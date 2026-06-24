use bevy::prelude::*;
use kurulus_core::prelude::*;

/// Plugin for flow‑field pathfinding — computes a single vector field per
/// destination that all units moving toward that destination share.
///
/// Flow fields are dramatically cheaper than per‑unit A* for groups:
/// - A*: O(n × A*) per destination
/// - Flow field: O(1 wavefront + n lookup) per destination
pub struct FlowFieldPlugin;

impl Plugin for FlowFieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlowFieldCache>().add_systems(
            FixedUpdate,
            (
                // Phase 3a: Detect group destinations and build flow fields
                build_flow_field_system.in_set(SimulationSet::Pathfinding),
                // Phase 3c: Move units along the flow field
                flow_field_movement_system.in_set(SimulationSet::FreeFormMovement),
                // Phase 3d: Clean up when destination reached
                cleanup_flow_field_system.in_set(SimulationSet::ComponentCleanup),
            ),
        );
    }
}

// ─── Constants ────────────────────────────────────────────────────────────

/// Flow fields are built for shared destinations only when at least this
/// many units are heading to the same cell.
const GROUP_SIZE_THRESHOLD: usize = 3;

/// Maximum cells to explore when building a flow field.
const MAX_FLOW_CELLS: usize = 20_000;

/// Search bounds (matches `PATHFINDING_BOUNDS` in unit_command/mod.rs).
const FLOW_BOUNDS: (IVec2, IVec2) = (IVec2::new(-100, -100), IVec2::new(100, 100));

// ─── Systems ──────────────────────────────────────────────────────────────

/// Builds flow fields for group destinations.
///
/// Scans all units with `Destination` but no `Path`, groups them by
/// destination, and if enough units share a destination, builds a single
/// flow field that all of them can use.  Those units get a
/// `FlowFieldMovement` component instead of the usual `Path`.
fn build_flow_field_system(
    mut commands: Commands,
    grid: Res<PathfindingGrid>,
    mut cache: ResMut<FlowFieldCache>,
    // Units with Destination but no Path and no FlowFieldMovement
    to_group: Query<
        (Entity, &Destination),
        (
            With<Unit>,
            Without<Path>,
            Without<FlowFieldMovement>,
            Without<Dead>,
        ),
    >,
) {
    // ── Phase 1: Group by destination ────────────────────────────────
    let mut groups: std::collections::HashMap<IVec2, Vec<Entity>> =
        std::collections::HashMap::new();

    for (entity, dest) in to_group.iter() {
        groups.entry(dest.target.0).or_default().push(entity);
    }

    if groups.is_empty() {
        return;
    }

    // ── Phase 2: Build flow fields for large enough groups ───────────
    for (&dest, entities) in groups.iter() {
        if entities.len() < GROUP_SIZE_THRESHOLD {
            continue; // too few units → use per‑unit A* instead
        }

        // Build (or ensure cached) the flow field
        if cache.get(dest).is_none() {
            let new_field = FlowField::build(dest, &grid, FLOW_BOUNDS, MAX_FLOW_CELLS);
            cache.insert(dest, new_field);
        }

        // Assign FlowFieldMovement to all units in the group
        for &entity in entities {
            commands.entity(entity).insert(FlowFieldMovement {
                destination: GridPos(dest),
            });
        }
    }
}

/// Moves units that have a `FlowFieldMovement` component.
///
/// Each tick, the unit reads its flow direction from the `FlowFieldCache`
/// and advances one grid‑cell step in that direction.
fn flow_field_movement_system(
    time: Res<Time>,
    cache: Res<FlowFieldCache>,
    mut units: Query<(Entity, &mut WorldPosition, &FlowFieldMovement, &Stats), Without<Dead>>,
) {
    for (_entity, mut pos, flow, stats) in units.iter_mut() {
        let Some(field) = cache.get(flow.destination.0) else {
            continue; // flow field not yet computed
        };

        let Some(dir) = field.direction(pos.grid.0) else {
            continue; // already at destination (or outside field bounds)
        };

        let current = pos.world_2d.0;
        let target_pos = Vec2::new(
            (pos.grid.0.x + dir.x) as f32 * CELL_SIZE,
            (pos.grid.0.y + dir.y) as f32 * CELL_SIZE,
        );
        let direction = (target_pos - current).normalize_or_zero();
        let step = stats.move_speed * time.delta_secs();

        let distance = current.distance(target_pos);
        if distance <= step {
            // Reached the next cell — update both world and grid positions
            pos.world_2d.0 = target_pos;
            pos.grid.0 += dir;
        } else {
            pos.world_2d.0 += direction * step;
        }
    }
}

