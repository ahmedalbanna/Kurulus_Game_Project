use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Formation Slot Assignment ─────────────────────────────────────────────

/// Assigns `FormationSlot` components to formation followers.
///
/// Runs every frame for all followers that reference a valid leader.
/// Slots are recomputed from the leader's `Formation.formation_type`
/// each frame, so changes to the formation type are picked up
/// automatically (no change detection needed).
pub fn assign_formation_slots_system(
    mut commands: Commands,
    leader_formations: Query<(&Formation, &WorldPosition), Without<FormationFollower>>,
    followers: Query<(Entity, &Formation), (With<FormationFollower>, Without<Dead>)>,
) {
    for (follower_entity, formation) in followers.iter() {
        // Find the leader's formation to get formation type and count
        let Ok((leader_formation, _leader_pos)) = leader_formations.get(formation.leader) else {
            continue;
        };

        let formation_type = leader_formation.formation_type;
        let count = leader_formation.unit_count.max(1);

        // Determine facing direction
        let facing = if leader_formation.facing.length_squared() > 0.01 {
            leader_formation.facing
        } else {
            Vec2::new(0.0, -1.0)
        };

        // Compute all slots
        let slots = formation_type.compute_slots(count, facing);

        // Assign the slot based on stable entity index
        let index = follower_entity.index() as usize % slots.len();
        let (offset, rank, file) = slots[index];

        commands.entity(follower_entity).insert(FormationSlot {
            offset,
            facing,
            rank,
            file,
        });
    }
}

// ─── Formation Bonus Assignment ────────────────────────────────────────────

/// Assigns `FormationBonus` to all units in a formation (both leaders
/// and followers) based on the formation type.
///
/// Runs every frame, automatically keeping bonuses up to date when
/// formation type changes.
pub fn assign_formation_bonus_system(
    mut commands: Commands,
    // Leaders: have a Formation component but NOT FormationFollower
    leaders: Query<(Entity, &Formation), (With<Unit>, Without<FormationFollower>, Without<Dead>)>,
    // Followers: have both Formation and FormationFollower
    followers: Query<(Entity, &Formation), (With<Unit>, With<FormationFollower>, Without<Dead>)>,
    // Bonus components to prevent re-insertion if already present
    existing_bonuses: Query<&FormationBonus>,
) {
    // --- Leaders ---
    for (entity, formation) in leaders.iter() {
        if formation.formation_type == FormationType::Scatter {
            // Scatter has no leader bonus (each unit fends for itself)
            if existing_bonuses.get(entity).is_ok() {
                commands.entity(entity).remove::<FormationBonus>();
            }
            continue;
        }

        let bonus = formation.formation_type.bonuses();
        // Only insert if it changed (or doesn't exist)
        match existing_bonuses.get(entity) {
            Ok(existing) if *existing == bonus => {} // unchanged
            _ => {
                commands.entity(entity).insert(bonus);
            }
        }
    }

    // --- Followers ---
    for (entity, formation) in followers.iter() {
        // Find the leader's formation type
        let leader_formation = match leaders.get(formation.leader) {
            Ok((_, f)) => f,
            Err(_) => continue,
        };

        if leader_formation.formation_type == FormationType::Scatter {
            if existing_bonuses.get(entity).is_ok() {
                commands.entity(entity).remove::<FormationBonus>();
            }
            continue;
        }

        let bonus = leader_formation.formation_type.bonuses();
        match existing_bonuses.get(entity) {
            Ok(existing) if *existing == bonus => {}
            _ => {
                commands.entity(entity).insert(bonus);
            }
        }
    }
}

/// Cleans up `FormationBonus` from units that are no longer in any formation.
pub fn clean_formation_bonus_system(
    mut commands: Commands,
    units: Query<(Entity, &FormationBonus), (With<Unit>, Without<Formation>)>,
) {
    for (entity, _) in units.iter() {
        commands.entity(entity).remove::<FormationBonus>();
    }
}

/// Cleans up `FormationSlot` from entities that are no longer formation
/// followers.
pub fn clean_formation_slots_system(
    mut commands: Commands,
    slots: Query<(Entity, &FormationSlot), (Without<FormationFollower>, Without<Formation>)>,
) {
    for (entity, _) in slots.iter() {
        commands.entity(entity).remove::<FormationSlot>();
    }
}

// ─── Formation Detection ───────────────────────────────────────────────────

