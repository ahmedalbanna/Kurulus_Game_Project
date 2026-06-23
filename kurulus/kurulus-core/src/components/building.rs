use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::unit::FactionId;
use crate::coord::GridPos;

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

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BuildCost {
    pub food: u32,
    pub gold: u32,
    pub wood: u32,
    pub stone: u32,
    pub population: u32,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum BuildingType {
    // Production
    Oba,           // Main tent / town center
    Pasture,       // Food production
    MiningCamp,    // Gold/stone
    LumberCamp,    // Wood
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
