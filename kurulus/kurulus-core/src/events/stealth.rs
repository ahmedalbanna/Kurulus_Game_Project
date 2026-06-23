use crate::components::AmbushPhase;
use bevy::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
//  DETECTION CHAIN EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a unit enters the suspicion phase (partial detection).
#[derive(Event, Clone, Debug)]
pub struct UnitSuspicionRaised {
    pub observer: Entity,
    pub suspected: Entity,
    pub suspicion_level: f32,
}

/// Emitted when a unit is fully confirmed as detected (confirmed sighting).
#[derive(Event, Clone, Debug)]
pub struct UnitConfirmed {
    pub observer: Entity,
    pub target: Entity,
}

/// Emitted when an alarm is raised — alerts all enemies within radius.
#[derive(Event, Clone, Debug)]
pub struct AlarmRaised {
    pub raiser: Entity,
    pub position: Vec2,
    pub radius: f32,
    pub duration: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH PHASE EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when an ambush phase changes.
#[derive(Event, Clone, Debug)]
pub struct AmbushPhaseChanged {
    pub entity: Entity,
    pub old_phase: AmbushPhase,
    pub new_phase: AmbushPhase,
}

/// Emitted when a staged ambush wave triggers.
#[derive(Event, Clone, Debug)]
pub struct StagedAmbushWave {
    pub ambusher: Entity,
    pub wave_number: u8,
    pub units: Vec<Entity>,
}

// ═════════════════════════════════════════════════════════════════════════════
//  SILENT KILL / STEALTH KILL EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a silent kill occurs (stealth assassination).
#[derive(Event, Clone, Debug)]
pub struct SilentKill {
    pub killer: Entity,
    pub victim: Entity,
    pub alarm_probability: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  LURE / DECOY EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a lure target attracts pursuers.
#[derive(Event, Clone, Debug)]
pub struct LureEngaged {
    pub lure_entity: Entity,
    pub pursuers: Vec<Entity>,
}

/// Emitted when a lure target escapes / leash breaks.
#[derive(Event, Clone, Debug)]
pub struct LureDisengaged {
    pub lure_entity: Entity,
    pub pursuers: Vec<Entity>,
    pub reason: LureDisengageReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LureDisengageReason {
    LeashBroken,
    LureKilled,
    PursuerKilled,
    AmbushTriggered,
}
