use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Integration test: Melee combat damage calculation
#[test]
fn test_melee_combat_damage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<DamageDealt>()
        .add_event::<UnitDied>()
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();

    // Spawn attacker with known stats
    let attacker = app
        .world_mut()
        .spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            Stats {
                move_speed: 4.0,
                attack_range: 2.0,
                attack_damage: 25.0,
                attack_speed: 100.0,
                vision_range: 8.0,
                morale: 70.0,
                morale_max: 100.0,
            },
            Health {
                current: 150.0,
                max: 150.0,
                armor: 10.0,
                armor_type: ArmorType::Light,
            },
            MeleeWeapon {
                damage: 25.0,
                weapon_type: WeaponType::Sword,
                range: 1.5,
                armor_penetration: 0.2,
            },
            AttackTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
        ))
        .id();

    // Spawn a same-faction unit (should be skipped by friendly-fire check)
    spawn_same_faction_ally(&mut app);

    // Spawn enemy defender
    let _ = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
        Stats {
            move_speed: 3.0,
            attack_range: 1.0,
            attack_damage: 15.0,
            attack_speed: 0.8,
            vision_range: 6.0,
            morale: 60.0,
            morale_max: 100.0,
        },
        Health {
            current: 100.0,
            max: 100.0,
            armor: 5.0,
            armor_type: ArmorType::Light,
        },
        MeleeWeapon {
            damage: 15.0,
            weapon_type: WeaponType::Spear,
            range: 1.5,
            armor_penetration: 0.1,
        },
        AttackTimer(Timer::from_seconds(1.25, TimerMode::Repeating)),
    ));

    // Run simulation and check for damage events during each frame
    let mut damage_dealt = false;

    for _ in 0..10 {
        app.update();
        let mut dmg_events = app.world_mut().resource_mut::<Events<DamageDealt>>();
        for event in dmg_events.drain() {
            if event.attacker == attacker {
                // Skip dodged/blocked attacks (amount == 0)
                if event.amount <= 0.0 {
                    continue;
                }
                // Slash vs LightInfantry effectiveness = 1.5.
                // Base weapon damage 25 × 1.5 = 37.5, minus armor ≈ 36.3 non-crit.
                // Crit (×1.5) = 54.5, charge (×2.0) could exceed that.
                if event.crit == CritType::None {
                    assert!(
                        event.amount <= 45.0,
                        "Non-crit damage {:?} expected ~36.3 with modifiers",
                        event.amount
                    );
                } else {
                    assert!(
                        event.amount <= 90.0,
                        "Crit damage {:?} expected ~54.5 max with modifiers",
                        event.amount
                    );
                }
                damage_dealt = true;
            }
        }
    }

    assert!(damage_dealt, "No damage events were emitted");
}

/// Helper to spawn an ally (same faction) for friendly-fire testing.
fn spawn_same_faction_ally(app: &mut App) {
    let _ = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 25.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 70.0,
            morale_max: 100.0,
        },
        Health {
            current: 150.0,
            max: 150.0,
            armor: 10.0,
            armor_type: ArmorType::Light,
        },
        MeleeWeapon {
            damage: 25.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.2,
        },
        AttackTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
    ));
}

/// Integration test: Unit death triggers UnitDied event
#[test]
fn test_unit_died_event() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<DamageDealt>()
        .add_event::<UnitDied>()
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();

    // Spawn attacker with high damage
    let attacker = app
        .world_mut()
        .spawn((
            Unit {
                archetype: UnitArchetype::HeavyInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            Stats {
                move_speed: 3.0,
                attack_range: 2.0,
                attack_damage: 100.0,
                attack_speed: 100.0,
                vision_range: 8.0,
                morale: 100.0,
                morale_max: 100.0,
            },
            Health {
                current: 500.0,
                max: 500.0,
                armor: 20.0,
                armor_type: ArmorType::Heavy,
            },
            MeleeWeapon {
                damage: 200.0,
                weapon_type: WeaponType::Axe,
                range: 1.5,
                armor_penetration: 0.5,
            },
            AttackTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
        ))
        .id();

    // Spawn defender with low health
    let defender = app
        .world_mut()
        .spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
            Stats {
                move_speed: 3.0,
                attack_range: 1.0,
                attack_damage: 5.0,
                attack_speed: 0.5,
                vision_range: 6.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            Health {
                current: 10.0,
                max: 100.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            MeleeWeapon {
                damage: 5.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
            AttackTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
        ))
        .id();

    // Run simulation and capture events during the loop
    let mut death_occurred = false;

    for _ in 0..10 {
        app.update();
        let death_events = app.world().resource::<Events<UnitDied>>();
        let mut reader = death_events.get_cursor();
        for event in reader.read(&death_events) {
            if event.killer == attacker && event.entity == defender {
                death_occurred = true;
            }
        }
    }

    assert!(
        death_occurred,
        "UnitDied event was not emitted for defender"
    );
}
