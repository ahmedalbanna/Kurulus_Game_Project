use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct UILogicPlugin;

impl Plugin for UILogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (selection_processing_system, notification_dispatch_system)
                .in_set(SimulationSet::InputProcessing),
        );
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
