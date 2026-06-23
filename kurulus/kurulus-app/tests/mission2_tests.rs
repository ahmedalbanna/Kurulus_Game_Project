use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::components::MissionStateComponent;
use kurulus_core::prelude::*;
use kurulus_core::states::MissionState as MissionStateEnum;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

// ─── Helpers ───────────────────────────────────────────────────────────────

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

fn make_m2_objectives() -> Vec<Objective> {
    vec![
        Objective {
            objective_id: ObjectiveId::BuildBarracks,
            objective_type: ObjectiveType::Build,
            description: "Construct a Barracks near the Oba".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 15,
                xp: 25,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RecruitTenUnits,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit 5 Light Infantry".into(),
            progress: 0.0,
            required_progress: 5.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 30,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::DestroyWall,
            objective_type: ObjectiveType::Destroy,
            description: "Destroy the Byzantine Watchtower".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 25,
                xp: 40,
                sovereignty_points: 2,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::CaptureRegion,
            objective_type: ObjectiveType::Capture,
            description: "Defeat all Byzantine forces".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 50,
                xp: 80,
                sovereignty_points: 3,
                item_unlock: None,
            },
        },
    ]
}

fn spawn_m2_minimal(app: &mut App) {
    use std::collections::HashMap;

    // Osman (hero)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 2,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        Health {
            current: 250.0,
            max: 250.0,
            armor: 10.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 35.0,
            attack_speed: 1.3,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 35.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.35,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // Orhan (hero)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Orhan,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(3, 5), CELL_SIZE),
        Health {
            current: 180.0,
            max: 180.0,
            armor: 8.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 25.0,
            attack_speed: 1.1,
            vision_range: 10.0,
            morale: 90.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 25.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.25,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // 4 Kayi infantry
    for i in 0..4 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(3 + i as i32, 6), CELL_SIZE),
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
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    // 1 Worker
    app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::Worker,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(2, 7), CELL_SIZE),
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
        AttackTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
    ));

    // Oba (Level 2)
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 2,
            max_level: 5,
            name: "Kayı Obağı".into(),
        },
        WorldPosition::new(IVec2::new(5, 8), CELL_SIZE),
        Health {
            current: 750.0,
            max: 750.0,
            armor: 25.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // Mission State
    app.world_mut().spawn(MissionStateComponent {
        mission_id: MissionId::Mission02,
        phase: MissionPhase::Intro,
        objectives: make_m2_objectives(),
        current_objective_index: 0,
        timer: None,
        time_limit: Some(480.0),
    });
}

/// Collects mission data without holding borrows across calls.
fn read_mission_data(
    world: &mut World,
) -> Option<(
    MissionPhase,
    Vec<(ObjectiveId, f32, f32, bool)>,
    Option<f32>,
)> {
    let mut query = world.query::<&MissionStateComponent>();
    let items: Vec<_> = query
        .iter(world)
        .map(|m| {
            let phase = m.phase;
            let objectives: Vec<_> = m
                .objectives
                .iter()
                .map(|o| (o.objective_id, o.progress, o.required_progress, o.completed))
                .collect();
            (phase, objectives, m.timer)
        })
        .collect();
    items.into_iter().next()
}

fn check_mission_ended(world: &mut World, mission_id: MissionId, expect_success: bool) -> bool {
    world
        .resource_mut::<Events<MissionEnded>>()
        .drain()
        .any(|e| e.mission_id == mission_id && e.success == expect_success)
}

fn update_n(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn assert_obj_progress(
    data: &Option<(
        MissionPhase,
        Vec<(ObjectiveId, f32, f32, bool)>,
        Option<f32>,
    )>,
    idx: usize,
    expected_id: ObjectiveId,
    expected: f32,
) {
    let (_, ref objectives, _) = data.as_ref().expect("Mission exists");
    assert_eq!(objectives[idx].0, expected_id);
    assert!(
        (objectives[idx].1 - expected).abs() < 0.01,
        "Obj {} progress: expected {}, got {}",
        idx,
        expected,
        objectives[idx].1
    );
}

fn assert_obj_completed(
    data: &Option<(
        MissionPhase,
        Vec<(ObjectiveId, f32, f32, bool)>,
        Option<f32>,
    )>,
    idx: usize,
    completed: bool,
) {
    let (_, ref objectives, _) = data.as_ref().expect("Mission exists");
    assert_eq!(
        objectives[idx].3, completed,
        "Obj {} completed mismatch",
        idx
    );
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_m2_state_transition() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Intro,
        "M2 should start in Intro"
    );

    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Active,
        "M2 should become Active"
    );

    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(*state.get(), MissionStateEnum::Active);
}

