use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{BuildingType, WorkerRole};

// ─── PopulationRegistry ──────────────────────────────────────────────────────

/// Detailed population breakdown by role.
///
/// Kept in sync with `CityState.population` and `ResourceStockpile.population`
/// by `population_sync_system`.  This is the single source of truth for the
/// breakdown of the total population into workers, soldiers, and idle citizens.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct PopulationRegistry {
    /// Total citizens (must equal `CityState.population` after sync).
    pub total: u32,
    /// Citizens currently available for work assignment.
    pub idle: u32,
    /// Citizens currently assigned as workers in production buildings.
    pub workers: u32,
    /// Citizens serving in the military (synced from `CityState.total_military_pop`).
    pub soldiers: u32,
    /// Maximum population capacity (synced from `CityState.max_population`).
    pub capacity: u32,
}

impl Default for PopulationRegistry {
    fn default() -> Self {
        Self {
            total: 5,
            idle: 5,
            workers: 0,
            soldiers: 0,
            capacity: 20,
        }
    }
}

impl PopulationRegistry {
    /// Recompute `idle` from `total`, `workers` and `soldiers`.
    pub fn recompute_idle(&mut self) {
        let used = self.workers.saturating_add(self.soldiers);
        self.idle = self.total.saturating_sub(used);
    }
}

// ─── WorkerAssignment (per-building record) ──────────────────────────────────

/// A single building's worker assignment record, stored in `WorkerAllocation`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkerAssignment {
    /// The building entity receiving workers.
    pub building_entity: Entity,
    /// Building type (for quick filtering / UI).
    pub building_type: BuildingType,
    /// Number of workers currently assigned.
    pub worker_count: u32,
    /// Role of these workers.
    pub role: WorkerRole,
}

// ─── WorkerAllocation ────────────────────────────────────────────────────────

/// Tracks all worker assignments globally.
///
/// Populated each tick by `worker_allocation_system` after it scans buildings
/// with `WorkerSlot` components.  The `worker_output_system` reads this to
/// scale production rates.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WorkerAllocation {
    /// Per-building assignment records.
    pub assignments: Vec<WorkerAssignment>,
    /// Total workers assigned across all buildings.
    pub total_assigned: u32,
}

impl Default for WorkerAllocation {
    fn default() -> Self {
        Self {
            assignments: Vec::new(),
            total_assigned: 0,
        }
    }
}

// ─── GrowthConfig ────────────────────────────────────────────────────────────

/// Configurable population growth parameters.
///
/// These can be loaded from a RON asset at startup or set to sensible defaults.
/// All rates are per `FixedUpdate` tick (~0.1 game-seconds).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GrowthConfig {
    /// Base growth rate per tick when all needs are fully satisfied.
    pub base_growth_rate: f32,
    /// Multiplier applied when `city.happiness > 80`.
    pub high_happiness_bonus: f32,
    /// Growth-rate multiplier from residential zone adjacency bonuses.
    pub zone_growth_mult: f32,
    /// Minimum `CitizenNeeds.food_satisfaction` required for population growth.
    pub food_growth_threshold: f32,
    /// Threshold below which starvation starts reducing population.
    pub starvation_threshold: f32,
    /// Death rate per tick when starvation is active.
    pub starvation_death_rate: f32,
    /// Housing saturation ratio (`pop / max_pop`) above which growth is dampened.
    pub housing_saturation_threshold: f32,
    /// Weight of faith satisfaction in the growth-rate formula.
    pub faith_growth_weight: f32,
    /// Weight of safety satisfaction in the growth-rate formula.
    pub safety_growth_weight: f32,
    /// Accumulated fractional population (carried between ticks).
    pub accumulator: f32,
    /// How many game-ticks must elapse between two allocation passes.
    /// A value of 1 means every tick; higher values reduce churn.
    pub allocation_interval_ticks: u32,
    /// Ticks since last allocation (internal counter).
    pub allocation_cooldown: u32,
}

