use bevy::prelude::*;
use kurulus_core::prelude::*;
use rand::Rng;

// ═════════════════════════════════════════════════════════════════════════════
//  RANGED COMBAT (GDD §6 — Ranged Combat)
// ═════════════════════════════════════════════════════════════════════════════

/// Resolves ranged attacks with accuracy checks, range penalties,
/// cover mechanics, ammo consumption, formation bonuses, and equipment stats.
pub fn ranged_combat_system(
    mut attack_triggers: EventWriter<AttackTriggered>,
    mut attackers: Query<
        (
            Entity,
            &WorldPosition,
            &Stats,
            &mut AttackTimer,
            &RangedWeapon,
            &Unit,
            &Health,
            Option<&mut Ammunition>,
            Option<&ActiveBuffs>,
            Option<&Equipment>,
            Option<&FormationBonus>,
            Option<&CurrentMoraleState>,
        ),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<
        (
            Entity,
            &WorldPosition,
            &Health,
            &Unit,
            Option<&FormationBonus>,
            Option<&CurrentMoraleState>,
        ),
        (With<Unit>, Without<Dead>),
    >,
    mut damage_events: EventWriter<DamageDealt>,
    config: Res<CombatConfig>,
    effectiveness_table: Res<DamageEffectivenessTable>,
    resistance_table: Res<ArmorResistanceTable>,
) {
    // Collect target data (with formation bonus + morale info)
    #[allow(clippy::type_complexity)]
    let target_list: Vec<(
        Entity,
        Vec2,
        UnitArchetype,
        FactionId,
        ArmorType,
        f32,
        Option<FormationBonus>,
        Option<CurrentMoraleState>,
    )> = targets
        .iter()
        .map(|(e, pos, health, unit, formation, morale)| {
            (
                e,
                pos.world_2d.0,
                unit.archetype,
                unit.faction,
                health.armor_type,
                health.armor,
                formation.copied(),
                morale.copied(),
            )
        })
        .collect();

    for (
        attacker_e,
        attacker_pos,
        stats,
        mut timer,
        weapon,
        unit,
        _health,
        mut ammo_opt,
        buffs_opt,
        equipment_opt,
        formation_opt,
        attacker_morale_opt,
    ) in attackers.iter_mut()
    {
        if !timer.0.just_finished() {
            continue;
        }

        // ─── Ammo check ────────────────────────────────────────────────
        let out_of_ammo = match ammo_opt.as_deref_mut() {
            Some(ammo) if ammo.current > 0 => {
                ammo.current -= 1;
                false
            }
            Some(_) => true, // Out of ammo
            None => false,   // No Ammo component = infinite ammo
        };

        if out_of_ammo {
            continue;
        }

        // ─── Target selection (closest enemy within range) ─────────────
        let mut best_target = None;
        let mut best_dist = f32::MAX;

        for (
            target_e,
            target_pos,
            _archetype,
            faction,
            _armor_type,
            _armor,
            _formation,
            _target_morale,
        ) in &target_list
        {
            if unit.faction == *faction || attacker_e == *target_e {
                continue;
            }
            let dist = attacker_pos.world_2d.0.distance(*target_pos);
            if dist <= stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(*target_e);
            }
        }

        if let Some(target_e) = best_target {
            // Get target data
            let t_data = target_list.iter().find(|(e, ..)| *e == target_e);
            let (
                _target_e,
                target_pos,
                target_archetype,
                _target_faction,
                target_armor_type,
                target_armor_val,
                target_formation_opt,
                target_morale_opt,
            ) = match t_data {
                Some(d) => d.clone(),
                None => continue,
            };

            // ─── Compute stat bonuses from equipment + buffs + morale ───
            let equipment_damage = equipment_opt
                .map(|e| {
                    e.weapon
                        .as_ref()
                        .map(|w| w.stats.damage_bonus)
                        .unwrap_or(0.0)
                        + e.armor
                            .as_ref()
                            .map(|a| a.stats.damage_bonus)
                            .unwrap_or(0.0)
                        + e.accessory_1
                            .as_ref()
                            .map(|a| a.stats.damage_bonus)
                            .unwrap_or(0.0)
                        + e.accessory_2
                            .as_ref()
                            .map(|a| a.stats.damage_bonus)
                            .unwrap_or(0.0)
                })
                .unwrap_or(0.0);

            let buff_damage_bonus = buffs_opt
                .map(|b| b.buffs.iter().map(|bf| bf.damage_bonus).sum::<f32>())
                .unwrap_or(0.0);

            let buff_attack_speed = buffs_opt
                .map(|b| b.buffs.iter().map(|bf| bf.attack_speed_bonus).sum::<f32>())
                .unwrap_or(0.0);

            let effective_attack_speed = (stats.attack_speed + buff_attack_speed).max(0.1);

            // Attacker morale damage multiplier
            let atk_morale_damage_mult = attacker_morale_opt.map(|m| m.damage_mult).unwrap_or(1.0);

            // Defender morale defense multiplier
            let def_morale_defense_mult = target_morale_opt.map(|m| m.defense_mult).unwrap_or(1.0);

            // Attacker formation
            let atk_formation = formation_opt.copied().unwrap_or_default();

            // Defender formation — typically Scatter spreads units, reducing ranged damage
            let def_formation = target_formation_opt.unwrap_or_default();
            let def_incoming_mult = def_formation.incoming_damage_mult;

            // ─── Distance penalty ───────────────────────────────────────
            let max_range = stats.attack_range.max(1.0);
            let dist_ratio = best_dist / max_range;
            let distance_penalty =
                1.0 - (dist_ratio * config.ranged_distance_falloff).min(config.max_range_penalty);

            // ─── Accuracy check ─────────────────────────────────────────
            let effective_accuracy = (weapon.base_accuracy * distance_penalty).max(0.1);
            let hit = rand::thread_rng().gen_range(0.0..1.0) < effective_accuracy;

            if !hit {
                // Miss — ammo already consumed
                continue;
            }

            // ─── Friendly fire check ────────────────────────────────────
            // Use the simplified target list for FF check (strip formation data)
            let ff_target_list: Vec<(Entity, Vec2, UnitArchetype, FactionId, ArmorType, f32)> =
                target_list
                    .iter()
                    .map(|(e, pos, arch, fac, at, av, _, _)| (*e, *pos, *arch, *fac, *at, *av))
                    .collect();

            let friendly_fire = check_friendly_fire(
                attacker_e,
                unit.faction,
                attacker_pos.world_2d.0,
                target_pos,
                &ff_target_list,
                config.friendly_fire_cone_angle,
                config.friendly_fire_max_check_dist,
            );

            let ff_mult = if friendly_fire {
                config.friendly_fire_damage_reduction
            } else {
                1.0
            };

            // ─── Damage calculation ─────────────────────────────────────
            // Base damage = weapon * distance * FF * equip * buff * formation * morale
            let raw_damage = weapon.damage
                * distance_penalty
                * ff_mult
                * (1.0 + equipment_damage)
                * (1.0 + buff_damage_bonus)
                * atk_formation.damage_mult
                * atk_morale_damage_mult;

            let dmg_type =
                kurulus_core::components::combat::damage_type_for_weapon(weapon.weapon_type);
            let final_damage = super::calculate_damage(
                raw_damage,
                dmg_type,
                target_armor_val,
                target_armor_type,
                weapon.armor_penetration,
                &effectiveness_table,
                &resistance_table,
                target_archetype,
            )
            .max(config.min_damage)
            * def_incoming_mult // Defender formation reduces incoming
            * def_morale_defense_mult; // Defender morale defense

            // ─── Emit events ────────────────────────────────────────────
            damage_events.send(DamageDealt {
                attacker: attacker_e,
                target: target_e,
                amount: final_damage,
                damage_type: dmg_type,
                crit: CritType::None,
                flanking: FlankingInfo::default(),
                is_friendly_fire: friendly_fire,
                source_ability: None,
            });

            attack_triggers.send(AttackTriggered {
                attacker: attacker_e,
                target: target_e,
                weapon_type: weapon.weapon_type,
                is_charge: false,
                is_from_stealth: false,
            });

            // Reset timer (with buffed attack speed)
            timer.0 =
                Timer::from_seconds(1.0 / effective_attack_speed.max(0.1), TimerMode::Repeating);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  RANGED FRIENDLY FIRE CHECK
// ═════════════════════════════════════════════════════════════════════════════

/// Checks if a friendly unit is within the cone between attacker and target.
#[allow(clippy::type_complexity)]
fn check_friendly_fire(
    attacker_e: Entity,
    faction: FactionId,
    from: Vec2,
    to: Vec2,
    targets: &[(Entity, Vec2, UnitArchetype, FactionId, ArmorType, f32)],
    cone_angle_deg: f32,
    max_check_dist: f32,
) -> bool {
    let attack_dir = to - from;
    let attack_len = attack_dir.length();
    if attack_len < 0.01 {
        return false;
    }
    let attack_dir_norm = attack_dir / attack_len;
    let check_len = attack_len.min(max_check_dist);

    for (e, pos, _arch, f_id, _armor_type, _armor) in targets {
        if *e == attacker_e || *f_id != faction {
            continue;
        }
        let to_friendly = *pos - from;
        let friendly_dist = to_friendly.length();
        if friendly_dist > check_len || friendly_dist < 0.1 {
            continue;
        }
        let friendly_dir = to_friendly / friendly_dist;
        let dot = friendly_dir.dot(attack_dir_norm);
        let angle = dot.acos().to_degrees();
        if angle <= cone_angle_deg * 0.5 {
            return true;
        }
    }
    false
}

// ═════════════════════════════════════════════════════════════════════════════
//  RANGED FRIENDLY FIRE SYSTEM
// ═════════════════════════════════════════════════════════════════════════════

/// Additional system that checks for and handles friendly fire separately.
pub fn ranged_ff_check_system(_ff_events: EventWriter<RangedFriendlyFire>) {
    // Stub — full implementation will emit events for UI/morale penalties
}