#[test]
fn test_m2_objective_build_barracks() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);
    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_obj_progress(&data, 0, ObjectiveId::BuildBarracks, 0.0);

    // Spawn a Kayi Barracks
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Barracks,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: "Test Barracks".into(),
        },
        WorldPosition::new(IVec2::new(5, 9), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
}

#[test]
fn test_m2_objective_recruit_infantry() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);
    update_n(&mut app, 5);

    // Advance current_objective_index to 1 (RecruitTenUnits) by
    // completing objective 0 first: spawn a Barracks.
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Barracks,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: "Test Barracks".into(),
        },
        WorldPosition::new(IVec2::new(5, 9), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    update_n(&mut app, 5); // complete obj 0, advance to index 1

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
    assert_obj_progress(&data, 1, ObjectiveId::RecruitTenUnits, 0.0);

    // Add 5 more Kayi Light Infantry (starting count: 7, after adding: 12)
    for i in 0..5 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(10 + i as i32, 10), CELL_SIZE),
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
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 1, true);
}

#[test]
fn test_m2_objective_capture_region() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);

    // Spawn Byzantine units BEFORE advancing objectives, so the capture
    // tracker sees alive enemies when it eventually reaches index 3.
    for i in 0..3 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(15 + i as i32, 5), CELL_SIZE),
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 10.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 40.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 10.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    update_n(&mut app, 5);

    // Complete obj 0: BuildBarracks
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Barracks,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: "Barracks".into(),
        },
        WorldPosition::new(IVec2::new(5, 9), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 0, true);

    // Obj 1: RecruitTenUnits — add 5 infantry to exceed starting 7
    for i in 0..5 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(10 + i as i32, 10), CELL_SIZE),
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
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 1, true);

    // Obj 2: DestroyWall — this will auto-complete (no watchtower exists),
    // advancing index to 3. The capture tracker will see alive Byzantines.
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 2, true);

    // Now index should be 3 — CaptureRegion. With Byzantines alive, not complete.
    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 3, false); // enemies still exist → not complete

    // Kill all Byzantine units
    let byz_entities: Vec<Entity> = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Unit>>();
        query
            .iter(world)
            .filter(|e| {
                world
                    .get::<Unit>(*e)
                    .is_some_and(|u| u.faction == FactionId::Byzantine)
            })
            .collect()
    };
    for e in byz_entities {
        app.world_mut().entity_mut(e).despawn();
    }

    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 3, true);
}

#[test]
fn test_m2_loss_osman_dead() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);
    update_n(&mut app, 5);

    // Find and kill Osman in a scoped block
    let osman_entity: Entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Hero>>();
        query
            .iter(world)
            .find(|e| {
                world
                    .get::<Hero>(*e)
                    .is_some_and(|h| h.hero_id == HeroId::Osman)
            })
            .expect("Osman should exist")
    };

    app.world_mut().entity_mut(osman_entity).despawn();
    update_n(&mut app, 5); // extra frames to propagate through event buffers

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission02, false),
        "MissionEnded(failure) should fire when Osman dies"
    );
}

#[test]
fn test_m2_loss_time_limit() {
    let mut app = make_app();
    spawn_m2_minimal(&mut app);
    update_n(&mut app, 5);

    // Set timer close to 480s limit
    let mission_entity: Entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<MissionStateComponent>>();
        query.iter(world).next().expect("Mission entity exists")
    };

    {
        let mut mission = app
            .world_mut()
            .get_mut::<MissionStateComponent>(mission_entity)
            .unwrap();
        mission.timer = Some(479.0);
    }

    update_n(&mut app, 120); // ~2 seconds

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission02, false),
        "MissionEnded(failure) should fire when M2 time limit expires"
    );
}
