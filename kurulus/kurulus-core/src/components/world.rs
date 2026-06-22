use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::unit::FactionId;
use crate::coord::GridPos;

/// A named region on the world map with ownership and resource data.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub region_id: RegionId,
    pub name: String,
    pub owner: FactionId,
    pub resource_bonus: RegionResources,
    pub control_points: Vec<GridPos>,
}

/// A faction entity on the world map.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub faction_id: FactionId,
    pub name: String,
    pub color: Color,
    pub is_player: bool,
    pub is_alive: bool,
}

/// Fog of war state for a single world-map tile.
/// Attached to tile entities on the tactical grid, or stored in the WorldMap resource.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FogOfWar {
    pub explored: bool,
    pub visible: bool,
    pub last_seen: f64,
}

impl Default for FogOfWar {
    fn default() -> Self {
        Self {
            explored: false,
            visible: false,
            last_seen: 0.0,
        }
    }
}

/// Fog state enum for the world map (strategic layer).
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FogState {
    /// Never seen — completely dark, no information
    Unexplored,
    /// Previously visible — dimmed, terrain visible but no units/activity
    Explored,
    /// Currently visible — fully lit, all activity visible
    Visible,
}

impl Default for FogState {
    fn default() -> Self {
        Self::Unexplored
    }
}

/// A single tile on the world map (strategic layer, not tactical).
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct WorldTile {
    pub position: IVec2,
    pub terrain: WorldTerrain,
    pub fog: FogState,
    pub region: Option<RegionId>,
    /// Elevation for visual shading (0.0 = sea level, 1.0 = peak)
    pub elevation: f32,
}

impl Default for WorldTile {
    fn default() -> Self {
        Self {
            position: IVec2::ZERO,
            terrain: WorldTerrain::Grassland,
            fog: FogState::Unexplored,
            region: None,
            elevation: 0.0,
        }
    }
}

/// Terrain types for the world map (strategic layer).
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorldTerrain {
    Grassland,
    Forest,
    Mountain,
    Water,
    Desert,
    Snow,
    Road,
    Settlement,
    Fortress,
}

impl WorldTerrain {
    /// Movement cost to traverse this terrain on the world map.
    pub fn travel_cost(&self) -> f32 {
        match self {
            WorldTerrain::Grassland => 1.0,
            WorldTerrain::Forest => 1.5,
            WorldTerrain::Mountain => 3.0,
            WorldTerrain::Water => 99.0, // impassable without ships
            WorldTerrain::Desert => 2.0,
            WorldTerrain::Snow => 2.5,
            WorldTerrain::Road => 0.6,
            WorldTerrain::Settlement => 1.0,
            WorldTerrain::Fortress => 1.0,
        }
    }

    /// Whether this terrain blocks vision on the world map.
    pub fn blocks_vision(&self) -> bool {
        matches!(self, WorldTerrain::Mountain)
    }
}

/// A defined region on the world map: its tile bounds, owner, and properties.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct RegionDef {
    pub region_id: RegionId,
    pub name: String,
    /// All tile positions belonging to this region
    pub tiles: Vec<IVec2>,
    pub owner: FactionId,
    pub resources: RegionResources,
    pub strategic_value: u32,
    /// How strong the garrison is (0.0 = undefended, 1.0 = heavily fortified)
    pub garrison_strength: f32,
}

/// Travel state for a hero or army moving between regions on the world map.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct WorldMapTravel {
    pub from_region: RegionId,
    pub to_region: RegionId,
    pub progress: f32,
    pub duration: f32,
    pub faction: FactionId,
}

/// Marker component for entities currently on the world map (not in tactical mission).
#[derive(Component, Clone, Debug, Default)]
pub struct OnWorldMap;

/// Resource data for a region (same as before but now used in RegionDef too).
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct RegionResources {
    pub food: f32,
    pub gold: f32,
    pub wood: f32,
    pub stone: f32,
    pub population_capacity: u32,
}

