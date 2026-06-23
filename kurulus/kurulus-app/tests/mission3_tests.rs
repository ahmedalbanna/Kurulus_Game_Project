use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_core::states::MissionState as MissionStateEnum;
use kurulus_core::components::MissionStateComponent;
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

fn make_m3_objectives() -> Vec<Objective> {
    vec![
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive for 90 seconds".into(),
            progress: 0.0,
            required_progress: 90.0,
            completed: false,
            rewards: MissionRewards { gold: 20, xp: 30, sovereignty_points: 1, item_unlock: None },
        },
        Objective {
            objective_id: ObjectiveId::RecruitTenUnits,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit 3 Light Cavalry".into(),
            progress: 0.0,
            required_progress: 3.0,
            completed: false,
            rewards: MissionRewards { gold: 25, xp: 35, sovereignty_points: 1, item_unlock: None },
        },
        Objective {
            objective_id: ObjectiveId::Custom(2),
            objective_type: ObjectiveType::Kill,
            description: "Defeat all Mongol raiders".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards { gold: 60, xp: 100, sovereignty_points: 4, item_unlock: None },
        },
    ]
}

fn spawn_m3_minimal(app: &mut App) {
    use std::collections::HashMap;

    // Osman (hero, level 3)
    app.world_mut().spawn((
        Hero { hero_id: HeroId::Osman, level: 3, xp: 0, skill_points: 0, unlocked_abilities: vec![], active_cooldowns: HashMap::new(), ..Default::default() },
        Unit { archetype: UnitArchetype::LightInfantry, faction: FactionId::Kayi, formation: FormationType::Line, stance: CombatStance::Aggressive },
        WorldPosition::new(IVec2::new(6, 5), CELL_SIZE),
        Health { current: 300.0, max: 300.0, armor: 15.0, armor_type: ArmorType::Medium },
        Stats { move_speed: 4.0, attack_range: 2.0, attack_damage: 40.0, attack_speed: 1.4, vision_range: 10.0, morale: 100.0, morale_max: 100.0 },
        MeleeWeapon { damage: 40.0, weapon_type: WeaponType::Sword, range: 2.0, armor_penetration: 0.4 },
        AttackTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
    ));

    // Bamsi (hero, level 2)
    app.world_mut().spawn((
        Hero { hero_id: HeroId::Bamsi, level: 2, xp: 0, skill_points: 0, unlocked_abilities: vec![], active_cooldowns: HashMap::new(), ..Default::default() },
        Unit { archetype: UnitArchetype::LightInfantry, faction: FactionId::Kayi, formation: FormationType::Line, stance: CombatStance::Aggressive },
        WorldPosition::new(IVec2::new(5, 4), CELL_SIZE),
        Health { current: 220.0, max: 220.0, armor: 12.0, armor_type: ArmorType::Medium },
        Stats { move_speed: 4.5, attack_range: 2.0, attack_damage: 28.0, attack_speed: 1.3, vision_range: 10.0, morale: 95.0, morale_max: 100.0 },
        MeleeWeapon { damage: 28.0, weapon_type: WeaponType::Spear, range: 2.0, armor_penetration: 0.2 },
        AttackTimer(Timer::from_seconds(0.85, TimerMode::Repeating)),
    ));

    // 2 Kayi infantry
    for i in 0..2 {
        app.world_mut().spawn((
            Unit { archetype: UnitArchetype::LightInfantry, faction: FactionId::Kayi, formation: FormationType::Line, stance: CombatStance::Defensive },
            WorldPosition::new(IVec2::new(4 + i as i32, 6), CELL_SIZE),
            Health { current: 80.0, max: 80.0, armor: 3.0, armor_type: ArmorType::Light },
            Stats { move_speed: 3.5, attack_range: 1.5, attack_damage: 12.0, attack_speed: 1.0, vision_range: 6.0, morale: 50.0, morale_max: 100.0 },
            MeleeWeapon { damage: 12.0, weapon_type: WeaponType::Sword, range: 1.5, armor_penetration: 0.1 },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    // Oba (Level 1)
    app.world_mut().spawn((
        Building { building_type: BuildingType::Oba, faction: FactionId::Kayi, level: 1, max_level: 5, name: "Kayı Obağı".into() },
        WorldPosition::new(IVec2::new(6, 8), CELL_SIZE),
        Health { current: 500.0, max: 500.0, armor: 20.0, armor_type: ArmorType::Fortification },
        BuildCost { food: 0, gold: 0, wood: 0, stone: 0, population: 0 },
    ));

    // Stable
    app.world_mut().spawn((
        Building { building_type: BuildingType::Stable, faction: FactionId::Kayi, level: 1, max_level: 3, name: "Kayı Stable".into() },
        WorldPosition::new(IVec2::new(4, 8), CELL_SIZE),
        Health { current: 200.0, max: 200.0, armor: 10.0, armor_type: ArmorType::Fortification },
        BuildCost { food: 0, gold: 0, wood: 0, stone: 0, population: 0 },
    ));

    // Mission State
    app.world_mut().spawn(MissionStateComponent {
        mission_id: MissionId::Mission03,
        phase: MissionPhase::Intro,
        objectives: make_m3_objectives(),
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(300.0),
    });
}

fn read_mission_data(world: &mut World) -> Option<(MissionPhase, Vec<(ObjectiveId, f32, f32, bool)>, Option<f32>)> {
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

#[allow(dead_code)]
fn assert_obj_progress(
    data: &Option<(MissionPhase, Vec<(ObjectiveId, f32, f32, bool)>, Option<f32>)>,
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
    data: &Option<(MissionPhase, Vec<(ObjectiveId, f32, f32, bool)>, Option<f32>)>,
    idx: usize,
    completed: bool,
) {
    let (_, ref objectives, _) = data.as_ref().expect("Mission exists");
    assert_eq!(objectives[idx].3, completed, "Obj {} completed mismatch", idx);
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_m3_state_transition() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);

    let data = read_mission_data(app.world_mut());
    assert_eq!(data.as_ref().unwrap().0, MissionPhase::Intro, "M3 should start in Intro");

    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(data.as_ref().unwrap().0, MissionPhase::Active, "M3 should become Active");

    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(*state.get(), MissionStateEnum::Active);
}

#[test]
fn test_m3_objective_survive_wave() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(data.as_ref().unwrap().1[0].2, 90.0); // required_progress
    // After 5 frames, some survive time has accumulated (~0.083s)
    assert!(
        data.as_ref().unwrap().1[0].1 < 0.5,
        "Progress should be near 0, got {}",
        data.as_ref().unwrap().1[0].1
    );

    // Run 90 seconds of game time (5400 frames at 1/60)
    update_n(&mut app, 5400);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
    assert!(
        (data.as_ref().unwrap().1[0].1 - 90.0).abs() < 1.0,
        "Progress should be ~90.0, got {}",
        data.as_ref().unwrap().1[0].1
    );
}

#[test]
fn test_m3_objective_recruit_cavalry() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    // Fast-forward through obj 0 (SurviveWave) by setting progress to required
    let mission_entity: Entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<MissionStateComponent>>();
        q.iter(app.world_mut()).next().expect("Mission entity")
    };
    {
        let mut mission = app.world_mut().get_mut::<MissionStateComponent>(mission_entity).unwrap();
        // Complete obj 0 by advancing timer past required
        mission.objectives[0].progress = 90.0;
    }
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true); // obj 0 should be completed now

    // Add 3 Kayi Light Cavalry
    for i in 0..3 {
        app.world_mut().spawn((
            Unit { archetype: UnitArchetype::LightCavalry, faction: FactionId::Kayi, formation: FormationType::Line, stance: CombatStance::Aggressive },
            WorldPosition::new(IVec2::new(10 + i as i32, 10), CELL_SIZE),
            Health { current: 100.0, max: 100.0, armor: 4.0, armor_type: ArmorType::Light },
            Stats { move_speed: 6.0, attack_range: 2.0, attack_damage: 15.0, attack_speed: 1.2, vision_range: 7.0, morale: 60.0, morale_max: 100.0 },
            MeleeWeapon { damage: 15.0, weapon_type: WeaponType::Spear, range: 2.0, armor_penetration: 0.1 },
            AttackTimer(Timer::from_seconds(0.9, TimerMode::Repeating)),
        ));
    }

    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 1, true);
}

