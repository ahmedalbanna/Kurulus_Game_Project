use bevy::prelude::*;
use crate::states::{GamePhase, TacticalPhase};
use crate::components::{HeroId, AbilityId, MissionId};
use crate::resources::ResourceStockpile;

// ═══════════════════════════════════════════════════════════════════════════
// Phase Transition Events
// ═══════════════════════════════════════════════════════════════════════════

/// Fired when the game phase transitions (e.g., Planning → Execution).
#[derive(Event, Clone, Debug)]
pub struct GamePhaseChanged {
    pub from: GamePhase,
    pub to: GamePhase,
}

/// Fired when the tactical sub-phase transitions (e.g., Setup → Complication).
#[derive(Event, Clone, Debug)]
pub struct TacticalPhaseChanged {
    pub from: TacticalPhase,
    pub to: TacticalPhase,
}

// ═══════════════════════════════════════════════════════════════════════════
// Harvest Events
// ═══════════════════════════════════════════════════════════════════════════

/// Fired when the Harvest phase begins.
#[derive(Event, Clone, Debug)]
pub struct HarvestStarted {
    pub mission_id: MissionId,
}

/// Fired when harvest calculation is complete and rewards are ready.
#[derive(Event, Clone, Debug)]
pub struct HarvestCompleted {
    pub mission_id: MissionId,
    pub total_xp: f32,
    pub total_sp: u32,
    pub total_loot: ResourceStockpile,
}

/// Fired when a hero gains XP during harvest.
#[derive(Event, Clone, Debug)]
pub struct HarvestXpAwarded {
    pub hero_id: HeroId,
    pub hero_entity: Entity,
    pub amount: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Hero Moment Events
// ═══════════════════════════════════════════════════════════════════════════

/// Fired when a Hero Moment trigger condition is met (before the moment plays).
#[derive(Event, Clone, Debug)]
pub struct HeroMomentTriggered {
    pub hero_id: HeroId,
    pub ability_id: AbilityId,
    pub description: String,
}

/// Fired when a Hero Moment is successfully completed by the player.
#[derive(Event, Clone, Debug)]
pub struct HeroMomentCompleted {
    pub hero_id: HeroId,
    pub ability_id: AbilityId,
    pub bonus_xp: f32,
}

/// Fired when a cinematic sequence should play for a Hero Moment.
/// Render systems can watch this to trigger camera cuts/slow-mo/etc.
#[derive(Event, Clone, Debug)]
pub struct CinematicTrigger {
    pub hero_id: HeroId,
    pub ability_id: AbilityId,
    pub description: String,
    pub duration_secs: f32,
    /// Whether this is a first-time viewing (can skip after first time).
    pub first_viewing: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// Mission Flow Events
// ═══════════════════════════════════════════════════════════════════════════

/// Fired when the Setup phase's initial base-building is complete.
/// Triggers the transition to the Complication phase.
#[derive(Event, Clone, Debug)]
pub struct MissionSetupComplete;

/// Fired when the first enemy wave or event arrives to complicate the mission.
#[derive(Event, Clone, Debug)]
pub struct ComplicationStarted;

/// Fired when the main objective becomes active (peak of the mission).
#[derive(Event, Clone, Debug)]
pub struct ClimaxTriggered;

/// Fired when all enemies are cleared and the loot/harvest phase begins.
#[derive(Event, Clone, Debug)]
pub struct ResolutionStarted;

// ═══════════════════════════════════════════════════════════════════════════
// Save / Load Events
// ═══════════════════════════════════════════════════════════════════════════

/// Fired when a game save is completed.
#[derive(Event, Clone, Debug)]
pub struct GameSaved {
    pub slot: usize,
    pub auto: bool,
}

/// Fired when a game is loaded from a save slot.
#[derive(Event, Clone, Debug)]
pub struct GameLoaded {
    pub slot: usize,
}
