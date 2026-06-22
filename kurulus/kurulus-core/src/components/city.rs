use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::building::BuildingType;

// ─── Zone Components ─────────────────────────────────────────────────────────

/// Marks a building as belonging to a city zone district.
/// Zones provide adjacency bonuses to buildings of the same zone type.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Zone {
    pub zone_type: ZoneType,
    pub level: u8,
}

/// The type of city zone district.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZoneType {
    /// Houses, population capacity (Oba, upgraded Obas)
    Residential,
    /// Barracks, ArcheryRange, Stable, SiegeWorkshop, Watchtower
    Military,
    /// Market, Warehouse, Pasture, MiningCamp, LumberCamp
    Economic,
    /// Mosque, Monument
    Religious,
    /// CouncilChamber, GuildHall
    Administrative,
}

impl ZoneType {
    /// Returns the bonus this zone type gives to buildings within it.
    /// Used for adjacency and same-zone bonuses.
    pub fn adjacency_bonus(&self) -> &str {
        match self {
            ZoneType::Residential => "population_growth",
            ZoneType::Military => "recruit_speed",
            ZoneType::Economic => "resource_output",
            ZoneType::Religious => "faith",
            ZoneType::Administrative => "happiness",
        }
    }

    /// Returns the buildings that naturally belong to this zone.
    pub fn matches_building(&self, bt: BuildingType) -> bool {
        match self {
            ZoneType::Residential => matches!(bt, BuildingType::Oba),
            ZoneType::Military => matches!(
                bt,
                BuildingType::Barracks
                    | BuildingType::ArcheryRange
                    | BuildingType::Stable
                    | BuildingType::SiegeWorkshop
                    | BuildingType::Watchtower
            ),
            ZoneType::Economic => matches!(
                bt,
                BuildingType::Market
                    | BuildingType::Warehouse
                    | BuildingType::Pasture
                    | BuildingType::MiningCamp
                    | BuildingType::LumberCamp
            ),
            ZoneType::Religious => matches!(bt, BuildingType::Mosque | BuildingType::Monument),
            ZoneType::Administrative => {
                matches!(bt, BuildingType::CouncilChamber | BuildingType::GuildHall)
            }
        }
    }
}

// ─── Service Building Component ──────────────────────────────────────────────

/// Marks a building that provides a city service with a coverage radius.
/// Buildings within the radius receive the service benefit.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ServiceBuilding {
    pub service_type: ServiceType,
    /// Coverage radius in grid cells.
    pub coverage_radius: f32,
    /// How effective the service is (0.0 – 1.0).
    pub effectiveness: f32,
}

/// Types of city services provided by specialized buildings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    /// Wells provide clean water (reduces disease, improves sanitation).
    Water,
    /// Markets enable trade (improves economy, happiness).
    Trade,
    /// Mosques provide faith (improves happiness, reduces crime).
    Faith,
    /// Monuments provide prestige (improves happiness, attracts settlers).
    Prestige,
}

impl ServiceBuilding {
    /// Create the appropriate service component for a building type.
    pub fn from_building_type(bt: BuildingType, level: u8) -> Option<Self> {
        let level_factor = 1.0 + (level as f32 - 1.0) * 0.25;
        match bt {
            BuildingType::Well => Some(ServiceBuilding {
                service_type: ServiceType::Water,
                coverage_radius: 8.0,
                effectiveness: (0.3 * level_factor).min(1.0),
            }),
            BuildingType::Market => Some(ServiceBuilding {
                service_type: ServiceType::Trade,
                coverage_radius: 12.0,
                effectiveness: (0.4 * level_factor).min(1.0),
            }),
            BuildingType::Mosque => Some(ServiceBuilding {
                service_type: ServiceType::Faith,
                coverage_radius: 15.0,
                effectiveness: (0.5 * level_factor).min(1.0),
            }),
            BuildingType::Monument => Some(ServiceBuilding {
                service_type: ServiceType::Prestige,
                coverage_radius: 20.0,
                effectiveness: (0.6 * level_factor).min(1.0),
            }),
            _ => None,
        }
    }
}

