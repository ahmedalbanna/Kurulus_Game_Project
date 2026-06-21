use bevy::prelude::*;
use kurulus_core::prelude::*;
use kurulus_core::states::GamePhase as GamePhaseEnum;

pub struct CoreLoopPlugin;

impl Plugin for CoreLoopPlugin {
    fn build(&self, app: &mut App) {
        // Chain all simulation sets in order using .after() relationships
        app.configure_sets(FixedUpdate, SimulationSet::InputProcessing);
        app.configure_sets(
            FixedUpdate,
            SimulationSet::CommandQueueing.after(SimulationSet::InputProcessing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::AIDecision.after(SimulationSet::CommandQueueing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::AICommandIssuing.after(SimulationSet::AIDecision),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::Pathfinding.after(SimulationSet::AICommandIssuing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::FormationMovement.after(SimulationSet::Pathfinding),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::FreeFormMovement.after(SimulationSet::FormationMovement),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::GridQuantization.after(SimulationSet::FreeFormMovement),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::VisionCalculation.after(SimulationSet::GridQuantization),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::StealthDetection.after(SimulationSet::VisionCalculation),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::AmbushTrigger.after(SimulationSet::StealthDetection),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::AttackTimers.after(SimulationSet::AmbushTrigger),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::MeleeResolution.after(SimulationSet::AttackTimers),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::RangedResolution.after(SimulationSet::MeleeResolution),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::SiegeResolution.after(SimulationSet::RangedResolution),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::DamageApplication.after(SimulationSet::SiegeResolution),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::MoraleUpdate.after(SimulationSet::DamageApplication),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::DeathProcessing.after(SimulationSet::MoraleUpdate),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::ResourceProduction.after(SimulationSet::DeathProcessing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::ResourceConsumption.after(SimulationSet::ResourceProduction),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::ConstructionProgress.after(SimulationSet::ResourceConsumption),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::ResearchProgress.after(SimulationSet::ConstructionProgress),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::XPGain.after(SimulationSet::ResearchProgress),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::MissionObjectiveCheck.after(SimulationSet::XPGain),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::FactionRelationUpdate.after(SimulationSet::MissionObjectiveCheck),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::WorldEventProcessing.after(SimulationSet::FactionRelationUpdate),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::DelegationProcessing.after(SimulationSet::WorldEventProcessing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::TrustUpdate.after(SimulationSet::DelegationProcessing),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::SynergyCalculation.after(SimulationSet::TrustUpdate),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::ComponentCleanup.after(SimulationSet::SynergyCalculation),
        );
        app.configure_sets(
            FixedUpdate,
            SimulationSet::PositionSync.after(SimulationSet::ComponentCleanup),
        );

        app.add_systems(
            FixedUpdate,
            (game_phase_transition_system, time_advance_system),
        );
    }
}

/// Transitions between Planning -> Execution -> Harvest based on events/conditions
fn game_phase_transition_system(
    mut next_phase: ResMut<NextState<GamePhaseEnum>>,
    phase: Res<State<GamePhaseEnum>>,
    game_time: Res<GameTime>,
) {
    // Placeholder: cycle through phases every 60 seconds for testing
    let phase_cycle = (game_time.total_seconds as u32 / 60) % 3;
    let target = match phase_cycle {
        0 => GamePhaseEnum::Planning,
        1 => GamePhaseEnum::Execution,
        _ => GamePhaseEnum::Harvest,
    };
    if *phase.get() != target {
        next_phase.set(target);
    }
}

/// Advances the global game time
fn time_advance_system(time: Res<bevy::time::Time>, mut game_time: ResMut<GameTime>) {
    game_time.advance(time.delta_secs_f64());
}
