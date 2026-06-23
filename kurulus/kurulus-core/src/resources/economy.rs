use crate::components::FactionId;
use crate::components::ResourceType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct ResourceStockpile {
    pub food: f32,
    pub gold: f32,
    pub wood: f32,
    pub stone: f32,
    pub iron: f32,

    // Population
    pub population: u32,
    pub max_population: u32,

    // Meta-progression
    pub sovereignty_points: u32,

    // ── Capacity limits ───────────────────────────────────────────
    pub max_food: f32,
    pub max_gold: f32,
    pub max_wood: f32,
    pub max_stone: f32,
    pub max_iron: f32,
}

impl Default for ResourceStockpile {
    fn default() -> Self {
        Self {
            food: 100.0,
            gold: 50.0,
            wood: 50.0,
            stone: 20.0,
            iron: 0.0,
            population: 5,
            max_population: 20,
            sovereignty_points: 0,
            // Base storage capacities (before Warehouse bonuses)
            max_food: 150.0,
            max_gold: 50.0,
            max_wood: 200.0,
            max_stone: 100.0,
            max_iron: 100.0,
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

// ═══════════════════════════════════════════════════════════════════════════════
// Market Trade Rates
// ═══════════════════════════════════════════════════════════════════════════════

/// Configurable trade exchange rates for the Market.
///
/// Uses a "base value" model: each resource has an intrinsic value, and the
/// amount received is `give_amount × give_value / receive_value`.
///
/// Default values use Food=1.0, Wood=1.0, Stone=1.5, Gold=2.0, Iron=3.0.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct TradeRates {
    /// Maps each `ResourceType` to its base value per unit.
    pub values: HashMap<ResourceType, f32>,
}

impl Default for TradeRates {
    fn default() -> Self {
        let mut values = HashMap::new();
        values.insert(ResourceType::Food, 1.0);
        values.insert(ResourceType::Wood, 1.0);
        values.insert(ResourceType::Stone, 1.5);
        values.insert(ResourceType::Gold, 2.0);
        values.insert(ResourceType::Iron, 3.0);
        Self { values }
    }
}

impl TradeRates {
    /// Compute how much `receive_resource` you get for `give_amount` of `give_resource`.
    pub fn trade_amount(
        &self,
        give_resource: ResourceType,
        give_amount: f32,
        receive_resource: ResourceType,
    ) -> f32 {
        let give_val = self.values.get(&give_resource).copied().unwrap_or(1.0);
        let recv_val = self.values.get(&receive_resource).copied().unwrap_or(1.0);
        if recv_val == 0.0 {
            return 0.0;
        }
        give_amount * give_val / recv_val
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Weather Modifiers
// ═══════════════════════════════════════════════════════════════════════════════

/// Per‑season / per‑time-of‑day gameplay modifiers computed from `GameTime`.
///
/// These are written once per tick (by `weather_modifier_system`) so that
/// other systems (construction, production, movement) can read them without
/// each querying `GameTime` directly.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherModifiers {
    /// Construction speed multiplier (1.0 = normal).  Winter = 0.7, night = 0.8.
    pub construction_speed_modifier: f32,
    /// Resource production multiplier.  Winter = 0.8, night = 0.9.
    pub production_modifier: f32,
    /// Unit movement speed multiplier.  Winter = 0.75, night = 1.0.
    pub movement_speed_modifier: f32,
}

impl Default for WeatherModifiers {
    fn default() -> Self {
        Self {
            construction_speed_modifier: 1.0,
            production_modifier: 1.0,
            movement_speed_modifier: 1.0,
        }
    }
}
