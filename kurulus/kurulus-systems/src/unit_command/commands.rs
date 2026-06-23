use bevy::prelude::*;
use kurulus_core::prelude::*;
use rand::Rng;

// ─── Attack Order ───────────────────────────────────────────────────────────

/// Reads `AttackOrder` events and sets/updates the `AttackTarget` component
/// on the issuing entity.
pub fn process_attack_order_system(
    mut commands: Commands,
    mut attack_events: EventReader<AttackOrder>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for order in attack_events.read() {
        if units.get(order.entity).is_err() {
            continue;
        }
        commands.entity(order.entity).insert(AttackTarget {
            target: order.target,
            aggressive: order.aggressive,
        });
    }
}

// ─── Attack-Move Order ──────────────────────────────────────────────────────

/// Reads `AttackMoveOrder` events, sets a `Destination` for movement, and
/// flags the entity as attack-moving so the engagement system can interrupt
/// movement when an enemy is spotted.
pub fn process_attack_move_order_system(
    mut commands: Commands,
    mut move_events: EventReader<AttackMoveOrder>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for order in move_events.read() {
        if units.get(order.entity).is_err() {
            continue;
        }
        commands.entity(order.entity).insert(Destination {
            target: order.target,
        });
        commands.entity(order.entity).insert(AttackTargetMarker);
    }
}

// ─── Stop Command ───────────────────────────────────────────────────────────

/// Removes movement- and attack-related components from the entity,
/// effectively halting it in place.
pub fn process_stop_command_system(
    mut commands: Commands,
    mut stop_events: EventReader<StopCommand>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for event in stop_events.read() {
        if units.get(event.entity).is_err() {
            continue;
        }
        let e = event.entity;
        commands.entity(e).remove::<Destination>();
        commands.entity(e).remove::<Path>();
        commands.entity(e).remove::<AttackTarget>();
        commands.entity(e).remove::<AttackTargetMarker>();
        commands.entity(e).remove::<HoldPosition>();
    }
}

// ─── Hold Command ───────────────────────────────────────────────────────────

/// Clears movement and sets `HoldPosition` so the unit stays in place but
/// auto-engages enemies within range.
pub fn process_hold_command_system(
    mut commands: Commands,
    mut hold_events: EventReader<HoldCommand>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for event in hold_events.read() {
        if units.get(event.entity).is_err() {
            continue;
        }
        let e = event.entity;
        commands.entity(e).remove::<Destination>();
        commands.entity(e).remove::<Path>();
        commands.entity(e).insert(HoldPosition);
    }
}

// ─── Retreat Command ────────────────────────────────────────────────────────

/// Sets the unit to Retreat stance, clears attack target, and inserts a
/// `Destination` toward the retreat direction or away from nearby enemies.
///
/// Uses a separate `&Unit` query to preserve existing archetype/formation
/// while updating the stance.
pub fn process_retreat_command_system(
    mut commands: Commands,
    mut retreat_events: EventReader<RetreatCommand>,
    units: Query<(Entity, &WorldPosition), (With<Unit>, Without<Dead>)>,
    unit_data: Query<&Unit>,
    enemies: Query<&WorldPosition, (With<Unit>, Without<Dead>)>,
) {
    for event in retreat_events.read() {
        let Ok((entity, pos)) = units.get(event.entity) else {
            continue;
        };

        // Preserve existing archetype/formation, update stance
        if let Ok(old) = unit_data.get(event.entity) {
            commands.entity(entity).insert(Unit {
                stance: CombatStance::Retreat,
                ..old.clone()
            });
        }

        // Determine retreat destination
        let retreat_target = if let Some(dir) = event.direction {
            GridPos(IVec2::new(
                (pos.grid.0.x as f32 + dir.x * 10.0).round() as i32,
                (pos.grid.0.y as f32 + dir.y * 10.0).round() as i32,
            ))
        } else {
            // Default: retreat away from nearest visible enemy
            let away = retreat_direction_from_enemies(pos, &enemies);
            GridPos(IVec2::new(
                (pos.grid.0.x as f32 + away.x * 10.0).round() as i32,
                (pos.grid.0.y as f32 + away.y * 10.0).round() as i32,
            ))
        };

        commands.entity(entity).remove::<AttackTarget>();
        commands.entity(entity).remove::<AttackTargetMarker>();
        commands.entity(entity).insert(Destination {
            target: retreat_target,
        });
    }
}

// ─── Scatter Command ────────────────────────────────────────────────────────

/// Moves the entity a short random distance from its current position.
/// Used to break formations when under fire.
pub fn process_scatter_command_system(
    mut commands: Commands,
    mut scatter_events: EventReader<ScatterCommand>,
    units: Query<(Entity, &WorldPosition), (With<Unit>, Without<Dead>)>,
) {
    let mut rng = rand::thread_rng();
    for event in scatter_events.read() {
        let Ok((entity, pos)) = units.get(event.entity) else {
            continue;
        };
        // Random offset within 2–4 cells
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let distance = rng.gen_range(2.0..=4.0);
        let target = GridPos(IVec2::new(
            (pos.grid.0.x as f32 + angle.cos() * distance).round() as i32,
            (pos.grid.0.y as f32 + angle.sin() * distance).round() as i32,
        ));
        commands.entity(entity).remove::<AttackTarget>();
        commands.entity(entity).remove::<AttackTargetMarker>();
        commands.entity(entity).insert(Destination { target });
    }
}

// ─── Formation Command ──────────────────────────────────────────────────────

/// Updates the `Unit.formation` field based on the command, preserving other
/// Unit fields.
pub fn process_formation_command_system(
    mut commands: Commands,
    mut formation_events: EventReader<FormationCommand>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
    unit_data: Query<&Unit>,
) {
    for event in formation_events.read() {
        if units.get(event.entity).is_err() {
            continue;
        }
        if let Ok(old) = unit_data.get(event.entity) {
            commands.entity(event.entity).insert(Unit {
                formation: event.formation,
                ..old.clone()
            });
        }
    }
}

// ─── Stance Change Command ──────────────────────────────────────────────────

/// Updates the `Unit.stance` field based on the command, preserving other
/// Unit fields.
pub fn process_stance_change_system(
    mut commands: Commands,
    mut stance_events: EventReader<StanceChangeCommand>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
    unit_data: Query<&Unit>,
) {
    for event in stance_events.read() {
        if units.get(event.entity).is_err() {
            continue;
        }
        if let Ok(old) = unit_data.get(event.entity) {
            commands.entity(event.entity).insert(Unit {
                stance: event.stance,
                ..old.clone()
            });
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Compute a direction vector pointing away from the nearest enemy.
fn retreat_direction_from_enemies(
    pos: &WorldPosition,
    enemies: &Query<&WorldPosition, (With<Unit>, Without<Dead>)>,
) -> Vec2 {
    let mut away = Vec2::ZERO;
    let my_pos = pos.world_2d.0;
    for enemy_pos in enemies.iter() {
        let diff = my_pos - enemy_pos.world_2d.0;
        let dist = diff.length();
        if dist > 0.0 && dist < 20.0 {
            // Closer enemies contribute more to the direction
            away += diff / (dist * dist);
        }
    }
    if away.length_squared() > 0.0 {
        away.normalize()
    } else {
        Vec2::new(0.0, 1.0) // default: retreat "south" (toward allied lines)
    }
}

// AttackTargetMarker is defined in kurulus-core/src/components/unit.rs
