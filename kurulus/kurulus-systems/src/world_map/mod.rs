use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (region_control_system, fog_of_war_system).in_set(SimulationSet::WorldEventProcessing),
        );
    }
}

/// Updates region ownership based on entities within control points
fn region_control_system(_regions: Query<&Region>, _units: Query<(&WorldPosition, &Unit)>) {
    // TODO: Implement region control logic
}

/// Updates fog of war based on unit vision ranges
fn fog_of_war_system(
    _units: Query<(&WorldPosition, &Stats)>,
    _fog: Query<(&GridPos, &mut FogOfWar)>,
) {
    // TODO: Implement fog of war
}
