use crate::components::{
    AbilityCost, AbilityEffect, AbilityId, AbilityType, ArmorType, BuildCost, BuildingCatalogEntry,
    BuildingCategory, BuildingType, HeroId, MissionId, MissionRewards, ObjectiveId, ObjectiveType,
    TargetType, UnitArchetype, ZoneType,
};
use crate::resources::city::CityLevel;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Configuration for the three-phase game loop transitions.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PhaseTransitionConfig {
    /// How many seconds of no combat before Execution → Harvest.
    pub combat_inactivity_timeout: f32,
    /// How many seconds Harvest lasts before auto-returning to Planning.
    pub harvest_duration: f32,
    /// Minimum Planning phase duration (so brief orders don't skip too fast).
    pub min_planning_duration: f32,
}

impl Default for PhaseTransitionConfig {
    fn default() -> Self {
        Self {
            combat_inactivity_timeout: 10.0,
            harvest_duration: 5.0,
            min_planning_duration: 2.0,
        }
    }
}

/// Tracks the last time any combat event occurred (for Execution→Harvest).
#[derive(Resource, Clone, Debug)]
pub struct CombatActivityTracker {
    /// Total game seconds at which the last DamageDealt or AttackTriggered was observed.
    pub last_combat_game_time: f64,
}

impl Default for CombatActivityTracker {
    fn default() -> Self {
        Self {
            last_combat_game_time: 0.0,
        }
    }
}

/// Flags that at least one order was issued during the current Planning phase.
#[derive(Resource, Clone, Debug, Default)]
pub struct OrdersIssuedDuringPlanning {
    pub any_order_issued: bool,
}

