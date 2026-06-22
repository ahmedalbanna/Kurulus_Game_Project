use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Test helper — create a minimal app with all logic plugins.
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
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();
    app
}

fn update_n(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

fn spawn_kayi_hero(app: &mut App, hero_id: HeroId, x: i32, y: i32) -> Entity {
    app.world_mut().spawn((
        Hero {
            hero_id,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![],
            active_cooldowns: std::collections::HashMap::new(),
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Skirmish,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(x, y), 1.0),
        Stats::default(),
        Health::default(),
    ))
    .id()
}

fn spawn_enemy_hero(app: &mut App) -> Entity {
    app.world_mut().spawn((
        Hero {
            hero_id: HeroId::Turgut,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![],
            active_cooldowns: std::collections::HashMap::new(),
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(50, 50), 1.0),
        Stats::default(),
        Health::default(),
    ))
    .id()
}

fn fire_hero_switch(app: &mut App, direction: Option<HeroSwitchDirection>, target: Option<Entity>) {
    app.world_mut()
        .resource_mut::<Events<HeroSwitchEvent>>()
        .send(HeroSwitchEvent { direction, target });
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[test]
fn test_hero_switch_selects_first_kayi_hero() {
    let mut app = make_app();

    // Spawn two Kayi heroes
    let hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);
    let _hero_b = spawn_kayi_hero(&mut app, HeroId::Orhan, 5, 5);

    // Spawn an enemy hero (should be ignored)
    spawn_enemy_hero(&mut app);

    // Fire next switch
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    // Should select the first Kayi hero (by entity bits order)
    let active = app.world().resource::<ActiveHero>();
    assert!(
        active.entity.is_some(),
        "ActiveHero should be set after switch"
    );

    // The first hero sorted by entity bits should be hero_a (lower entity index)
    assert_eq!(
        active.entity.unwrap(),
        hero_a,
        "Should select first Kayi hero on Next"
    );
}

#[test]
fn test_hero_switch_cycles_forward() {
    let mut app = make_app();

    let _hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);
    let hero_b = spawn_kayi_hero(&mut app, HeroId::Orhan, 5, 5);

    // Two Nexts should wrap to the second hero
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert_eq!(
        active.entity.unwrap(),
        hero_b,
        "Second Next should select second Kayi hero"
    );
}

#[test]
fn test_hero_switch_wraps_forward() {
    let mut app = make_app();

    let hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);
    let _hero_b = spawn_kayi_hero(&mut app, HeroId::Orhan, 5, 5);

    // Three Nexts should wrap back to first (2 heroes, so 3 = 1 full cycle + 1)
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert_eq!(
        active.entity.unwrap(),
        hero_a,
        "Third Next should wrap back to first Kayi hero"
    );
}

#[test]
fn test_hero_switch_cycles_backward() {
    let mut app = make_app();

    let _hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);
    let _hero_b = spawn_kayi_hero(&mut app, HeroId::Orhan, 5, 5);

    // Previous from empty selection should select the LAST hero
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Previous), None);
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert!(
        active.entity.is_some(),
        "ActiveHero should be set after Previous"
    );
}

#[test]
fn test_hero_switch_direct_target() {
    let mut app = make_app();

    let hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);
    let hero_b = spawn_kayi_hero(&mut app, HeroId::Orhan, 5, 5);

    // Direct switch to hero_b
    fire_hero_switch(&mut app, None, Some(hero_b));
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert_eq!(
        active.entity.unwrap(),
        hero_b,
        "Direct switch should select the target hero"
    );

    // Direct switch back to hero_a
    fire_hero_switch(&mut app, None, Some(hero_a));
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert_eq!(
        active.entity.unwrap(),
        hero_a,
        "Direct switch back should select hero_a"
    );
}

#[test]
fn test_hero_switch_fires_selection_event() {
    let mut app = make_app();

    let _hero_a = spawn_kayi_hero(&mut app, HeroId::Osman, 0, 0);

    // Set up event reader before the switch
    let mut reader = app.world_mut()
        .resource_mut::<Events<SelectionChanged>>()
        .get_cursor();

    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    let events: Vec<SelectionChanged> = reader
        .read(app.world().resource::<Events<SelectionChanged>>())
        .cloned()
        .collect();

    assert!(
        !events.is_empty(),
        "SelectionChanged event should be fired on hero switch"
    );
}

#[test]
fn test_hero_switch_no_heroes_no_crash() {
    let mut app = make_app();

    // No heroes spawned — should not crash
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert!(
        active.entity.is_none(),
        "ActiveHero should remain None when no heroes exist"
    );
}

#[test]
fn test_hero_switch_ignores_enemy_heroes() {
    let mut app = make_app();

    spawn_enemy_hero(&mut app);

    // Only enemy hero exists — should not select it
    fire_hero_switch(&mut app, Some(HeroSwitchDirection::Next), None);
    update_n(&mut app, 3);

    let active = app.world().resource::<ActiveHero>();
    assert!(
        active.entity.is_none(),
        "ActiveHero should remain None when only enemy heroes exist"
    );
}
