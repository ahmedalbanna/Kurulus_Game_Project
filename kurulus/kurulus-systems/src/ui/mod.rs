use bevy::prelude::*;
use kurulus_core::prelude::*;

mod city_event_log;
use city_event_log::*;

mod zone_legend;
use zone_legend::*;

pub mod world_map;

pub struct UILogicPlugin;

impl Plugin for UILogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityEventLog>()
            .add_systems(
                FixedUpdate,
                (
                    selection_processing_system,
                    notification_dispatch_system,
                    // City event log logic — runs after city events are generated
                    city_event_log_logic_system
                        .in_set(SimulationSet::CityProgression),
                ),
            )
            // Render systems update the bevy_ui tree each frame.
            // Safe in headless mode (UiPlugin absent → entities invisible but harmless).
            .add_systems(Update, (
                city_event_log_render_system,
                zone_legend_render_system,
                world_map::world_map_tooltip_system,
                world_map::world_map_legend_system,
                world_map::world_map_selection_panel_system,
            ));
    }
}

/// Processes unit/building selection events
fn selection_processing_system(_selection_events: EventWriter<SelectionChanged>) {
    // TODO: Implement selection logic (data only, no rendering)
}

/// Dispatches notifications based on game events
fn notification_dispatch_system(
    _damage_events: EventReader<DamageDealt>,
    _death_events: EventReader<UnitDied>,
    _objective_events: EventReader<ObjectiveCompleted>,
    _notifications: EventWriter<Notification>,
) {
    // TODO: Implement notification dispatch
}
