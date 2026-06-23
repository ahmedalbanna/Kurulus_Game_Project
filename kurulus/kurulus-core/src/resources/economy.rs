use crate::components::FactionId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Per-capita resource consumption rates, applied each tick.
///
/// Each field is the amount consumed **per citizen per game-second**.
/// The actual per-tick deduction is:
/// `population × rate × delta × policy_multiplier`
///
/// Only **food** is affected by the `RationingLevel` multiplier from
/// `PolicySettings` (if that resource is available).  Gold and wood
/// are consumed at flat rates representing non-discretionary upkeep
/// (luxury goods, administrative costs, fuel, tool repair, etc.).
///
/// Stone is **not** consumed daily — it is only drawn from the stockpile
/// for construction projects.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ResourceConsumption {
    /// Food consumed per citizen per game-second (before rationing).
    pub food_per_second_per_capita: f32,
    /// Gold consumed per citizen per game-second (luxury / admin upkeep).
    pub gold_per_second_per_capita: f32,
    /// Wood consumed per citizen per game-second (fuel / tools).
    pub wood_per_second_per_capita: f32,
}

impl Default for ResourceConsumption {
    fn default() -> Self {
        Self {
            // 5 people consume 0.0025/tick (same as old hardcoded rate);
            // 20 people consume 0.01/tick (10% of base production);
            // 100 people consume 0.05/tick (50% of base production).
            food_per_second_per_capita: 0.005,
            // 5 people consume 0.001/tick;
            // 100 people consume 0.02/tick (40% of gold production).
            gold_per_second_per_capita: 0.002,
            // 5 people consume 0.0015/tick;
            // 100 people consume 0.03/tick (60% of wood production).
            wood_per_second_per_capita: 0.003,
        }
    }
}