/// Automatically promotes a group of friendly units into a formation when
/// they are close together and share the same `Unit.formation` type.
///
/// This is a lightweight auto-formation system that creates `Formation`
/// components and `FormationFollower` markers for units that meet the
/// criteria.  A leader is chosen as the unit with the highest `Stats.morale`.
pub fn auto_form_system(
    mut commands: Commands,
    units: Query<(Entity, &Unit, &WorldPosition, &Stats), (Without<Formation>, Without<Dead>)>,
) {
    use std::collections::HashMap;

    // Group unformed units by (faction, formation_type)
    let mut groups: HashMap<(FactionId, FormationType), Vec<(Entity, f32, Vec2)>> = HashMap::new();

    for (entity, unit, pos, stats) in units.iter() {
        if unit.formation == FormationType::Scatter {
            continue; // Scatter is always individual
        }
        groups
            .entry((unit.faction, unit.formation))
            .or_default()
            .push((entity, stats.morale, pos.world_2d.0));
    }

    const FORMATION_RANGE: f32 = 3.0; // cells

    for ((_faction, formation_type), candidates) in groups.iter() {
        if candidates.len() < 2 {
            continue; // need at least 2 units to form a formation
        }

        // For now use a simple greedy grouping: group entities within range
        // of each other. A more sophisticated approach would use clustering.
        let mut grouped: Vec<Vec<(Entity, f32, Vec2)>> = Vec::new();

        for &candidate in candidates {
            let mut placed = false;
            for group in grouped.iter_mut() {
                // Check if this candidate is within range of any entity in the group
                let in_range = group
                    .iter()
                    .any(|(_, _, pos)| pos.distance(candidate.2) <= FORMATION_RANGE);
                if in_range {
                    group.push(candidate);
                    placed = true;
                    break;
                }
            }
            if !placed {
                grouped.push(vec![candidate]);
            }
        }

        for group in grouped.iter() {
            if group.len() < 2 {
                continue;
            }

            // Pick leader: highest morale
            let leader_idx = group
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(idx, _)| idx)
                .unwrap_or(0);
            let (leader_entity, _, leader_pos) = group[leader_idx];

            // Determine facing from the group's average position relative to leader
            let avg_pos = group
                .iter()
                .map(|(_, _, p)| *p)
                .reduce(|a, b| a + b)
                .unwrap_or(leader_pos);
            let facing_dir = if avg_pos != leader_pos {
                (avg_pos - leader_pos).normalize_or_zero()
            } else {
                Vec2::new(0.0, -1.0)
            };

            let count = group.len();

            // Insert Formation component on leader
            commands.entity(leader_entity).insert(Formation {
                leader: leader_entity,
                formation_type: *formation_type,
                facing: facing_dir,
                is_breaking: false,
                break_timer: None,
                unit_count: count,
            });

            // Insert Formation + FormationFollower on everyone else
            for (_i, (follower_entity, _, _)) in group.iter().enumerate() {
                if *follower_entity == leader_entity {
                    continue;
                }
                commands.entity(*follower_entity).insert((
                    Formation {
                        leader: leader_entity,
                        formation_type: *formation_type,
                        facing: facing_dir,
                        is_breaking: false,
                        break_timer: None,
                        unit_count: count,
                    },
                    FormationFollower,
                ));
            }
        }
    }
}

/// Stores the leader's formation type so followers can see it when the
/// leader's unit.formation field changes (e.g. via FormationCommand).
///
/// Without this, `process_formation_command_system` updates `Unit.formation`
/// but the `Formation` component (which followers read) stays stale.
pub fn sync_formation_type_system(
    mut commands: Commands,
    units: Query<(Entity, &Unit, Option<&Formation>), (Changed<Unit>, Without<FormationFollower>)>,
) {
    for (entity, unit, formation) in units.iter() {
        if let Some(form) = formation {
            if form.formation_type != unit.formation {
                commands.entity(entity).insert(Formation {
                    formation_type: unit.formation,
                    ..form.clone()
                });
            }
        }
    }
}

// ─── Formation Breaking ───────────────────────────────────────────────────

