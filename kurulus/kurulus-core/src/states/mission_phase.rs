use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Sub-phases within a mission that drive narrative pacing.
///
/// These are **not** Bevy states (to avoid state-machine explosion) — they
/// are tracked as a `Resource` so multiple systems can read/write them
/// without scheduling conflicts.
///
/// NOTE: This is distinct from `kurulus_core::components::mission::MissionPhase`
/// which tracks the high-level mission script phase (Intro/Active/Complete/Failed).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Reflect)]
pub enum TacticalPhase {
    /// First ~2 minutes: explore, collect initial resources, build first base.
    #[default]
    Setup,
    /// ~4-6 minutes: enemies arrive, secondary objectives appear.
    Complication,
    /// ~3-5 minutes: main objective active, peak enemy density.
    Climax,
    /// ~2 minutes: combat ends, collect loot, distribute rewards.
    Resolution,
}

impl TacticalPhase {
    /// Returns the human-readable label for the phase.
    pub fn label(&self) -> &'static str {
        match self {
            TacticalPhase::Setup => "Setup",
            TacticalPhase::Complication => "Complication",
            TacticalPhase::Climax => "Climax",
            TacticalPhase::Resolution => "Resolution",
        }
    }

    /// Returns the next phase in sequence, if any.
    pub fn next(self) -> Option<Self> {
        match self {
            TacticalPhase::Setup => Some(TacticalPhase::Complication),
            TacticalPhase::Complication => Some(TacticalPhase::Climax),
            TacticalPhase::Climax => Some(TacticalPhase::Resolution),
            TacticalPhase::Resolution => None,
        }
    }

    /// Returns the recommended minimum duration of this phase (in game-seconds).
    pub fn min_duration(&self) -> f32 {
        match self {
            TacticalPhase::Setup => 120.0,
            TacticalPhase::Complication => 240.0,
            TacticalPhase::Climax => 180.0,
            TacticalPhase::Resolution => 90.0,
        }
    }
}
