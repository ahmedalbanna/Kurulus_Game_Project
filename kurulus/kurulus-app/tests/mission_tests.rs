use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::components::MissionStateComponent;
use kurulus_core::prelude::*;
use kurulus_core::states::MissionState as MissionStateEnum;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Integration test: Mission starts in Loading state and transitions
#[test]
fn test_mission_state_transition() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>()
        .init_state::<MissionStateEnum>();

    // Default state should be Loading
    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(
        *state.get(),
        MissionStateEnum::Loading,
        "Mission should start in Loading state"
    );

    // Start a mission
    app.world_mut().send_event(MissionStarted {
        mission_id: MissionId::Mission01,
    });

    for _ in 0..5 {
        app.update();
    }

    // Should transition to Active
    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(
        *state.get(),
        MissionStateEnum::Active,
        "Mission should transition to Active state after start event"
    );
}

/// Integration test: Objective completes when progress meets requirements
#[test]
fn test_objective_completion() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<ObjectiveCompleted>()
        .add_event::<MissionEnded>()
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>()
        .init_state::<MissionStateEnum>();

    // Create a mission with an objective that's almost complete
    let objective = Objective {
        objective_id: ObjectiveId::Custom(1),
        objective_type: ObjectiveType::Gather,
        description: "Gather resources".to_string(),
        progress: 95.0,
        required_progress: 100.0,
        completed: false,
        rewards: MissionRewards {
            gold: 50,
            xp: 100,
            sovereignty_points: 5,
            item_unlock: None,
        },
    };

    let mission_entity = app
        .world_mut()
        .spawn(MissionStateComponent {
            mission_id: MissionId::Mission01,
            phase: MissionPhase::Active,
            objectives: vec![objective],
            current_objective_index: 0,
            timer: None,
            time_limit: None,
        })
        .id();

    // Set the objective progress to complete
    let mut mission = app
        .world_mut()
        .get_mut::<MissionStateComponent>(mission_entity)
        .unwrap();
    mission.objectives[0].progress = 100.0;

    // Run simulation and check for objective completed event during each frame
    let mut completed = false;

    for _ in 0..5 {
        app.update();
        let mut events = app.world_mut().resource_mut::<Events<ObjectiveCompleted>>();
        for event in events.drain() {
            if event.objective_id == ObjectiveId::Custom(1) {
                completed = true;
            }
        }
    }

    assert!(
        completed,
        "ObjectiveCompleted event should fire when progress meets requirement"
    );
}