#[test]
fn test_m3_objective_defeat_mongols() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    // Spawn both LightCavalry (for obj 1) and Mongol units (for obj 2) BEFORE
    // advancing objectives, so the trackers see non-zero counts at the right time.
    for i in 0..3 {
        app.world_mut().spawn((
            Unit { archetype: UnitArchetype::LightCavalry, faction: FactionId::Kayi, formation: FormationType::Line, stance: CombatStance::Aggressive },
            WorldPosition::new(IVec2::new(16 + i as i32, 12), CELL_SIZE),
            Health { current: 100.0, max: 100.0, armor: 4.0, armor_type: ArmorType::Light },
            Stats { move_speed: 6.0, attack_range: 2.0, attack_damage: 15.0, attack_speed: 1.2, vision_range: 7.0, morale: 60.0, morale_max: 100.0 },
            MeleeWeapon { damage: 15.0, weapon_type: WeaponType::Spear, range: 2.0, armor_penetration: 0.1 },
            AttackTimer(Timer::from_seconds(0.9, TimerMode::Repeating)),
        ));
    }
    for i in 0..3 {
        app.world_mut().spawn((
            Unit { archetype: UnitArchetype::LightCavalry, faction: FactionId::Mongol, formation: FormationType::Line, stance: CombatStance::Aggressive },
            WorldPosition::new(IVec2::new(20 + i as i32, 5), CELL_SIZE),
            Health { current: 70.0, max: 70.0, armor: 2.0, armor_type: ArmorType::Light },
            Stats { move_speed: 6.0, attack_range: 2.0, attack_damage: 18.0, attack_speed: 1.3, vision_range: 8.0, morale: 60.0, morale_max: 100.0 },
            MeleeWeapon { damage: 18.0, weapon_type: WeaponType::Spear, range: 2.0, armor_penetration: 0.1 },
            AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
        ));
    }

    // Advance obj 0 progress so it completes (then obj 1 tracker sees 3 cavalry)
    let mission_entity: Entity = {
        let mut q = app.world_mut().query_filtered::<Entity, With<MissionStateComponent>>();
        q.iter(app.world_mut()).next().expect("Mission entity")
    };
    {
        let mut mission = app.world_mut().get_mut::<MissionStateComponent>(mission_entity).unwrap();
        mission.objectives[0].progress = 90.0;
    }
    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
    assert_obj_completed(&data, 1, true);

    // Index is now 2 (Custom(2) — defeat Mongols). Mongols are alive, so not complete.
    assert_obj_completed(&data, 2, false);

    // Kill all Mongol units
    let mongol_entities: Vec<Entity> = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Unit>>();
        query
            .iter(world)
            .filter(|e| world.get::<Unit>(*e).is_some_and(|u| u.faction == FactionId::Mongol))
            .collect()
    };

    for e in mongol_entities {
        app.world_mut().entity_mut(e).despawn();
    }

    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 2, true);
}