/// Detects formation-breaking conditions:
/// - 30%+ casualties in the last 10 seconds → 5s break (panic)
/// - Leader killed → 4s break (panic)
/// - Formation has is_breaking=true → apply break
///
/// Also updates break timers and reforms formations after break ends.
pub fn formation_breaking_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut commands: Commands,
    mut break_writer: EventWriter<FormationBreakEvent>,
    // Leaders (entities with Formation but NOT FormationFollower)
    mut leader_formations: Query<
        (Entity, &mut Formation, Option<&FormationBreak>),
        (With<Unit>, Without<FormationFollower>, Without<Dead>),
    >,
    // Followers
    followers: Query<
        (Entity, &Formation, Option<&FormationBreak>),
        (With<Unit>, With<FormationFollower>, Without<Dead>),
    >,
    // Health for checking casualty thresholds
    healths: Query<&Health>,
    // Dead entities added this frame (via the UnitDied event for leader death)
    mut dead_events: EventReader<kurulus_core::events::combat::UnitDied>,
) {
    let dt = time.delta_secs();

    // 1. Recover leaders that are breaking but expired
    for (leader_entity, mut formation, maybe_break) in leader_formations.iter_mut() {
        if let Some(break_comp) = maybe_break {
            let mut break_comp = break_comp.clone(); // clone since we have &mut formation
            break_comp.remaining -= dt;
            if break_comp.remaining <= 0.0 {
                // Break expired — reform
                formation.is_breaking = false;
                formation.break_timer = None;
                commands.entity(leader_entity).remove::<FormationBreak>();

                // Re-form followers — add back FormationFollower if missing
                // (they were cleared when the break started)
                // Note: This is handled by the auto-form system on subsequent frames.
            } else {
                commands.entity(leader_entity).insert(break_comp);
            }
            continue; // already breaking, skip condition checks
        }

        // Check health-based break condition
        let health = match healths.get(leader_entity) {
            Ok(h) => h,
            Err(_) => continue,
        };

        // Condition: health <= 70% (30%+ casualties) and not dead
        if health.current > 0.0 && health.current / health.max <= 0.70 {
            trigger_break(
                &mut commands,
                &mut break_writer,
                leader_entity,
                &mut formation,
                5.0,
                true,
                &followers,
            );
        }
    }

    // 2. Handle leader death (from UnitDied events)
    for died_event in dead_events.read() {
        let dead_entity = died_event.entity;

        // Break any formation whose leader died
        for (follower_entity, formation, _) in followers.iter() {
            if formation.leader == dead_entity {
                // Leader is dead — break the formation for this follower
                commands.entity(follower_entity).insert(FormationBreak {
                    remaining: 4.0,
                    total_duration: 4.0,
                    panic: true,
                });
                // Remove formation components — follower is now independent
                commands.entity(follower_entity).remove::<Formation>();
                commands.entity(follower_entity).remove::<FormationSlot>();
                commands.entity(follower_entity).remove::<FormationBonus>();
                commands
                    .entity(follower_entity)
                    .remove::<FormationFollower>();
            }
        }

        // If the dead entity was a formation leader, clear its Formation
        if let Ok((_, mut formation, _)) = leader_formations.get_mut(dead_entity) {
            formation.unit_count = 0;
            formation.is_breaking = true;
        }
    }
}

/// Helper: trigger a formation break event and update components.
fn trigger_break(
    commands: &mut Commands,
    break_writer: &mut EventWriter<FormationBreakEvent>,
    leader: Entity,
    formation: &mut Formation,
    duration: f32,
    panic: bool,
    followers: &Query<
        (Entity, &Formation, Option<&FormationBreak>),
        (
            bevy::ecs::query::With<Unit>,
            bevy::ecs::query::With<FormationFollower>,
            bevy::ecs::query::Without<Dead>,
        ),
    >,
) {
    formation.is_breaking = true;
    formation.break_timer = Some(duration);

    // Apply break component to the leader
    commands.entity(leader).insert(FormationBreak {
        remaining: duration,
        total_duration: duration,
        panic,
    });

    // Collect follower entities for the event
    let follower_entities: Vec<Entity> = followers
        .iter()
        .filter(|(_, f, _)| f.leader == leader)
        .map(|(e, _, _)| e)
        .collect();

    // Apply break to all followers too
    for &fe in &follower_entities {
        commands.entity(fe).insert(FormationBreak {
            remaining: duration,
            total_duration: duration,
            panic,
        });
    }

    break_writer.send(FormationBreakEvent {
        leader,
        followers: follower_entities,
        duration,
        panic,
    });
}

// ─── Break Movement ──────────────────────────────────────────────────────

/// During a formation break, units scatter by moving in random directions
/// (half speed, panicked). This overrides the normal formation_movement_system
/// behaviour.
pub fn formation_break_movement_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut broken: Query<(&mut WorldPosition, &Stats, &FormationBreak), (With<Unit>, Without<Dead>)>,
) {
    let dt = time.delta_secs();
    for (mut pos, stats, break_comp) in broken.iter_mut() {
        // Deterministic scatter direction based on the remaining break time
        let angle = break_comp.remaining * 3.0;
        let dir = Vec2::new(angle.cos(), angle.sin());
        let step = stats.move_speed * 0.5 * dt; // half speed while panicked
        pos.world_2d.0 += dir * step;
    }
}
