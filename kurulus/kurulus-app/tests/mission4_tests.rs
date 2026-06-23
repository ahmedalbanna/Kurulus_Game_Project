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

fn make_m4_objectives() -> Vec<Objective> {
    vec![
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive the Mongol scout assault for 30 seconds".into(),
            progress: 0.0,
            required_progress: 30.0,
            completed: false,
            rewards: MissionRewards {
                gold: 15,
                xp: 25,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::DestroyWall,
            objective_type: ObjectiveType::Destroy,
            description: "Destroy the Mongol Sentry Post".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 35,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RescueAlly,
            objective_type: ObjectiveType::Rescue,
            description: "Rescue the captured Kayi scout".into(),
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
    ]
}

fn spawn_m4_minimal(app: &mut App) {
    use std::collections::HashMap;

    // Osman (hero, level 3)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 3,
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
            current: 300.0,
            max: 300.0,
            armor: 15.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.4,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 40.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.4,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // Alaeddin (hero, level 1)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Alaeddin,
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
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(6, 6), CELL_SIZE),
        Health {
            current: 150.0,
            max: 150.0,
            armor: 6.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 18.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 80.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 18.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.2,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // 3 Kayi infantry
    for i in 0..3 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(4 + i as i32, 7), CELL_SIZE),
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
        WorldPosition::new(IVec2::new(3, 8), CELL_SIZE),
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
            name: "Kayı Obağı — Frontier".into(),
        },
        WorldPosition::new(IVec2::new(5, 9), CELL_SIZE),
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
            population: 0,
        },
    ));

    // Mission State
    app.world_mut().spawn(MissionStateComponent {
        mission_id: MissionId::Mission04,
        phase: MissionPhase::Intro,
        objectives: make_m4_objectives(),
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(300.0),
    });
}

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

#[allow(dead_code)]
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
fn test_m4_state_transition() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Intro,
        "M4 should start in Intro"
    );

    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Active,
        "M4 should become Active"
    );

    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(*state.get(), MissionStateEnum::Active);
}

#[test]
fn test_m4_objective_survive_wave() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(data.as_ref().unwrap().1[0].2, 30.0); // required_progress
    assert!(
        data.as_ref().unwrap().1[0].1 < 0.5,
        "Progress should be near 0, got {}",
        data.as_ref().unwrap().1[0].1
    );

    // Run 30 seconds of game time (1800 frames at 1/60)
    update_n(&mut app, 1800);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
    assert!(
        (data.as_ref().unwrap().1[0].1 - 30.0).abs() < 1.0,
        "Progress should be ~30.0, got {}",
        data.as_ref().unwrap().1[0].1
    );
}

#[test]
fn test_m4_objective_destroy_sentry() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    // Spawn a Mongol Watchtower BEFORE completing obj 0, so the destroy
    // tracker sees it when it reaches index 1.
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Watchtower,
            faction: FactionId::Mongol,
            level: 1,
            max_level: 1,
            name: "Mongol Sentry".into(),
        },
        WorldPosition::new(IVec2::new(22, 6), CELL_SIZE),
        Health {
            current: 250.0,
            max: 250.0,
            armor: 25.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            population: 0,
        },
    ));

    // Fast-forward through obj 0 (SurviveWave) by setting progress
    let mission_entity: Entity = {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<MissionStateComponent>>();
        q.iter(app.world_mut()).next().expect("Mission entity")
    };
    {
        let mut mission = app
            .world_mut()
            .get_mut::<MissionStateComponent>(mission_entity)
            .unwrap();
        mission.objectives[0].progress = 30.0;
    }
    update_n(&mut app, 3);
    assert_obj_completed(&read_mission_data(app.world_mut()), 0, true);

    // Obj 1: DestroyWall. Watchtower still exists → not complete.
    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 1, false);

    // Destroy the Watchtower
    let sentry: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Building>>();
        q.iter(world)
            .find(|e| {
                world.get::<Building>(*e).is_some_and(|b| {
                    b.building_type == BuildingType::Watchtower && b.faction == FactionId::Mongol
                })
            })
            .expect("Mongol Watchtower exists")
    };
    app.world_mut().entity_mut(sentry).despawn();
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 1, true);
}

#[test]
fn test_m4_objective_rescue_ally() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    // Spawn the captured scout (RescueTarget) BEFORE advancing objectives
    app.world_mut().spawn((
        RescueTarget { extracted: false },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(21, 5), CELL_SIZE),
        Health {
            current: 40.0,
            max: 40.0,
            armor: 2.0,
            armor_type: ArmorType::Light,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 8.0,
            attack_speed: 0.8,
            vision_range: 5.0,
            morale: 20.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 8.0,
            weapon_type: WeaponType::Unarmed,
            range: 1.0,
            armor_penetration: 0.0,
        },
        AttackTimer(Timer::from_seconds(1.5, TimerMode::Repeating)),
    ));

    // Fast-forward through obj 0 (SurviveWave) and obj 1 (DestroyWall)
    let mission_entity: Entity = {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<MissionStateComponent>>();
        q.iter(app.world_mut()).next().expect("Mission entity")
    };
    {
        let mut mission = app
            .world_mut()
            .get_mut::<MissionStateComponent>(mission_entity)
            .unwrap();
        mission.objectives[0].progress = 30.0;
    }
    // Obj 1 auto-completes (no Watchtower exists), advancing index to 2 (RescueAlly)
    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
    assert_obj_completed(&data, 1, true);
    // With RescueTarget alive, obj 2 is NOT complete
    assert_obj_completed(&data, 2, false);

    // "Rescue" the scout by despawning it (escorted to safety)
    let scout: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<RescueTarget>>();
        q.iter(world).next().expect("RescueTarget exists")
    };
    app.world_mut().entity_mut(scout).despawn();
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 2, true);
}

#[test]
fn test_m4_loss_osman_dead() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    let osman_entity: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Hero>>();
        q.iter(world)
            .find(|e| {
                world
                    .get::<Hero>(*e)
                    .is_some_and(|h| h.hero_id == HeroId::Osman)
            })
            .expect("Osman exists")
    };
    app.world_mut().entity_mut(osman_entity).despawn();
    update_n(&mut app, 5);

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission04, false),
        "MissionEnded(failure) should fire when Osman dies"
    );
}

#[test]
fn test_m4_loss_alaeddin_dead() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    let alaeddin_entity: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Hero>>();
        q.iter(world)
            .find(|e| {
                world
                    .get::<Hero>(*e)
                    .is_some_and(|h| h.hero_id == HeroId::Alaeddin)
            })
            .expect("Alaeddin exists")
    };
    app.world_mut().entity_mut(alaeddin_entity).despawn();
    update_n(&mut app, 5);

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission04, false),
        "MissionEnded(failure) should fire when Alaeddin dies"
    );
}

#[test]
fn test_m4_loss_time_limit() {
    let mut app = make_app();
    spawn_m4_minimal(&mut app);
    update_n(&mut app, 5);

    // Set timer close to 300s limit
    let mission_entity: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<MissionStateComponent>>();
        q.iter(world).next().expect("Mission entity exists")
    };
    {
        let mut mission = app
            .world_mut()
            .get_mut::<MissionStateComponent>(mission_entity)
            .unwrap();
        mission.timer = Some(299.0);
    }

    update_n(&mut app, 120); // ~2 seconds

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission04, false),
        "MissionEnded(failure) should fire when M4 time limit expires"
    );
}
