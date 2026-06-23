use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct BaseBuildingPlugin;

impl Plugin for BaseBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecruitmentTimer>()
            .add_event::<BuildRejected>()
            .add_systems(
                FixedUpdate,
                (
                    construction_progress_system.in_set(SimulationSet::ConstructionProgress),
                    complete_construction_system,
                    building_upgrade_system,
                    build_from_order_system.in_set(SimulationSet::InputProcessing),
                    process_recruit_orders_system,
                    recruitment_progress_system,
                ),
            );
    }
}

// ─── Construction progress ─────────────────────────────────────────────────

/// Tick construction progress and consume resources from the stockpile
/// proportionally as the building is built.
fn construction_progress_system(
    game_time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut sites: Query<(&mut ConstructionSite, &BuildCost)>,
) {
    for (mut site, cost) in sites.iter_mut() {
        if site.progress >= site.max_progress {
            continue;
        }

        let increment = game_time.time_scale * 10.0;
        let new_progress = (site.progress + increment).min(site.max_progress);

        // Consume resources proportionally to progress made this tick
        let progress_fraction = (new_progress - site.progress) / site.max_progress;
        stockpile.food = (stockpile.food - cost.food as f32 * progress_fraction).max(0.0);
        stockpile.gold = (stockpile.gold - cost.gold as f32 * progress_fraction).max(0.0);
        stockpile.wood = (stockpile.wood - cost.wood as f32 * progress_fraction).max(0.0);
        stockpile.stone = (stockpile.stone - cost.stone as f32 * progress_fraction).max(0.0);

        site.progress = new_progress;
    }
}

