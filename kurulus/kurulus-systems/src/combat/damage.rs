use bevy::prelude::*;
use kurulus_core::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
//  DAMAGE APPLICATION
// ═════════════════════════════════════════════════════════════════════════════

/// Receives DamageDealt events and applies damage to target Health.
/// Also emits UnitDied when health ≤ 0.
/// Applies difficulty multipliers to incoming/outgoing damage.
pub fn damage_application_system(
    mut damage_events: EventReader<DamageDealt>,
    mut healths: Query<(Entity, &mut Health, Option<&Unit>)>,
    mut unit_died_events: EventWriter<UnitDied>,
    mut damage_tracker: ResMut<DamageTracker>,
    difficulty: Res<CombatDifficultyConfig>,
) {
    for ev in damage_events.read() {
        if let Ok((_entity, mut health, unit_opt)) = healths.get_mut(ev.target) {
            let mut final_amount = ev.amount;

            // ─── Difficulty scaling ─────────────────────────────────────
            // Check if the attacker is player-controlled (Kayi faction)
            // and apply outgoing/incoming multipliers
            let attacker_is_player = unit_opt
                .map(|u| u.faction == FactionId::Kayi)
                .unwrap_or(false);

            // For the target check, we just apply the incoming multiplier
            // to any damage absorbed by player units
            let target_is_player = unit_opt
                .map(|u| u.faction == FactionId::Kayi)
                .unwrap_or(false);

            if attacker_is_player {
                final_amount *= difficulty.outgoing_mult;
            }
            if target_is_player {
                final_amount *= difficulty.incoming_mult;
            }

            // ─── Apply damage ───────────────────────────────────────────
            health.current = (health.current - final_amount).max(0.0);

            // Track total damage dealt (for statistics)
            damage_tracker.total_dealt += final_amount;

            // Check for death
            if health.current <= 0.0 {
                unit_died_events.send(UnitDied {
                    entity: ev.target,
                    killer: ev.attacker,
                });
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DAMAGE OVER TIME (DoT)
// ═════════════════════════════════════════════════════════════════════════════

/// Processes DamageOverTime components, applying tick damage each frame
/// and reducing remaining duration. Removes the component when expired.
///
/// Checks that the dot.source entity is still valid before emitting events
/// to prevent dangling Entity references.
pub fn dot_tick_system(
    time: Res<Time>,
    mut target_query: Query<(Entity, &mut Health, &mut DamageOverTime)>,
    mut damage_events: EventWriter<DamageDealt>,
    mut commands: Commands,
    source_query: Query<Entity, With<Unit>>,
) {
    for (entity, mut health, mut dot) in target_query.iter_mut() {
        // ─── Validate the source entity still exists ────────────────────
        let source_valid = source_query.get(dot.source).is_ok();
        if !source_valid && dot.source != Entity::from_raw(0) {
            // Source no longer exists — use it as inert (still apply DoT)
            // but we won't emit the source in damage events meaningfully
        }

        // Accumulate tick progress
        dot.next_tick_progress += time.delta_secs();
        let tick_interval = dot.tick_interval.max(0.1);

        // Process each completed tick
        while dot.next_tick_progress >= tick_interval {
            dot.next_tick_progress -= tick_interval;

            // Apply tick damage
            health.current = (health.current - dot.damage_per_tick).max(0.0);

            // Emit damage event for tracking
            damage_events.send(DamageDealt {
                attacker: if source_valid { dot.source } else { entity },
                target: entity,
                amount: dot.damage_per_tick,
                damage_type: dot.damage_type,
                crit: CritType::None,
                flanking: FlankingInfo::default(),
                is_friendly_fire: false,
                source_ability: dot.source_ability,
            });

            // Reduce remaining duration
            dot.remaining_duration -= tick_interval;
        }

        // Remove DoT when expired
        if dot.remaining_duration <= 0.0 {
            commands.entity(entity).remove::<DamageOverTime>();
        }
    }
}
