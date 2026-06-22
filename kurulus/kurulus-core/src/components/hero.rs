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

/// A temporary stat modifier applied by abilities (e.g. damage buff, defense).
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Buff {
    pub source: AbilityId,
    pub duration: f32,       // remaining seconds
    pub max_duration: f32,   // total seconds (for UI display)
    pub damage_bonus: f32,
    pub defense_bonus: f32,
    pub move_speed_bonus: f32,
    pub heal_per_second: f32,
    pub damage_reduction: f32, // 0.0–1.0 fraction
    pub attack_speed_bonus: f32, // added to stats.attack_speed (e.g. 0.2 = +20%)
}

impl Buff {
    pub fn new(source: AbilityId, duration: f32) -> Self {
        Self {
            source,
            duration,
            max_duration: duration,
            damage_bonus: 0.0,
            defense_bonus: 0.0,
            move_speed_bonus: 0.0,
            heal_per_second: 0.0,
            damage_reduction: 0.0,
            attack_speed_bonus: 0.0,
        }
    }
}

/// Marks a building as immune to enemy attacks (from DiplomaticImmunity ability).
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiplomaticImmunity {
    pub remaining: f32,
}

/// Tracks the AdministrativeMind production buff duration.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct AdministrativeMindBuff {
    pub remaining: f32,
    pub active: bool,
    pub original_food_rate: f32,
    pub original_gold_rate: f32,
    pub original_wood_rate: f32,
    pub original_stone_rate: f32,
}

impl Default for AdministrativeMindBuff {
    fn default() -> Self {
        Self {
            remaining: 0.0,
            active: false,
            original_food_rate: 1.0,
            original_gold_rate: 0.5,
            original_wood_rate: 0.5,
            original_stone_rate: 0.2,
        }
    }
}

/// Tracks the CulturalRenaissance XP buff duration.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CulturalRenaissanceBuff {
    pub remaining: f32,
    pub active: bool,
}

/// Marks an entity as being affected by one or more active buffs.
/// Separate from `Buff` so we can query for entities that have any buff.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActiveBuffs {
    pub buffs: Vec<Buff>,
}

/// Tracks which hero entity the player is currently controlling.
/// Updated by the hero switch system when the player cycles heroes.
#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct ActiveHero {
    pub entity: Option<Entity>,
}

impl Default for ActiveHero {
    fn default() -> Self {
        Self { entity: None }
    }
}

/// XP thresholds per level. Level 1→2 needs 100 XP, 2→3 needs 250, etc.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct XpConfig {
    pub thresholds: Vec<u32>, // index = level-1, value = XP needed
}

impl Default for XpConfig {
    fn default() -> Self {
        Self {
            thresholds: vec![100, 250, 500, 1000, 1800, 3000, 5000, 8000, 12000],
        }
    }
}

impl XpConfig {
    pub fn xp_for_level(&self, level: u8) -> u32 {
        let idx = (level as usize).saturating_sub(1);
        self.thresholds.get(idx).copied().unwrap_or(u32::MAX)
    }

    pub fn max_level(&self) -> u8 {
        self.thresholds.len() as u8 + 1
    }
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
