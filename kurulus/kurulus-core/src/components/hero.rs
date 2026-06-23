use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::combat::DamageType;

// ─── Hero resource type ─────────────────────────────────────────────────────

/// Per-hero resource system: Inspiration (Osman), Fury (Turgut), Focus (Buran).
/// Replaces the generic `stamina` system with hero-specific ability resources.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum HeroResource {
    /// Osman — +1/s base, +3/s in combat. Max 100.
    Inspiration,
    /// Turgut — +0.8/s base, +2 on taking damage. Max 120.
    Fury,
    /// Buran — +1.5/s base, +5 while stealthed and stationary. Max 80.
    Focus,
}

// ─── Hero core component ────────────────────────────────────────────────────

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Hero {
    pub hero_id: HeroId,
    pub level: u8,
    pub xp: u32,
    pub skill_points: u8,
    pub unlocked_abilities: Vec<AbilityId>,
    pub active_cooldowns: HashMap<AbilityId, f32>,
    /// Per-hero ability resource type.
    pub resource_type: HeroResource,
    /// Current resource amount.
    pub resource_current: f32,
    /// Maximum resource amount.
    pub resource_max: f32,
    /// Base resource regen per second.
    pub resource_regen: f32,
    /// Talent choices made at levels 6 and 10 (level → choice).
    pub talent_choices: HashMap<u8, TalentChoice>,
    /// Specialization path chosen at level 8.
    pub specialization: Option<SpecializationPath>,
    /// Named unit-specific: Konur, Ayazbey, Bamsi, Salur, Salvador/AbuBakr
    pub named_unit_id: Option<NamedUnitId>,
    /// Whether this named unit has permanently died (only for NamedUnits, not Heroes).
    pub is_permanently_dead: bool,
}

impl Default for Hero {
    fn default() -> Self {
        Self {
            hero_id: HeroId::Osman,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: Vec::new(),
            active_cooldowns: HashMap::new(),
            resource_type: HeroResource::Inspiration,
            resource_current: 100.0,
            resource_max: 100.0,
            resource_regen: 1.0,
            talent_choices: HashMap::new(),
            specialization: None,
            named_unit_id: None,
            is_permanently_dead: false,
        }
    }
}

impl Hero {
    pub fn new(hero_id: HeroId) -> Self {
        Self {
            hero_id,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: Vec::new(),
            active_cooldowns: HashMap::new(),
            resource_type: match hero_id {
                HeroId::Osman => HeroResource::Inspiration,
                HeroId::Turgut => HeroResource::Fury,
                HeroId::Buran => HeroResource::Focus,
                _ => HeroResource::Inspiration,
            },
            resource_current: 100.0,
            resource_max: match hero_id {
                HeroId::Turgut => 120.0,
                HeroId::Buran => 80.0,
                _ => 100.0,
            },
            resource_regen: match hero_id {
                HeroId::Osman => 1.0,
                HeroId::Turgut => 0.8,
                HeroId::Buran => 1.5,
                _ => 1.0,
            },
            talent_choices: HashMap::new(),
            specialization: None,
            named_unit_id: None,
            is_permanently_dead: false,
        }
    }
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
            // Levels 1→2 through 9→10 (9 thresholds for 10 levels)
            // From Feature Spec §5: 200, 450, 750, 1100, 1500, 2000, 2600, 3300, 4000
            thresholds: vec![200, 450, 750, 1100, 1500, 2000, 2600, 3300, 4000],
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

// ─── Per-hero base stats (loaded from RON, defines GDD stat block) ──────────

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct HeroStats {
    pub base_health: f32,
    pub base_damage: f32,
    pub base_defense: f32,
    pub base_speed: f32,
    pub base_vision: f32,
    pub base_attack_speed: f32,
    pub health_per_level: f32,
    pub damage_per_level: f32,
    pub defense_per_level: f32,
    pub damage_type: DamageType,
}

impl Default for HeroStats {
    fn default() -> Self {
        Self {
            base_health: 1000.0,
            base_damage: 30.0,
            base_defense: 20.0,
            base_speed: 3.0,
            base_vision: 15.0,
            base_attack_speed: 1.0,
            health_per_level: 150.0,
            damage_per_level: 4.0,
            defense_per_level: 3.0,
            damage_type: DamageType::Melee,
        }
    }
}

// ─── Equipment system ───────────────────────────────────────────────────────

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct Equipment {
    pub weapon: Option<EquipmentItem>,
    pub armor: Option<EquipmentItem>,
    pub accessory_1: Option<EquipmentItem>,
    pub accessory_2: Option<EquipmentItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct EquipmentItem {
    pub item_id: String,
    pub name: String,
    pub rarity: Rarity,
    pub slot: EquipmentSlot,
    pub stats: EquipmentStats,
    pub passive_effect: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Accessory1,
    Accessory2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct EquipmentStats {
    pub damage_bonus: f32,
    pub defense_bonus: f32,
    pub health_bonus: f32,
    pub speed_bonus: f32,
    pub attack_speed_bonus: f32,
}

// ─── Hero state machine ─────────────────────────────────────────────────────

#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum HeroState {
    Idle,
    Moving,
    Running,
    Attacking,
    UsingAbility,
    Channeling,
    KnockedOut { timer: f32 },
    Recovering { timer: f32 },
    Dead,
}

impl PartialEq for HeroState {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Default for HeroState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct HeroDeathState {
    pub is_dead_this_mission: bool,
    pub knockout_timer: f32,
}

// ─── Named Units ────────────────────────────────────────────────────────────

#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct NamedUnit {
    pub unit_id: NamedUnitId,
    pub is_permanently_dead: bool,
}

impl Default for NamedUnit {
    fn default() -> Self {
        Self {
            unit_id: NamedUnitId::KonurAlp,
            is_permanently_dead: false,
        }
    }
}

/// Tracks which unique passive abilities a named unit has.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct NamedUnitAbilities {
    pub passive_ids: Vec<NamedUnitAbilityId>,
}

/// Stat block for a named unit (from GDD roster).
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct NamedUnitStatsBlock {
    pub max_health: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub armor: f32,
    pub move_speed: f32,
    pub vision_range: f32,
}

/// All named unit stat blocks, keyed by unit ID.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct NamedUnitRoster {
    pub units: HashMap<NamedUnitId, NamedUnitStatsBlock>,
}

