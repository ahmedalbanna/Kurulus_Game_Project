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

fn make_m6_objectives() -> Vec<Objective> {
    vec![
        Objective {
            objective_id: ObjectiveId::KillEnemyHero,
            objective_type: ObjectiveType::Kill,
            description: "Kill the Byzantine Commander".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 40,
                xp: 60,
                sovereignty_points: 2,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive the onslaught for 60 seconds".into(),
            progress: 0.0,
            required_progress: 60.0,
            completed: false,
            rewards: MissionRewards {
                gold: 30,
                xp: 50,
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
                gold: 100,
                xp: 150,
                sovereignty_points: 6,
                item_unlock: None,
            },
        },
    ]
}

fn spawn_m6_minimal(app: &mut App) {
    use std::collections::HashMap;

    // Osman (hero, level 4)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 4,
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
            current: 400.0,
            max: 400.0,
            armor: 20.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 50.0,
            attack_speed: 1.5,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 50.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.45,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // Orhan (hero, level 3)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Orhan,
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
        WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        Health {
            current: 280.0,
            max: 280.0,
            armor: 14.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 35.0,
            attack_speed: 1.3,
            vision_range: 10.0,
            morale: 95.0,
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

    // Alaeddin (hero, level 2)
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Alaeddin,
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
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(6, 6), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 22.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 85.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 22.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.25,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // 8 Kayi infantry
    for i in 0..8 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(
                IVec2::new(4 + (i % 4) as i32, 7 + (i / 4) as i32),
                CELL_SIZE,
            ),
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

    // 2 Kayi Light Cavalry
    for i in 0..2 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(3 + i as i32 * 2, 9), CELL_SIZE),
            Health {
                current: 100.0,
                max: 100.0,
                armor: 4.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 6.0,
                attack_range: 2.0,
                attack_damage: 15.0,
                attack_speed: 1.2,
                vision_range: 7.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Spear,
                range: 2.0,
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
        WorldPosition::new(IVec2::new(2, 10), CELL_SIZE),
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

    // Oba (Level 3)
    app.world_mut().spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 3,
            max_level: 5,
            name: "Field HQ Oba".into(),
        },
        WorldPosition::new(IVec2::new(6, 10), CELL_SIZE),
        Health {
            current: 1000.0,
            max: 1000.0,
            armor: 30.0,
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
        mission_id: MissionId::Mission06,
        phase: MissionPhase::Intro,
        objectives: make_m6_objectives(),
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(600.0),
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
fn test_m6_state_transition() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Intro,
        "M6 should start in Intro"
    );

    update_n(&mut app, 5);

    let data = read_mission_data(app.world_mut());
    assert_eq!(
        data.as_ref().unwrap().0,
        MissionPhase::Active,
        "M6 should become Active"
    );

    let state = app.world().resource::<State<MissionStateEnum>>();
    assert_eq!(*state.get(), MissionStateEnum::Active);
}

#[test]
fn test_m6_objective_kill_enemy_hero() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);

    // Spawn Byzantine Commander (HP > 200) BEFORE advancing
    app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(23, 5), CELL_SIZE),
        Health {
            current: 350.0,
            max: 350.0,
            armor: 25.0,
            armor_type: ArmorType::Heavy,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.3,
            vision_range: 9.0,
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

    update_n(&mut app, 5);

    // Obj 0: KillEnemyHero — commander alive, not complete
    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, false);

    // Kill the Byzantine Commander
    let commander: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Unit>>();
        q.iter(world)
            .find(|e| {
                world
                    .get::<Unit>(*e)
                    .is_some_and(|u| u.faction == FactionId::Byzantine)
            })
            .expect("Byzantine commander exists")
    };
    app.world_mut().entity_mut(commander).despawn();
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 0, true);
}

