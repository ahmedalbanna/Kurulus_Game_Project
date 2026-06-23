use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Types of terrain that affect stealth, movement, and vision.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    OpenField,
    Forest,
    Water,
    Mountain,
    Road,
    Building,
}

impl TerrainType {
    /// Stealth modifier for this terrain type.
    /// Positive = easier to hide, negative = harder.
    pub fn stealth_modifier(&self) -> f32 {
        match self {
            TerrainType::Forest => 0.30,
            TerrainType::OpenField => -0.10,
            TerrainType::Water => 0.0,
            TerrainType::Mountain => 0.15,
            TerrainType::Road => -0.05,
            TerrainType::Building => 0.20,
        }
    }

    /// Whether this terrain blocks line of sight (for fog of war).
    pub fn blocks_vision(&self) -> bool {
        matches!(self, TerrainType::Mountain | TerrainType::Building)
    }

    /// Movement cost multiplier for pathfinding.
    pub fn movement_cost(&self) -> f32 {
        match self {
            TerrainType::OpenField => 1.0,
            TerrainType::Forest => 1.5,
            TerrainType::Water => 3.0,
            TerrainType::Mountain => 2.5,
            TerrainType::Road => 0.8,
            TerrainType::Building => 2.0,
        }
    }
}

/// Component attached to tile entities or used in a grid resource.
/// Associates a grid position with its terrain properties.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct TerrainTile {
    pub position: GridPos,
    pub terrain: TerrainType,
}
