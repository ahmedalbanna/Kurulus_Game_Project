use super::hero::AbilityId;
use super::unit::ArmorType;
use super::unit::UnitArchetype;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═════════════════════════════════════════════════════════════════════════════
//  WEAPON COMPONENTS
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct MeleeWeapon {
    pub damage: f32,
    pub weapon_type: WeaponType,
    pub range: f32,
    pub armor_penetration: f32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct RangedWeapon {
    pub damage: f32,
    pub weapon_type: WeaponType,
    pub range: f32,
    pub projectile_speed: f32,
    pub armor_penetration: f32,
    pub ammo: u32,
    pub max_ammo: u32,
    pub reload_time: f32,
    /// Base accuracy (0.0–1.0), used by ranged_combat_system
    pub base_accuracy: f32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Armor {
    pub armor_type: ArmorType,
    pub base_armor: f32,
    pub bonus_armor: f32, // From abilities/formations
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Damage {
    pub amount: f32,
    pub damage_type: DamageType,
    pub source: Entity,
}

// ═════════════════════════════════════════════════════════════════════════════
//  GDD-ALIGNED DAMAGE TYPES (GDD §5.3 — Damage Model)
// ═════════════════════════════════════════════════════════════════════════════

/// GDD §5.3 — Six-element damage type system with rock-paper-scissors counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    /// Swords, axes — effective vs light/leather armor (×1.5), weak vs plate (×0.5)
    Slash,
    /// Arrows, spears, daggers — effective vs cavalry (×1.5), weak vs stone (×0.2)
    Pierce,
    /// Maces, war hammers, Turgut's axe — effective vs heavy plate (×1.5), structures (×2.0)
    Blunt,
    /// Flaming arrows, fire abilities — effective vs buildings (×3.0), weak vs plate (×0.5)
    Fire,
    /// Charges, knockbacks, tramples — effective vs formations (×2.0), breaks pikes
    Impact,
    /// Fear effects, hero ultimates — causes panic checks, no effect on structures
    Morale,
}

// ═════════════════════════════════════════════════════════════════════════════
//  WEAPON TYPES (expanded with Mace, Dagger)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,    // Slash
    Axe,      // Slash (light) or Blunt (heavy)
    Spear,    // Pierce
    Bow,      // Pierce
    Crossbow, // Pierce
    Mace,     // Blunt (NEW)
    Dagger,   // Pierce, fast (NEW)
    Siege,    // Blunt/Impact (Catapult, Ram)
    Unarmed,
}

/// Maps a WeaponType to its primary DamageType for GDD effectiveness.
pub fn damage_type_for_weapon(weapon_type: WeaponType) -> DamageType {
    match weapon_type {
        WeaponType::Sword | WeaponType::Axe => DamageType::Slash,
        WeaponType::Spear | WeaponType::Bow | WeaponType::Crossbow | WeaponType::Dagger => {
            DamageType::Pierce
        }
        WeaponType::Mace => DamageType::Blunt,
        WeaponType::Siege => DamageType::Impact,
        WeaponType::Unarmed => DamageType::Blunt,
    }
}

/// Computes base block chance from archetype and armor type.
pub fn compute_block_chance(archetype: UnitArchetype) -> f32 {
    match archetype {
        UnitArchetype::HeavyInfantry | UnitArchetype::HeavyCavalry => 0.15,
        UnitArchetype::LightInfantry => 0.08,
        UnitArchetype::Archer | UnitArchetype::Scout => 0.03,
        UnitArchetype::LightCavalry => 0.05,
        _ => 0.02,
    }
}

/// Computes base dodge chance from archetype.
pub fn compute_dodge_chance(archetype: UnitArchetype) -> f32 {
    match archetype {
        UnitArchetype::Scout | UnitArchetype::LightCavalry => 0.15,
        UnitArchetype::LightInfantry | UnitArchetype::Archer => 0.08,
        UnitArchetype::HeavyInfantry | UnitArchetype::HeavyCavalry => 0.02,
        _ => 0.01,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CRITICAL HIT SYSTEM (GDD §2 — Critical Damage)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CritType {
    None,
    Flanking,      // Attacking from side/rear
    Charge,        // First hit after 10m+ charge distance
    Assassination, // Stealth attack from behind (×3.0)
    HighGround,    // Ranged from elevation (+5% crit)
}

impl Default for CritType {
    fn default() -> Self {
        Self::None
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  FLANKING SYSTEM (GDD §1.2 — Encirclement)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackZone {
    Front, // ±45° — ×1.0, full block chance
    Side,  // 45°–135° — ×1.25, -15% block
    Rear,  // 135°–180° — ×1.5, -30% block, ignores bonus armor
}

impl Default for AttackZone {
    fn default() -> Self {
        Self::Front
    }
}

/// Per-attack-pair flanking info computed by flanking_calculation_system.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlankingInfo {
    /// Attack angle relative to target's facing direction (0° = front).
    pub attack_angle: f32,
    /// Zone classification.
    pub zone: AttackZone,
    /// Damage multiplier for this flank zone (×1.0 / ×1.25 / ×1.5).
    pub damage_multiplier: f32,
    /// Block chance reduction for the defender (percentage points).
    pub block_reduction: f32,
}

impl Default for FlankingInfo {
    fn default() -> Self {
        Self {
            attack_angle: 0.0,
            zone: AttackZone::Front,
            damage_multiplier: 1.0,
            block_reduction: 0.0,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  CHARGE SYSTEM (GDD §3 — Charge)
// ═════════════════════════════════════════════════════════════════════════════

/// Tracks charge state for a unit preparing or executing a charge.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct ChargeState {
    pub is_charging: bool,
    pub charge_start_pos: Vec2,
    pub charge_distance_traveled: f32,
    pub required_distance: f32, // 10.0m minimum for charge bonus
    pub has_impacted: bool,
}

impl Default for ChargeState {
    fn default() -> Self {
        Self {
            is_charging: false,
            charge_start_pos: Vec2::ZERO,
            charge_distance_traveled: 0.0,
            required_distance: 10.0,
            has_impacted: false,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DAMAGE OVER TIME (GDD §4 — DoT)
// ═════════════════════════════════════════════════════════════════════════════

/// Applied by weapons/abilities that cause bleeding, burning, or poison.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct DamageOverTime {
    pub damage_per_tick: f32,
    pub tick_interval: f32,
    pub next_tick_progress: f32,
    pub damage_type: DamageType,
    pub remaining_duration: f32,
    pub max_duration: f32,
    pub stacks: u8,
    pub max_stacks: u8,
    pub source: Entity,
    pub source_ability: Option<AbilityId>,
}

impl Default for DamageOverTime {
    fn default() -> Self {
        Self {
            damage_per_tick: 5.0,
            tick_interval: 1.0,
            next_tick_progress: 0.0,
            damage_type: DamageType::Fire,
            remaining_duration: 5.0,
            max_duration: 5.0,
            stacks: 1,
            max_stacks: 3,
            source: Entity::from_raw(0),
            source_ability: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DotType {
    Bleed,
    Burn,
    Poison,
    Slow,
}

/// Percentage damage: deals % of target's max HP (GDD §4.2).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PercentageDamage {
    pub fraction: f32, // e.g. 0.05 = 5% of max HP
    pub max_damage: f32,
    pub damage_type: DamageType,
}

// ═════════════════════════════════════════════════════════════════════════════
//  FACING DIRECTION (for flanking calculations)
// ═════════════════════════════════════════════════════════════════════════════

/// The direction a unit is looking/travelling.
/// Updated by the movement system each frame.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Facing {
    pub direction: Vec2,
}

impl Facing {
    pub fn new(direction: Vec2) -> Self {
        Self {
            direction: direction.normalize_or_zero(),
        }
    }
}

impl Default for Facing {
    fn default() -> Self {
        Self {
            direction: Vec2::new(0.0, -1.0), // facing "down" by default
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  ARMOR RESISTANCE TABLE (GDD §2.2 — Damage Type vs Armor Type)
// ═════════════════════════════════════════════════════════════════════════════

/// Maps (ArmorType, DamageType) → resistance fraction (0.0–1.0).
/// Higher resistance = more damage blocked.
/// Loaded from RON config; sensible defaults provided for the 4 legacy armor types.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ArmorResistanceTable {
    pub resistances: HashMap<(ArmorType, DamageType), f32>,
}

impl Default for ArmorResistanceTable {
    fn default() -> Self {
        // GDD §2.2 — Armor Type Resistances for the 4 legacy types
        // Light ≈ LightLeather, Medium ≈ PartialPlate, Heavy ≈ FullPlate, Fortification ≈ Stone
        let mut r = HashMap::new();
        // Light leather (Light): weak vs Slash, moderate vs Pierce+Fire, strong vs Blunt
        r.insert((ArmorType::Light, DamageType::Slash), 0.3);
        r.insert((ArmorType::Light, DamageType::Pierce), 0.5);
        r.insert((ArmorType::Light, DamageType::Blunt), 0.6);
        r.insert((ArmorType::Light, DamageType::Fire), 0.4);
        r.insert((ArmorType::Light, DamageType::Impact), 0.5);
        r.insert((ArmorType::Light, DamageType::Morale), 0.0);
        // Partial plate (Medium): balanced
        r.insert((ArmorType::Medium, DamageType::Slash), 0.6);
        r.insert((ArmorType::Medium, DamageType::Pierce), 0.4);
        r.insert((ArmorType::Medium, DamageType::Blunt), 0.4);
        r.insert((ArmorType::Medium, DamageType::Fire), 0.5);
        r.insert((ArmorType::Medium, DamageType::Impact), 0.3);
        r.insert((ArmorType::Medium, DamageType::Morale), 0.0);
        // Full plate (Heavy): strong vs Slash+Pierce+Fire, weak vs Blunt
        r.insert((ArmorType::Heavy, DamageType::Slash), 0.8);
        r.insert((ArmorType::Heavy, DamageType::Pierce), 0.7);
        r.insert((ArmorType::Heavy, DamageType::Blunt), 0.2);
        r.insert((ArmorType::Heavy, DamageType::Fire), 0.7);
        r.insert((ArmorType::Heavy, DamageType::Impact), 0.6);
        r.insert((ArmorType::Heavy, DamageType::Morale), 0.0);
        // Stone (Fortification): very high resistance except Blunt
        r.insert((ArmorType::Fortification, DamageType::Slash), 0.9);
        r.insert((ArmorType::Fortification, DamageType::Pierce), 0.9);
        r.insert((ArmorType::Fortification, DamageType::Blunt), 0.4);
        r.insert((ArmorType::Fortification, DamageType::Fire), 0.6);
        r.insert((ArmorType::Fortification, DamageType::Impact), 0.7);
        r.insert((ArmorType::Fortification, DamageType::Morale), 0.0);
        Self { resistances: r }
    }
}

impl ArmorResistanceTable {
    /// Returns the resistance fraction for the given (armor_type, damage_type) pair.
    /// Falls back to 0.0 (no resistance) if the pair is not explicitly defined.
    pub fn resistance(&self, armor_type: ArmorType, damage_type: DamageType) -> f32 {
        self.resistances
            .get(&(armor_type, damage_type))
            .copied()
            .unwrap_or(0.0)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DAMAGE EFFECTIVENESS TABLE (GDD §2.1 — Damage Type vs Unit Archetpye)
// ═════════════════════════════════════════════════════════════════════════════

/// Maps (DamageType, UnitArchetype) → effectiveness multiplier (0.1–3.0).
/// Used in the damage formula: `raw_damage × effectiveness_mult`.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct DamageEffectivenessTable {
    pub multipliers: HashMap<(DamageType, UnitArchetype), f32>,
}

impl Default for DamageEffectivenessTable {
    fn default() -> Self {
        // GDD §5.3 — Damage Effectiveness Matrix (rows: DamageType, cols: UnitArchetype)
        let mut m = HashMap::new();
        // Slash: effective vs LightInfantry(×1.5), weak vs HeavyInfantry(×0.5)
        m.insert((DamageType::Slash, UnitArchetype::LightInfantry), 1.5);
        m.insert((DamageType::Slash, UnitArchetype::HeavyInfantry), 0.5);
        m.insert((DamageType::Slash, UnitArchetype::Archer), 1.5);
        m.insert((DamageType::Slash, UnitArchetype::LightCavalry), 1.0);
        m.insert((DamageType::Slash, UnitArchetype::HeavyCavalry), 0.7);
        m.insert((DamageType::Slash, UnitArchetype::Scout), 1.5);
        m.insert((DamageType::Slash, UnitArchetype::SiegeCrew), 1.2);
        m.insert((DamageType::Slash, UnitArchetype::Worker), 1.5);
        // Pierce: effective vs LightCavalry(×1.5), weak vs HeavyInfantry(×0.8)
        m.insert((DamageType::Pierce, UnitArchetype::LightInfantry), 1.0);
        m.insert((DamageType::Pierce, UnitArchetype::HeavyInfantry), 0.8);
        m.insert((DamageType::Pierce, UnitArchetype::Archer), 1.2);
        m.insert((DamageType::Pierce, UnitArchetype::LightCavalry), 1.5);
        m.insert((DamageType::Pierce, UnitArchetype::HeavyCavalry), 1.2);
        m.insert((DamageType::Pierce, UnitArchetype::Scout), 1.0);
        m.insert((DamageType::Pierce, UnitArchetype::SiegeCrew), 0.8);
        m.insert((DamageType::Pierce, UnitArchetype::Worker), 0.7);
        // Blunt: effective vs HeavyInfantry(×1.5), structures(×2.0), weak vs LightInfantry(×0.8)
        m.insert((DamageType::Blunt, UnitArchetype::LightInfantry), 0.8);
        m.insert((DamageType::Blunt, UnitArchetype::HeavyInfantry), 1.5);
        m.insert((DamageType::Blunt, UnitArchetype::Archer), 0.8);
        m.insert((DamageType::Blunt, UnitArchetype::LightCavalry), 0.8);
        m.insert((DamageType::Blunt, UnitArchetype::HeavyCavalry), 1.2);
        m.insert((DamageType::Blunt, UnitArchetype::Scout), 0.6);
        m.insert((DamageType::Blunt, UnitArchetype::SiegeCrew), 1.0);
        m.insert((DamageType::Blunt, UnitArchetype::Worker), 0.8);
        // Fire: effective vs Archers(×1.5), weak vs HeavyInfantry(×0.5)
        m.insert((DamageType::Fire, UnitArchetype::LightInfantry), 1.2);
        m.insert((DamageType::Fire, UnitArchetype::HeavyInfantry), 0.5);
        m.insert((DamageType::Fire, UnitArchetype::Archer), 1.5);
        m.insert((DamageType::Fire, UnitArchetype::LightCavalry), 1.2);
        m.insert((DamageType::Fire, UnitArchetype::HeavyCavalry), 0.5);
        m.insert((DamageType::Fire, UnitArchetype::Scout), 1.2);
        m.insert((DamageType::Fire, UnitArchetype::SiegeCrew), 1.5);
        m.insert((DamageType::Fire, UnitArchetype::Worker), 1.2);
        // Impact: effective vs Archers(×2.0), weak vs HeavyInfantry(×0.7)
        m.insert((DamageType::Impact, UnitArchetype::LightInfantry), 1.5);
        m.insert((DamageType::Impact, UnitArchetype::HeavyInfantry), 0.7);
        m.insert((DamageType::Impact, UnitArchetype::Archer), 2.0);
        m.insert((DamageType::Impact, UnitArchetype::LightCavalry), 1.0);
        m.insert((DamageType::Impact, UnitArchetype::HeavyCavalry), 1.0);
        m.insert((DamageType::Impact, UnitArchetype::Scout), 1.5);
        m.insert((DamageType::Impact, UnitArchetype::SiegeCrew), 1.0);
        m.insert((DamageType::Impact, UnitArchetype::Worker), 1.2);
        // Morale: effective vs LightInfantry+Archer, no effect on heavy armor
        m.insert((DamageType::Morale, UnitArchetype::LightInfantry), 1.5);
        m.insert((DamageType::Morale, UnitArchetype::HeavyInfantry), 0.5);
        m.insert((DamageType::Morale, UnitArchetype::Archer), 1.5);
        m.insert((DamageType::Morale, UnitArchetype::LightCavalry), 0.8);
        m.insert((DamageType::Morale, UnitArchetype::HeavyCavalry), 0.5);
        m.insert((DamageType::Morale, UnitArchetype::Scout), 1.2);
        m.insert((DamageType::Morale, UnitArchetype::SiegeCrew), 0.5);
        m.insert((DamageType::Morale, UnitArchetype::Worker), 0.8);
        Self { multipliers: m }
    }
}

impl DamageEffectivenessTable {
    /// Returns the effectiveness multiplier. Defaults to 1.0 if not explicitly defined.
    pub fn multiplier(&self, damage_type: DamageType, archetype: UnitArchetype) -> f32 {
        self.multipliers
            .get(&(damage_type, archetype))
            .copied()
            .unwrap_or(1.0)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMBAT GLOBAL CONFIG
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CombatConfig {
    /// Base critical hit chance (0.05 = 5%)
    pub base_crit_chance: f32,
    /// Critical damage multiplier (×1.5)
    pub crit_damage_multiplier: f32,
    /// Additional crit chance from side attack
    pub flank_crit_bonus_side: f32,
    /// Additional crit chance from rear attack
    pub flank_crit_bonus_rear: f32,
    /// Minimum distance in meters for a charge to activate
    pub charge_min_distance: f32,
    /// Charge damage multiplier
    pub charge_damage_multiplier: f32,
    /// Cavalry knockdown duration in seconds
    pub cavalry_knockdown_duration: f32,
    /// Formation break duration from charge impact (seconds)
    pub charge_formation_break_duration: f32,
    /// Minimum damage floor (cannot be reduced below this)
    pub min_damage: f32,
    /// Base chance for ranged friendly fire
    pub ranged_ff_base_chance: f32,
    /// Maximum stacks for DoT effects
    pub dot_max_stacks: u8,
    /// Panic propagation radius in meters
    pub panic_propagation_radius: f32,
    /// Panic propagation chance per neighbor
    pub panic_propagation_chance: f32,
    /// Ranged distance falloff factor (0.0–1.0)
    pub ranged_distance_falloff: f32,
    /// Maximum range penalty (0.0–1.0)
    pub max_range_penalty: f32,
    /// Friendly fire cone angle in degrees
    pub friendly_fire_cone_angle: f32,
    /// Max distance in meters for friendly fire check
    pub friendly_fire_max_check_dist: f32,
    /// Damage reduction multiplier for friendly fire (0.0–1.0)
    pub friendly_fire_damage_reduction: f32,
    /// Morale recovery rate multiplier (applied per second)
    pub morale_recovery_multiplier: f32,
    /// Panic recovery rate multiplier (applied per second)
    pub panic_recovery_multiplier: f32,
    /// Base panic duration in seconds
    pub panic_base_duration: f32,
    /// Speed multiplier for fleeing units
    pub panic_flee_speed_mult: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            base_crit_chance: 0.05,
            crit_damage_multiplier: 1.5,
            flank_crit_bonus_side: 0.10,
            flank_crit_bonus_rear: 0.15,
            charge_min_distance: 10.0,
            charge_damage_multiplier: 2.0,
            cavalry_knockdown_duration: 1.5,
            charge_formation_break_duration: 2.0,
            min_damage: 1.0,
            ranged_ff_base_chance: 0.15,
            dot_max_stacks: 3,
            panic_propagation_radius: 5.0,
            panic_propagation_chance: 0.3,
            ranged_distance_falloff: 0.5,
            max_range_penalty: 0.6,
            friendly_fire_cone_angle: 30.0,
            friendly_fire_max_check_dist: 15.0,
            friendly_fire_damage_reduction: 0.5,
            morale_recovery_multiplier: 0.1,
            panic_recovery_multiplier: 0.05,
            panic_base_duration: 5.0,
            panic_flee_speed_mult: 1.5,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMMUNITION & COVER (future Phase C)
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Ammunition {
    pub current: u32,
    pub max: u32,
    pub resupply_point: Option<GridPos>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Cover {
    pub cover_type: CoverType,
    pub hp: f32,
    pub direction: Vec2, // which side provides cover
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverType {
    Solid,   // Stone wall — -90% damage, 100% dodge
    Wood,    // Barrel/door — -50% damage, 70% dodge
    Natural, // Tree/bush — -30% damage, 50% dodge
    Low,     // Small wall — -40% damage, 60% dodge
    Mobile,  // Shield — -60% damage, 80% dodge
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE SYSTEM TYPES (used by events and morale subsystems)
// ═════════════════════════════════════════════════════════════════════════════

/// Triggers for morale checks (GDD §2 — Morale Checks).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MoraleCheckTrigger {
    AllyDeathNearby { distance: f32 },
    AmbushReceived,
    CavalryChargeIncoming,
    CommanderDeathNearby,
    Outnumbered { ratio: f32 },
    UnderFire { duration: f32 },
    NightStart,
    BattleLost,
    Victory,
}

/// Types of panic behavior (GDD §3 — Panic States).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanicType {
    Rout,     // 5s, retreat 10m, then stop
    Flee,     // 8s, run away at max speed
    Panic,    // 10s, random direction changes
    Stampede, // 10s, run straight, trample friends
    Broken,   // Permanent, flee off-map (equivalent to death)
}

// ═════════════════════════════════════════════════════════════════════════════
//  PANIC STATE
// ═════════════════════════════════════════════════════════════════════════════

/// Indicates a unit is panicking (fleeing, disordered).
/// Added by panic_processing_system and removed by morale_update_system.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PanicState {
    pub is_panicking: bool,
    pub duration: f32,
    pub flee_speed_multiplier: f32,
}

impl Default for PanicState {
    fn default() -> Self {
        Self {
            is_panicking: false,
            duration: 5.0,
            flee_speed_multiplier: 1.5,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DAMAGE TRACKER (Resource)
// ═════════════════════════════════════════════════════════════════════════════

/// Total damage dealt in the current game session.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct DamageTracker {
    pub total_dealt: f32,
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self { total_dealt: 0.0 }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DIFFICULTY CONFIG (GDD §5.12 — Difficulty Levels)
// ═════════════════════════════════════════════════════════════════════════════

/// Per-difficulty combat damage multipliers.
/// Renamed to `CombatDifficultyConfig` to avoid collision with
/// `kurulus_core::resources::phase::DifficultyConfig`.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CombatDifficultyConfig {
    /// Incoming damage multiplier for player-controlled units (e.g. 0.6 = take less)
    pub incoming_mult: f32,
    /// Outgoing damage multiplier for player units (e.g. 1.4 = deal more)
    pub outgoing_mult: f32,
    /// Resource multiplier for enemy (e.g. 0.7 = enemy has fewer resources)
    pub enemy_resource_mult: f32,
    /// Enemy reaction speed multiplier (lower = slower reaction)
    pub enemy_reaction_mult: f32,
}

impl Default for CombatDifficultyConfig {
    fn default() -> Self {
        Self {
            incoming_mult: 1.0,
            outgoing_mult: 1.0,
            enemy_resource_mult: 1.0,
            enemy_reaction_mult: 1.0,
        }
    }
}

impl CombatDifficultyConfig {
    /// Returns the config for the given difficulty preset.
    pub fn for_difficulty(difficulty: DifficultyPreset) -> Self {
        match difficulty {
            // Story mode: player deals 140%, takes 60%
            DifficultyPreset::Story => Self {
                incoming_mult: 0.6,
                outgoing_mult: 1.4,
                enemy_resource_mult: 0.7,
                enemy_reaction_mult: 0.8,
            },
            // Normal: 1.0 all around
            DifficultyPreset::Normal => Self::default(),
            // Hard: player deals 80%, takes 130%
            DifficultyPreset::Hard => Self {
                incoming_mult: 1.3,
                outgoing_mult: 0.8,
                enemy_resource_mult: 1.3,
                enemy_reaction_mult: 1.2,
            },
            // Epic: player deals 65%, takes 160%
            DifficultyPreset::Epic => Self {
                incoming_mult: 1.6,
                outgoing_mult: 0.65,
                enemy_resource_mult: 1.6,
                enemy_reaction_mult: 1.5,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DifficultyPreset {
    Story,
    Normal,
    Hard,
    Epic,
}

impl Default for DifficultyPreset {
    fn default() -> Self {
        Self::Normal
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE STATE (GDD §5.3 — Five-Level Morale)
// ═════════════════════════════════════════════════════════════════════════════

/// The five morale states from GDD §5.3, ordered from best to worst.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoraleState {
    High,       // 75–100 : +15% damage, +10% defense
    Normal,     // 50–74  : no modifier
    Shaken,     // 25–49  : -10% damage, 20% panic chance
    Frightened, // 1–24   : -25% damage, 40% panic chance
    Panic,      // 0      : flee 5–10s
}

impl MoraleState {
    pub fn from_morale_value(value: f32) -> Self {
        if value >= 75.0 {
            Self::High
        } else if value >= 50.0 {
            Self::Normal
        } else if value >= 25.0 {
            Self::Shaken
        } else if value > 0.0 {
            Self::Frightened
        } else {
            Self::Panic
        }
    }

    /// Damage multiplier from this morale state (1.0 = normal).
    pub fn damage_mult(&self) -> f32 {
        match self {
            Self::High => 1.15,
            Self::Normal => 1.0,
            Self::Shaken => 0.90,
            Self::Frightened => 0.75,
            Self::Panic => 0.0, // Panicking units don't attack
        }
    }

    /// Defense multiplier from this morale state (1.0 = normal).
    pub fn defense_mult(&self) -> f32 {
        match self {
            Self::High => 1.10,
            Self::Normal => 1.0,
            Self::Shaken => 1.0,
            Self::Frightened => 1.0,
            Self::Panic => 0.5, // Panicking units take more damage
        }
    }

    /// Panic chance (0.0–1.0) when taking damage in this state.
    pub fn panic_chance(&self) -> f32 {
        match self {
            Self::High | Self::Normal => 0.0,
            Self::Shaken => 0.20,
            Self::Frightened => 0.40,
            Self::Panic => 1.0,
        }
    }
}

/// Cached morale state for a unit, updated by `morale_state_system`.
/// Provides fast access to damage/defense multipliers without recomputing.
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CurrentMoraleState {
    pub state: MoraleState,
    /// Damage multiplier from this morale state (1.0 = normal).
    pub damage_mult: f32,
    /// Defense multiplier from this morale state (1.0 = normal).
    pub defense_mult: f32,
}

impl Default for CurrentMoraleState {
    fn default() -> Self {
        Self {
            state: MoraleState::Normal,
            damage_mult: 1.0,
            defense_mult: 1.0,
        }
    }
}

impl CurrentMoraleState {
    /// Create from a raw morale value.
    pub fn from_morale(morale: f32) -> Self {
        let state = MoraleState::from_morale_value(morale);
        Self {
            state,
            damage_mult: state.damage_mult(),
            defense_mult: state.defense_mult(),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  COMBAT-RELATED RE-EXPORTS
// ═════════════════════════════════════════════════════════════════════════════

pub use crate::coord::GridPos;