// Config asset types used for loading .ron files

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct UnitConfig {
    pub archetype: UnitArchetype,
    pub name: String,
    pub stats: UnitStats,
    pub health: UnitHealth,
    pub cost: UnitCost,
    pub build_time: f32,
    pub abilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitStats {
    pub move_speed: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub vision_range: f32,
    pub morale: f32,
    pub morale_max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitHealth {
    pub current: f32,
    pub max: f32,
    pub armor: f32,
    pub armor_type: ArmorType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitCost {
    pub food: u32,
    pub gold: u32,
    pub population: u32,
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct BuildingConfig {
    pub building_type: BuildingType,
    pub name: String,
    pub health: f32,
    pub armor: f32,
    pub cost: BuildCost,
    pub build_time: f32,
    pub production: Option<BuildingProduction>,
    pub requires_tech: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingProduction {
    pub resource_type: String,
    pub amount_per_second: f32,
}

// ─── Building Catalog (config-driven building definitions) ──────────────────

/// The master building definitions catalog, loaded from RON at startup.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct BuildingCatalog {
    pub entries: Vec<BuildingCatalogEntry>,
}

impl BuildingCatalog {
    /// Look up a catalog entry for the given building type.
    pub fn entry_for(&self, bt: BuildingType) -> Option<&BuildingCatalogEntry> {
        self.entries.iter().find(|e| e.building_type == bt)
    }

    /// Return the level definition for a building at the given level.
    /// Returns `None` if the entry or level does not exist.
    pub fn level_def(
        &self,
        bt: BuildingType,
        level: u8,
    ) -> Option<&crate::components::building::BuildingLevelDef> {
        self.entry_for(bt)
            .and_then(|e| e.levels.iter().find(|l| l.level == level))
    }

    /// Return the cost for building at the given level (level 1 = build cost).
    pub fn build_cost(&self, bt: BuildingType) -> Option<BuildCost> {
        self.level_def(bt, 1).map(|ld| ld.cost.clone())
    }
}

impl Default for BuildingCatalog {
    fn default() -> Self {
        use crate::components::building::BuildingLevelDef;
        use BuildingType::*;

        // Hardcoded defaults mirroring the GDD tables.
        // In production these are overridden by RON assets.
        let mut entries = Vec::new();

        macro_rules! entry {
            ($bt:ident, $cat:ident, [$(($lvl:expr, $food:expr, $gold:expr, $wood:expr, $stone:expr, $iron:expr, $pop:expr, $time:expr, $hp:expr, $armor:expr)),+], [$(($dep:ident, $dlvl:expr)),*]) => {
                entries.push(BuildingCatalogEntry {
                    building_type: $bt,
                    category: crate::components::building::BuildingCategory::$cat,
                    levels: vec![$(BuildingLevelDef {
                        level: $lvl,
                        cost: BuildCost { food: $food, gold: $gold, wood: $wood, stone: $stone, iron: $iron, population: $pop },
                        build_time: $time as f32,
                        health: $hp as f32,
                        armor: $armor as f32,
                    }),+],
                    dependencies: vec![$(crate::components::building::BuildingDependency {
                        required_building: $dep,
                        required_level: $dlvl,
                    }),*],
                });
            };
        }

        // ── Basic / Core ────────────────────────────────────────────
        entry!(
            Oba,
            Basic,
            [
                (1, 0, 0, 100, 0, 0, 0, 30, 500, 0),
                (2, 0, 0, 200, 0, 100, 0, 60, 800, 2),
                (3, 0, 50, 400, 0, 200, 0, 120, 1200, 5),
                (4, 0, 100, 600, 0, 300, 0, 180, 1600, 8),
                (5, 0, 200, 800, 0, 500, 0, 300, 2500, 12)
            ],
            []
        );
        entry!(
            Pasture,
            Basic,
            [
                (1, 0, 0, 80, 0, 0, 0, 15, 200, 0),
                (2, 0, 0, 150, 0, 30, 0, 35, 400, 1),
                (3, 0, 0, 250, 0, 60, 0, 60, 600, 2)
            ],
            []
        );
        entry!(
            MiningCamp,
            Basic,
            [
                (1, 0, 0, 100, 60, 0, 0, 25, 300, 1),
                (2, 0, 0, 200, 100, 40, 0, 50, 500, 2),
                (3, 0, 0, 350, 200, 100, 0, 90, 800, 4)
            ],
            []
        );
        entry!(
            LumberCamp,
            Basic,
            [
                (1, 0, 0, 80, 0, 0, 0, 20, 200, 0),
                (2, 0, 0, 150, 0, 30, 0, 40, 400, 1),
                (3, 0, 0, 250, 0, 80, 0, 70, 600, 2)
            ],
            []
        );
        entry!(
            Barracks,
            Basic,
            [
                (1, 0, 0, 120, 0, 0, 0, 25, 300, 0),
                (2, 0, 0, 200, 0, 50, 0, 50, 500, 2),
                (3, 0, 0, 350, 0, 150, 0, 90, 800, 4)
            ],
            []
        );
        entry!(
            ArcheryRange,
            Basic,
            [
                (1, 0, 0, 140, 0, 30, 0, 30, 300, 1),
                (2, 0, 0, 250, 0, 80, 0, 55, 500, 2),
                (3, 0, 0, 400, 0, 150, 0, 100, 800, 4)
            ],
            []
        );
        entry!(
            Stable,
            Basic,
            [
                (1, 0, 50, 200, 0, 80, 0, 45, 400, 2),
                (2, 0, 50, 400, 0, 150, 0, 75, 700, 4),
                (3, 0, 100, 600, 0, 250, 0, 120, 1100, 6)
            ],
            []
        );
        entry!(
            Market,
            Basic,
            [
                (1, 0, 0, 200, 0, 0, 0, 45, 400, 0),
                (2, 0, 100, 300, 0, 0, 0, 75, 600, 2)
            ],
            [(Oba, 1), (Warehouse, 1)]
        );
        entry!(
            Warehouse,
            Basic,
            [
                (1, 0, 0, 50, 0, 0, 0, 15, 200, 0),
                (2, 0, 0, 100, 0, 50, 0, 30, 400, 1),
                (3, 0, 0, 200, 0, 100, 0, 60, 600, 2)
            ],
            []
        );
        entry!(
            Mosque,
            Basic,
            [
                (1, 0, 0, 80, 0, 0, 0, 20, 200, 0),
                (2, 0, 50, 150, 0, 0, 0, 45, 400, 1),
                (3, 0, 20, 300, 0, 0, 0, 90, 600, 2)
            ],
            []
        );
        entry!(
            Well,
            Basic,
            [
                (1, 0, 0, 50, 0, 0, 0, 20, 150, 0),
                (2, 0, 0, 100, 0, 20, 0, 35, 250, 1),
                (3, 0, 0, 150, 0, 40, 0, 60, 400, 2)
            ],
            []
        );
        entry!(Road, Basic, [(1, 0, 0, 5, 0, 0, 0, 5, 50, 0)], []);
        entry!(
            CouncilChamber,
            Basic,
            [
                (1, 0, 0, 200, 0, 0, 0, 45, 600, 2),
                (2, 0, 0, 350, 0, 100, 0, 75, 900, 4)
            ],
            [(Oba, 2)]
        );
        entry!(
            GuildHall,
            Basic,
            [
                (1, 0, 0, 150, 0, 0, 0, 35, 400, 1),
                (2, 0, 0, 250, 0, 50, 0, 60, 700, 3)
            ],
            []
        );
        entry!(
            Monument,
            Basic,
            [(1, 0, 0, 300, 0, 100, 0, 80, 1000, 5)],
            []
        );

        // ── Defense ────────────────────────────────────────────────
        entry!(
            Palisade,
            Defense,
            [
                (1, 0, 0, 50, 0, 0, 0, 10, 500, 2),
                (2, 0, 0, 80, 30, 0, 0, 15, 1000, 5),
                (3, 0, 0, 0, 150, 50, 0, 25, 2000, 8)
            ],
            []
        );
        entry!(
            StoneWall,
            Defense,
            [
                (1, 0, 0, 0, 100, 0, 0, 15, 800, 8),
                (2, 0, 0, 0, 200, 50, 0, 25, 1500, 12),
                (3, 0, 0, 0, 350, 100, 0, 40, 2500, 16)
            ],
            [(Palisade, 1)]
        );
        entry!(
            Gate,
            Defense,
            [
                (1, 0, 0, 80, 0, 0, 0, 15, 400, 3),
                (2, 0, 0, 150, 0, 60, 0, 30, 800, 6),
                (3, 0, 30, 300, 0, 150, 0, 60, 1500, 10)
            ],
            []
        );

        // ── Special ────────────────────────────────────────────────
        entry!(
            SiegeWorkshop,
            Special,
            [
                (1, 0, 0, 400, 0, 200, 0, 90, 800, 4),
                (2, 0, 0, 600, 0, 350, 0, 150, 1400, 6),
                (3, 0, 50, 800, 0, 500, 0, 240, 2000, 8)
            ],
            [(Oba, 2)]
        );

        Self { entries }
    }
}

// ─── Building Limits ─────────────────────────────────────────────────────────

/// Tracks the current building count per category and the
/// maximum allowed per category (scales with Command Tent level).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct BuildingLimits {
    pub basic_count: u32,
    pub defense_count: u32,
    pub special_count: u32,
    pub max_basic: u32,
    pub max_defense: u32,
    pub max_special: u32,
}

impl BuildingLimits {
    /// Recompute max limits from the highest-level Oba in the world.
    pub fn compute_from_oba_level(oba_level: u8) -> Self {
        match oba_level {
            0 | 1 => Self {
                basic_count: 0,
                defense_count: 0,
                special_count: 0,
                max_basic: 4,
                max_defense: 2,
                max_special: 0,
            },
            2 => Self {
                basic_count: 0,
                defense_count: 0,
                special_count: 0,
                max_basic: 8,
                max_defense: 4,
                max_special: 1,
            },
            _ => Self {
                // Level 3+ unlocks full capacity
                basic_count: 0,
                defense_count: 0,
                special_count: 0,
                max_basic: 12,
                max_defense: 6,
                max_special: 3,
            },
        }
    }

    /// Returns `Ok` if a building of the given category can be placed.
    pub fn can_place(&self, cat: BuildingCategory) -> Result<(), BuildingLimitExceeded> {
        match cat {
            BuildingCategory::Basic if self.basic_count < self.max_basic => Ok(()),
            BuildingCategory::Defense if self.defense_count < self.max_defense => Ok(()),
            BuildingCategory::Special if self.special_count < self.max_special => Ok(()),
            BuildingCategory::Basic => Err(BuildingLimitExceeded::Basic(self.max_basic)),
            BuildingCategory::Defense => Err(BuildingLimitExceeded::Defense(self.max_defense)),
            BuildingCategory::Special => Err(BuildingLimitExceeded::Special(self.max_special)),
        }
    }
}

impl Default for BuildingLimits {
    fn default() -> Self {
        Self::compute_from_oba_level(1)
    }
}

/// Reason why a building-limit check failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingLimitExceeded {
    Basic(u32),
    Defense(u32),
    Special(u32),
}

/// Configures how food decay works.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct FoodDecayConfig {
    /// Seconds before food starts decaying (base).
    pub base_decay_delay: f32,
    /// Fraction of stored food lost per minute after the delay.
    pub decay_rate_per_min: f32,
    /// Extra delay in seconds per Warehouse level.
    pub delay_per_warehouse_level: f32,
}

impl Default for FoodDecayConfig {
    fn default() -> Self {
        Self {
            base_decay_delay: 600.0,          // 10 minutes
            decay_rate_per_min: 0.10,         // 10% per minute
            delay_per_warehouse_level: 300.0, // +5 minutes per level
        }
    }
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct HeroConfig {
    pub hero_id: HeroId,
    pub name: String,
    pub archetype: HeroArchetype,
    pub base_stats: HeroBaseStats,
    pub growth_per_level: HeroGrowth,
    pub abilities: Vec<AbilityDef>,
    pub skill_tree: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeroArchetype {
    Warrior,
    Strategist,
    Administrator,
    Diplomat,
    HybridLeader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroBaseStats {
    pub health: f32,
    pub stamina: f32,
    pub attack: f32,
    pub move_speed: f32,
    pub vision: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroGrowth {
    pub health: f32,
    pub stamina: f32,
    pub attack: f32,
    pub move_speed: f32,
    pub vision: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityDef {
    pub id: AbilityId,
    pub ability_type: AbilityType,
    pub target_type: TargetType,
    pub range: f32,
    pub cooldown: f32,
    pub cost: AbilityCost,
    pub effect: AbilityEffect,
}

#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct MissionDef {
    pub mission_id: MissionId,
    pub name: String,
    pub description: String,
    pub objectives: Vec<ObjectiveDef>,
    pub rewards: MissionRewards,
    pub time_limit: Option<f32>,
    pub intro_dialogue: Option<String>,
    pub failure_conditions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectiveDef {
    pub objective_id: ObjectiveId,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub required_progress: f32,
    pub rewards: MissionRewards,
}

// ─── Zone Config Asset ──────────────────────────────────────────────────────

/// Configures zone prices, unlock requirements, and level caps.
/// Loaded from `assets/zones/zone_definitions.ron` at runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneConfigEntry {
    pub zone_type: ZoneType,
    pub unlocked: bool,
    pub bonus_multiplier: f32,
    pub placement_gold_cost: u32,
    pub placement_wood_cost: u32,
    pub upgrade_gold_cost: u32,
    pub upgrade_wood_cost: u32,
    pub max_level: u8,
    pub max_district_size: u32,
    pub color: [f32; 4],
    pub required_city_level: CityLevel,
    pub required_population: u32,
    pub sp_cost: f32,
}

/// A list of zone config entries, loaded from a single RON asset.
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct ZoneConfigList {
    pub zones: Vec<ZoneConfigEntry>,
}
