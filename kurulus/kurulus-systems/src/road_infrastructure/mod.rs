use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct RoadInfrastructurePlugin;

impl Plugin for RoadInfrastructurePlugin {
    fn build(&self, app: &mut App) {
        app
            // Road-access validation runs after construction completes, in the
            // CityServices phase (alongside other city-level infrastructure checks).
            .add_systems(
                FixedUpdate,
                (
                    road_access_check_system.in_set(SimulationSet::CityServices),
                    // Road maintenance is a recurring cost, grouped with other
                    // resource consumption systems.
                    road_maintenance_system.in_set(SimulationSet::ResourceConsumption),
                    // Production penalty for buildings without road access.
                    road_access_production_penalty_system.in_set(SimulationSet::ResourceProduction),
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Road Access Check
// ═══════════════════════════════════════════════════════════════════════════════

/// Every tick, scans all buildings that need road access and checks whether
/// they have an adjacent `Road` entity.  Updates the `RoadAccess` component
/// accordingly.
///
/// **Adjacency** = any of the 8 neighbouring cells (including diagonals)
/// contains a completed `Building` whose `building_type == Road`.
fn road_access_check_system(
    mut consumers: Query<(Entity, &WorldPosition, &mut RoadAccess, &RequiresRoadAccess)>,
    roads: Query<&WorldPosition, (With<Building>, Without<ConstructionSite>)>,
    new_buildings: Query<
        (
            Entity,
            &Building,
            Option<&ConstructionSite>,
            Has<RequiresRoadAccess>,
        ),
        Added<Building>,
    >,
    mut commands: Commands,
) {
    // ── Collect road positions into a spatial hash set ────────────────
    let road_positions: std::collections::HashSet<IVec2> =
        roads.iter().map(|wp| wp.grid.0).collect();

    // ── Helper: check if any neighbouring cell contains a road ─────────
    let has_adjacent_road = |pos: IVec2| -> bool {
        const NEIGHBOURS: [IVec2; 8] = [
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ];
        NEIGHBOURS
            .iter()
            .any(|&offset| road_positions.contains(&(pos + offset)))
    };

    // ── Attach RequiresRoadAccess to newly completed buildings ─────────
    for (_entity, building, site, already_has) in new_buildings.iter() {
        if site.is_some() {
            continue; // still under construction
        }
        if !already_has && requires_road_access(building.building_type) {
            commands.entity(_entity).insert(RequiresRoadAccess);
        }
    }

    // ── Update road access for all consumers ──────────────────────────
    for (_entity, pos, mut access, _req) in consumers.iter_mut() {
        let now_has = has_adjacent_road(pos.grid.0);
        if access.has_access != now_has {
            access.has_access = now_has;
            if now_has {
                trace!("Road access granted at {:?}", pos.grid.0);
            } else {
                trace!("Road access lost at {:?}", pos.grid.0);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Road Maintenance
// ═══════════════════════════════════════════════════════════════════════════════

/// Deducts a small gold upkeep per road entity each tick.
///
/// Formula: `gold_cost = road_count × ROAD_MAINTENANCE_PER_TICK × delta`.
/// A single road costs ~0.5 gold per game-day (at 1 tick/sec).
const ROAD_MAINTENANCE_PER_TICK: f32 = 0.02; // gold per road per game-second

fn road_maintenance_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    roads: Query<&Building, (With<WorldPosition>, Without<ConstructionSite>)>,
) {
    let delta = time.time_scale * 0.1;
    if delta <= 0.0 {
        return;
    }

    let road_count = roads
        .iter()
        .filter(|b| b.building_type == BuildingType::Road)
        .count() as f32;

    if road_count == 0.0 {
        return;
    }

    let total_cost = road_count * ROAD_MAINTENANCE_PER_TICK * delta;
    stockpile.gold = (stockpile.gold - total_cost).max(0.0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Production Penalty for No Road Access
// ═══════════════════════════════════════════════════════════════════════════════

/// Buildings that require road access but *don't* have it produce at 50 %
/// efficiency.  This system modifies `BuildingResourceProduction` *only when
/// the access state changes*, avoiding floating-point drift from repeated
/// re-scaling.
const ROAD_PENALTY_MULT: f32 = 0.5; // 50 % production with no road access

fn road_access_production_penalty_system(
    mut consumers: Query<(
        &mut RoadAccess,
        &RequiresRoadAccess,
        &mut BuildingResourceProduction,
    )>,
) {
    for (mut access, _req, mut prod) in consumers.iter_mut() {
        if access.has_access && access.penalty_applied {
            // Road was just built — restore production (inverse of penalty).
            let inv = 1.0 / ROAD_PENALTY_MULT;
            prod.food_per_sec *= inv;
            prod.gold_per_sec *= inv;
            prod.wood_per_sec *= inv;
            prod.stone_per_sec *= inv;
            prod.iron_per_sec *= inv;
            access.penalty_applied = false;
        } else if !access.has_access && !access.penalty_applied {
            // Road was just removed — apply penalty.
            prod.food_per_sec *= ROAD_PENALTY_MULT;
            prod.gold_per_sec *= ROAD_PENALTY_MULT;
            prod.wood_per_sec *= ROAD_PENALTY_MULT;
            prod.stone_per_sec *= ROAD_PENALTY_MULT;
            prod.iron_per_sec *= ROAD_PENALTY_MULT;
            access.penalty_applied = true;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use kurulus_core::components::building::BuildingType;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .add_systems(FixedUpdate, road_access_check_system);
        app
    }

    /// Spawn a road at the given grid position using the world directly
    /// (so the entity is immediately available for later `entity_mut` calls).
    fn spawn_road(world: &mut World, pos: IVec2) -> Entity {
        world
            .spawn((
                Building {
                    building_type: BuildingType::Road,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 1,
                    name: "Road".into(),
                },
                WorldPosition::new(pos, 1.0),
            ))
            .id()
    }

    /// Spawn a building that needs road access at the given grid position
    /// using the world directly (not commands).
    fn spawn_consumer(world: &mut World, pos: IVec2) -> Entity {
        world
            .spawn((
                Building {
                    building_type: BuildingType::Barracks,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 3,
                    name: "Barracks".into(),
                },
                WorldPosition::new(pos, 1.0),
                RoadAccess::default(),
                RequiresRoadAccess,
                BuildingResourceProduction::default(),
            ))
            .id()
    }

    fn run_frames(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    // ─── Road Access Check ────────────────────────────────────────────

    #[test]
    fn test_adjacent_road_grants_access() {
        let mut app = test_app();

        spawn_road(app.world_mut(), IVec2::new(5, 5));
        let consumer = spawn_consumer(app.world_mut(), IVec2::new(5, 6));

        run_frames(&mut app, 2);

        let access = app.world().get::<RoadAccess>(consumer).unwrap();
        assert!(
            access.has_access,
            "Building adjacent to road should have access"
        );
    }

    #[test]
    fn test_no_road_no_access() {
        let mut app = test_app();

        let consumer = spawn_consumer(app.world_mut(), IVec2::new(10, 10));

        run_frames(&mut app, 2);

        let access = app.world().get::<RoadAccess>(consumer).unwrap();
        assert!(
            !access.has_access,
            "Building with no adjacent road should lack access"
        );
    }

    #[test]
    fn test_diagonal_road_grants_access() {
        let mut app = test_app();

        spawn_road(app.world_mut(), IVec2::new(5, 5));
        let consumer = spawn_consumer(app.world_mut(), IVec2::new(6, 6));

        run_frames(&mut app, 2);

        let access = app.world().get::<RoadAccess>(consumer).unwrap();
        assert!(
            access.has_access,
            "Building diagonal to road should have access"
        );
    }

    #[test]
    fn test_road_lost_when_road_removed() {
        let mut app = test_app();

        let road = spawn_road(app.world_mut(), IVec2::new(5, 5));
        let consumer = spawn_consumer(app.world_mut(), IVec2::new(5, 6));

        run_frames(&mut app, 2);

        assert!(
            app.world().get::<RoadAccess>(consumer).unwrap().has_access,
            "Should start with access"
        );

        app.world_mut().despawn(road);
        run_frames(&mut app, 2);

        assert!(
            !app.world().get::<RoadAccess>(consumer).unwrap().has_access,
            "Access should be lost after road removal"
        );
    }

    // ─── Road Maintenance ─────────────────────────────────────────────

    #[test]
    fn test_road_maintenance_deducts_gold() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, road_maintenance_system);

        spawn_road(app.world_mut(), IVec2::new(0, 0));
        spawn_road(app.world_mut(), IVec2::new(0, 1));

        app.world_mut().resource_mut::<ResourceStockpile>().gold = 100.0;

        run_frames(&mut app, 10);

        let gold = app.world().resource::<ResourceStockpile>().gold;
        assert!(
            gold < 100.0,
            "Gold should decrease due to road maintenance (got {})",
            gold
        );
    }

    #[test]
    fn test_road_maintenance_no_negative_gold() {
        let mut app = test_app();
        app.add_systems(FixedUpdate, road_maintenance_system);

        spawn_road(app.world_mut(), IVec2::new(0, 0));
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 0.01;

        run_frames(&mut app, 10);

        let gold = app.world().resource::<ResourceStockpile>().gold;
        assert!(gold >= 0.0, "Gold should not go negative (got {})", gold);
    }

    // ─── Production Penalty ──────────────────────────────────────────

    #[test]
    fn test_production_penalty_applied() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            (
                road_access_check_system,
                road_access_production_penalty_system,
            ),
        );

        // Consumer with no adjacent road — spawn without BuildingResourceProduction,
        // then insert it after so the test is explicit.
        let consumer = spawn_consumer(app.world_mut(), IVec2::new(10, 10));
        app.world_mut()
            .entity_mut(consumer)
            .insert(BuildingResourceProduction {
                food_per_sec: 10.0,
                gold_per_sec: 5.0,
                ..default()
            });

        run_frames(&mut app, 2);

        // Since there's no road, access is false and penalty_applied should toggle to true.
        let prod = app
            .world()
            .get::<BuildingResourceProduction>(consumer)
            .unwrap();
        assert!(
            (prod.food_per_sec - 5.0).abs() < 0.01,
            "Expected food production halved ({} ≈ 5.0)",
            prod.food_per_sec
        );
        assert!(
            (prod.gold_per_sec - 2.5).abs() < 0.01,
            "Expected gold production halved ({} ≈ 2.5)",
            prod.gold_per_sec
        );
    }

    #[test]
    fn test_production_restored_when_road_added() {
        let mut app = test_app();
        app.add_systems(
            FixedUpdate,
            (
                road_access_check_system,
                road_access_production_penalty_system,
            ),
        );

        let consumer = spawn_consumer(app.world_mut(), IVec2::new(5, 6));
        app.world_mut()
            .entity_mut(consumer)
            .insert(BuildingResourceProduction {
                food_per_sec: 10.0,
                gold_per_sec: 5.0,
                ..default()
            });

        // First frame: no road, penalty applied
        run_frames(&mut app, 2);

        {
            let prod = app
                .world()
                .get::<BuildingResourceProduction>(consumer)
                .unwrap();
            assert!(
                (prod.food_per_sec - 5.0).abs() < 0.01,
                "Should be penalized initially"
            );
        }

        // Add a road now
        spawn_road(app.world_mut(), IVec2::new(5, 5));
        run_frames(&mut app, 3);

        // Production should be restored
        let prod = app
            .world()
            .get::<BuildingResourceProduction>(consumer)
            .unwrap();
        assert!(
            (prod.food_per_sec - 10.0).abs() < 0.01,
            "Production should be restored when road is added (got {})",
            prod.food_per_sec
        );
    }
}
