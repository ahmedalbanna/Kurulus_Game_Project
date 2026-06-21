mod pathfinding;

use bevy::prelude::*;
use kurulus_core::prelude::*;
use pathfinding::{a_star, collect_obstacles, ObstacleMap};

pub struct UnitCommandPlugin;

impl Plugin for UnitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObstacleMap>()
            .add_systems(
                FixedUpdate,
                // Phase 1b: Respond to move orders by inserting Destination
                process_move_orders_system.in_set(SimulationSet::CommandQueueing),
            )
            .add_systems(
                FixedUpdate,
                (
                    // Phase 3a: Build obstacle map
                    build_obstacle_map_system.in_set(SimulationSet::Pathfinding),
                    // Phase 3a: Calculate paths for units with a Destination but no Path
                    apply_paths_system.in_set(SimulationSet::Pathfinding),
                    // Phase 3b: Move formations
                    formation_movement_system.in_set(SimulationSet::FormationMovement),
                    // Phase 3c: Move non-formation units along their path
                    free_form_movement_system.in_set(SimulationSet::FreeFormMovement),
                    // Phase 3d: Snap continuous positions back to grid
                    grid_quantization_system.in_set(SimulationSet::GridQuantization),
                ),
            );
    }
}

// ─── Move order processing ─────────────────────────────────────────────────

/// Listens for `MoveOrder` events and inserts a `Destination` component on
/// the target entity, which the pathfinding system then picks up.
fn process_move_orders_system(
    mut commands: Commands,
    mut move_events: EventReader<MoveOrder>,
    units: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for order in move_events.read() {
        if units.get(order.entity).is_ok() {
            commands.entity(order.entity).insert(Destination {
                target: order.target,
            });
        }
    }
}

// ─── Obstacle map ──────────────────────────────────────────────────────────

/// Rebuilds the obstacle map each frame so pathfinding always has fresh data.
fn build_obstacle_map_system(
    buildings: Query<&WorldPosition, With<Building>>,
    mut map: ResMut<ObstacleMap>,
) {
    map.obstacles = collect_obstacles(buildings);
}

// ─── Pathfinding ──────────────────────────────────────────────────────────

/// For each entity that has a `Destination` but no `Path`, calculate a
/// new A* path and insert it via Commands.
fn apply_paths_system(
    mut commands: Commands,
    map: Res<ObstacleMap>,
    positions: Query<&WorldPosition>,
    to_pathfind: Query<
        (Entity, &Destination),
        (With<Unit>, Without<Path>, Without<Dead>),
    >,
) {
    for (entity, destination) in to_pathfind.iter() {
        let start_pos = match positions.get(entity) {
            Ok(p) => p.grid.0,
            Err(_) => continue,
        };
        let goal = destination.target.0;

        // Reasonable bounds for the Mission 1 map
        let bounds = Some((IVec2::splat(-100), IVec2::splat(100)));
        if let Some(waypoints) = a_star(start_pos, goal, &map.obstacles, bounds) {
            let grid_waypoints: Vec<GridPos> =
                waypoints.into_iter().map(|iv| GridPos(iv)).collect();
            commands.entity(entity).insert(Path::new(grid_waypoints));
        }
    }
}

// ─── Formation movement ────────────────────────────────────────────────────

/// Moves units in formation relative to their leader.
fn formation_movement_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    _move_events: EventReader<MoveOrder>,
    leaders: Query<(Entity, &WorldPosition), (With<Unit>, Without<FormationFollower>)>,
    mut followers: Query<
        (Entity, &mut WorldPosition, &Formation, &Stats),
        With<FormationFollower>,
    >,
) {
    for (_follower_e, mut follower_pos, formation, stats) in followers.iter_mut() {
        if let Ok((_leader_e, leader_pos)) = leaders.get(formation.leader) {
            let target = leader_pos.world_2d.0 + formation.offset;
            let current = follower_pos.world_2d.0;
            let direction = (target - current).normalize_or_zero();
            let distance = stats.move_speed * time.delta_secs();
            follower_pos.world_2d.0 += direction * distance;
        }
    }
}

// ─── Free-form movement along paths ────────────────────────────────────────

/// Moves units along their computed A* path waypoint-by-waypoint.
fn free_form_movement_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut units: Query<
        (&mut WorldPosition, &Stats, &mut Path),
        (With<Unit>, Without<FormationFollower>, Without<Dead>),
    >,
) {
    for (mut pos, stats, mut path) in units.iter_mut() {
        let Some(next_wp) = path.next_waypoint() else {
            continue; // path is exhausted
        };

        let target_pos = Vec2::new(
            next_wp.0.x as f32 * CELL_SIZE,
            next_wp.0.y as f32 * CELL_SIZE,
        );
        let current = pos.world_2d.0;
        let direction = (target_pos - current).normalize_or_zero();
        let step = stats.move_speed * time.delta_secs();

        let distance = current.distance(target_pos);
        if distance <= step {
            // Arrived at this waypoint; snap to it and advance
            pos.world_2d.0 = target_pos;
            path.advance();
        } else {
            pos.world_2d.0 += direction * step;
        }
    }
}

// ─── Grid quantization ─────────────────────────────────────────────────────

/// Quantizes continuous positions back to grid after movement.
fn grid_quantization_system(_time: Res<GameTime>, mut positions: Query<&mut WorldPosition>) {
    for mut pos in positions.iter_mut() {
        pos.quantize(CELL_SIZE);
    }
}
