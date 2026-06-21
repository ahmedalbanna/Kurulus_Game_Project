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

        // Register phase-transition resources
        app.init_resource::<PhaseTransitionConfig>()
            .init_resource::<CombatActivityTracker>()
            .init_resource::<OrdersIssuedDuringPlanning>();

        // Phase transition systems — each runs in a dedicated set early in the tick
        // so that the rest of the frame sees the correct GamePhase.
        app.add_systems(
            FixedUpdate,
            (
                // Detect new commands issued during Planning phase
                flag_orders_system.in_set(SimulationSet::InputProcessing),
                // Track combat activity (updated by combat events)
                track_combat_activity_system.in_set(SimulationSet::AttackTimers),
                // Phase transition logic
                planning_to_execution_system,
                execution_to_harvest_system,
                harvest_to_planning_system,
                // Core time advancement
                time_advance_system,
            ),
        );
    }
}

// ─── Combat activity tracking ─────────────────────────────────────────────

/// Watches for damage/attack events and updates the activity tracker so the
/// Execution→Harvest transition knows when combat has gone quiet.
fn track_combat_activity_system(
    game_time: Res<GameTime>,
    damage_events: EventReader<DamageDealt>,
    attack_events: EventReader<AttackTriggered>,
    ambush_events: EventReader<AmbushTriggered>,
    mut tracker: ResMut<CombatActivityTracker>,
) {
    let any_combat = !damage_events.is_empty()
        || !attack_events.is_empty()
        || !ambush_events.is_empty();

    if any_combat {
        tracker.last_combat_game_time = game_time.total_seconds;
    }
}

// ─── Flag orders issued during Planning ───────────────────────────────────

/// Fires whenever any order-type event is sent.  Because events are
/// drained in order, we must reset the flag at the *beginning* of each
/// Planning phase and set it as soon as any order arrives.
fn flag_orders_system(
    phase: Res<State<GamePhaseEnum>>,
    mut flags: ResMut<OrdersIssuedDuringPlanning>,
    // Reading, not consuming — other systems also need these events.
    move_events: EventReader<MoveOrder>,
    attack_events: EventReader<AttackOrder>,
    build_events: EventReader<BuildOrder>,
    recruit_events: EventReader<RecruitOrder>,
    research_events: EventReader<ResearchOrder>,
    ability_events: EventReader<AbilityUsed>,
) {
    if *phase.get() != GamePhaseEnum::Planning {
        return;
    }

    let has_orders = !move_events.is_empty()
        || !attack_events.is_empty()
        || !build_events.is_empty()
        || !recruit_events.is_empty()
        || !research_events.is_empty()
        || !ability_events.is_empty();

    if has_orders {
        flags.any_order_issued = true;
    }
}

// ─── Phase transitions ────────────────────────────────────────────────────

/// Planning → Execution: transition when at least one order was issued
/// during the Planning phase and the minimum planning duration has elapsed.
fn planning_to_execution_system(
    game_time: Res<GameTime>,
    config: Res<PhaseTransitionConfig>,
    phase: Res<State<GamePhaseEnum>>,
    mut next_phase: ResMut<NextState<GamePhaseEnum>>,
    orders: Res<OrdersIssuedDuringPlanning>,
    mut phase_start_time: Local<f64>,
) {
    if *phase.get() != GamePhaseEnum::Planning {
        return;
    }

    // Track when we entered Planning phase
    let current = game_time.total_seconds;

    // Initialise phase_start_time when we first enter Planning
    if *phase_start_time == 0.0 {
        *phase_start_time = current;
    }

    let elapsed = current - *phase_start_time;

    // Must wait at least min_planning_duration
    if elapsed < config.min_planning_duration as f64 {
        return;
    }

    // If any orders were issued, transition to Execution
    if orders.any_order_issued {
        next_phase.set(GamePhaseEnum::Execution);
        *phase_start_time = 0.0; // reset for next phase
    }
}

/// Execution → Harvest: transition when combat has been inactive for
/// `combat_inactivity_timeout` seconds.
fn execution_to_harvest_system(
    game_time: Res<GameTime>,
    config: Res<PhaseTransitionConfig>,
    phase: Res<State<GamePhaseEnum>>,
    mut next_phase: ResMut<NextState<GamePhaseEnum>>,
    combat_tracker: Res<CombatActivityTracker>,
) {
    if *phase.get() != GamePhaseEnum::Execution {
        return;
    }

    let time_since_combat = game_time.total_seconds - combat_tracker.last_combat_game_time;
    if time_since_combat >= config.combat_inactivity_timeout as f64 {
        next_phase.set(GamePhaseEnum::Harvest);
    }
}

/// Harvest → Planning: transition after harvest_duration seconds.
fn harvest_to_planning_system(
    game_time: Res<GameTime>,
    config: Res<PhaseTransitionConfig>,
    phase: Res<State<GamePhaseEnum>>,
    mut next_phase: ResMut<NextState<GamePhaseEnum>>,
    mut orders: ResMut<OrdersIssuedDuringPlanning>,
    mut harvest_start_time: Local<f64>,
) {
    if *phase.get() != GamePhaseEnum::Harvest {
        return;
    }

    let current = game_time.total_seconds;

    if *harvest_start_time == 0.0 {
        *harvest_start_time = current;
    }

    let elapsed = current - *harvest_start_time;
    if elapsed >= config.harvest_duration as f64 {
        // Reset orders flag for the next Planning phase
        orders.any_order_issued = false;
        next_phase.set(GamePhaseEnum::Planning);
        *harvest_start_time = 0.0; // reset for next cycle
    }
}

// ─── Time advancement (unchanged) ──────────────────────────────────────────

/// Advances the global game time.
fn time_advance_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut game_time: ResMut<GameTime>,
) {
    game_time.advance(time.delta_secs_f64());
}
