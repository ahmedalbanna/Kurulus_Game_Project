use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct CoRulePlugin;

impl Plugin for CoRulePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                co_command_dispatch_system.in_set(SimulationSet::DelegationProcessing),
                trust_update_system.in_set(SimulationSet::TrustUpdate),
                synergy_calculation_system.in_set(SimulationSet::SynergyCalculation),
                delegation_order_system,
            ),
        );
    }
}

/// Dispatches co-command orders between primary and secondary heroes
fn co_command_dispatch_system(_commands: Commands, _co_commands: Query<&CoCommand>) {
    // TODO: Implement co-command dispatch
}

/// Updates trust levels between co-rulers
fn trust_update_system(
    _time: Res<GameTime>,
    _trust: ResMut<TrustMatrix>,
    _objective_events: EventReader<ObjectiveCompleted>,
) {
    // TODO: Implement trust updates
}

/// Calculates synergy bonus between co-ruling heroes
fn synergy_calculation_system(_co_commands: Query<&CoCommand>, _state: ResMut<CoCommandState>) {
    // TODO: Implement synergy calculation
}

/// Processes delegation orders between heroes
fn delegation_order_system(_commands: Commands, _co_commands: Query<&mut CoCommand>) {
    // TODO: Implement delegation order processing
}
