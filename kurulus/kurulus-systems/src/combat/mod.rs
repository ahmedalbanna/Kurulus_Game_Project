use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeathQueue>().add_systems(
            FixedUpdate,
            (
                attack_timer_system.in_set(SimulationSet::AttackTimers),
                melee_combat_system.in_set(SimulationSet::MeleeResolution),
                ranged_combat_system.in_set(SimulationSet::RangedResolution),
                damage_application_system.in_set(SimulationSet::DamageApplication),
                morale_update_system.in_set(SimulationSet::MoraleUpdate),
                death_queue_system.in_set(SimulationSet::DeathProcessing),
            ),
        );
    }
}

// ─── Attack timer ───────────────────────────────────────────────────────────

/// Ticks attack timers for all units.
/// Ensures each timer duration matches `1.0 / attack_speed` on each tick,
/// so that faster units attack more frequently.
fn attack_timer_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut timers: Query<(&mut AttackTimer, &Stats)>,
) {
    for (mut timer, stats) in timers.iter_mut() {
        let desired = 1.0 / stats.attack_speed.max(0.1);
        if (timer.0.duration().as_secs_f32() - desired).abs() > 0.001 {
            timer.0 = Timer::from_seconds(desired, TimerMode::Repeating);
        }
        timer.0.tick(time.delta());
    }
}

// ─── Melee combat ───────────────────────────────────────────────────────────

/// Resolves melee combat between attackers and targets in range.
/// Respects faction alignment (no friendly fire).
fn melee_combat_system(
    mut attack_triggers: EventWriter<AttackTriggered>,
    mut attackers: Query<
        (
            Entity,
            &WorldPosition,
            &Stats,
            &mut AttackTimer,
            &MeleeWeapon,
            &Unit,
        ),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<
        (Entity, &WorldPosition, &Health, &Unit),
        (With<Unit>, Without<Dead>),
    >,
    mut damage_events: EventWriter<DamageDealt>,
) {
    for (attacker_e, attacker_pos, attacker_stats, mut timer, weapon, attacker_unit) in
        attackers.iter_mut()
    {
        if !timer.0.just_finished() {
            continue;
        }

        let mut best_target = None;
        let mut best_dist = f32::MAX;

        for (target_e, target_pos, _target_health, target_unit) in targets.iter() {
            // --- Friendly fire prevention ---
            if attacker_unit.faction == target_unit.faction {
                continue;
            }
            if attacker_e == target_e {
                continue;
            }

            let dist = attacker_pos.world_2d.0.distance(target_pos.world_2d.0);
            if dist <= attacker_stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(target_e);
            }
        }

        if let Some(target_e) = best_target {
            let base_damage = weapon.damage;

            // Fire the damage event; the `damage_application_system` applies
            // armour calculations.
            damage_events.send(DamageDealt {
                attacker: attacker_e,
                target: target_e,
                amount: base_damage,
                damage_type: DamageType::Melee,
            });

            // Reset timer so the unit must wait before striking again
            timer.0 = Timer::from_seconds(
                1.0 / attacker_stats.attack_speed.max(0.1),
                TimerMode::Repeating,
            );

            // Fire attack-triggered event (used by stealth/ambush)
            attack_triggers.send(AttackTriggered {
                attacker: attacker_e,
                target: target_e,
                weapon_type: weapon.weapon_type,
            });
        }
    }
}

// ─── Ranged combat (placeholder) ────────────────────────────────────────────

/// Resolves ranged combat (stub — not needed for Mission 1).
fn ranged_combat_system(
    _commands: Commands,
    _attackers: Query<(&WorldPosition, &Stats, &RangedWeapon)>,
    _targets: Query<(&WorldPosition, &Health)>,
    _damage_events: EventWriter<DamageDealt>,
) {
    // TODO: Implement ranged combat with projectile simulation
}

// ─── Damage application ────────────────────────────────────────────────────

/// Central damage pipeline:
///   1. Apply armour reduction  (effective_armour = armour × (1 - penetration))
///   2. Subtract from target's Health
///   3. Insert `Dead` marker + fire `UnitDied` if health ≤ 0
fn damage_application_system(
    mut commands: Commands,
    mut damage_events: EventReader<DamageDealt>,
    mut healths: Query<&mut Health>,
    weapons: Query<&MeleeWeapon>,
    mut death_events: EventWriter<UnitDied>,
) {
    for event in damage_events.read() {
        let Ok(mut health) = healths.get_mut(event.target) else {
            continue;
        };

        // Look up the attacker's weapon to get armour penetration
        let armor_pen = weapons
            .get(event.attacker)
            .map(|w| w.armor_penetration)
            .unwrap_or(0.0);

        // Effective armour after penetration
        let effective_armour = (health.armor * (1.0 - armor_pen)).max(0.0);
        let final_damage = (event.amount - effective_armour).max(1.0); // minimum 1

        health.current = (health.current - final_damage).max(0.0);

        if health.current <= 0.0 {
            commands.entity(event.target).insert(Dead);
            death_events.send(UnitDied {
                entity: event.target,
                killer: event.attacker,
            });
        }
    }
}

