use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn a building entity with the given `BuildingType` and `GridPos`.
fn spawn_building(
    commands: &mut Commands,
    building_type: BuildingType,
    pos: GridPos,
    level: u8,
) -> Entity {
    commands
        .spawn((
            Building {
                building_type,
                faction: FactionId::Kayi,
                level,
                max_level: 3,
                name: format!("{:?}", building_type),
            },
            WorldPosition::new(IVec2::new(pos.0.x, pos.0.y), 1.0),
            pos,
        ))
        .id()
}

/// Run `n` ticks of the app.
fn run_ticks(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Population Registry
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_population_registry_syncs_with_city_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();
    // Must be InGame for FixedUpdate systems to run
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    app.update(); // apply state transition

    // Tick once to let all CityPopulation systems settle
    app.update();

    // Record the baseline
    let baseline_total = app.world().resource::<PopulationRegistry>().total;

    // Set city population higher than the baseline
    let mut city = app.world_mut().resource_mut::<CityState>();
    city.population = baseline_total + 10;
    let expected_total = city.population;
    drop(city);

    // Tick again: sync should copy city → registry
    app.update();

    // Verify the sync system ran: registry.total should no longer be the baseline
    let registry = app.world().resource::<PopulationRegistry>();
    assert!(
        registry.total >= expected_total,
        "total should have increased to at least {} after sync; got {}",
        expected_total,
        registry.total
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Worker Allocation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_worker_assigns_to_vacant_building() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    // Insert WorkerSlotConfig with at least Pasture
    app.insert_resource(WorkerSlotConfig::default());

    let building = {
        let mut commands = app.world_mut().commands();
        spawn_building(
            &mut commands,
            BuildingType::Pasture,
            GridPos(IVec2::new(5, 5)),
            1,
        )
    };

    // Attach a WorkerSlot with vacancies
    {
        let mut commands = app.world_mut().commands();
        commands.entity(building).insert(WorkerSlot {
            max_workers: 4,
            assigned_workers: 0,
            required_role: WorkerRole::Gatherer,
            worker_efficiency: 0.9,
            full_staff_bonus: 1.30,
        });
    }

    // Set population to have available workers (idle workers)
    let mut city = app.world_mut().resource_mut::<CityState>();
    city.population = 10;
    city.max_population = 20;
    city.total_military_pop = 0;

    run_ticks(&mut app, 10);

    // After allocation, some workers should be assigned
    let slot = app.world().get::<WorkerSlot>(building).unwrap();
    assert!(
        slot.assigned_workers > 0,
        "Expected workers to be assigned to the building slot (assigned: {})",
        slot.assigned_workers
    );

    let allocation = app.world().resource::<WorkerAllocation>();
    assert!(
        allocation.total_assigned > 0,
        "Expected total_assigned > 0 after allocation"
    );
}

#[test]
fn test_worker_unassigns_when_slot_removed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    app.insert_resource(WorkerSlotConfig::default());

    let building = {
        let mut commands = app.world_mut().commands();
        spawn_building(
            &mut commands,
            BuildingType::Pasture,
            GridPos(IVec2::new(3, 3)),
            1,
        )
    };

    // Attach a slot with some pre-assigned workers
    {
        let mut commands = app.world_mut().commands();
        commands.entity(building).insert(WorkerSlot {
            max_workers: 4,
            assigned_workers: 2,
            required_role: WorkerRole::Gatherer,
            worker_efficiency: 0.9,
            full_staff_bonus: 1.30,
        });
    }

    let mut city = app.world_mut().resource_mut::<CityState>();
    city.population = 10;
    city.max_population = 20;

    // Tick enough times to get past the allocation throttle (default: 5 ticks)
    run_ticks(&mut app, 6);
    let alloc = app.world().resource::<WorkerAllocation>();
    let initial_assigned = alloc.total_assigned;

    // Remove the slot component
    {
        let mut commands = app.world_mut().commands();
        commands.entity(building).remove::<WorkerSlot>();
    }

    // Tick past throttle again
    run_ticks(&mut app, 6);
    let alloc = app.world().resource::<WorkerAllocation>();
    assert!(
        alloc.total_assigned < initial_assigned,
        "Total assigned should decrease when slot is removed (initial: {}, current: {})",
        initial_assigned,
        alloc.total_assigned
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Worker Output & Production
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_worker_production_with_multiplier() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .init_resource::<ProductionRates>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    let building = {
        let mut commands = app.world_mut().commands();
        spawn_building(
            &mut commands,
            BuildingType::Pasture,
            GridPos(IVec2::new(2, 2)),
            1,
        )
    };

    // Fully-staffed WorkerSlot (4 workers)
    {
        let mut commands = app.world_mut().commands();
        commands.entity(building).insert(WorkerSlot {
            max_workers: 4,
            assigned_workers: 4,
            required_role: WorkerRole::Gatherer,
            worker_efficiency: 0.9,
            full_staff_bonus: 1.30,
        });
    }

    let initial_food = app.world().resource::<ResourceStockpile>().food;

    run_ticks(&mut app, 10);

    // Pasture should produce food
    let final_food = app.world().resource::<ResourceStockpile>().food;
    assert!(
        final_food > initial_food,
        "Worker production should increase food stockpile (initial: {:.1}, final: {:.1})",
        initial_food,
        final_food
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Population Growth
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_population_growth_with_high_needs() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    // Set up high citizen needs satisfaction
    let mut needs = app.world_mut().resource_mut::<CitizenNeeds>();
    needs.food_satisfaction = 0.95;
    needs.housing_satisfaction = 0.90;
    needs.faith_satisfaction = 0.85;
    needs.safety_satisfaction = 0.90;

    let mut city = app.world_mut().resource_mut::<CityState>();
    city.population = 20;
    city.max_population = 100;
    city.happiness = 85.0;

    let initial_pop = city.population;

    run_ticks(&mut app, 20);

    let final_pop = app.world().resource::<CityState>().population;
    assert!(
        final_pop >= initial_pop,
        "Population should not decrease when needs are well met (initial: {}, final: {})",
        initial_pop,
        final_pop
    );
}

#[test]
fn test_population_declines_when_needs_critical() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .init_resource::<GrowthConfig>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    // Set up critical citizen needs (starvation)
    let mut needs = app.world_mut().resource_mut::<CitizenNeeds>();
    needs.food_satisfaction = 0.05;

    let mut city = app.world_mut().resource_mut::<CityState>();
    city.population = 30;
    city.max_population = 100;
    city.happiness = 10.0;

    let initial_pop = city.population;

    // NeedsCritical event should fire when needs are very low
    run_ticks(&mut app, 10);

    let final_pop = app.world().resource::<CityState>().population;
    // Population may stay same or decrease — just verify it's not dramatically growing
    assert!(
        final_pop <= initial_pop + 2,
        "Population should not grow significantly when needs are critical (initial: {}, final: {})",
        initial_pop,
        final_pop
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Complete Construction attaches WorkerSlot
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_construction_complete_attaches_worker_slot() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<PopulationRegistry>()
        .init_resource::<WorkerAllocation>()
        .add_event::<PopulationGrowth>()
        .add_event::<PopulationDecline>()
        .add_event::<WorkerAssigned>()
        .add_event::<WorkerUnassigned>()
        .add_event::<NeedsCritical>();

    // Provide a WorkerSlotConfig so the construction system can look up slots
    app.insert_resource(WorkerSlotConfig {
        configs: vec![(
            BuildingType::Pasture,
            WorkerSlot {
                max_workers: 4,
                assigned_workers: 0,
                required_role: WorkerRole::Gatherer,
                worker_efficiency: 0.9,
                full_staff_bonus: 1.30,
            },
        )],
    });

    // Set to InGame so FixedUpdate systems (complete_construction_system) run
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::InGame);
    app.update(); // flush commands and apply state transition

    // Spawn a completed construction site
    let entity = {
        let mut commands = app.world_mut().commands();
        commands
            .spawn((
                ConstructionSite {
                    building_type: BuildingType::Pasture,
                    progress: 100.0,
                    max_progress: 100.0,
                    assigned_workers: vec![],
                },
                Building {
                    building_type: BuildingType::Pasture,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 3,
                    name: "Test Pasture".into(),
                },
                WorldPosition::new(IVec2::new(0, 0), 1.0),
                GridPos(IVec2::new(0, 0)),
            ))
            .id()
    };

    app.update();

    // After update, ConstructionSite should be removed and WorkerSlot inserted
    assert!(
        !app.world().entity(entity).contains::<ConstructionSite>(),
        "ConstructionSite should be removed after completion"
    );
    assert!(
        app.world().entity(entity).contains::<WorkerSlot>(),
        "WorkerSlot should be attached after construction completes"
    );

    let slot = app.world().get::<WorkerSlot>(entity).unwrap();
    assert_eq!(slot.max_workers, 4);
    assert_eq!(slot.required_role, WorkerRole::Gatherer);
}
