use bevy::prelude::*;
use kurulus_core::prelude::*;
use rand::Rng;

// ═════════════════════════════════════════════════════════════════════════════
//  FLANKING CALCULATION (GDD §1.2 — Encirclement)
// ═════════════════════════════════════════════════════════════════════════════

/// Computes the attack zone (Front/Side/Rear) for each attacker-target pair
/// based on the target's facing direction. Stores the result as a FlankingInfo
/// component on the attacker so the melee system can read it.
///
/// Zone thresholds:
///   Front: ±45°  — ×1.0, full block
///   Side:  45–135° — ×1.25, -15% block
///   Rear:  135–180° — ×1.5, -30% block
pub fn flanking_calculation_system(
    mut attackers: Query<
        (Entity, &WorldPosition, &mut FlankingInfo, Option<&Facing>),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<(Entity, &WorldPosition, Option<&Facing>), (With<Unit>, Without<Dead>)>,
) {
    // Collect target positions and facings into a Vec for iteration
    let target_list: Vec<(Entity, Vec2, Vec2)> = targets
        .iter()
        .map(|(e, pos, facing)| {
            let fwd = facing.map(|f| f.direction).unwrap_or(Vec2::NEG_Y);
            (e, pos.world_2d.0, fwd)
        })
        .collect();

    for (attacker_e, attacker_pos, mut flank_info, _attacker_facing_opt) in attackers.iter_mut() {
        for (target_e, target_pos, target_facing) in &target_list {
            if *target_e == attacker_e {
                continue;
            }

            // Vector from target to attacker
            let attack_vec = attacker_pos.world_2d.0 - *target_pos;
            let dist = attack_vec.length();
            if dist < 0.01 || dist > 10.0 {
                continue;
            }

            let attack_dir = attack_vec / dist;
            let dot = attack_dir.dot(*target_facing);
            let angle_deg = dot.acos().to_degrees().clamp(0.0, 180.0);

            let (zone, dmg_mult, block_reduction) = if angle_deg <= 45.0 {
                (AttackZone::Front, 1.0, 0.0)
            } else if angle_deg <= 135.0 {
                (AttackZone::Side, 1.25, 0.15)
            } else {
                (AttackZone::Rear, 1.5, 0.30)
            };

            *flank_info = FlankingInfo {
                attack_angle: angle_deg,
                zone,
                damage_multiplier: dmg_mult,
                block_reduction,
            };
            break;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CHARGE SYSTEM (GDD §3 — Charge)
// ═════════════════════════════════════════════════════════════════════════════

/// Tracks charging units, accumulates distance traveled, and detects impact
/// when a charging unit enters attack range of its target.
pub fn charge_update_system(
    mut commands: Commands,
    mut chargers: Query<
        (
            Entity,
            &mut ChargeState,
            &WorldPosition,
            &Stats,
            &MeleeWeapon,
            &Unit,
        ),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<(Entity, &WorldPosition, &Unit), Without<Dead>>,
    config: Res<CombatConfig>,
    mut charge_impact_events: EventWriter<ChargeImpact>,
    mut damage_events: EventWriter<DamageDealt>,
    mut attack_triggers: EventWriter<AttackTriggered>,
) {
    #[allow(clippy::type_complexity)]
    let target_list: Vec<(Entity, Vec2, FactionId)> = targets
        .iter()
        .map(|(e, pos, unit)| (e, pos.world_2d.0, unit.faction))
        .collect();

    for (entity, mut charge, pos, stats, weapon, unit) in chargers.iter_mut() {
        if !charge.is_charging || charge.has_impacted {
            continue;
        }

        if charge.charge_distance_traveled < charge.required_distance {
            continue;
        }

        // Find nearest enemy in attack range
        let mut best_target = None;
        let mut best_dist = f32::MAX;

        for (target_e, target_pos, faction) in &target_list {
            if *faction == unit.faction || *target_e == entity {
                continue;
            }
            let dist = pos.world_2d.0.distance(*target_pos);
            if dist <= stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(*target_e);
            }
        }

        if let Some(target_e) = best_target {
            charge.has_impacted = true;

            let is_cavalry = matches!(
                unit.archetype,
                UnitArchetype::LightCavalry | UnitArchetype::HeavyCavalry
            );

            let charge_damage = weapon.damage * config.charge_damage_multiplier;
            let knockdown = if is_cavalry {
                config.cavalry_knockdown_duration
            } else {
                0.0
            };

            damage_events.send(DamageDealt {
                attacker: entity,
                target: target_e,
                amount: charge_damage,
                damage_type: DamageType::Impact,
                crit: CritType::Charge,
                flanking: FlankingInfo::default(),
                is_friendly_fire: false,
                source_ability: None,
            });

            charge_impact_events.send(ChargeImpact {
                charger: entity,
                target: target_e,
                damage: charge_damage,
                knockdown_duration: knockdown,
                formation_break_duration: config.charge_formation_break_duration,
            });

            attack_triggers.send(AttackTriggered {
                attacker: entity,
                target: target_e,
                weapon_type: weapon.weapon_type,
                is_charge: true,
                is_from_stealth: false,
            });

            commands.entity(entity).remove::<ChargeState>();
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  MELEE COMBAT (GDD §5 — Melee Combat)
// ═════════════════════════════════════════════════════════════════════════════

/// Resolves melee combat with full damage pipeline including:
///   - Nearest-target selection
///   - Flanking zone multipliers
///   - Block/Parry checks
///   - Dodge checks
///   - Critical hit rolls
///   - Formation bonus integration (equipment, buffs, morale)
pub fn melee_combat_system(
    mut attack_triggers: EventWriter<AttackTriggered>,
    mut attackers: Query<
        (
            Entity,
            &WorldPosition,
            &Stats,
            &mut AttackTimer,
            &MeleeWeapon,
            &Unit,
            Option<&FlankingInfo>,
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
        attacker_stats,
        mut timer,
        weapon,
        attacker_unit,
        flanking_opt,
        buffs_opt,
        equipment_opt,
        formation_opt,
        attacker_morale_opt,
    ) in attackers.iter_mut()
    {
        if !timer.0.just_finished() {
            continue;
        }

        let mut best_target = None;
        let mut best_dist = f32::MAX;

        for (
            target_e,
            target_pos,
            _archetype,
            target_faction,
            _armor_type,
            _armor,
            _formation,
            _target_morale,
        ) in &target_list
        {
            if attacker_unit.faction == *target_faction || attacker_e == *target_e {
                continue;
            }
            let dist = attacker_pos.world_2d.0.distance(*target_pos);
            if dist <= attacker_stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(*target_e);
            }
        }

        if let Some(target_e) = best_target {
            // Retrieve target data
            let t_data = target_list.iter().find(|(e, ..)| *e == target_e);
            let (
                _target_e,
                _target_pos,
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

            // ─── 0. Compute stat bonuses from equipment + buffs + morale ─────────
            let equipment = equipment_opt
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

            let armor_from_equipment = equipment_opt
                .map(|e| {
                    e.weapon
                        .as_ref()
                        .map(|w| w.stats.defense_bonus)
                        .unwrap_or(0.0)
                        + e.armor
                            .as_ref()
                            .map(|a| a.stats.defense_bonus)
                            .unwrap_or(0.0)
                        + e.accessory_1
                            .as_ref()
                            .map(|a| a.stats.defense_bonus)
                            .unwrap_or(0.0)
                        + e.accessory_2
                            .as_ref()
                            .map(|a| a.stats.defense_bonus)
                            .unwrap_or(0.0)
                })
                .unwrap_or(0.0);

            // Buffs
            let buff_damage_bonus = buffs_opt
                .map(|b| b.buffs.iter().map(|bf| bf.damage_bonus).sum::<f32>())
                .unwrap_or(0.0);
            let buff_attack_speed = buffs_opt
                .map(|b| b.buffs.iter().map(|bf| bf.attack_speed_bonus).sum::<f32>())
                .unwrap_or(0.0);

            // Attacker formation bonus
            let atk_formation = formation_opt.copied().unwrap_or_default();
            let atk_formation_dmg = atk_formation.damage_mult;
            let atk_formation_cav_mult = if target_archetype == UnitArchetype::LightCavalry
                || target_archetype == UnitArchetype::HeavyCavalry
            {
                atk_formation.anti_cavalry_mult
            } else {
                1.0
            };

            // Defender formation bonus (reduces incoming damage)
            let def_formation = target_formation_opt.unwrap_or_default();
            let def_incoming_mult = def_formation.incoming_damage_mult;

            // Apply morale state multipliers
            let atk_morale_damage_mult = attacker_morale_opt.map(|m| m.damage_mult).unwrap_or(1.0);
            let def_morale_defense_mult = target_morale_opt.map(|m| m.defense_mult).unwrap_or(1.0);

            // Apply effective attack speed from buffs
            let effective_attack_speed = (attacker_stats.attack_speed + buff_attack_speed).max(0.1);

            // ─── 1. Flanking damage multiplier ──────────────────────────
            let flanking = flanking_opt.copied().unwrap_or_default();
            let flank_mult = flanking.damage_multiplier;

            // ─── 2. Block check (with formation bonus) ───────────────────
            let base_block =
                kurulus_core::components::combat::compute_block_chance(attacker_unit.archetype);
            let block_reduction = flanking.block_reduction;
            let formation_block_bonus = atk_formation.block_bonus / 100.0; // percentage points to fraction
            let block_chance = (base_block - block_reduction + formation_block_bonus).max(0.0);
            let blocked =
                block_chance > 0.0 && rand::thread_rng().gen_range(0.0..1.0) < block_chance;

            // ─── 3. Dodge check ─────────────────────────────────────────
            let dodge_chance =
                kurulus_core::components::combat::compute_dodge_chance(attacker_unit.archetype);
            let dodged = !blocked
                && dodge_chance > 0.0
                && rand::thread_rng().gen_range(0.0..1.0) < dodge_chance;

            // ─── 4. Critical hit roll ───────────────────────────────────
            let base_crit = config.base_crit_chance;
            let flank_crit = match flanking.zone {
                AttackZone::Front => 0.0,
                AttackZone::Side => config.flank_crit_bonus_side,
                AttackZone::Rear => config.flank_crit_bonus_rear,
            };
            let crit_chance = base_crit + flank_crit;
            let is_crit = !blocked
                && crit_chance > 0.0
                && rand::thread_rng().gen_range(0.0..1.0) < crit_chance;

            // ─── 5. Calculate final damage ──────────────────────────────
            // Base damage = weapon * flank * formation * equip * buffs * anti-cav * morale
            let raw_damage = weapon.damage
                * flank_mult
                * atk_formation_dmg
                * (1.0 + equipment)
                * (1.0 + buff_damage_bonus)
                * atk_formation_cav_mult
                * atk_morale_damage_mult;

            let damage_type =
                kurulus_core::components::combat::damage_type_for_weapon(weapon.weapon_type);
            let final_damage = if blocked {
                raw_damage * 0.5
            } else if dodged {
                0.0
            } else if is_crit {
                let base = super::calculate_damage(
                    raw_damage,
                    damage_type,
                    target_armor_val + armor_from_equipment,
                    target_armor_type,
                    weapon.armor_penetration,
                    &effectiveness_table,
                    &resistance_table,
                    target_archetype,
                );
                // Apply defender formation + morale defense reduction to critical
                (base * config.crit_damage_multiplier * def_incoming_mult * def_morale_defense_mult)
                    .max(config.min_damage)
            } else {
                let base = super::calculate_damage(
                    raw_damage,
                    damage_type,
                    target_armor_val + armor_from_equipment,
                    target_armor_type,
                    weapon.armor_penetration,
                    &effectiveness_table,
                    &resistance_table,
                    target_archetype,
                );
                (base * def_incoming_mult * def_morale_defense_mult).max(config.min_damage)
            };

            damage_events.send(DamageDealt {
                attacker: attacker_e,
                target: target_e,
                amount: final_damage,
                damage_type,
                crit: if is_crit {
                    CritType::Flanking
                } else {
                    CritType::None
                },
                flanking,
                is_friendly_fire: false,
                source_ability: None,
            });

            timer.0 =
                Timer::from_seconds(1.0 / effective_attack_speed.max(0.1), TimerMode::Repeating);

            attack_triggers.send(AttackTriggered {
                attacker: attacker_e,
                target: target_e,
                weapon_type: weapon.weapon_type,
                is_charge: false,
                is_from_stealth: false,
            });
        }
    }
}
