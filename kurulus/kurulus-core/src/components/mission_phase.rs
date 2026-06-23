use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::states::TacticalPhase;

/// Tracks the current tactical sub-phase and related timing.
///
/// Stored as a `Resource` for simplicity (can be a Component on a
/// mission-control entity later).
///
/// This is distinct from `mission::MissionPhase` (Intro/Active/Complete/Failed)
/// which tracks the high-level mission script state.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct TacticalPhaseTracker {
    pub current_phase: TacticalPhase,
    /// Game-time seconds when the current phase started.
    pub phase_start_time: f64,
    /// Whether dynamic (event-driven) phase transitions are enabled.
    pub dynamic_triggers: bool,
}

impl Default for TacticalPhaseTracker {
    fn default() -> Self {
        Self {
            current_phase: TacticalPhase::Setup,
            phase_start_time: 0.0,
            dynamic_triggers: true,
        }
    }
}

impl TacticalPhaseTracker {
    /// Advance to the next phase, recording the new start time.
    pub fn advance(&mut self, game_time_seconds: f64) {
        if let Some(next) = self.current_phase.next() {
            self.current_phase = next;
            self.phase_start_time = game_time_seconds;
        }
    }

    /// Reset to Setup phase (for new missions).
    pub fn reset(&mut self) {
        self.current_phase = TacticalPhase::Setup;
        self.phase_start_time = 0.0;
    }
}