impl Default for GrowthConfig {
    fn default() -> Self {
        Self {
            base_growth_rate: 0.15,
            high_happiness_bonus: 1.5,
            zone_growth_mult: 1.25,
            food_growth_threshold: 0.3,
            starvation_threshold: 0.15,
            starvation_death_rate: 0.05,
            housing_saturation_threshold: 0.85,
            faith_growth_weight: 0.15,
            safety_growth_weight: 0.10,
            accumulator: 0.0,
            allocation_interval_ticks: 5,
            allocation_cooldown: 0,
        }
    }
}

// ─── WorkerSlotConfig ────────────────────────────────────────────────────────

/// Building-specific `WorkerSlot` definitions, keyed by `BuildingType`.
///
/// When a building is completed the `complete_construction_system` looks up
/// the appropriate slot configuration from this resource instead of hardcoding
/// values.  The resource can be loaded from a RON asset at startup.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WorkerSlotConfig {
    /// Ordered (type, config) pairs.
    pub configs: Vec<(BuildingType, crate::components::population::WorkerSlot)>,
}

impl Default for WorkerSlotConfig {
    fn default() -> Self {
        use crate::components::building::BuildingType;
        use crate::components::population::{WorkerRole, WorkerSlot};
        Self {
            configs: vec![
                (
                    BuildingType::Pasture,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Gatherer,
                        worker_efficiency: 0.9,
                        full_staff_bonus: 1.30,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::MiningCamp,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Gatherer,
                        worker_efficiency: 0.85,
                        full_staff_bonus: 1.25,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::LumberCamp,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Gatherer,
                        worker_efficiency: 0.95,
                        full_staff_bonus: 1.20,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::Market,
                    WorkerSlot {
                        max_workers: 4,
                        required_role: WorkerRole::Merchant,
                        worker_efficiency: 0.80,
                        full_staff_bonus: 1.35,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::Warehouse,
                    WorkerSlot {
                        max_workers: 2,
                        required_role: WorkerRole::Merchant,
                        worker_efficiency: 0.90,
                        full_staff_bonus: 1.15,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::Barracks,
                    WorkerSlot {
                        max_workers: 2,
                        required_role: WorkerRole::Trainer,
                        worker_efficiency: 0.85,
                        full_staff_bonus: 1.20,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::ArcheryRange,
                    WorkerSlot {
                        max_workers: 2,
                        required_role: WorkerRole::Trainer,
                        worker_efficiency: 0.85,
                        full_staff_bonus: 1.20,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::Stable,
                    WorkerSlot {
                        max_workers: 2,
                        required_role: WorkerRole::Trainer,
                        worker_efficiency: 0.80,
                        full_staff_bonus: 1.25,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::SiegeWorkshop,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Trainer,
                        worker_efficiency: 0.75,
                        full_staff_bonus: 1.30,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::Mosque,
                    WorkerSlot {
                        max_workers: 2,
                        required_role: WorkerRole::Civic,
                        worker_efficiency: 0.90,
                        full_staff_bonus: 1.40,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::CouncilChamber,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Civic,
                        worker_efficiency: 0.85,
                        full_staff_bonus: 1.30,
                        ..Default::default()
                    },
                ),
                (
                    BuildingType::GuildHall,
                    WorkerSlot {
                        max_workers: 3,
                        required_role: WorkerRole::Civic,
                        worker_efficiency: 0.80,
                        full_staff_bonus: 1.35,
                        ..Default::default()
                    },
                ),
            ],
        }
    }
}

impl WorkerSlotConfig {
    /// Look up the `WorkerSlot` for a given `BuildingType`.
    /// Returns a default `WorkerSlot` (with `max_workers: 0`) if no entry exists,
    /// so that callers can treat all building types uniformly.
    pub fn slot_for(&self, bt: BuildingType) -> crate::components::population::WorkerSlot {
        self.configs
            .iter()
            .find(|(t, _)| *t == bt)
            .map(|(_, s)| s.clone())
            .unwrap_or_default()
    }
}
