mod commands;
mod engagement;
mod formation;
mod pathfinding;
mod selection;

use bevy::prelude::*;
use commands::*;
use engagement::*;
use formation::*;
use kurulus_core::prelude::*;
use pathfinding::{a_star, collect_obstacles, is_path_blocked, ObstacleMap};
use selection::*;

pub struct UnitCommandPlugin;

impl Plugin for UnitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObstacleMap>()
            .init_resource::<PathfindingGrid>()
            .init_resource::<SelectionState>()
            .init_resource::<EngagementConfig>()
            .add_systems(
                FixedUpdate,
                (
                    // ── Phase 1a: Input Processing (Selection) ─────────────
                    process_selection_system.in_set(SimulationSet::InputProcessing),
                    process_control_groups_system.in_set(SimulationSet::InputProcessing),
                    // ── Phase 1b: Input Processing (Core Commands) ──────────
                    process_attack_order_system.in_set(SimulationSet::InputProcessing),
                    process_attack_move_order_system.in_set(SimulationSet::InputProcessing),
                    process_stop_command_system.in_set(SimulationSet::InputProcessing),
                    process_hold_command_system.in_set(SimulationSet::InputProcessing),
                    process_retreat_command_system.in_set(SimulationSet::InputProcessing),
                    process_scatter_command_system.in_set(SimulationSet::InputProcessing),
                    process_formation_command_system.in_set(SimulationSet::InputProcessing),
                    process_stance_change_system.in_set(SimulationSet::InputProcessing),
                    // ── Phase D: Formation auto-form & type sync ──────────
                    auto_form_system.in_set(SimulationSet::InputProcessing),
                    sync_formation_type_system.in_set(SimulationSet::InputProcessing),
                    // ── Phase 1c: Command Queueing ────────────────────────
                    process_move_orders_system.in_set(SimulationSet::CommandQueueing),
                    // Phase C1: Engagement movement runs after move orders so
                    // it can set Destination for units approaching their target
                    engagement_movement_system.in_set(SimulationSet::CommandQueueing),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    // ── Phase 5: Attack Timers (target acquisition) ────────
                    // Phase C2: HoldPosition auto-target
                    hold_position_auto_target_system.in_set(SimulationSet::AttackTimers),
                    // Phase C3: Attack-move interrupt
                    attack_move_interrupt_system.in_set(SimulationSet::AttackTimers),
                    // Phase C4: Target priority / re-targeting
                    target_priority_system.in_set(SimulationSet::AttackTimers),
                    // Phase D: Formation breaking detection
                    formation_breaking_system.in_set(SimulationSet::AttackTimers),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    // Phase 3a: Build obstacle map from building positions
                    build_obstacle_map_system.in_set(SimulationSet::Pathfinding),
                    // Phase 3a: Build combined pathfinding grid (terrain + obstacles)
                    build_pathfinding_grid_system.in_set(SimulationSet::Pathfinding),
                    // Phase 3a: Calculate paths for units with a Destination but no Path
                    apply_paths_system.in_set(SimulationSet::Pathfinding),
                    // Phase 3a: Recalculate paths for units whose path is now blocked
                    recalculate_blocked_paths_system.in_set(SimulationSet::Pathfinding),
                    // Phase D: Formation slot & bonus assignment (before movement)
                    assign_formation_slots_system.in_set(SimulationSet::FormationMovement),
                    assign_formation_bonus_system.in_set(SimulationSet::FormationMovement),
                    // Phase 3b: Move formations
                    formation_movement_system.in_set(SimulationSet::FormationMovement),
                    // Phase D: Break scatter movement (overrides normal movement)
                    formation_break_movement_system.in_set(SimulationSet::FreeFormMovement),
                    // Phase 3c: Move non-formation units along their path
                    free_form_movement_system.in_set(SimulationSet::FreeFormMovement),
                    // Phase 3d: Snap continuous positions back to grid
                    grid_quantization_system.in_set(SimulationSet::GridQuantization),
                ),
            )
            .add_systems(
                FixedUpdate,
                // Dead entity cleanup from selection (runs every tick but cheap)
                (
                    cleanup_dead_selection_system,
                    // Phase D: Formation component cleanup
                    clean_formation_bonus_system,
                    clean_formation_slots_system,
                )
                    .in_set(SimulationSet::ComponentCleanup),
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

// ─── Pathfinding grid (terrain + obstacles) ────────────────────────────────

/// Builds the combined `PathfindingGrid` from terrain data and building
/// obstacles.  This runs before `apply_paths_system` so A* can account for
/// terrain movement costs when calculating paths.
fn build_pathfinding_grid_system(
    terrain: Res<TerrainGrid>,
    obstacles: Res<ObstacleMap>,
    mut grid: ResMut<PathfindingGrid>,
) {
    *grid = PathfindingGrid::build(&terrain, &obstacles.obstacles);
}

// ─── Pathfinding ──────────────────────────────────────────────────────────

/// Bounds constant used for pathfinding searches (matches Mission 1 map).
const PATHFINDING_BOUNDS: Option<(IVec2, IVec2)> =
    Some((IVec2::new(-100, -100), IVec2::new(100, 100)));

/// For each entity that has a `Destination` but no `Path`, calculate a
/// new A* path and insert it via Commands.
///
/// Uses the combined `PathfindingGrid` for terrain-aware routing.
fn apply_paths_system(
    mut commands: Commands,
    grid: Res<PathfindingGrid>,
    positions: Query<&WorldPosition>,
    to_pathfind: Query<(Entity, &Destination), (With<Unit>, Without<Path>, Without<Dead>)>,
) {
    for (entity, destination) in to_pathfind.iter() {
        let start_pos = match positions.get(entity) {
            Ok(p) => p.grid.0,
            Err(_) => continue,
        };
        let goal = destination.target.0;

        if let Some(waypoints) = a_star(start_pos, goal, &*grid, PATHFINDING_BOUNDS) {
            let grid_waypoints: Vec<GridPos> =
                waypoints.into_iter().map(|iv| GridPos(iv)).collect();
            commands.entity(entity).insert(Path::new(grid_waypoints));
        }
    }
}

/// Recalculates paths for units whose current path is blocked by new
/// obstacles (e.g., a building was constructed mid-march, or terrain
/// changed). Only recalculates if the forward portion of the path
/// (from current_index onward) is blocked.
fn recalculate_blocked_paths_system(
    mut commands: Commands,
    grid: Res<PathfindingGrid>,
    positions: Query<&WorldPosition>,
    to_recalc: Query<(Entity, &Destination, &Path), (With<Unit>, Without<Dead>)>,
) {
    for (entity, destination, path) in to_recalc.iter() {
        // Only recalculate if the remaining path is blocked
        if !is_path_blocked(&path.waypoints, path.current_index, &grid) {
            continue;
        }

        let start_pos = match positions.get(entity) {
            Ok(p) => p.grid.0,
            Err(_) => continue,
        };
        let goal = destination.target.0;

        if let Some(waypoints) = a_star(start_pos, goal, &*grid, PATHFINDING_BOUNDS) {
            let grid_waypoints: Vec<GridPos> =
                waypoints.into_iter().map(|iv| GridPos(iv)).collect();
            commands.entity(entity).insert(Path::new(grid_waypoints));
        }
    }
}

// ─── Formation movement ────────────────────────────────────────────────────

/// Moves units in formation relative to their leader.
/// Uses `FormationSlot` if present (computed by Phase D's formation
/// positioning system), otherwise defaults to following the leader
/// with basic spacing.
fn formation_movement_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    leaders: Query<&WorldPosition, Without<FormationFollower>>,
    mut followers: Query<
        (
            Entity,
            &mut WorldPosition,
            &Formation,
            Option<&FormationSlot>,
            &Stats,
        ),
        With<FormationFollower>,
    >,
) {
    for (_follower_e, mut follower_pos, formation, slot, stats) in followers.iter_mut() {
        if let Ok(leader_pos) = leaders.get(formation.leader) {
            // If a FormationSlot has been computed (Phase D+), use its offset.
            // Otherwise fall back to a simple trailing offset behind the leader.
            let base_offset = match slot {
                Some(s) => s.offset,
                None => Vec2::new(0.0, -1.0), // 1 cell behind leader
            };
            let target = leader_pos.world_2d.0 + base_offset;
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
