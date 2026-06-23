use bevy::prelude::*;
use kurulus_core::prelude::*;

pub mod damage;
pub mod feedback;
pub mod hero_combat;
pub mod melee;
pub mod morale;
pub mod ranged;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DeathQueue>()
            .init_resource::<CombatConfig>()
            .init_resource::<ArmorResistanceTable>()
            .init_resource::<DamageEffectivenessTable>()
            .init_resource::<CombatDifficultyConfig>()
            .add_event::<HitFeedbackEvent>()
            .add_event::<HeroAuraApplied>()
            .add_systems(
                FixedUpdate,
                (
                    // Phase 5a: Pre-resolution
                    (
                        melee::flanking_calculation_system
                            .in_set(SimulationSet::FlankingCalculation),
                        melee::charge_update_system.in_set(SimulationSet::ChargeUpdate),
                        // Hero auras apply passive buffs to nearby allies before combat resolution
                        hero_combat::hero_aura_system.in_set(SimulationSet::FlankingCalculation),
                    ),
                    // Phase 5b: Resolution
                    (
                        attack_timer_system.in_set(SimulationSet::AttackTimers),
                        melee::melee_combat_system.in_set(SimulationSet::MeleeResolution),
                        ranged::ranged_combat_system.in_set(SimulationSet::RangedResolution),
                        ranged::ranged_ff_check_system.in_set(SimulationSet::RangedResolution),
                    ),
                    // Phase 5c: Application
                    (
                        damage::damage_application_system.in_set(SimulationSet::DamageApplication),
                        damage::dot_tick_system.in_set(SimulationSet::DotApplication),
                        // Morale drain reduces Stats.morale from DamageDealt events
                        morale::morale_drain_system.in_set(SimulationSet::MoraleCheck),
                        // Morale check emits panic events when morale hits 0
                        morale::morale_check_system.in_set(SimulationSet::MoraleCheck),
                        // Panic processing inserts PanicState from panic events
                        morale::panic_processing_system.in_set(SimulationSet::PanicProcessing),
                        // State machine keeps CurrentMoraleState in sync
                        morale::morale_state_machine_system.in_set(SimulationSet::MoraleUpdate),
                        // Morale update recovers morale over time
                        morale::morale_update_system.in_set(SimulationSet::MoraleUpdate),
                    ),
                    // Phase 5d: Hero knockout (intercept UnitDied for heroes)
                    hero_combat::hero_knockout_system.in_set(SimulationSet::DeathProcessing),
                    // Phase 5e: Death
                    death_queue_system.in_set(SimulationSet::DeathProcessing),
                    // Phase 5f: Feedback
                    feedback::hit_feedback_system.in_set(SimulationSet::HitFeedback),
                    // Phase 5g: Hero knockout recovery (runs every frame)
                    hero_combat::hero_knockout_recovery_system
                        .in_set(SimulationSet::HitFeedback),
                ),
            );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  ATTACK TIMER
// ═════════════════════════════════════════════════════════════════════════════

/// Ticks attack timers for all units.
/// Ensures each timer duration matches `1.0 / attack_speed` on each tick.
pub fn attack_timer_system(
    time: Res<Time<bevy::time::Fixed>>,
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

// ═════════════════════════════════════════════════════════════════════════════
//  DEATH QUEUE
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct DeathQueue {
    pub entries: Vec<(Entity, Timer)>,
}

/// Receives `UnitDied` events, starts a 2-second despawn timer, and
/// eventually despawns the entity with a simple loot drop.
fn death_queue_system(
    mut commands: Commands,
    time: Res<Time<bevy::time::Fixed>>,
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
                UnitArchetype::HeavyInfantry | UnitArchetype::HeavyCavalry => (10.0, 5.0),
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

/// Helper: calculate damage with armour penetration and resistance table.
pub fn calculate_damage(
    raw_damage: f32,
    damage_type: DamageType,
    armor: f32,
    armor_type: ArmorType,
    armor_pen: f32,
    effectiveness_table: &DamageEffectivenessTable,
    resistance_table: &ArmorResistanceTable,
    target_archetype: UnitArchetype,
) -> f32 {
    // 1. Apply damage-type effectiveness vs target archetype
    let effectiveness = effectiveness_table.multiplier(damage_type, target_archetype);
    let effective_damage = raw_damage * effectiveness;

    // 2. Apply armor penetration to get effective armor
    let effective_armor = (armor * (1.0 - armor_pen)).max(0.0);

    // 3. Look up armor type resistance for this damage type
    let resistance = resistance_table.resistance(armor_type, damage_type);

    // 4. Final formula: (damage × effectiveness) - (armor × resistance)
    (effective_damage - effective_armor * resistance).max(1.0)
}
