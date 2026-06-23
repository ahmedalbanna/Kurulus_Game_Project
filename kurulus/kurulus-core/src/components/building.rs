use super::unit::FactionId;
use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub faction: FactionId,
    pub level: u8,
    pub max_level: u8,
    pub name: String,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ConstructionSite {
    pub building_type: BuildingType,
    pub progress: f32,
    pub max_progress: f32,
    pub assigned_workers: Vec<Entity>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BuildQueue {
    pub queue: Vec<BuildQueueItem>,
    pub max_queue_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildQueueItem {
    pub building_type: BuildingType,
    pub position: GridPos,
    pub progress: f32,
    pub total_cost: BuildCost,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct BuildCost {
    pub food: u32,
    pub gold: u32,
    pub wood: u32,
    pub stone: u32,
    pub iron: u32,
    pub population: u32,
}

/// Resource type for production, trading, and cost definitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum ResourceType {
    Food,
    Gold,
    Wood,
    Stone,
    Iron,
    Population,
}

/// Categorises buildings for limit enforcement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum BuildingCategory {
    /// Basic & production buildings (most structures).
    Basic,
    /// Defensive walls, gates, watchtowers.
    Defense,
    /// Siege Workshop and other special structures.
    Special,
}

/// One prerequisite for constructing or upgrading a building.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct BuildingDependency {
    /// The building type that must exist.
    pub required_building: BuildingType,
    /// Minimum level required.
    pub required_level: u8,
}

/// Component attached to buildings that records their construction
/// prerequisites (loaded from the catalog at spawn time).
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BuildingPrerequisites {
    pub dependencies: Vec<BuildingDependency>,
}

/// Per-level upgrade cost and time definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeCost {
    pub level: u8,
    pub cost: BuildCost,
    pub build_time: f32,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum BuildingType {
    // Production
    Oba,        // Main tent / town center
    Pasture,    // Food production
    MiningCamp, // Gold/stone
    LumberCamp, // Wood
    // Military
    Barracks,
    ArcheryRange,
    Stable,
    SiegeWorkshop,
    Watchtower,
    // Infrastructure
    Road,
    Market,
    Warehouse,
    Well,
    // Defensive
    Palisade,
    StoneWall,
    Gate,
    // Special
    Mosque,
    CouncilChamber,
    GuildHall,
    Monument,
}

impl BuildingType {
    /// Returns the category used for building-limit enforcement.
    pub fn category(&self) -> BuildingCategory {
        match self {
            // Defense structures
            BuildingType::Palisade
            | BuildingType::StoneWall
            | BuildingType::Gate
            | BuildingType::Watchtower => BuildingCategory::Defense,
            // Special structures
            BuildingType::SiegeWorkshop => BuildingCategory::Special,
            // Everything else is Basic
            _ => BuildingCategory::Basic,
        }
    }

    pub fn build_time(&self) -> f32 {
        match self {
            BuildingType::Oba => 60.0,
            BuildingType::Pasture => 20.0,
            BuildingType::MiningCamp => 25.0,
            BuildingType::LumberCamp => 15.0,
            BuildingType::Barracks => 40.0,
            BuildingType::ArcheryRange => 35.0,
            BuildingType::Stable => 45.0,
            BuildingType::SiegeWorkshop => 60.0,
            BuildingType::Watchtower => 30.0,
            BuildingType::Road => 5.0,
            BuildingType::Market => 30.0,
            BuildingType::Warehouse => 25.0,
            BuildingType::Well => 20.0,
            BuildingType::Palisade => 15.0,
            BuildingType::StoneWall => 40.0,
            BuildingType::Gate => 30.0,
            BuildingType::Mosque => 50.0,
            BuildingType::CouncilChamber => 45.0,
            BuildingType::GuildHall => 35.0,
            BuildingType::Monument => 80.0,
        }
    }

    pub fn max_level(&self) -> u8 {
        match self {
            BuildingType::Oba => 5,
            BuildingType::Road => 1,
            _ => 3,
        }
    }
}

/// A single level definition from the building catalog.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingLevelDef {
    pub level: u8,
    pub cost: BuildCost,
    pub build_time: f32,
    pub health: f32,
    pub armor: f32,
}

/// An entry in the building catalog describing all levels of one building type.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingCatalogEntry {
    pub building_type: BuildingType,
    pub category: BuildingCategory,
    pub levels: Vec<BuildingLevelDef>,
    pub dependencies: Vec<BuildingDependency>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 3 — Resource Production & Upkeep
// ═══════════════════════════════════════════════════════════════════════════════

/// Per‑tick resource production rates attached to a completed `Building`.
///
/// Added by `complete_construction_system` using hardcoded defaults
/// from `BuildingResourceProduction::for_building_type`.  In Phase 4 these
/// values will be fetched from `BuildingCatalog` instead.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct BuildingResourceProduction {
    /// Food produced per game‑second.
    pub food_per_sec: f32,
    /// Gold produced per game‑second.
    pub gold_per_sec: f32,
    /// Wood produced per game‑second.
    pub wood_per_sec: f32,
    /// Stone produced per game‑second.
    pub stone_per_sec: f32,
    /// Iron produced per game‑second.
    pub iron_per_sec: f32,
}

