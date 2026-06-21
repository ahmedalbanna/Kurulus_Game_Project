use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                ai_decision_system,
                ai_command_issuing_system,
                threat_evaluation_system,
            )
                .in_set(SimulationSet::AIDecision),
        );
    }
}

/// AI controllers evaluate world state and set goals
fn ai_decision_system(
    _time: Res<GameTime>,
    _stockpile: Res<ResourceStockpile>,
    mut _controllers: Query<&mut AIController>,
    _threats: Query<&Threat>,
) {
    // TODO: Implement AI decision making
}

/// AI issues commands based on current goals
fn ai_command_issuing_system(
    _controllers: Query<&AIController>,
    _move_events: EventWriter<MoveOrder>,
    _attack_events: EventWriter<AttackOrder>,
    _build_events: EventWriter<BuildOrder>,
) {
    // TODO: Implement AI command issuing
}

/// Evaluates threats from nearby enemies
fn threat_evaluation_system(
    _units: Query<(Entity, &WorldPosition, &FactionId)>,
    _threats: Query<&mut Threat>,
) {
    // TODO: Implement threat evaluation
}
