use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Engagement Movement ───────────────────────────────────────────────────

/// Moves units toward their `AttackTarget` until they are within attack range.
///
/// Once in range, the existing combat system (`melee_combat_system`) picks up
/// the attacker and resolves damage automatically.
pub fn engagement_movement_system(
    mut commands: Commands,
    attackers: Query<
        (Entity, &AttackTarget, &WorldPosition, &Stats, &Unit),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<&WorldPosition, (With<Unit>, Without<Dead>)>,
) {
    for (entity, attack_target, pos, stats, _unit) in attackers.iter() {
        // If target is dead/gone, skip — let target_priority_system handle re-targeting
        let Ok(target_pos) = targets.get(attack_target.target) else {
            continue;
        };

        let distance = pos.world_2d.0.distance(target_pos.world_2d.0);

        // If we're already in range, let combat handle it — no movement needed
        if distance <= stats.attack_range {
            continue;
        }

        // Move toward the target
        commands.entity(entity).insert(Destination {
            target: target_pos.grid,
        });
    }
}

// ─── Hold Position Auto-Target ─────────────────────────────────────────────

/// Scans for enemies within attack range for units with `HoldPosition`.
/// Sets `AttackTarget` to the nearest valid enemy.
pub fn hold_position_auto_target_system(
    mut commands: Commands,
    holders: Query<
        (Entity, &WorldPosition, &Stats, &Unit),
        (With<Unit>, With<HoldPosition>, Without<Dead>),
    >,
    potential_targets: Query<(Entity, &WorldPosition, &Health, &Unit), Without<Dead>>,
) {
    for (entity, pos, stats, my_unit) in holders.iter() {
        // Skip if already has a target (the target system re-evaluates)
        // We let the combat system handle in-range enemies; this system only
        // sets the *initial* target for Hold units.
        // For re-targeting, see `target_priority_system`.

        let mut best_target: Option<Entity> = None;
        let mut best_dist = f32::MAX;

        for (target_e, target_pos, _health, target_unit) in potential_targets.iter() {
            // No friendly fire
            if my_unit.faction == target_unit.faction || entity == target_e {
                continue;
            }

            let dist = pos.world_2d.0.distance(target_pos.world_2d.0);

            // Only consider targets within attack range
            if dist <= stats.attack_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(target_e);
            }
        }

        if let Some(target) = best_target {
            commands.entity(entity).insert(AttackTarget {
                target,
                aggressive: true,
            });
        }
    }
}

// ─── Attack-Move Interrupt ─────────────────────────────────────────────────

/// Units with `AttackTargetMarker` scan for enemies within their vision range.
/// When an enemy is detected, the marker is removed, movement is stopped, and
/// `AttackTarget` is set so the unit engages.
pub fn attack_move_interrupt_system(
    mut commands: Commands,
    movers: Query<
        (Entity, &WorldPosition, &Stats, &Unit),
        (
            With<Unit>,
            With<AttackTargetMarker>,
            Without<Dead>,
        ),
    >,
    potential_targets: Query<(Entity, &WorldPosition, &Unit), Without<Dead>>,
) {
    for (entity, pos, stats, my_unit) in movers.iter() {
        let mut best_target: Option<Entity> = None;
        let mut best_dist = f32::MAX;

        // Use vision_range for detection radius during attack-move
        let detect_range = stats.vision_range;

        for (target_e, target_pos, target_unit) in potential_targets.iter() {
            if my_unit.faction == target_unit.faction || entity == target_e {
                continue;
            }

            let dist = pos.world_2d.0.distance(target_pos.world_2d.0);
            if dist <= detect_range && dist < best_dist {
                best_dist = dist;
                best_target = Some(target_e);
            }
        }

        if let Some(target) = best_target {
            // Interrupt movement: stop and engage
            commands.entity(entity).remove::<AttackTargetMarker>();
            commands.entity(entity).remove::<Destination>();
            commands.entity(entity).remove::<Path>();
            commands.entity(entity).insert(AttackTarget {
                target,
                aggressive: true,
            });
        }
    }
}

// ─── Target Priority ───────────────────────────────────────────────────────

/// Evaluates all units with `AttackTarget` and optionally re-targets based on
/// priority rules (nearest, lowest HP, highest threat).
///
/// Currently implements: **nearest-first** (matches the combat system's default).
/// Future: faction-agnostic priority config via `EngagementConfig`.
pub fn target_priority_system(
    mut commands: Commands,
    attackers: Query<
        (Entity, &AttackTarget, &WorldPosition, &Stats, &Unit),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<(Entity, &WorldPosition, &Health, &Unit), Without<Dead>>,
) {
    for (entity, current_target, pos, stats, my_unit) in attackers.iter() {
        // Verify the current target still exists
        let target_alive = targets.get(current_target.target).is_ok();

        if target_alive {
            // Check if target is still in a reasonable range (3x attack_range)
            if let Ok((_target_e, target_pos, _health, _target_unit)) = targets.get(current_target.target) {
                let dist = pos.world_2d.0.distance(target_pos.world_2d.0);
                if dist <= stats.attack_range * 3.0 {
                    // Current target is still valid — keep it
                    continue;
                }
            }
        }

        // Current target is dead or too far — find a new one
        let mut best_target: Option<Entity> = None;
        let mut best_dist = f32::MAX;
        let mut best_hp = f32::MAX;

        for (target_e, target_pos, health, target_unit) in targets.iter() {
            if my_unit.faction == target_unit.faction || entity == target_e {
                continue;
            }

            let dist = pos.world_2d.0.distance(target_pos.world_2d.0);

            // Priority: nearest, then lowest HP as tiebreaker
            if dist < best_dist || (dist == best_dist && health.current < best_hp) {
                best_dist = dist;
                best_hp = health.current;
                best_target = Some(target_e);
            }
        }

        if let Some(target) = best_target {
            commands.entity(entity).insert(AttackTarget {
                target,
                aggressive: current_target.aggressive,
            });
        } else {
            // No valid targets — clear the attack order
            commands.entity(entity).remove::<AttackTarget>();
        }
    }
}
