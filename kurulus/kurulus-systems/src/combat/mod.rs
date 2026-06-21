use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                attack_timer_system.in_set(SimulationSet::AttackTimers),
                melee_combat_system.in_set(SimulationSet::MeleeResolution),
                ranged_combat_system.in_set(SimulationSet::RangedResolution),
                damage_application_system.in_set(SimulationSet::DamageApplication),
                morale_update_system.in_set(SimulationSet::MoraleUpdate),
                death_processing_system.in_set(SimulationSet::DeathProcessing),
            ),
        );
    }
}

/// Ticks attack timers for all units
fn attack_timer_system(time: Res<bevy::time::Time>, mut timers: Query<&mut AttackTimer>) {
    for mut timer in timers.iter_mut() {
        timer.0.tick(time.delta());
    }
}

/// Resolves melee combat between attackers and targets in range
fn melee_combat_system(
    mut commands: Commands,
    attackers: Query<
        (
            Entity,
            &WorldPosition,
            &Stats,
            &mut AttackTimer,
            &MeleeWeapon,
        ),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<(Entity, &WorldPosition, &Health, &Stats), (With<Unit>, Without<Dead>)>,
    mut damage_events: EventWriter<DamageDealt>,
    mut death_events: EventWriter<UnitDied>,
) {
    for (attacker_e, attacker_pos, attacker_stats, timer, weapon) in attackers.iter() {
        if !timer.0.just_finished() {
            continue;
        }

        let mut best_target = None;
        let mut best_dist = f32::MAX;

        for (target_e, target_pos, target_health, _target_stats) in targets.iter() {
            if attacker_e == target_e {
                continue;
            }
            let dist = attacker_pos.world_2d.0.distance(target_pos.world_2d.0);
            if dist <= attacker_stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some((target_e, target_health));
            }
        }

        if let Some((target_e, target_health)) = best_target {
            let damage = calculate_damage(weapon, target_health.armor, target_health.armor_type);
            commands.entity(target_e).insert(Health {
                current: (target_health.current - damage).max(0.0),
                ..*target_health
            });
            damage_events.send(DamageDealt {
                attacker: attacker_e,
                target: target_e,
                amount: damage,
                damage_type: DamageType::Melee,
            });
            if target_health.current - damage <= 0.0 {
                death_events.send(UnitDied {
                    entity: target_e,
                    killer: attacker_e,
                });
            }
        }
    }
}

/// Resolves ranged combat
fn ranged_combat_system(
    _commands: Commands,
    _attackers: Query<(&WorldPosition, &Stats, &RangedWeapon)>,
    _targets: Query<(&WorldPosition, &Health)>,
    _damage_events: EventWriter<DamageDealt>,
) {
    // TODO: Implement ranged combat with projectile simulation
}

/// Applies damage from events (allows damage modifiers, armor calculations)
fn damage_application_system(
    _damage_events: EventReader<DamageDealt>,
    _healths: Query<&mut Health>,
) {
    // TODO: Implement damage modifiers (armor, resistances, etc.)
}

/// Updates morale based on recent combat results
fn morale_update_system(
    _damage_events: EventReader<DamageDealt>,
    _death_events: EventReader<UnitDied>,
    _stats: Query<&mut Stats>,
) {
    // TODO: Implement morale calculations
}

/// Processes deaths (removes dead entities, spawns loot, etc.)
fn death_processing_system(mut commands: Commands, mut death_events: EventReader<UnitDied>) {
    for event in death_events.read() {
        commands.entity(event.entity).insert(Dead);
        // TODO: Despawn after death animation, spawn loot
    }
}

/// Calculates damage with armor penetration and type matchups
fn calculate_damage(weapon: &MeleeWeapon, armor: f32, _armor_type: ArmorType) -> f32 {
    let effective_armor = (armor * (1.0 - weapon.armor_penetration)).max(0.0);
    (weapon.damage - effective_armor).max(1.0)
}