impl BuildingResourceProduction {
    /// Return the base (no workers) per‑second production for a building type.
    /// Returns `None` for buildings that do not produce resources.
    pub fn for_building_type(bt: BuildingType, level: u8) -> Option<Self> {
        use BuildingType::*;
        // Level multiplier: L1 = 1.0, L2 = 2.0, L3 = 3.5
        let mult = match level {
            0 | 1 => 1.0,
            2 => 2.0,
            _ => 3.5,
        };
        let base = match bt {
            Pasture => Self {
                food_per_sec: 2.0 * mult,
                ..Default::default()
            },
            MiningCamp => Self {
                stone_per_sec: 1.0 * mult,
                gold_per_sec: 0.5 * mult,
                ..Default::default()
            },
            LumberCamp => Self {
                wood_per_sec: 2.0 * mult,
                ..Default::default()
            },
            Oba => Self {
                gold_per_sec: 0.5 * mult,
                ..Default::default()
            },
            Market => Self {
                gold_per_sec: 1.0 * mult,
                ..Default::default()
            },
            // All other building types produce nothing
            _ => return None,
        };
        Some(base)
    }
}

/// Per‑tick upkeep costs attached to a completed `Building`.
///
/// Deducted from the stockpile each game‑second.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct BuildingUpkeep {
    /// Gold consumed per game‑second.
    pub gold_per_sec: f32,
    /// Wood consumed per game‑second (walls, palisades).
    pub wood_per_sec: f32,
}

impl BuildingUpkeep {
    /// Return the per‑second upkeep cost for a building type.
    /// Returns `None` for buildings with no upkeep.
    pub fn for_building_type(bt: BuildingType, level: u8) -> Option<Self> {
        use BuildingType::*;
        let mult = match level {
            0 | 1 => 1.0,
            2 => 1.8,
            _ => 3.5,
        };
        let base = match bt {
            Barracks => Some(Self {
                gold_per_sec: 0.5 * mult,
                ..Default::default()
            }),
            ArcheryRange => Some(Self {
                gold_per_sec: 0.3 * mult,
                ..Default::default()
            }),
            Stable => Some(Self {
                gold_per_sec: 0.8 * mult,
                ..Default::default()
            }),
            SiegeWorkshop => Some(Self {
                gold_per_sec: 1.0 * mult,
                ..Default::default()
            }),
            Palisade => Some(Self {
                wood_per_sec: 0.1 * mult,
                ..Default::default()
            }),
            StoneWall => Some(Self {
                gold_per_sec: 0.2 * mult,
                ..Default::default()
            }),
            Mosque => Some(Self {
                gold_per_sec: 0.3 * mult,
                ..Default::default()
            }),
            CouncilChamber => Some(Self {
                gold_per_sec: 0.5 * mult,
                ..Default::default()
            }),
            GuildHall => Some(Self {
                gold_per_sec: 0.2 * mult,
                ..Default::default()
            }),
            Monument => Some(Self {
                gold_per_sec: 1.0 * mult,
                ..Default::default()
            }),
            // Buildings with no upkeep
            Pasture | MiningCamp | LumberCamp | Oba | Market | Watchtower | Road | Warehouse
            | Well | Gate => None,
        };
        base
    }
}

/// Passive healing aura emitted by a Well building.
///
/// Each game-second, all allied units within `range` grid cells have their
/// `Health::current` increased by `heal_per_sec`.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct HealingAura {
    /// Radius in grid cells (world units).
    pub range: f32,
    /// HP restored per second per unit.
    pub heal_per_sec: f32,
    /// Which faction this aura heals (usually Kayi).
    pub faction: FactionId,
}

impl HealingAura {
    /// Return the default aura for a Well at the given level.
    pub fn for_well_level(level: u8) -> Self {
        let mult = match level {
            0 | 1 => 1.0,
            2 => 1.8,
            _ => 3.5,
        };
        Self {
            range: 4.0,
            heal_per_sec: 2.0 * mult,
            faction: FactionId::Kayi,
        }
    }
}

impl Default for HealingAura {
    fn default() -> Self {
        Self {
            range: 4.0,
            heal_per_sec: 2.0,
            faction: FactionId::Kayi,
        }
    }
}

/// Cardinal direction a building faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum CardinalDirection {
    North,
    East,
    South,
    West,
}

impl Default for CardinalDirection {
    fn default() -> Self {
        Self::South
    }
}

/// Rotation / facing of a building, affecting its sprite orientation.
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BuildingRotation {
    pub direction: CardinalDirection,
}

impl Default for BuildingRotation {
    fn default() -> Self {
        Self {
            direction: CardinalDirection::South,
        }
    }
}
