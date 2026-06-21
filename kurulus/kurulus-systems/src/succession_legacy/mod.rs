use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct SuccessionLegacyPlugin;

impl Plugin for SuccessionLegacyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                succession_order_system,
                legacy_scoring_system,
                dynasty_tree_system,
            ),
        );
    }
}

/// Determines the succession order when a ruler dies
fn succession_order_system(_commands: Commands, _heroes: Query<&Hero>, _trust: Res<TrustMatrix>) {
    // TODO: Implement succession order
}

/// Calculates legacy score based on achievements
fn legacy_scoring_system(_objective_events: EventReader<ObjectiveCompleted>) {
    // TODO: Implement legacy scoring
}

/// Manages the dynasty tree across generations
fn dynasty_tree_system(_commands: Commands) {
    // TODO: Implement dynasty tree
}
