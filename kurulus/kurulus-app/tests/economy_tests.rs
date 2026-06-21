use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Integration test: Resource stockpile increases over time
#[test]
fn test_resource_production() {
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

    // Record initial stockpile
    let initial_food = app.world().resource::<ResourceStockpile>().food;

    // Run simulation for several ticks
    for _ in 0..50 {
        app.update();
    }

    // Food should have increased
    let current_food = app.world().resource::<ResourceStockpile>().food;
    assert!(current_food > initial_food,
        "Food should increase over time (initial: {:.1}, current: {:.1})",
        initial_food, current_food);
}

/// Integration test: Game time advances properly
#[test]
fn test_game_time_advancement() {
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

    let initial_time = app.world().resource::<GameTime>().total_seconds;

    for _ in 0..100 {
        app.update();
    }

    let current_time = app.world().resource::<GameTime>().total_seconds;
    assert!(current_time > initial_time,
        "Game time should advance (initial: {:.1}, current: {:.1})",
        initial_time, current_time);
}
