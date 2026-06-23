use bevy::prelude::*;
use crate::components::{HeroId, SpecializationPath, TalentChoice};

// ─── Hero state machine events ───────────────────────────────────────────────

/// Fired when a hero enters the KnockedOut state.
#[derive(Event, Clone, Debug)]
pub struct HeroKnockedOut {
    pub hero: Entity,
    pub hero_id: HeroId,
    pub knockout_duration: f32,
}

/// Fired when a hero receives a death blow while KnockedOut.
#[derive(Event, Clone, Debug)]
pub struct HeroDeathBlow {
    pub hero: Entity,
    pub hero_id: HeroId,
    pub killer: Entity,
}

/// Fired when a hero transitions from KnockedOut to Recovering.
#[derive(Event, Clone, Debug)]
pub struct HeroRecovering {
    pub hero: Entity,
    pub hero_id: HeroId,
}

/// Fired when a hero fully recovers and returns to Idle.
#[derive(Event, Clone, Debug)]
pub struct HeroReturnedToIdle {
    pub hero: Entity,
    pub hero_id: HeroId,
}

/// Fired when a hero permanently dies in a mission (death blow confirmed).
#[derive(Event, Clone, Debug)]
pub struct HeroKilledInMission {
    pub hero: Entity,
    pub hero_id: HeroId,
    pub killer: Entity,
}

// ─── Talent / Specialization events ──────────────────────────────────────────

/// Fired when a hero reaches a level that grants a talent choice (6 or 10).
/// UI should display two options and send `TalentChoiceSelected` back.
#[derive(Event, Clone, Debug)]
pub struct TalentChoiceRequest {
    pub hero: Entity,
    pub hero_id: HeroId,
    pub level: u8,
    /// Option A description (short).
    pub option_a: String,
    /// Option B description (short).
    pub option_b: String,
}

/// Fired by the UI when the player selects a talent option.
#[derive(Event, Clone, Debug)]
pub struct TalentChoiceSelected {
    pub hero: Entity,
    pub level: u8,
    pub choice: TalentChoice,
}

/// Fired when a hero reaches level 8 (specialization path choice).
/// UI should display three paths and send `SpecializationSelected` back.
#[derive(Event, Clone, Debug)]
pub struct SpecializationRequest {
    pub hero: Entity,
    pub hero_id: HeroId,
    pub paths: Vec<SpecializationPath>,
}

/// Fired by the UI when the player selects a specialization path.
#[derive(Event, Clone, Debug)]
pub struct SpecializationSelected {
    pub hero: Entity,
    pub path: SpecializationPath,
}

// ─── Hero switch events (from command.rs) ────────────────────────────────────
// HeroSwitchEvent is already defined in command.rs — re-exported for convenience.
