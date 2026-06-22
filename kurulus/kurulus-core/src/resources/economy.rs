use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::FactionId;

#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct ResourceStockpile {
    pub food: f32,
    pub gold: f32,
    pub wood: f32,
    pub stone: f32,
    pub population: u32,
    pub max_population: u32,
    pub sovereignty_points: u32,
}

impl Default for ResourceStockpile {
    fn default() -> Self {
        Self {
            food: 100.0,
            gold: 50.0,
            wood: 50.0,
            stone: 20.0,
            population: 5,
            max_population: 20,
            sovereignty_points: 0,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct TaxPolicy {
    pub rate: f32,
    pub happiness_impact: f32,
    pub last_collection: f32,
}

impl Default for TaxPolicy {
    fn default() -> Self {
        Self {
            rate: 0.1,
            happiness_impact: 0.0,
            last_collection: 0.0,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ProductionRates {
    pub food_per_second: f32,
    pub gold_per_second: f32,
    pub wood_per_second: f32,
    pub stone_per_second: f32,
}

impl Default for ProductionRates {
    fn default() -> Self {
        Self {
            food_per_second: 1.0,
            gold_per_second: 0.5,
            wood_per_second: 0.5,
            stone_per_second: 0.2,
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize, Default)]
pub struct FactionRelations {
    pub relations: Vec<(FactionId, FactionId, f32)>,
}
