use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Integration test: Hidden unit is detected when detector is in range
#[test]
fn test_stealth_detection_in_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<StealthRevealed>()
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();

    // Spawn a hidden unit
    let hidden_unit = app
        .world_mut()
        .spawn((
            Stealth {
                level: 0.3,
                is_hidden: true,
                hidden_timer: 10.0,
                ambush_zone: None,
            },
            Detection {
                radius: 5.0,
                height_bonus: 0.0,
                active: true,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction: FactionId::Kayi,
                formation: FormationType::Scatter,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id();

    // Spawn a detector very close
    let _detector = app
        .world_mut()
        .spawn((
            Detection {
                radius: 10.0,
                height_bonus: 0.0,
                active: true,
            },
            WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction: FactionId::Byzantine,
                formation: FormationType::Scatter,
                stance: CombatStance::Aggressive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id();

    // Run simulation
    for _ in 0..5 {
        app.update();
    }

    // Check if hidden unit was revealed
    let stealth = app.world().get::<Stealth>(hidden_unit).unwrap();
    assert!(!stealth.is_hidden, "Hidden unit should have been detected");
}

/// Integration test: Hidden unit stays hidden when far from detectors
#[test]
fn test_stealth_stays_hidden_out_of_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<StealthRevealed>()
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();

    // Spawn a hidden unit
    let hidden_unit = app
        .world_mut()
        .spawn((
            Stealth {
                level: 0.9,
                is_hidden: true,
                hidden_timer: 10.0,
                ambush_zone: None,
            },
            Detection {
                radius: 5.0,
                height_bonus: 0.0,
                active: true,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction: FactionId::Kayi,
                formation: FormationType::Scatter,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id();

    // Spawn a detector far away
    let _detector = app
        .world_mut()
        .spawn((
            Detection {
                radius: 5.0,
                height_bonus: 0.0,
                active: true,
            },
            WorldPosition::new(IVec2::new(100, 100), CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction: FactionId::Byzantine,
                formation: FormationType::Scatter,
                stance: CombatStance::Aggressive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id();

    // Run simulation
    for _ in 0..5 {
        app.update();
    }

    // Check hidden unit is still hidden
    let stealth = app.world().get::<Stealth>(hidden_unit).unwrap();
    assert!(
        stealth.is_hidden,
        "Hidden unit should remain hidden when far from detectors"
    );
}
