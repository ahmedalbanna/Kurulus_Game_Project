use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Hero {
    pub hero_id: HeroId,
    pub level: u8,
    pub xp: u32,
    pub skill_points: u8,
    pub unlocked_abilities: Vec<AbilityId>,
    pub active_cooldowns: HashMap<AbilityId, f32>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Ability {
    pub id: AbilityId,
    pub ability_type: AbilityType,
    pub target_type: TargetType,
    pub range: f32,
    pub cooldown: f32,
    pub cost: AbilityCost,
    pub effect: AbilityEffect,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct SkillTree {
    pub hero_id: HeroId,
    pub nodes: Vec<SkillNode>,
    pub available_points: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillNode {
    pub id: SkillNodeId,
    pub name: String,
    pub description: String,
    pub cost: u8,
    pub prerequisites: Vec<SkillNodeId>,
    pub unlocked: bool,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeroId {
    Osman,
    Orhan,
    Alaeddin,
    Malhun,
    Bala,
    Turgut,
    Bamsi,
    Abdurrahman,
    Gunduz,
    Savci,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityId {
    // Osman
    WarCry,
    SovereignStrike,
    LionsHeart,
    TreesVision,
    // Orhan
    TacticalGenius,
    ShieldWall,
    CavalryCharge,
    FortressBuilder,
    // Alaeddin
    AdministrativeMind,
    TaxReform,
    DiplomaticImmunity,
    CulturalRenaissance,
    // Generic
    FirstAid,
    Scout,
    Sabotage,
    Inspire,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AbilityType {
    Active,
    Passive,
    Ultimate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetType {
    Self_,
    Point,
    Unit,
    Area,
    Direction,
    Global,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityCost {
    pub stamina: f32,
    pub gold: Option<u32>,
    pub resource: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AbilityEffect {
    pub damage_multiplier: Option<f32>,
    pub heal_amount: Option<f32>,
    pub buff_radius: Option<f32>,
    pub duration: Option<f32>,
    pub damage_bonus: Option<f32>,
    pub defense_bonus: Option<f32>,
    pub move_speed_bonus: Option<f32>,
    pub knockback: Option<bool>,
    pub dash_distance: Option<f32>,
    pub reveal_duration: Option<f32>,
    pub show_enemies: Option<bool>,
    pub show_ambush_spots: Option<bool>,
    pub hp_threshold: Option<f32>,
    pub cooldown_reduction: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillNodeId {
    // Osman tree
    SwordMastery,
    LeaderAura,
    GuerrillaTactics,
    WarLeadership,
    TribalUnity,
    EpicWarcry,
    ShadowStrike,
    Sovereignty,
    // Orhan tree
    SiegeMastery,
    ArmyOrganization,
    Fortification,
    Logistics,
    MilitaryReform,
    GrandArmy,
    // Alaeddin tree
    Taxation,
    Bureaucracy,
    Diplomacy,
    CulturalPatronage,
    LegalCode,
    GrandVizier,
}
