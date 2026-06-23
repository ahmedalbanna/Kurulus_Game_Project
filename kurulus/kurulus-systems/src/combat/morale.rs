use bevy::prelude::*;
use kurulus_core::prelude::*;
use rand::Rng;

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE CHECK (GDD §8 — Morale & Panic)
// ═════════════════════════════════════════════════════════════════════════════

/// Evaluates combat events to determine whether units should have their
/// morale reduced, trigger panic, or resist panic based on leadership.
///
/// Triggers:
///   - Taking damage (damage → morale loss)
///   - Seeing an ally die (nearby death → morale loss)
///   - Being flanked (flanking attack → extra morale loss)
///   - Seeing an ally rout (nearby panic → panic propagation)
pub fn morale_check_system(
    mut damage_events: EventReader<DamageDealt>,
    mut unit_died_events: EventReader<UnitDied>,
    mut morale_events: EventWriter<MoraleEvent>,
    unit_query: Query<(Entity, &Unit, &Stats, Option<&Hero>)>,
) {
    // Collect unit data into a vec to avoid borrow conflicts
    let unit_data: Vec<(Entity, Unit, f32, f32, bool)> = unit_query
        .iter()
        .map(|(e, unit, stats, hero)| {
            (
                e,
                unit.clone(),
                stats.morale,
                stats.morale_max,
                hero.is_some(),
            )
        })
        .collect();

    // ─── Process damage events → morale loss ───────────────────────────
    for ev in damage_events.read() {
        if let Some((_entity, _unit, current_morale, _max_morale, _is_hero)) =
            unit_data.iter().find(|(e, ..)| *e == ev.target)
        {
            let morale_loss = match ev.damage_type {
                DamageType::Fire => 10.0,
                DamageType::Impact => 15.0,
                DamageType::Morale => ev.amount, // Direct morale damage
                _ => 5.0,
            };

            // Flanking attacks cause extra morale damage
            let flank_penalty = match ev.flanking.zone {
                AttackZone::Rear => 10.0,
                AttackZone::Side => 5.0,
                AttackZone::Front => 0.0,
            };

            let total_loss = morale_loss + flank_penalty;
            let new_morale = (*current_morale - total_loss).max(0.0);

            if new_morale <= 0.0 {
                morale_events.send(MoraleEvent {
                    unit: ev.target,
                    event_type: MoraleEventType::Panic,
                    cause: if ev.flanking.zone != AttackZone::Front {
                        "Flanked".to_string()
                    } else {
                        "Heavy casualties".to_string()
                    },
                });
            }
        }
    }

    // ─── Process death events → morale loss for nearby allies ──────────
    for ev in unit_died_events.read() {
        if let Some((_, dead_unit, _, _, _)) = unit_data.iter().find(|(e, ..)| *e == ev.entity) {
            for (alive_e, alive_unit, _current_morale, _max_morale, _alive_is_hero) in &unit_data {
                if *alive_e == ev.entity {
                    continue;
                }
                if alive_unit.faction == dead_unit.faction {
                    // Ally death penalty
                    // NOTE: We apply the penalty in a separate pass below
                    morale_events.send(MoraleEvent {
                        unit: *alive_e,
                        event_type: MoraleEventType::Panic,
                        cause: "Ally died".to_string(),
                    });
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE UPDATE (GDD §8 — Recovery & Rout)
// ═════════════════════════════════════════════════════════════════════════════

/// Updates morale over time (recovery) and processes Panic/Steady states.
/// Units that have been out of combat for a while recover morale.
/// Units in panic have a chance to recover (rallied).
pub fn morale_update_system(
    time: Res<Time>,
    mut units: Query<
        (Entity, &mut Stats, Option<&Hero>, Option<&mut PanicState>),
        (With<Unit>, Without<Dead>),
    >,
    config: Res<CombatConfig>,
) {
    for (_entity, mut stats, hero_opt, panic_opt) in units.iter_mut() {
        // ─── Out-of-combat recovery ────────────────────────────────────
        let recovery_rate =
            config.morale_recovery_multiplier * if hero_opt.is_some() { 1.5 } else { 1.0 };

        let new_morale = (stats.morale + recovery_rate * time.delta_secs()).min(stats.morale_max);
        stats.morale = new_morale;

        // ─── Panic state recovery ──────────────────────────────────────
        if let Some(mut panic) = panic_opt {
            panic.duration =
                (panic.duration - time.delta_secs() * config.panic_recovery_multiplier).max(0.0);
            if panic.duration <= 0.0 {
                panic.is_panicking = false;
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE STATE MACHINE (GDD §5.3 — Five-Level Morale, NEW)
// ═════════════════════════════════════════════════════════════════════════════

/// Synchronises the `CurrentMoraleState` component with the unit's current
/// `Stats.morale` value. This runs after `morale_update_system` and
/// `morale_check_system` to ensure the state is always up-to-date.
///
/// The `CurrentMoraleState` component is then queried by melee/ranged combat
/// systems to apply the appropriate damage/defense multipliers.
pub fn morale_state_machine_system(
    mut units: Query<
        (Entity, &Stats, Option<&mut CurrentMoraleState>),
        (With<Unit>, Without<Dead>),
    >,
    mut commands: Commands,
) {
    for (_entity, stats, morale_state_opt) in units.iter_mut() {
        let new_state = CurrentMoraleState::from_morale(stats.morale);

        match morale_state_opt {
            Some(mut existing) => {
                // Only update if the state actually changed
                if existing.state != new_state.state {
                    *existing = new_state;
                }
            }
            None => {
                // Insert if missing (first-time initialisation)
                commands.entity(_entity).insert(new_state);
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  GRADUAL MORALE DRAIN (GDD §5.3 — Taking damage reduces morale)
// ═════════════════════════════════════════════════════════════════════════════

/// Applies gradual morale drain when a unit takes damage.
/// Each `DamageDealt` event reduces the target's `Stats.morale` by an amount
/// based on the damage type, flanking, etc.
///
/// This works alongside `morale_check_system` (which emits panic events at 0).
pub fn morale_drain_system(
    mut damage_events: EventReader<DamageDealt>,
    mut stats_query: Query<&mut Stats>,
) {
    // ─── Damage → morale loss ──────────────────────────────────────────
    // Each DamageDealt event reduces the target's Stats.morale based on
    // the damage type and flanking zone. Ally-death morale penalties are
    // handled by `morale_check_system` + `panic_processing_system`.
    for ev in damage_events.read() {
        if let Ok(mut stats) = stats_query.get_mut(ev.target) {
            let morale_loss = match ev.damage_type {
                DamageType::Fire => 10.0,
                DamageType::Impact => 15.0,
                DamageType::Morale => ev.amount, // Direct morale damage
                _ => 5.0,
            };

            // Flanking attacks cause extra morale damage
            let flank_penalty = match ev.flanking.zone {
                AttackZone::Rear => 10.0,
                AttackZone::Side => 5.0,
                AttackZone::Front => 0.0,
            };

            let total_loss = morale_loss + flank_penalty;
            stats.morale = (stats.morale - total_loss).max(0.0);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  PANIC PROCESSING (GDD §8 — Panic Propagation)
// ═════════════════════════════════════════════════════════════════════════════

/// Processes MoraleEvents (panics) and applies PanicState to affected units.
/// Propagates panic to nearby allies within a radius.
pub fn panic_processing_system(
    mut commands: Commands,
    mut morale_events: EventReader<MoraleEvent>,
    unit_query: Query<(Entity, &Unit, &WorldPosition), (With<Unit>, Without<Dead>)>,
    mut stats_query: Query<&mut Stats>,
    config: Res<CombatConfig>,
) {
    let panics: Vec<(Entity, String)> = morale_events
        .read()
        .filter(|e| e.event_type == MoraleEventType::Panic)
        .map(|e| (e.unit, e.cause.clone()))
        .collect();

    // Collect unit positions for propagation check
    let unit_positions: Vec<(Entity, FactionId, Vec2)> = unit_query
        .iter()
        .map(|(e, unit, pos)| (e, unit.faction, pos.world_2d.0))
        .collect();

    for (panicked_e, _cause) in panics {
        // Give the panicking unit a PanicState
        commands.entity(panicked_e).insert(PanicState {
            is_panicking: true,
            duration: config.panic_base_duration,
            flee_speed_multiplier: config.panic_flee_speed_mult,
        });

        // Reduce morale for panicked unit
        if let Ok(mut stats) = stats_query.get_mut(panicked_e) {
            stats.morale = 0.0;
        }

        // Find the panicked unit's position and faction
        let panicked_data = unit_positions.iter().find(|(e, ..)| *e == panicked_e);
        let (_, panicked_faction, panicked_pos) = match panicked_data {
            Some(d) => *d,
            None => continue,
        };

        // Propagate panic to nearby allies
        let propagation_radius = config.panic_propagation_radius;
        for (ally_e, ally_faction, ally_pos) in &unit_positions {
            if *ally_e == panicked_e {
                continue;
            }
            if *ally_faction != panicked_faction {
                continue;
            }
            let dist = panicked_pos.distance(*ally_pos);
            if dist <= propagation_radius
                && rand::thread_rng().gen_range(0.0..1.0) < config.panic_propagation_chance
            {
                commands.entity(*ally_e).insert(PanicState {
                    is_panicking: true,
                    duration: config.panic_base_duration * 0.5,
                    flee_speed_multiplier: config.panic_flee_speed_mult,
                });
            }
        }
    }
}