// ─── Morale ─────────────────────────────────────────────────────────────────

/// Updates morale based on:
///   - Damage taken (scale: 1–5 points per hit)
///   - Ally death (scale: 10 points to all units — simplified for Phase 2)
///   - Recovery during Harvest phase (+5/s)
#[allow(clippy::too_many_arguments)]
fn morale_update_system(
    game_time: Res<GameTime>,
    phase: Res<State<GamePhase>>,
    mut damage_events: EventReader<DamageDealt>,
    mut death_events: EventReader<UnitDied>,
    mut stats_q: Query<(&mut Stats, &Health)>,
    mut broken_events: EventWriter<MoraleBroken>,
) {
    // Morale loss from taking damage
    for event in damage_events.read() {
        if let Ok((mut stats, health)) = stats_q.get_mut(event.target) {
            let damage_pct = event.amount / health.max;
            let morale_loss = (damage_pct * 20.0).clamp(1.0, 5.0);
            stats.morale = (stats.morale - morale_loss).max(0.0);

            if stats.morale <= 0.0 {
                broken_events.send(MoraleBroken {
                    entity: event.target,
                    morale_value: 0.0,
                });
            }
        }
    }

    // Morale loss from seeing an ally die (affects all units in a simplified
    // manner for Phase 2)
    for _event in death_events.read() {
        for (mut stats, _health) in stats_q.iter_mut() {
            stats.morale = (stats.morale - 10.0).max(0.0);
        }
    }

    // Morale recovery during Harvest
    if *phase.get() == GamePhase::Harvest {
        for (mut stats, _health) in stats_q.iter_mut() {
            stats.morale =
                (stats.morale + 5.0 * game_time.time_scale).min(stats.morale_max);
        }
    }
}

// ─── Death queue ───────────────────────────────────────────────────────────

/// Tracks dead entities and despawns them after a visual delay.
#[derive(Resource)]
pub struct DeathQueue {
    pub entries: Vec<(Entity, Timer)>,
}

impl Default for DeathQueue {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// Receives `UnitDied` events, starts a 2-second despawn timer, and
/// eventually despawns the entity with a simple loot drop.
fn death_queue_system(
    mut commands: Commands,
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut queue: ResMut<DeathQueue>,
    mut unit_died_events: EventReader<UnitDied>,
    mut stockpile: ResMut<ResourceStockpile>,
    units: Query<&Unit>,
) {
    // Add newly dead entities to the queue
    for event in unit_died_events.read() {
        queue
            .entries
            .push((event.entity, Timer::from_seconds(2.0, TimerMode::Once)));
    }

    // Tick all timers and collect finished ones
    let mut to_despawn = Vec::new();
    for (entity, timer) in queue.entries.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            to_despawn.push(*entity);
        }
    }
    queue.entries.retain(|(e, _)| !to_despawn.contains(e));

    // Loot drop & despawn for finished entities
    for entity in to_despawn {
        if let Ok(unit) = units.get(entity) {
            let loot = match unit.archetype {
                UnitArchetype::LightInfantry | UnitArchetype::Archer => (5.0, 2.0),
                UnitArchetype::HeavyInfantry | UnitArchetype::HeavyCavalry => {
                    (10.0, 5.0)
                }
                UnitArchetype::LightCavalry | UnitArchetype::Scout => (3.0, 5.0),
                UnitArchetype::Worker => (15.0, 0.0),
                UnitArchetype::SiegeCrew => (0.0, 10.0),
            };
            stockpile.food += loot.0;
            stockpile.gold += loot.1;
        }
        commands.entity(entity).despawn();
    }
}

/// Calculates raw damage with armour penetration (used externally by ambush).
pub fn calculate_damage(weapon: &MeleeWeapon, armor: f32, _armor_type: ArmorType) -> f32 {
    let effective_armor = (armor * (1.0 - weapon.armor_penetration)).max(0.0);
    (weapon.damage - effective_armor).max(1.0)
}
