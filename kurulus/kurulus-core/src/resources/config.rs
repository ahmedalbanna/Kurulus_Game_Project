use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{
    UnitArchetype, ArmorType,
    BuildingType, BuildCost,
    HeroId, AbilityId, AbilityType, TargetType, AbilityCost, AbilityEffect,
    MissionId, ObjectiveId, ObjectiveType, MissionRewards,
};

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