impl Default for RegionResources {
    fn default() -> Self {
        Self {
            food: 0.0,
            gold: 0.0,
            wood: 0.0,
            stone: 0.0,
            population_capacity: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Points of Interest
// ═══════════════════════════════════════════════════════════════════════════

/// A point of interest on the world map that can be discovered and interacted with.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct PointOfInterest {
    pub poi_id: PoiId,
    pub name: String,
    pub position: IVec2,
    pub poi_type: PoiType,
    pub discovered: bool,
    pub description: String,
    pub region: Option<RegionId>,
}

/// Category of a point of interest.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoiType {
    Village,
    Fort,
    Ruin,
    TradePost,
    MountainPass,
    RiverCrossing,
    SacredSite,
    Cave,
    Camp,
}

impl PoiType {
    pub fn display_name(&self) -> &str {
        match self {
            PoiType::Village => "Village",
            PoiType::Fort => "Fort",
            PoiType::Ruin => "Ruin",
            PoiType::TradePost => "Trade Post",
            PoiType::MountainPass => "Mountain Pass",
            PoiType::RiverCrossing => "River Crossing",
            PoiType::SacredSite => "Sacred Site",
            PoiType::Cave => "Cave",
            PoiType::Camp => "Camp",
        }
    }
}

/// Named POIs matching Ottoman history in the game's region.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoiId {
    /// Osman's initial camp / tribe centre
    KayiCamp,
    /// Domaniç Plateau — summer pasture
    DomanicPlateau,
    /// Ermeni Beli — strategic mountain pass
    ErmeniBeliPass,
    /// Karacahisar Fort — first Ottoman conquest
    KaracahisarFort,
    /// Bilecik Bazaar — local trade hub
    BilecikBazaar,
    /// Yarhisar Watchtower — Byzantine fortress
    YarhisarTower,
    /// İnegöl Market — Byzantine town centre
    InegolMarket,
    /// Köprühisar Bridge — strategic bridge over gorge
    KopruhisarBridge,
    /// Uludağ slopes — sacred mountain
    UludagShrine,
    /// Lake İznik — ancient walls and lake
    IznikLake,
    /// Sakarya River — major river crossing
    SakaryaCrossing,
    /// Söğüt Caves — hidden refuge
    SogutCaves,
    /// Marmara Coast — access to the sea
    MarmaraCoast,
    /// Harmankaya Fortress — Byzantine stronghold
    HarmankayaFort,
}

impl PoiId {
    pub fn display_name(&self) -> &str {
        match self {
            PoiId::KayiCamp => "Kayı Camp",
            PoiId::DomanicPlateau => "Domaniç Plateau",
            PoiId::ErmeniBeliPass => "Ermeni Beli Pass",
            PoiId::KaracahisarFort => "Karacahisar Fort",
            PoiId::BilecikBazaar => "Bilecik Bazaar",
            PoiId::YarhisarTower => "Yarhisar Watchtower",
            PoiId::InegolMarket => "İnegöl Market",
            PoiId::KopruhisarBridge => "Köprühisar Bridge",
            PoiId::UludagShrine => "Uludağ Shrine",
            PoiId::IznikLake => "Lake İznik",
            PoiId::SakaryaCrossing => "Sakarya Crossing",
            PoiId::SogutCaves => "Söğüt Caves",
            PoiId::MarmaraCoast => "Marmara Coast",
            PoiId::HarmankayaFort => "Harmankaya Fortress",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Encounters
// ═══════════════════════════════════════════════════════════════════════════

/// A random encounter that can trigger while travelling on the world map.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct Encounter {
    pub encounter_type: EncounterType,
    pub description: String,
    pub difficulty: EncounterDifficulty,
    /// Possible choices the player can make (2-3 options)
    pub choices: Vec<EncounterChoice>,
    /// Which faction is involved (if any)
    pub faction: Option<FactionId>,
    /// Tile position where the encounter occurs
    pub position: IVec2,
}

/// Types of random encounters.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncounterType {
    Merchants,
    Bandits,
    Refugees,
    Scouts,
    Nomads,
    WildAnimals,
    Hermit,
    Omen,
    Patrol,
    Traveler,
}

impl EncounterType {
    pub fn display_name(&self) -> &str {
        match self {
            EncounterType::Merchants => "Merchants",
            EncounterType::Bandits => "Bandits",
            EncounterType::Refugees => "Refugees",
            EncounterType::Scouts => "Scouts",
            EncounterType::Nomads => "Nomads",
            EncounterType::WildAnimals => "Wild Animals",
            EncounterType::Hermit => "Hermit",
            EncounterType::Omen => "Omen",
            EncounterType::Patrol => "Patrol",
            EncounterType::Traveler => "Traveler",
        }
    }
}

/// Difficulty of an encounter.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncounterDifficulty {
    Easy,
    Medium,
    Hard,
    Deadly,
}

/// A choice the player can make when encountering an event.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct EncounterChoice {
    pub text: String,
    pub outcome: EncounterOutcome,
}

/// The result of choosing a particular encounter option.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct EncounterOutcome {
    /// Gold gained (positive) or lost (negative)
    pub gold_delta: f32,
    /// Reputation change with the involved faction (if any)
    pub reputation_delta: f32,
    /// A short flavour text describing the result
    pub result_text: String,
}

// --- Enums ---

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionId {
    Sogut,
    Domanic,
    ErmeniBeli,
    Karacahisar,
    Bilecik,
    Yarhisar,
    Inegol,
    Kopruhisar,
    Bursa,
    Iznik,
    Izmit,
    Sakarya,
}

impl RegionId {
    /// Human-readable display name
    pub fn display_name(&self) -> &str {
        match self {
            RegionId::Sogut => "Söğüt",
            RegionId::Domanic => "Domaniç",
            RegionId::ErmeniBeli => "Ermeni Beli",
            RegionId::Karacahisar => "Karacahisar",
            RegionId::Bilecik => "Bilecik",
            RegionId::Yarhisar => "Yarhisar",
            RegionId::Inegol => "İnegöl",
            RegionId::Kopruhisar => "Köprühisar",
            RegionId::Bursa => "Bursa",
            RegionId::Iznik => "İznik",
            RegionId::Izmit => "İzmit",
            RegionId::Sakarya => "Sakarya",
        }
    }
}
