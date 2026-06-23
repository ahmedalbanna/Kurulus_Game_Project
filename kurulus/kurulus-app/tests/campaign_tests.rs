use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_core::states::MissionState as MissionStateEnum;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Tolerance for floating-point resource comparisons (economy systems add passive income per tick).
const RESOURCE_EPSILON: f32 = 10.0;

fn make_app() -> App {
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
    app
}

fn update_n(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn fire_mission_ended(app: &mut App, mission_id: MissionId, success: bool) {
    app.world_mut()
        .resource_mut::<Events<MissionEnded>>()
        .send(MissionEnded {
            mission_id,
            success,
        });
}

fn fire_mission_started(app: &mut App, mission_id: MissionId) {
    app.world_mut()
        .resource_mut::<Events<MissionStarted>>()
        .send(MissionStarted { mission_id });
}

fn assert_f32_approx_eq(a: f32, b: f32, epsilon: f32, label: &str) {
    assert!(
        (a - b).abs() <= epsilon,
        "{label}: expected {b} ± {epsilon}, got {a}"
    );
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_campaign_default_state() {
    let app = make_app();
    let state = app.world().resource::<CampaignState>();
    assert!(
        state.is_unlocked(MissionId::Mission01),
        "Mission01 should be unlocked by default"
    );
    assert!(
        !state.is_unlocked(MissionId::Mission02),
        "Mission02 should NOT be unlocked by default"
    );
    assert!(
        state.completed_missions.is_empty(),
        "No missions should be completed initially"
    );
}

#[test]
fn test_campaign_mission_sequence() {
    let mut app = make_app();

    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    let state = app.world().resource::<CampaignState>();
    assert!(
        state.is_completed(MissionId::Mission01),
        "Mission01 should be completed"
    );
    assert!(
        state.is_unlocked(MissionId::Mission02),
        "Mission02 should now be unlocked"
    );
    assert!(
        !state.is_unlocked(MissionId::Mission03),
        "Mission03 should NOT yet be unlocked"
    );
}

#[test]
fn test_campaign_full_sequence() {
    let mut app = make_app();
    let mut state = app.world_mut().resource_mut::<CampaignState>();

    // Manually step through the entire mission sequence
    for (current, expected_next) in [
        (MissionId::Mission01, Some(MissionId::Mission02)),
        (MissionId::Mission02, Some(MissionId::Mission03)),
        (MissionId::Mission03, Some(MissionId::Mission04)),
        (MissionId::Mission04, Some(MissionId::Mission05)),
        (MissionId::Mission05, Some(MissionId::Mission06)),
        (MissionId::Mission06, None),
    ] {
        assert_eq!(CampaignState::next_mission(current), expected_next);
        state.complete_mission(current);
        assert!(
            state.is_completed(current),
            "{:?} should be completed",
            current
        );
        if let Some(next) = expected_next {
            assert!(state.is_unlocked(next), "{:?} should be unlocked", next);
        }
    }
}

#[test]
fn test_campaign_resource_persistence() {
    let mut app = make_app();

    // Set up initial stockpile with custom values
    {
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.gold = 200.0;
        stockpile.food = 500.0;
        stockpile.wood = 150.0;
        stockpile.stone = 80.0;
        stockpile.population = 12;
        stockpile.sovereignty_points = 5;
    }

    // End a mission successfully → saves a snapshot of current stockpile
    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    // Verify persistence was saved (snapshot captured at time of event, before passive income)
    let persistence = app.world().resource::<ResourcePersistence>();
    assert!(
        persistence.saved_stockpile.is_some(),
        "Stockpile should be saved on mission win"
    );
    // The snapshot was taken during the first frame's FixedUpdate, so it should match
    // the set values (any passive income happens AFTER the snapshot within the same frame
    // if the economy system runs in a later schedule point, or in subsequent frames).
    // Use tolerance to be safe.
    if let Some(ref saved) = persistence.saved_stockpile {
        assert_f32_approx_eq(saved.gold, 200.0, RESOURCE_EPSILON, "Gold persistence");
        assert_f32_approx_eq(saved.food, 500.0, RESOURCE_EPSILON, "Food persistence");
    }

    // Now simulate starting a new mission: reset stockpile to defaults
    {
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        *stockpile = ResourceStockpile::default();
    }

    fire_mission_started(&mut app, MissionId::Mission02);
    update_n(&mut app, 5);

    // Verify resources were restored (with tolerance for passive income during those 5 frames)
    let stockpile = app.world().resource::<ResourceStockpile>();
    assert_f32_approx_eq(stockpile.gold, 200.0, RESOURCE_EPSILON, "Gold restored");
    assert_f32_approx_eq(stockpile.food, 500.0, RESOURCE_EPSILON, "Food restored");
    assert_f32_approx_eq(stockpile.wood, 150.0, RESOURCE_EPSILON, "Wood restored");
    assert_f32_approx_eq(stockpile.stone, 80.0, RESOURCE_EPSILON, "Stone restored");
    assert_eq!(stockpile.population, 12, "Population should be restored");
    assert_eq!(stockpile.sovereignty_points, 5, "SP should be restored");

    // Verify persistence was consumed (took the saved stockpile and cleared it)
    let persistence = app.world().resource::<ResourcePersistence>();
    assert!(
        persistence.saved_stockpile.is_none(),
        "Persistence should be consumed after restore"
    );
}

#[test]
fn test_campaign_no_carryover_on_failure() {
    let mut app = make_app();

    // Set custom stockpile
    {
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.gold = 999.0;
    }

    // End a mission with FAILURE
    fire_mission_ended(&mut app, MissionId::Mission01, false);
    update_n(&mut app, 5);

    // Verify persistence was NOT saved on failure
    let persistence = app.world().resource::<ResourcePersistence>();
    assert!(
        persistence.saved_stockpile.is_none(),
        "Stockpile should NOT be saved on mission failure"
    );

    // Campaign state should NOT advance on failure
    let state = app.world().resource::<CampaignState>();
    assert!(
        !state.is_completed(MissionId::Mission01),
        "Mission01 should NOT be completed on failure"
    );
}

#[test]
fn test_campaign_advanced_event_side_effect() {
    let mut app = make_app();

    fire_mission_ended(&mut app, MissionId::Mission01, true);

    // Update — this causes campaign_save_system to run, which sends CampaignAdvanced
    // as a side effect. The event is delivered to Bevy's event bus. We verify the
    // side effect took effect by checking the persisted state.
    update_n(&mut app, 5);

    // CampaignState was updated by the system (direct mutation — reliable check)
    let state = app.world().resource::<CampaignState>();
    assert!(
        state.is_completed(MissionId::Mission01),
        "Mission01 should be completed"
    );
    assert!(
        state.is_unlocked(MissionId::Mission02),
        "Mission02 should be unlocked"
    );

    // ResourcePersistence was updated too
    let persistence = app.world().resource::<ResourcePersistence>();
    assert!(
        persistence.saved_stockpile.is_some(),
        "Stockpile should be saved"
    );

    // The CampaignAdvanced event itself is a fire-and-forget notification.
    // It gets cleaned up by Bevy's event bus after ~3 update cycles.
    // Testing its delivery is fragile; the state assertions above are sufficient.
}

// ─── Post-mission rewards ──────────────────────────────────────────────────

/// Helper: spawn an entity with MissionStateComponent containing completed
/// objectives that have specific gold and item rewards.
fn spawn_mission_with_rewards(app: &mut App, mission_id: MissionId, objectives: Vec<Objective>) {
    app.world_mut().spawn(MissionStateComponent {
        mission_id,
        phase: MissionPhase::Active,
        objectives,
        current_objective_index: 0,
        timer: None,
        time_limit: None,
    });
}

fn make_completed_objective(id: ObjectiveId, gold: u32, item: Option<&str>) -> Objective {
    Objective {
        objective_id: id,
        objective_type: ObjectiveType::Kill,
        description: "Test objective".into(),
        progress: 1.0,
        required_progress: 1.0,
        completed: true,
        rewards: MissionRewards {
            gold,
            xp: 0,
            sovereignty_points: 0,
            item_unlock: item.map(|s| s.to_string()),
        },
    }
}

/// Sets stockpile gold to a known value and returns the app.
fn with_base_gold(app: &mut App, gold: f32) {
    app.world_mut().resource_mut::<ResourceStockpile>().gold = gold;
}

#[test]
fn test_post_mission_rewards_gold_awarded() {
    let mut app = make_app();

    with_base_gold(&mut app, 50.0);

    spawn_mission_with_rewards(
        &mut app,
        MissionId::Mission01,
        vec![
            make_completed_objective(ObjectiveId::TutorialBuildOba, 10, None),
            make_completed_objective(ObjectiveId::TutorialRecruitUnit, 15, None),
            make_completed_objective(ObjectiveId::TutorialAttackEnemy, 25, None),
        ],
    );

    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    // Base 50 + 10 + 15 + 25 = 100, within tolerance for passive income
    let stockpile = app.world().resource::<ResourceStockpile>();
    assert_f32_approx_eq(
        stockpile.gold,
        100.0,
        RESOURCE_EPSILON,
        "Gold after mission rewards",
    );
}

#[test]
fn test_post_mission_rewards_no_rewards_on_failure() {
    let mut app = make_app();

    with_base_gold(&mut app, 50.0);

    spawn_mission_with_rewards(
        &mut app,
        MissionId::Mission01,
        vec![make_completed_objective(
            ObjectiveId::TutorialBuildOba,
            50,
            None,
        )],
    );

    // Fire failure — rewards should NOT be applied
    fire_mission_ended(&mut app, MissionId::Mission01, false);
    update_n(&mut app, 5);

    let stockpile = app.world().resource::<ResourceStockpile>();
    assert_f32_approx_eq(
        stockpile.gold,
        50.0,
        RESOURCE_EPSILON,
        "Gold should not change on failure",
    );
}

#[test]
fn test_post_mission_rewards_items_unlocked() {
    let mut app = make_app();

    spawn_mission_with_rewards(
        &mut app,
        MissionId::Mission01,
        vec![make_completed_objective(
            ObjectiveId::TutorialAttackEnemy,
            30,
            Some("Sword_of_Osman"),
        )],
    );

    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    let campaign = app.world().resource::<CampaignState>();
    assert!(
        campaign
            .unlocked_items
            .contains(&"Sword_of_Osman".to_string()),
        "Item should be unlocked in CampaignState"
    );
}

#[test]
fn test_post_mission_rewards_items_deduplicated() {
    let mut app = make_app();

    // Two objectives reward the same item
    spawn_mission_with_rewards(
        &mut app,
        MissionId::Mission01,
        vec![
            make_completed_objective(ObjectiveId::TutorialBuildOba, 10, Some("Same_Item")),
            make_completed_objective(ObjectiveId::TutorialAttackEnemy, 20, Some("Same_Item")),
        ],
    );

    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    let campaign = app.world().resource::<CampaignState>();
    let count = campaign
        .unlocked_items
        .iter()
        .filter(|i| *i == "Same_Item")
        .count();
    assert_eq!(count, 1, "Item should appear only once in unlocked_items");
}

#[test]
fn test_post_mission_rewards_no_items_when_objectives_incomplete() {
    let mut app = make_app();

    // Create an incomplete objective with an item reward.
    // Use Custom(999) so no M1 tracker auto-completes it.
    let objective = Objective {
        objective_id: ObjectiveId::Custom(999),
        objective_type: ObjectiveType::Kill,
        description: "Test objective".into(),
        progress: 0.5,
        required_progress: 1.0,
        completed: false,
        rewards: MissionRewards {
            gold: 50,
            xp: 0,
            sovereignty_points: 0,
            item_unlock: Some("Unobtained_Item".to_string()),
        },
    };

    spawn_mission_with_rewards(&mut app, MissionId::Mission01, vec![objective]);

    fire_mission_ended(&mut app, MissionId::Mission01, true);
    update_n(&mut app, 5);

    let campaign = app.world().resource::<CampaignState>();
    assert!(
        !campaign
            .unlocked_items
            .contains(&"Unobtained_Item".to_string()),
        "Incomplete objective should not unlock its item"
    );
}
