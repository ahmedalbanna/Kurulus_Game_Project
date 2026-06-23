use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::building::BuildingType;

// ─── WorkerSlot Component ─────────────────────────────────────────────────────

/// Per-building component that defines worker capacity and requirements.
/// Added to `Building` entities at spawn time by `complete_construction_system` (or at
/// entity spawn if pre-placed).  The `worker_allocation_system` reads this to assign
/// idle population labourers to production buildings.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct WorkerSlot {
    /// Maximum number of workers this building can employ.
    pub max_workers: u32,
    /// Current number of workers assigned (set by `worker_allocation_system`).
    pub assigned_workers: u32,
    /// What kind of labour this building requires (affects assignment priority).
    pub required_role: WorkerRole,
    /// Base effectiveness per worker, 0.0 – 1.0.
    /// At 1.0 each worker contributes full output.
    pub worker_efficiency: f32,
    /// Production bonus multiplier when the building is fully staffed
    /// (e.g. 1.50 = +50 % output over the sum of individual worker contributions).
    pub full_staff_bonus: f32,
}

impl WorkerSlot {
    /// How many more workers can be assigned (may be negative if over-assigned).
    pub fn available_slots(&self) -> i32 {
        self.max_workers as i32 - self.assigned_workers as i32
    }

    /// Whether the building has reached its maximum worker count.
    pub fn is_full(&self) -> bool {
        self.assigned_workers >= self.max_workers
    }

    /// Current staffing fraction, 0.0 – 1.0.
    pub fn staffing_fraction(&self) -> f32 {
        if self.max_workers == 0 {
            0.0
        } else {
            self.assigned_workers as f32 / self.max_workers as f32
        }
    }

    /// Effective production multiplier for this building.
    ///
    /// Formula:
    ///   `(staffing_fraction × worker_efficiency) + full_staff_bonus(lift)`
    ///
    /// where the lift is only applied when fully staffed.
    pub fn production_multiplier(&self) -> f32 {
        let base = self.staffing_fraction() * self.worker_efficiency;
        if self.is_full() {
            base + (self.full_staff_bonus - 1.0)
        } else {
            base
        }
    }
}

impl Default for WorkerSlot {
    fn default() -> Self {
        Self {
            max_workers: 2,
            assigned_workers: 0,
            required_role: WorkerRole::Idle,
            worker_efficiency: 0.8,
            full_staff_bonus: 1.25,
        }
    }
}

/// Satisfies `WorkerSlotConfig` which returns default entries.
/// Used when spawning buildings from config / editor.
impl WorkerSlot {
    /// Return the default `WorkerSlot` configuration for a given `BuildingType`.
    /// Returns `None` for buildings that do not employ workers (walls, gates, roads, etc.).
    pub fn for_building_type(bt: BuildingType) -> Option<Self> {
        use BuildingType::*;
        match bt {
            Pasture => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Gatherer,
                worker_efficiency: 0.9,
                full_staff_bonus: 1.30,
                assigned_workers: 0,
            }),
            MiningCamp => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Gatherer,
                worker_efficiency: 0.85,
                full_staff_bonus: 1.25,
                assigned_workers: 0,
            }),
            LumberCamp => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Gatherer,
                worker_efficiency: 0.95,
                full_staff_bonus: 1.20,
                assigned_workers: 0,
            }),
            Market => Some(Self {
                max_workers: 4,
                required_role: WorkerRole::Merchant,
                worker_efficiency: 0.80,
                full_staff_bonus: 1.35,
                assigned_workers: 0,
            }),
            Warehouse => Some(Self {
                max_workers: 2,
                required_role: WorkerRole::Merchant,
                worker_efficiency: 0.90,
                full_staff_bonus: 1.15,
                assigned_workers: 0,
            }),
            Barracks => Some(Self {
                max_workers: 2,
                required_role: WorkerRole::Trainer,
                worker_efficiency: 0.85,
                full_staff_bonus: 1.20,
                assigned_workers: 0,
            }),
            ArcheryRange => Some(Self {
                max_workers: 2,
                required_role: WorkerRole::Trainer,
                worker_efficiency: 0.85,
                full_staff_bonus: 1.20,
                assigned_workers: 0,
            }),
            Stable => Some(Self {
                max_workers: 2,
                required_role: WorkerRole::Trainer,
                worker_efficiency: 0.80,
                full_staff_bonus: 1.25,
                assigned_workers: 0,
            }),
            SiegeWorkshop => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Trainer,
                worker_efficiency: 0.75,
                full_staff_bonus: 1.30,
                assigned_workers: 0,
            }),
            Mosque => Some(Self {
                max_workers: 2,
                required_role: WorkerRole::Civic,
                worker_efficiency: 0.90,
                full_staff_bonus: 1.40,
                assigned_workers: 0,
            }),
            CouncilChamber => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Civic,
                worker_efficiency: 0.85,
                full_staff_bonus: 1.30,
                assigned_workers: 0,
            }),
            GuildHall => Some(Self {
                max_workers: 3,
                required_role: WorkerRole::Civic,
                worker_efficiency: 0.80,
                full_staff_bonus: 1.35,
                assigned_workers: 0,
            }),
            // Buildings that do not employ workers
            Oba | Watchtower | Palisade | StoneWall | Gate | Road | Well | Monument => None,
        }
    }
}

// ─── WorkerRole ───────────────────────────────────────────────────────────────

/// The role a worker performs in a building, used for priority matching during
/// worker allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum WorkerRole {
    /// Resource gathering: Pasture, MiningCamp, LumberCamp
    Gatherer,
    /// Manufacturing / trade: Market, Warehouse
    Merchant,
    /// Military training: Barracks, ArcheryRange, Stable, SiegeWorkshop
    Trainer,
    /// Civic / religious: Mosque, CouncilChamber, GuildHall
    Civic,
    /// Construction site (any building under construction)
    Builder,
    /// Unassigned — available for allocation
    Idle,
}

impl Default for WorkerRole {
    fn default() -> Self {
        Self::Idle
    }
}