#[test]
fn test_m3_loss_oba_destroyed() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    // Find Oba
    let oba_entity: Entity = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Building>>();
        query
            .iter(world)
            .find(|e| {
                world
                    .get::<Building>(*e)
                    .is_some_and(|b| b.building_type == BuildingType::Oba && b.faction == FactionId::Kayi)
            })
            .expect("Oba should exist")
    };

    app.world_mut().entity_mut(oba_entity).despawn();
    update_n(&mut app, 3);

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission03, false),
        "MissionEnded(failure) should fire when Oba destroyed"
    );
}

#[test]
fn test_m3_loss_all_units_dead() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    // Collect all Kayi units
    let kayi_entities: Vec<Entity> = {
        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Unit>>();
        query
            .iter(world)
            .filter(|e| world.get::<Unit>(*e).is_some_and(|u| u.faction == FactionId::Kayi))
            .collect()
    };

    for e in kayi_entities {
        app.world_mut().entity_mut(e).despawn();
    }

    update_n(&mut app, 3);

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission03, false),
        "MissionEnded(failure) should fire when all Kayi units dead"
    );
}

#[test]
fn test_m3_timer_tracking() {
    let mut app = make_app();
    spawn_m3_minimal(&mut app);
    update_n(&mut app, 5);

    // Run 1 second (60 frames at 1/60)
    update_n(&mut app, 60);

    let data = read_mission_data(app.world_mut());
    let timer = data.as_ref().and_then(|(_, _, t)| *t).expect("Timer should be Some");
    assert!(
        (timer - 1.0).abs() < 0.15,
        "Timer should be ~1.0 after 60 frames, got {}",
        timer
    );
}
