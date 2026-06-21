use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Integration test: Formation followers stay close to their leader
#[test]
fn test_formation_movement() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0 / 60.0)))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn a formation leader
    let leader = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        Stats {
            move_speed: 4.0,
            attack_range: 1.0,
            attack_damage: 10.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 70.0,
            morale_max: 100.0,
        },
        Health {
            current: 100.0,
            max: 100.0,
            armor: 5.0,
            armor_type: ArmorType::Light,
        },
    )).id();

    // Spawn a formation follower
    let follower = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(7, 5), CELL_SIZE),
        Stats {
            move_speed: 4.0,
            attack_range: 1.0,
            attack_damage: 10.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 70.0,
            morale_max: 100.0,
        },
        Health {
            current: 100.0,
            max: 100.0,
            armor: 5.0,
            armor_type: ArmorType::Light,
        },
        Formation {
            leader,
            offset: Vec2::new(-1.0, 0.0),
            slot_index: 0,
        },
        FormationFollower,
    )).id();

    // Run simulation
    let initial_follower_pos = app.world().get::<WorldPosition>(follower).unwrap().world_2d.0;
    for _ in 0..10 {
        app.update();
    }

    // Follower should move toward leader
    let final_leader_pos = app.world().get::<WorldPosition>(leader).unwrap().world_2d.0;
    let final_follower_pos = app.world().get::<WorldPosition>(follower).unwrap().world_2d.0;

    let initial_dist = initial_follower_pos.distance(final_leader_pos);
    let final_dist = final_follower_pos.distance(final_leader_pos);

    assert!(final_dist <= initial_dist + 0.1,
        "Follower should move toward leader (initial: {:.2}, final: {:.2})",
        initial_dist, final_dist);
}

/// Integration test: Grid quantization snaps positions to integer grid
#[test]
fn test_grid_quantization() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0 / 60.0)))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn a unit at non-integer position
    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition {
            grid: GridPos(IVec2::new(0, 0)),
            world_2d: WorldPos2D(Vec2::new(1.7, 3.4)),
            world_3d: WorldPos3D(Vec3::new(1.7, 0.0, 3.4)),
        },
        Stats::default(),
        Health::default(),
    )).id();

    // Run simulation
    for _ in 0..3 {
        app.update();
    }

    let pos = app.world().get::<WorldPosition>(unit).unwrap();
    assert_eq!(pos.grid.0.x, 2, "Grid X should be quantized to 2");
    assert_eq!(pos.grid.0.y, 3, "Grid Y should be quantized to 3");
}
