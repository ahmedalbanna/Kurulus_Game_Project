use bevy::prelude::*;
use bevy::math::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::components::TerrainType;

/// Global terrain grid — maps grid positions to their terrain type.
/// Systems read this to determine stealth modifiers, movement costs,
/// vision blocking, and other terrain-dependent properties.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct TerrainGrid {
    pub tiles: HashMap<IVec2, TerrainType>,
}

impl Default for TerrainGrid {
    fn default() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }
}

impl TerrainGrid {
    pub fn get(&self, pos: IVec2) -> TerrainType {
        self.tiles.get(&pos).copied().unwrap_or(TerrainType::OpenField)
    }

    pub fn stealth_modifier(&self, pos: IVec2) -> f32 {
        self.get(pos).stealth_modifier()
    }

    pub fn blocks_vision(&self, pos: IVec2) -> bool {
        self.get(pos).blocks_vision()
    }

    pub fn movement_cost(&self, pos: IVec2) -> f32 {
        self.get(pos).movement_cost()
    }

    pub fn set(&mut self, pos: IVec2, terrain: TerrainType) {
        self.tiles.insert(pos, terrain);
    }
}
