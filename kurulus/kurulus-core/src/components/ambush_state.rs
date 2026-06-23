use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Phases of the ambush lifecycle as defined in GDD Ambush Mechanics.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmbushPhase {
    /// Unit is not part of any ambush.
    Idle,
    /// Unit is being positioned into ambush locations.
    Setup,
    /// Unit is in ready state — hidden, not auto-attacking, waiting for trigger.
    Ready,
    /// Ambush has been triggered — bonuses active.
    Triggered,
    /// Ambush execution phase — dealing bonus damage, effects active.
    Executing,
    /// Ambush has ended — cooldown before next ambush possible.
    Cooldown,
}

/// Types of ambush trigger available to the player.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    /// Player presses a key to trigger manually.
    Manual,
    /// Auto-triggers when enemy reaches a designated contact point.
    AutoFirstContact,
    /// Three-stage trigger: archers → infantry → cavalry.
    Staged,
    /// Solo ambush (Boran only) — single-target assassination.
    Solo,
}

/// Ambush state component — attached to units participating in an ambush.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AmbushState {
    /// Current ambush phase.
    pub phase: AmbushPhase,
    /// Trigger type for this ambush.
    pub trigger_type: TriggerType,
    /// Current staged wave (0 = not staged, 1/2/3 = wave number).
    pub staged_wave: u8,
    /// Total number of staged waves planned.
    pub total_waves: u8,
    /// Time since entering ready state.
    pub ready_since: f32,
    /// Remaining duration of bonus damage window (seconds).
    pub bonus_damage_remaining: f32,
    /// Remaining shock duration on enemies (seconds).
    pub shock_remaining: f32,
    /// Whether this unit's ambush has been discovered prematurely.
    pub discovered: bool,
    /// The ambush zone entity this unit is part of.
    pub ambush_zone: Option<Entity>,
}

impl Default for AmbushState {
    fn default() -> Self {
        Self {
            phase: AmbushPhase::Idle,
            trigger_type: TriggerType::Manual,
            staged_wave: 0,
            total_waves: 1,
            ready_since: 0.0,
            bonus_damage_remaining: 0.0,
            shock_remaining: 0.0,
            discovered: false,
            ambush_zone: None,
        }
    }
}
