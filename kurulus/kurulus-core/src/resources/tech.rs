use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{UnitArchetype, BuildingType, AbilityId};

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct TechTree {
    pub techs: Vec<TechNode>,
    pub current_research: Option<ResearchProject>,
}

impl Default for TechTree {
    fn default() -> Self {
        Self {
            techs: Vec::new(),
            current_research: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechNode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: u32,
    pub research_time: f32,
    pub prerequisites: Vec<String>,
    pub unlocked: bool,
    pub category: TechCategory,
    pub effects: Vec<TechEffect>,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ResearchProject {
    pub tech_id: String,
    pub progress: f32,
    pub max_progress: f32,
    pub assigned_hero: Option<Entity>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechCategory {
    Military,
    Economy,
    Culture,
    Diplomacy,
    Building,
    Naval,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TechEffect {
    UnitStatBonus(UnitArchetype, String, f32),
    BuildingCostReduction(BuildingType, f32),
    ResourceRateBonus(String, f32),
    UnlockAbility(AbilityId),
    UnlockBuilding(BuildingType),
    UnlockUnit(UnitArchetype),
    PassiveBonus(String, f32),
}