/// Removes `FlowFieldMovement` when the unit reaches its destination,
/// preventing re‑evaluation on subsequent frames.
fn cleanup_flow_field_system(
    mut commands: Commands,
    cache: Res<FlowFieldCache>,
    units: Query<(Entity, &WorldPosition, &FlowFieldMovement), Without<Dead>>,
) {
    for (entity, pos, flow) in units.iter() {
        let Some(field) = cache.get(flow.destination.0) else {
            continue;
        };

        // If the unit has no direction (at destination or outside field), clean up
        if field.direction(pos.grid.0).is_none() && pos.grid.0 == flow.destination.0 {
            commands.entity(entity).remove::<FlowFieldMovement>();
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn flow_field_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .insert_resource(GameTime::default())
            .init_resource::<PathfindingGrid>()
            .init_resource::<FlowFieldCache>()
            .add_systems(
                Update,
                (
                    super::build_flow_field_system.in_set(SimulationSet::Pathfinding),
                    super::flow_field_movement_system.in_set(SimulationSet::FreeFormMovement),
                    super::cleanup_flow_field_system.in_set(SimulationSet::ComponentCleanup),
                ),
            );
        app
    }

    fn seed_terrain_grid(app: &mut App) {
        let mut terrain = TerrainGrid::default();
        for x in -10..=10 {
            for y in -10..=10 {
                terrain.set(IVec2::new(x, y), TerrainType::OpenField);
            }
        }
        app.insert_resource(terrain);
    }

    fn seed_pathfinding_grid(app: &mut App) {
        let terrain = app.world().resource::<TerrainGrid>();
        let grid = PathfindingGrid::build(terrain, &std::collections::HashSet::new());
        app.insert_resource(grid);
    }

    #[test]
    fn test_flow_field_builds_for_group() {
        let mut app = flow_field_test_app();
        seed_terrain_grid(&mut app);
        seed_pathfinding_grid(&mut app);
        app.update(); // init readers

        // Spawn 3 units all with the same destination
        let dest = GridPos(IVec2::new(5, 0));
        for x in 0..3 {
            app.world_mut().spawn((
                Unit::default(),
                WorldPosition::new(IVec2::new(x, 0), CELL_SIZE),
                Destination { target: dest },
                Stats::default(),
                Health::default(),
            ));
        }

        app.update();

        // All 3 should now have FlowFieldMovement
        let flow_count = app
            .world_mut()
            .query_filtered::<Entity, With<FlowFieldMovement>>()
            .iter(app.world())
            .count();
        assert_eq!(flow_count, 3, "All 3 units should get FlowFieldMovement");

        // Flow field cache should have the destination
        let cache = app.world().resource::<FlowFieldCache>();
        assert!(cache.get(dest.0).is_some(), "Flow field should be cached");
    }

    #[test]
    fn test_flow_field_not_built_for_few_units() {
        let mut app = flow_field_test_app();
        seed_terrain_grid(&mut app);
        seed_pathfinding_grid(&mut app);
        app.update();

        // Only 2 units — below GROUP_SIZE_THRESHOLD (3)
        let dest = GridPos(IVec2::new(5, 0));
        for x in 0..2 {
            app.world_mut().spawn((
                Unit::default(),
                WorldPosition::new(IVec2::new(x, 0), CELL_SIZE),
                Destination { target: dest },
                Stats::default(),
                Health::default(),
            ));
        }

        app.update();

        // No flow field should be built
        let flow_count = app
            .world_mut()
            .query_filtered::<Entity, With<FlowFieldMovement>>()
            .iter(app.world())
            .count();
        assert_eq!(
            flow_count, 0,
            "Fewer than threshold should not get flow field"
        );
    }

    #[test]
    fn test_flow_field_moves_units_toward_destination() {
        let mut app = flow_field_test_app();
        seed_terrain_grid(&mut app);
        seed_pathfinding_grid(&mut app);
        app.update();

        let dest = GridPos(IVec2::new(5, 0));

        // A unit at (0, 0) with destination (5, 0)
        let unit = app
            .world_mut()
            .spawn((
                Unit::default(),
                WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
                Destination { target: dest },
                Stats::default(),
                Health::default(),
            ))
            .id();

        // Build the flow field manually and assign movement
        let grid = app.world().resource::<PathfindingGrid>();
        let field = FlowField::build(dest.0, grid, FLOW_BOUNDS, MAX_FLOW_CELLS);
        {
            let mut cache = app.world_mut().resource_mut::<FlowFieldCache>();
            cache.insert(dest.0, field);
        }
        app.world_mut()
            .entity_mut(unit)
            .insert(FlowFieldMovement { destination: dest });

        // Move for enough ticks to cross several cells (move_speed=3, delta≈0.0167,
        // so ≈0.05 grid-units per tick; 30 ticks → ~1.5 cells)
        for _ in 0..30 {
            app.update();
        }

        // The unit should have moved toward dest
        let pos = app.world().get::<WorldPosition>(unit).unwrap();
        assert!(
            pos.grid.0.x > 0,
            "Unit should have moved toward dest, x={}",
            pos.grid.0.x
        );
        // Verify significant progress
        assert!(
            pos.grid.0.x >= 1,
            "Unit should have advanced at least 1 cell, x={}",
            pos.grid.0.x
        );
    }
}
