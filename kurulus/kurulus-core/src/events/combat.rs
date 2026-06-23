use crate::components::{
    AbilityId, CritType, DamageType, FlankingInfo, MoraleCheckTrigger, PanicType, WeaponType,
};
use bevy::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
//  CORE COMBAT EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Central damage event — emitted by melee/ranged/ability systems,
/// consumed by damage_application_system.
#[derive(Event, Clone, Debug)]
pub struct DamageDealt {
    pub attacker: Entity,
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
    /// Critical hit classification (None by default).
    pub crit: CritType,
    /// Flanking info for this attack.
    pub flanking: FlankingInfo,
    /// Whether this damage was friendly fire.
    pub is_friendly_fire: bool,
    /// The ability that caused this damage, if any.
    pub source_ability: Option<AbilityId>,
}

impl Default for DamageDealt {
    fn default() -> Self {
        Self {
            attacker: Entity::from_raw(0),
            target: Entity::from_raw(0),
            amount: 0.0,
            damage_type: DamageType::Slash,
            crit: CritType::None,
            flanking: FlankingInfo::default(),
            is_friendly_fire: false,
            source_ability: None,
        }
    }
}

/// Unit has died (health <= 0).
#[derive(Event, Clone, Debug)]
pub struct UnitDied {
    pub entity: Entity,
    pub killer: Entity,
}

/// Fired when a unit's attack timer expires and damage is about to be dealt.
/// Read by the ambush system to apply ambush bonuses.
#[derive(Event, Clone, Debug)]
pub struct AttackTriggered {
    pub attacker: Entity,
    pub target: Entity,
    pub weapon_type: WeaponType,
    pub is_charge: bool,
    pub is_from_stealth: bool,
}

impl Default for AttackTriggered {
    fn default() -> Self {
        Self {
            attacker: Entity::from_raw(0),
            target: Entity::from_raw(0),
            weapon_type: WeaponType::Sword,
            is_charge: false,
            is_from_stealth: false,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a unit's morale drops to 0.
#[derive(Event, Clone, Debug)]
pub struct MoraleBroken {
    pub entity: Entity,
    pub morale_value: f32,
}

/// Triggers a morale check on the target entity.
/// Consumed by morale_check_system.
#[derive(Event, Clone, Debug)]
pub struct MoraleCheckTriggered {
    pub entity: Entity,
    pub trigger: MoraleCheckTrigger,
    pub panic_multiplier: f32,
}

/// Emitted when a unit panics. Consumed by panic_processing_system.
#[derive(Event, Clone, Debug)]
pub struct PanicTriggered {
    pub entity: Entity,
    pub panic_type: PanicType,
    pub duration: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH & STEALTH EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when an ambush deals bonus damage.
#[derive(Event, Clone, Debug)]
pub struct AmbushTriggered {
    pub ambusher: Entity,
    pub target: Entity,
    pub damage_bonus: f32,
    pub zone: Entity,
}

/// Emitted when a stealthed unit is detected.
#[derive(Event, Clone, Debug)]
pub struct StealthRevealed {
    pub entity: Entity,
    pub revealer: Entity,
}

// ═════════════════════════════════════════════════════════════════════════════
//  CHARGE EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a charging unit impacts a target.
#[derive(Event, Clone, Debug)]
pub struct ChargeImpact {
    pub charger: Entity,
    pub target: Entity,
    pub damage: f32,
    pub knockdown_duration: f32,
    pub formation_break_duration: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  RANGED COMBAT EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted after a ranged attack resolves.
#[derive(Event, Clone, Debug)]
pub struct RangedAttackResolved {
    pub attacker: Entity,
    pub target: Entity,
    pub hit: bool,
    pub damage: f32,
    pub friendly_fire: bool,
    pub accuracy_roll: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  SIEGE EVENTS
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a wall segment takes damage from a siege engine.
#[derive(Event, Clone, Debug)]
pub struct SiegeWallDamaged {
    pub wall: Entity,
    pub damage: f32,
    pub source: Entity,
    pub is_breached: bool,
}

// ═════════════════════════════════════════════════════════════════════════════
//  MORALE EVENTS (expanded)
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a unit's morale changes in a significant way (panics, rallies).
#[derive(Event, Clone, Debug)]
pub struct MoraleEvent {
    pub unit: Entity,
    pub event_type: MoraleEventType,
    pub cause: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MoraleEventType {
    Panic,
    Rally,
    Broken,
}

// ═════════════════════════════════════════════════════════════════════════════
//  RANGED FRIENDLY FIRE EVENT
// ═════════════════════════════════════════════════════════════════════════════

/// Emitted when a ranged attack hits a friendly unit.
#[derive(Event, Clone, Debug)]
pub struct RangedFriendlyFire {
    pub attacker: Entity,
    pub target: Entity,
    pub damage: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  HIT FEEDBACK EVENTS (Phase 5e — NEW)
// ═════════════════════════════════════════════════════════════════════════════

/// Floating damage text / visual feedback data for UI rendering.
/// Emitted by hit_feedback_system after damage application.
#[derive(Event, Clone, Debug)]
pub struct HitFeedbackEvent {
    pub entity: Entity,
    pub feedback_type: HitFeedbackType,
    pub amount: f32,
    pub position: Vec2,
}

/// Classification of a combat hit for visual feedback purposes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HitFeedbackType {
    Hit,
    Block,
    Dodge,
    Critical,
    AmbushBonus,
    ChargeImpact,
    FriendlyFire,
    DotTick,
    Heal,
}

// ═════════════════════════════════════════════════════════════════════════════
//  HERO AURA EVENTS (Phase C — NEW)
// ═════════════════════════════════════════════════════════════════════════════

/// Applied when a hero ability affects nearby units.
#[derive(Event, Clone, Debug)]
pub struct HeroAuraApplied {
    pub hero: Entity,
    pub hero_id: crate::components::hero::HeroId,
    pub ability_id: crate::components::hero::AbilityId,
    pub affected_units: Vec<Entity>,
    pub radius: f32,
}