// ─── Upgrade Chain Component ────────────────────────────────────────────────

/// Defines a named upgrade tier for a building within its upgrade chain.
/// E.g., Oba -> Manor -> Governor's Estate -> Bey's Palace -> Sultan's Court.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeChain {
    /// The base building type this chain applies to.
    pub base: BuildingType,
    /// Human-readable name for each level.
    pub tier_names: Vec<String>,
}

impl UpgradeChain {
    /// Returns the name for the given level (1-indexed).
    pub fn name_for_level(&self, level: u8) -> &str {
        if level == 0 || level as usize > self.tier_names.len() {
            "Unknown"
        } else {
            &self.tier_names[(level - 1) as usize]
        }
    }

    /// Returns the total number of tiers in this chain.
    pub fn tier_count(&self) -> u8 {
        self.tier_names.len() as u8
    }

    /// Get the default upgrade chain for a building type.
    pub fn for_building_type(bt: BuildingType) -> Option<Self> {
        let (base, tier_names): (BuildingType, Vec<&str>) = match bt {
            BuildingType::Oba => (
                BuildingType::Oba,
                vec![
                    "Oba",
                    "Manor",
                    "Governor's Estate",
                    "Bey's Palace",
                    "Sultan's Court",
                ],
            ),
            BuildingType::Pasture => (
                BuildingType::Pasture,
                vec!["Pasture", "Ranch", "Horse Farm"],
            ),
            BuildingType::MiningCamp => (
                BuildingType::MiningCamp,
                vec!["Mining Camp", "Mine", "Deep Quarry"],
            ),
            BuildingType::LumberCamp => (
                BuildingType::LumberCamp,
                vec!["Lumber Camp", "Sawmill", "Timber Yard"],
            ),
            BuildingType::Barracks => (
                BuildingType::Barracks,
                vec!["Barracks", "Garrison", "Fortress"],
            ),
            BuildingType::ArcheryRange => (
                BuildingType::ArcheryRange,
                vec!["Archery Range", "Training Grounds", "Marksman Hall"],
            ),
            BuildingType::Stable => (
                BuildingType::Stable,
                vec!["Stable", "Cavalry Post", "Horse Breeder"],
            ),
            BuildingType::SiegeWorkshop => (
                BuildingType::SiegeWorkshop,
                vec!["Siege Workshop", "Engineer Guild", "Arsenal"],
            ),
            BuildingType::Watchtower => (
                BuildingType::Watchtower,
                vec!["Watchtower", "Guard Tower", "Citadel"],
            ),
            BuildingType::Market => (
                BuildingType::Market,
                vec!["Market", "Bazaar", "Grand Bazaar"],
            ),
            BuildingType::Warehouse => (
                BuildingType::Warehouse,
                vec!["Warehouse", "Depot", "Trade Center"],
            ),
            BuildingType::Well => (
                BuildingType::Well,
                vec!["Well", "Fountain", "Aqueduct"],
            ),
            BuildingType::Mosque => (
                BuildingType::Mosque,
                vec!["Mosque", "Great Mosque", "Külliye"],
            ),
            BuildingType::CouncilChamber => (
                BuildingType::CouncilChamber,
                vec!["Council Chamber", "Divan Hall", "Palace"],
            ),
            BuildingType::GuildHall => (
                BuildingType::GuildHall,
                vec!["Guild Hall", "Merchant Guild", "Trade Consortium"],
            ),
            BuildingType::Monument => (
                BuildingType::Monument,
                vec!["Monument", "Great Monument", "Landmark"],
            ),
            _ => return None, // Palisade, StoneWall, Gate, Road have no upgrade chain
        };
        Some(Self {
            base,
            tier_names: tier_names.into_iter().map(String::from).collect(),
        })
    }
}

// ─── City Marker Component ───────────────────────────────────────────────────

/// Simple marker component to identify the city's "core" entity
/// (the main settlement entity on the world map).
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CityMarker;