// ─── Equipment loot table ───────────────────────────────────────────────────

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct EquipmentLootTable {
    pub entries: Vec<LootEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct LootEntry {
    pub item_id: String,
    pub rarity: Rarity,
    pub drop_chance: f32,
    pub enemies: Vec<String>,
}

// ─── Cross-mission death penalties ──────────────────────────────────────────

#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct HeroDeathPenalties {
    pub penalties: HashMap<HeroId, DeathPenalty>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct DeathPenalty {
    pub rating_loss: f32,
    pub xp_loss: u32,
    pub morale_penalty: f32,
}

/// Marker component for visual feedback during/after a Hero Moment.
/// Render systems can read this to show screen flash, VFX, etc.
/// Timer counts down; at zero the component should be removed.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
pub struct HeroMomentFlash {
    pub remaining: f32,
    pub ability_name: String,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum HeroId {
    // Season 1 — Core 3 heroes
    Osman,
    Turgut,
    Buran,
    // Season 2+ — forward-compatible
    Orhan,
    Alaeddin,
    Malhun,
    Bala,
    Bamsi,
    Abdurrahman,
    Gunduz,
    Savci,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum NamedUnitId {
    KonurAlp,
    Ayazbey,
    BamsiBeyrek,
    Salvador,
    SalurAlp,
}

/// Unique passive abilities for named units (GDD §7 — Named Units Roster).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum NamedUnitAbilityId {
    /// Konur Alp: +15% damage when charging 10m+, +20% move speed for 5s after kill
    RidersSwiftness,
    /// Ayazbey: +25% attack range, detect stealth at +5m
    HawksEye,
    /// Bamsi Beyrek: +15% morale aura 15m, +10% defense to adjacent units
    VeteransCry,
    /// Salvador/Abu Bakr: +15% defense to adjacent, -10% siege damage
    RomanTactics,
    /// Salur Alp: +20% scouting range, +5% move speed to squad
    TribesEye,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum AbilityId {
    // Osman (Season 1 core)
    WarCry,           // Q - Active: AoE morale + damage buff for 10s
    CallOfTheTribe,   // W - Active: summon 5 alps to location
    Lionheart,        // E - Passive: triggers at <30% HP, damage reduction + attack speed
    DreamOfTheTree,   // R - Ultimate: area reveal + buff allies / debuff enemies
    // Turgut (Season 1 core)
    GroundStrike,     // Q - Active: AoE blunt damage + knockback
    EarthquakeChallenge, // W - Active: leap + taunt + self defense buff
    IronWall,         // E - Passive: % damage reduction, increases while still
    BullsWrath,       // R - Ultimate: dash + enrage (attack speed + damage, -defense)
    // Buran (Season 1 core)
    FlamingArrow,     // Q - Active: fire DoT + area reveal
    BlindingShot,     // W - Active: single target blind + silence
    ShadowLightness,  // E - Passive: +15% speed, stealth after 3s still
    DeathVolley,      // R - Ultimate: 20 arrows on area
    // Orhan (Season 2+)
    TacticalGenius,
    ShieldWall,
    CavalryCharge,
    FortressBuilder,
    // Alaeddin (Season 2+)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum AbilityType {
    Active,
    Passive,
    Ultimate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TargetType {
    Self_,
    Point,
    Unit,
    Area,
    Direction,
    Global,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct AbilityCost {
    /// Generic cost — for heroes using the HeroResource system, this is the
    /// amount of their specific resource consumed.
    pub stamina: f32,
    pub gold: Option<u32>,
    pub resource: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
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
    /// taunt: forces enemies to target the caster
    pub taunt: Option<bool>,
    /// blind: target vision = 0 for duration
    pub blind: Option<bool>,
    /// silence: target cannot use abilities
    pub silence: Option<bool>,
    /// summon_count: number of units to summon
    pub summon_count: Option<u32>,
    /// summon_archetype: the type of unit to summon
    pub summon_archetype: Option<String>,
    /// fire_damage_per_second: burning DoT
    pub fire_damage_per_second: Option<f32>,
    /// stealth: makes caster enter stealth
    pub stealth: Option<bool>,
}

// ─── Talent system ───────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TalentChoice {
    OptionA,
    OptionB,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum SpecializationPath {
    // Osman
    Conquest,
    Sovereignty,
    Faith,
    // Turgut
    Mountain,
    Storm,
    Guardian,
    // Buran
    Fire,
    Shadow,
    Eye,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
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
    // Turgut tree
    GroundPounder,
    Unyielding,
    BattleRoar,
    StoneSkin,
    // Buran tree
    EagleEye,
    SilentStep,
    EmberArrow,
    ShadowForm,
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