#[test]
fn test_m6_objective_survive_wave() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);
    update_n(&mut app, 5);

    // Fast-forward through obj 0 (KillEnemyHero) by spawning and killing commander
    let commander = app
        .world_mut()
        .spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(23, 5), CELL_SIZE),
            Health {
                current: 350.0,
                max: 350.0,
                armor: 25.0,
                armor_type: ArmorType::Heavy,
            },
            Stats {
                move_speed: 4.0,
                attack_range: 2.0,
                attack_damage: 40.0,
                attack_speed: 1.3,
                vision_range: 9.0,
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
        ))
        .id();
    // Kill it so obj 0 completes
    app.world_mut().entity_mut(commander).despawn();
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 0, true);

    // Obj 1: SurviveWave (60s). After a few frames, progress should be small.
    let data = read_mission_data(app.world_mut());
    assert!(
        data.as_ref().unwrap().1[1].1 < 1.0,
        "Survive progress should be small, got {}",
        data.as_ref().unwrap().1[1].1
    );

    // Fast-forward by setting progress manually
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
        mission.objectives[1].progress = 60.0;
    }
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 1, true);
}

#[test]
fn test_m6_objective_capture_region() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);

    // Spawn Byzantine units (including commander with HP > 200) BEFORE advancing
    app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(23, 5), CELL_SIZE),
        Health {
            current: 350.0,
            max: 350.0,
            armor: 25.0,
            armor_type: ArmorType::Heavy,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.3,
            vision_range: 9.0,
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
    for i in 0..3 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(22 + i as i32, 7), CELL_SIZE),
            Health {
                current: 90.0,
                max: 90.0,
                armor: 5.0,
                armor_type: ArmorType::Medium,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 65.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.15,
            },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    update_n(&mut app, 5);

    // Fast-forward through obj 0 by killing the commander (max HP > 200)
    let commander: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Unit>>();
        q.iter(world)
            .find(|e| {
                world
                    .get::<Unit>(*e)
                    .is_some_and(|u| u.faction == FactionId::Byzantine)
                    && world.get::<Health>(*e).is_some_and(|h| h.max > 200.0)
            })
            .expect("Byzantine commander exists")
    };
    app.world_mut().entity_mut(commander).despawn();
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 0, true);

    // Fast-forward through obj 1 (SurviveWave)
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
        mission.objectives[1].progress = 60.0;
    }
    update_n(&mut app, 5);
    assert_obj_completed(&read_mission_data(app.world_mut()), 1, true);

    // Obj 2: CaptureRegion — Byzantines still alive, not complete
    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 2, false);

    // Kill remaining Byzantine units
    let remaining: Vec<Entity> = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Unit>>();
        q.iter(world)
            .filter(|e| {
                world
                    .get::<Unit>(*e)
                    .is_some_and(|u| u.faction == FactionId::Byzantine)
            })
            .collect()
    };
    for e in remaining {
        app.world_mut().entity_mut(e).despawn();
    }
    update_n(&mut app, 3);

    let data = read_mission_data(app.world_mut());
    assert_obj_completed(&data, 2, true);
}

#[test]
fn test_m6_loss_osman_dead() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);
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
        check_mission_ended(app.world_mut(), MissionId::Mission06, false),
        "MissionEnded(failure) should fire when Osman dies"
    );
}

#[test]
fn test_m6_loss_orhan_dead() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);
    update_n(&mut app, 5);

    let orhan_entity: Entity = {
        let world = app.world_mut();
        let mut q = world.query_filtered::<Entity, With<Hero>>();
        q.iter(world)
            .find(|e| {
                world
                    .get::<Hero>(*e)
                    .is_some_and(|h| h.hero_id == HeroId::Orhan)
            })
            .expect("Orhan exists")
    };
    app.world_mut().entity_mut(orhan_entity).despawn();
    update_n(&mut app, 5);

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission06, false),
        "MissionEnded(failure) should fire when Orhan dies"
    );
}

#[test]
fn test_m6_loss_alaeddin_dead() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);
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
        check_mission_ended(app.world_mut(), MissionId::Mission06, false),
        "MissionEnded(failure) should fire when Alaeddin dies"
    );
}

#[test]
fn test_m6_loss_time_limit() {
    let mut app = make_app();
    spawn_m6_minimal(&mut app);
    update_n(&mut app, 5);

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
        mission.timer = Some(599.0);
    }

    update_n(&mut app, 120); // ~2 seconds

    assert!(
        check_mission_ended(app.world_mut(), MissionId::Mission06, false),
        "MissionEnded(failure) should fire when M6 time limit expires"
    );
}