/// When a `ConstructionSite` reaches full progress, replace it with a
/// `Building` component so the structure becomes operational.
fn complete_construction_system(
    mut commands: Commands,
    sites: Query<(Entity, &ConstructionSite, &Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    for (entity, site, building_template, pos) in sites.iter() {
        if site.progress < site.max_progress {
            continue;
        }

        // Calculate max_population contribution from Oba
        let pop_bonus = if building_template.building_type == BuildingType::Oba {
            match building_template.level {
                1 => 5,
                2 => 10,
                3 => 15,
                4 => 25,
                _ => 40,
            }
        } else {
            0
        };
        stockpile.max_population += pop_bonus;

        // Remove ConstructionSite, keep Building and other components
        commands.entity(entity).remove::<ConstructionSite>();

        // Log completion
        info!(
            "Construction complete: {:?} at {:?}",
            building_template.building_type,
            pos.grid.0
        );
    }
}

/// Reasonable default build costs for each building type.
/// Used when spawning a new `BuildCost` component from a `BuildOrder`.
fn build_cost_for_type(bt: BuildingType) -> BuildCost {
    use BuildingType::*;
    match bt {
        Oba => BuildCost { food: 20, gold: 10, wood: 30, stone: 0, population: 0 },
        Pasture => BuildCost { food: 5, gold: 0, wood: 15, stone: 0, population: 0 },
        MiningCamp => BuildCost { food: 0, gold: 5, wood: 10, stone: 0, population: 0 },
        LumberCamp => BuildCost { food: 0, gold: 0, wood: 5, stone: 0, population: 0 },
        Barracks => BuildCost { food: 15, gold: 20, wood: 25, stone: 0, population: 0 },
        ArcheryRange => BuildCost { food: 10, gold: 15, wood: 20, stone: 0, population: 0 },
        Stable => BuildCost { food: 20, gold: 25, wood: 30, stone: 0, population: 0 },
        SiegeWorkshop => BuildCost { food: 30, gold: 40, wood: 50, stone: 10, population: 0 },
        Watchtower => BuildCost { food: 5, gold: 10, wood: 20, stone: 15, population: 0 },
        Market => BuildCost { food: 0, gold: 15, wood: 20, stone: 0, population: 0 },
        Warehouse => BuildCost { food: 0, gold: 10, wood: 25, stone: 0, population: 0 },
        Well => BuildCost { food: 0, gold: 5, wood: 10, stone: 5, population: 0 },
        Mosque => BuildCost { food: 10, gold: 30, wood: 20, stone: 20, population: 0 },
        CouncilChamber => BuildCost { food: 20, gold: 50, wood: 30, stone: 30, population: 0 },
        GuildHall => BuildCost { food: 15, gold: 35, wood: 25, stone: 20, population: 0 },
        Monument => BuildCost { food: 50, gold: 80, wood: 40, stone: 60, population: 0 },
        // Defensive & infrastructure have minimal costs
        Palisade => BuildCost { food: 0, gold: 0, wood: 10, stone: 0, population: 0 },
        StoneWall => BuildCost { food: 0, gold: 5, wood: 10, stone: 20, population: 0 },
        Gate => BuildCost { food: 0, gold: 10, wood: 15, stone: 15, population: 0 },
        Road => BuildCost { food: 0, gold: 0, wood: 2, stone: 0, population: 0 },
    }
}

/// Reads `BuildOrder` events, validates against existing zones, and spawns
/// construction entities if the building is allowed at that position.
///
/// ## Zone validation rules
///
/// | Condition | Result |
/// |-----------|--------|
/// | No zone at target cell | ✅ Allowed (backwards compatible) |
/// | Zone type matches building type | ✅ Allowed |
/// | Zone type does NOT match | ❌ Rejected → emits `BuildRejected` |
/// | Target cell already has a building/construction | ❌ Rejected → emits `BuildRejected` |
fn build_from_order_system(
    mut orders: EventReader<BuildOrder>,
    mut rejected_writer: EventWriter<BuildRejected>,
    mut commands: Commands,
    zones: Query<(&Zone, &WorldPosition)>,
    existing: Query<&WorldPosition, Or<(With<Building>, With<ConstructionSite>)>>,
) {
    for order in orders.read() {
        let target = order.position;
        let building_type = order.building_type;

        // ── Check for existing building/construction at this cell ──────
        let cell_occupied = existing.iter().any(|wp| wp.grid.0 == target.0);
        if cell_occupied {
            rejected_writer.send(BuildRejected {
                order: order.clone(),
                reason: BuildRejectionReason::CellOccupied,
            });
            continue;
        }

        // ── Check zone compatibility ──────────────────────────────────
        let zone_at_target = zones.iter().find(|(_, wp)| wp.grid.0 == target.0);

        if let Some((zone, _)) = zone_at_target {
            if !zone.zone_type.matches_building(building_type) {
                rejected_writer.send(BuildRejected {
                    order: order.clone(),
                    reason: BuildRejectionReason::ZoneMismatch(zone.zone_type, building_type),
                });
                continue;
            }
        }

        // ── All checks passed — spawn the construction ────────────────
        let max_level = building_type.max_level();
        commands.spawn((
            ConstructionSite {
                building_type,
                progress: 0.0,
                max_progress: building_type.build_time(),
                assigned_workers: vec![order.entity],
            },
            Building {
                building_type,
                faction: FactionId::Kayi, // default faction, overridden by mission setup
                level: 1,
                max_level,
                name: format!("{:?} I", building_type),
            },
            build_cost_for_type(building_type),
            WorldPosition::new(target.0, CELL_SIZE),
        ));
    }
}

// ─── Building upgrades ─────────────────────────────────────────────────────

/// Handles building level upgrades (triggered by a `BuildOrder` with an
/// existing building as target, or via direct component mutation).
///
/// For Phase 2, the upgrade cost is hardcoded per building type.
fn building_upgrade_system(
    mut buildings: Query<(&mut Building, &BuildCost), Without<UpgradeChain>>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut production_rates: ResMut<ProductionRates>,
) {
    for (mut building, upgrade_cost) in buildings.iter_mut() {
        if building.level >= building.max_level {
            continue;
        }

        // Upgrade is triggered when a player sends a BuildOrder targeting
        // an existing building.  In Phase 2 we simulate this automatically
        // when resources are sufficient (for testing).
        // A full game would use BuildOrder events.

        let can_upgrade = stockpile.food >= upgrade_cost.food as f32
            && stockpile.gold >= upgrade_cost.gold as f32
            && stockpile.wood >= upgrade_cost.wood as f32
            && stockpile.stone >= upgrade_cost.stone as f32;

        if !can_upgrade {
            continue;
        }

        // Deduct cost
        stockpile.food -= upgrade_cost.food as f32;
        stockpile.gold -= upgrade_cost.gold as f32;
        stockpile.wood -= upgrade_cost.wood as f32;
        stockpile.stone -= upgrade_cost.stone as f32;

        // Apply level
        building.level += 1;

        // Oba-specific: increase max_population per level
        if building.building_type == BuildingType::Oba {
            stockpile.max_population += 5; // flat bonus per level
            production_rates.food_per_second *= 1.2; // Oba upgrades improve food production
        }

        info!(
            "{:?} upgraded to level {}",
            building.building_type, building.level
        );

        // For now, upgrade one at a time per frame
        break;
    }
}

// ─── Recruitment ───────────────────────────────────────────────────────────

/// Tracks the recruitment timer so units are spawned one at a time with a
/// delay between them.
#[derive(Resource, Default)]
pub struct RecruitmentTimer {
    pub active_recruitments: Vec<ActiveRecruitment>,
}

pub struct ActiveRecruitment {
    pub building: Entity,
    pub archetype: UnitArchetype,
    pub remaining: u32,
    pub timer: f32,
    pub interval: f32, // seconds between each unit
}

/// Reads `RecruitOrder` events and starts recruitment queues on buildings.
fn process_recruit_orders_system(
    mut orders: EventReader<RecruitOrder>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<&Building>,
) {
    for order in orders.read() {
        let Ok(building) = buildings.get(order.building) else {
            continue;
        };

        // Only Oba can recruit in Phase 2
        if building.building_type != BuildingType::Oba {
            continue;
        }

        let interval = match order.archetype {
            UnitArchetype::Worker => 5.0,
            UnitArchetype::LightInfantry => 8.0,
            _ => 10.0,
        };

        // Add to recruitment queue
        timer.active_recruitments.push(ActiveRecruitment {
            building: order.building,
            archetype: order.archetype,
            remaining: order.count,
            timer: 0.0,
            interval,
        });
    }
}

/// Processes the recruitment queue, spawning units when their timer reaches
/// the interval threshold.
fn recruitment_progress_system(
    mut commands: Commands,
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<(&Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    let mut completed = Vec::new();

    for (idx, recruitment) in timer.active_recruitments.iter_mut().enumerate() {
        if recruitment.remaining == 0 {
            completed.push(idx);
            continue;
        }

        recruitment.timer += time.delta_secs();

        if recruitment.timer >= recruitment.interval {
            recruitment.timer -= recruitment.interval;
            recruitment.remaining -= 1;

            // Spawn the unit near the building
            if let Ok((_building, pos)) = buildings.get(recruitment.building) {
                let spawn_offset = IVec2::new(
                    pos.grid.0.x + 2, // spawn to the right of the building
                    pos.grid.0.y,
                );

                let (health, stats, weapon) = unit_archetype_stats(recruitment.archetype);
                let attack_speed = stats.attack_speed;

                commands.spawn((
                    Unit {
                        archetype: recruitment.archetype,
                        faction: FactionId::Kayi,
                        formation: FormationType::Line,
                        stance: CombatStance::Defensive,
                    },
                    WorldPosition::new(spawn_offset, CELL_SIZE),
                    health,
                    stats,
                    weapon,
                    AttackTimer(Timer::from_seconds(
                        1.0 / attack_speed.max(0.1),
                        TimerMode::Repeating,
                    )),
                ));

                // Consume population
                stockpile.population += 1;

                info!("Recruited {:?} at {:?}", recruitment.archetype, spawn_offset);
            }
        }
    }

    // Remove completed recruitments (reverse order)
    for idx in completed.into_iter().rev() {
        timer.active_recruitments.remove(idx);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Helper: build an app with `build_from_order_system` and basic resources.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .add_event::<BuildOrder>()
            .add_event::<BuildRejected>()
            .add_systems(FixedUpdate, build_from_order_system);
        app
    }

    /// Marker resource that collects `BuildRejected` events during simulation.
    #[derive(Resource, Default)]
    struct RejectionCollector(Vec<BuildRejected>);

    /// System that runs after `build_from_order_system` to capture rejections.
    fn collect_rejections(mut orders: EventReader<BuildRejected>, mut collector: ResMut<RejectionCollector>) {
        for ev in orders.read() {
            collector.0.push(ev.clone());
        }
    }

    /// Count construction-site entities at a given grid position.
    fn constructions_at(app: &mut App, pos: IVec2) -> usize {
        app.world_mut()
            .query_filtered::<&WorldPosition, With<ConstructionSite>>()
            .iter(app.world())
            .filter(|wp| wp.grid.0 == pos)
            .count()
    }

    /// Build a test app that includes the rejection collector.
    fn test_app_with_collector() -> App {
        let mut app = test_app();
        app.init_resource::<RejectionCollector>()
            .add_systems(FixedUpdate, collect_rejections.after(build_from_order_system));
        app
    }

    // ── No zone → allowed (backwards compatible) ──────────────────────

    #[test]
    fn test_build_without_zone_is_allowed() {
        let mut app = test_app_with_collector();

        // Send a BuildOrder for a cell with no zone
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(5, 5)),
            });

        // Run enough frames for FixedUpdate to fire
        for _ in 0..5 {
            app.update();
        }

        // ConstructionSite should exist at (5, 5)
        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            1,
            "Barracks should be placed without a zone"
        );
        // No rejection event
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.0.is_empty(),
            "No rejection expected without zone"
        );
    }

    // ── Matching zone → allowed ───────────────────────────────────────

    #[test]
    fn test_build_in_matching_zone() {
        let mut app = test_app_with_collector();

        // Place a Military zone at (3, 3)
        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Military,
                level: 1,
            },
            WorldPosition::new(IVec2::new(3, 3), CELL_SIZE),
        ));

        // Barracks matches Military zone
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(3, 3)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(3, 3)),
            1,
            "Barracks should be placed in Military zone"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.0.is_empty(),
            "No rejection for matching zone"
        );
    }

    // ── Mismatched zone → rejected ────────────────────────────────────

    #[test]
    fn test_build_in_wrong_zone_is_rejected() {
        let mut app = test_app_with_collector();

        // Residential zone at (2, 2)
        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(2, 2), CELL_SIZE),
        ));

        // Barracks does NOT match Residential
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(2, 2)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(2, 2)),
            0,
            "Barracks should NOT be placed in Residential zone"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            !collector.0.is_empty(),
            "Should emit at least one BuildRejected"
        );
        assert!(
            collector.0.iter().any(|r| matches!(r.reason, BuildRejectionReason::ZoneMismatch(ZoneType::Residential, BuildingType::Barracks))),
            "Expected ZoneMismatch(Residential, Barracks), got: {:?}",
            collector.0.first().map(|r| &r.reason)
        );
    }

    // ── All zone-type / building-type match pairs ─────────────────────

    #[test]
    fn test_all_zone_type_matches() {
        let pairs = vec![
            (ZoneType::Residential, BuildingType::Oba),
            (ZoneType::Military, BuildingType::Barracks),
            (ZoneType::Military, BuildingType::ArcheryRange),
            (ZoneType::Military, BuildingType::Stable),
            (ZoneType::Military, BuildingType::SiegeWorkshop),
            (ZoneType::Military, BuildingType::Watchtower),
            (ZoneType::Economic, BuildingType::Market),
            (ZoneType::Economic, BuildingType::Warehouse),
            (ZoneType::Economic, BuildingType::Pasture),
            (ZoneType::Economic, BuildingType::MiningCamp),
            (ZoneType::Economic, BuildingType::LumberCamp),
            (ZoneType::Religious, BuildingType::Mosque),
            (ZoneType::Religious, BuildingType::Monument),
            (ZoneType::Administrative, BuildingType::CouncilChamber),
            (ZoneType::Administrative, BuildingType::GuildHall),
        ];

        for (zone_type, building_type) in pairs {
            let mut app = test_app_with_collector();
            app.world_mut().spawn((
                Zone {
                    zone_type,
                    level: 1,
                },
                WorldPosition::new(IVec2::new(7, 7), CELL_SIZE),
            ));

            app.world_mut()
                .resource_mut::<Events<BuildOrder>>()
                .send(BuildOrder {
                    entity: Entity::from_raw(1),
                    building_type,
                    position: GridPos(IVec2::new(7, 7)),
                });

            for _ in 0..5 {
                app.update();
            }

            assert_eq!(
                constructions_at(&mut app, IVec2::new(7, 7)),
                1,
                "{:?} should be placeable in {:?} zone",
                building_type,
                zone_type
            );
            let collector = app.world().resource::<RejectionCollector>();
            assert!(
                collector.0.is_empty(),
                "No rejection for {:?} in {:?}",
                building_type,
                zone_type
            );
        }
    }

    // ── Cell occupied → rejected ──────────────────────────────────────

    #[test]
    fn test_build_on_occupied_cell_is_rejected() {
        let mut app = test_app_with_collector();

        // An existing Building at (4, 4)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Test Oba".into(),
            },
            WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        ));

        // Try to build on the same cell
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(2),
                building_type: BuildingType::Market,
                position: GridPos(IVec2::new(4, 4)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(4, 4)),
            0,
            "No construction on occupied cell"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.0.iter().any(|r| matches!(r.reason, BuildRejectionReason::CellOccupied)),
            "Expected CellOccupied rejection"
        );
    }

    // ── Mismatched zone does NOT prevent other builds ─────────────────

    #[test]
    fn test_rejected_build_does_not_block_other_orders() {
        let mut app = test_app_with_collector();

        // Residential zone at (1, 1)
        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(1, 1), CELL_SIZE),
        ));

        // Send TWO orders: Barracks (wrong for Residential) and Oba (right)
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(1, 1)),
            });
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(2),
                building_type: BuildingType::Oba,
                position: GridPos(IVec2::new(1, 1)),
            });

        for _ in 0..5 {
            app.update();
        }

        // Oba should be placed (matches Residential)
        assert_eq!(
            constructions_at(&mut app, IVec2::new(1, 1)),
            1,
            "Oba should be placed after Barracks was rejected"
        );
        // At least one rejection for Barracks
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.0.iter().any(|r| matches!(r.reason, BuildRejectionReason::ZoneMismatch(ZoneType::Residential, BuildingType::Barracks))),
            "Should have rejected Barracks in Residential zone"
        );
    }

    // ── Construction proceeds normally after zone-validated spawn ─────

    #[test]
    fn test_construction_progresses_after_zone_validated_spawn() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .add_event::<BuildOrder>()
            .add_event::<BuildRejected>()
            .add_systems(
                FixedUpdate,
                (
                    build_from_order_system,
                    construction_progress_system,
                    complete_construction_system,
                ),
            );

        // Send a BuildOrder for Oba in a Residential zone
        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Oba,
                position: GridPos(IVec2::new(0, 0)),
            });

        // Run many frames — construction should start, progress, and complete
        for _ in 0..120 {
            app.update();
        }

        // After many ticks, the ConstructionSite should be gone (completed)
        let const_count = app
            .world_mut()
            .query_filtered::<&ConstructionSite, With<WorldPosition>>()
            .iter(app.world())
            .filter(|site| site.building_type == BuildingType::Oba)
            .count();
        assert_eq!(
            const_count, 0,
            "Oba construction should have completed"
        );

        // There should be a Building with 'level >= 1'
        let building_count = app
            .world_mut()
            .query_filtered::<&Building, With<WorldPosition>>()
            .iter(app.world())
            .filter(|b| b.building_type == BuildingType::Oba)
            .count();
        assert_eq!(
            building_count, 1,
            "Oba should exist as a completed Building"
        );
    }
}

/// Returns default health, stats, and weapon for a given archetype.
fn unit_archetype_stats(
    archetype: UnitArchetype,
) -> (Health, Stats, MeleeWeapon) {
    match archetype {
        UnitArchetype::LightInfantry => (
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.5,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 1.0,
                vision_range: 6.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Sword,
                range: 1.5,
                armor_penetration: 0.1,
            },
        ),
        UnitArchetype::Worker => (
            Health {
                current: 50.0,
                max: 50.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.0,
                attack_damage: 3.0,
                attack_speed: 0.5,
                vision_range: 4.0,
                morale: 30.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 3.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
        ),
        UnitArchetype::Archer => (
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 6.0,
                attack_damage: 15.0,
                attack_speed: 0.7,
                vision_range: 8.0,
                morale: 40.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Bow,
                range: 6.0,
                armor_penetration: 0.0,
            },
        ),
        _ => (
            Health::default(),
            Stats::default(),
            MeleeWeapon {
                damage: 10.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
        ),
    }
}
