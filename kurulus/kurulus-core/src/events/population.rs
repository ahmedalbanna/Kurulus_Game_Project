use bevy::prelude::*;
use crate::components::{BuildingType, WorkerRole};

// ─── Population Growth / Decline ────────────────────────────────────────────

/// Fired when population grows by one or more.
#[derive(Event, Clone, Debug)]
pub struct PopulationGrowth {
    /// Number of new citizens added this tick.
    pub amount: u32,
    /// Total population after growth.
    pub total: u32,
}

/// Fired when population declines.
#[derive(Event, Clone, Debug)]
pub struct PopulationDecline {
    /// Number of citizens lost this tick.
    pub amount: u32,
    /// Total population after decline.
    pub total: u32,
    /// The cause of the decline.
    pub reason: DeclineReason,
}

/// Why population decreased.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeclineReason {
    /// Food satisfaction dropped below the starvation threshold.
    Starvation,
    /// Plague (or other disease) event.
    Plague,
    /// Military casualties removed from the population count.
    MilitaryLoss,
    /// Population exceeds capacity (overcrowding migration).
    Overcrowding,
}

// ─── Worker Assignment ──────────────────────────────────────────────────────

/// Fired when workers are assigned to a building.
#[derive(Event, Clone, Debug)]
pub struct WorkerAssigned {
    /// The building entity receiving workers.
    pub building: Entity,
    /// Building type (for UI / log display).
    pub building_type: BuildingType,
    /// Number of workers assigned in this event.
    pub worker_count: u32,
    /// The role these workers perform.
    pub role: WorkerRole,
}

/// Fired when workers are removed from a building.
#[derive(Event, Clone, Debug)]
pub struct WorkerUnassigned {
    /// The building entity losing workers.
    pub building: Entity,
    /// Building type.
    pub building_type: BuildingType,
    /// Number of workers removed.
    pub worker_count: u32,
}

// ─── Needs Critical Warning ─────────────────────────────────────────────────

/// Fired when a citizen need drops below a critical threshold (default < 0.2).
#[derive(Event, Clone, Debug)]
pub struct NeedsCritical {
    /// Which need is critical.
    pub need_type: CriticalNeedType,
    /// Current satisfaction value (0.0 – 1.0) indicating severity.
    pub severity: f32,
}

/// Which citizen need is at a critical level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CriticalNeedType {
    Food,
    Housing,
    Faith,
    Safety,
}
