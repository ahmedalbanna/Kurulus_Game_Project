use super::building::BuildingType;
use super::unit::FactionId;
use super::world::RegionId;
use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CoCommand {
    pub primary_hero: Entity,
    pub secondary_hero: Entity,
    pub active_mode: CoCommandMode,
    pub delegated_orders: Vec<DelegationOrder>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CoCommandStateComponent {
    pub primary_hero: Entity,
    pub secondary_hero: Entity,
    pub mode: CoCommandMode,
    pub trust_level: f32,
    pub synergy_bonus: f32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct TrustMeter {
    pub primary_trust: f32,
    pub secondary_trust: f32,
    pub max_trust: f32,
    pub decay_rate: f32,
    pub gain_rate: f32,
}

// --- Enums & Structs ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum CoCommandMode {
    Solo,
    Delegated,
    Joint,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct DelegationOrder {
    pub from_hero: Entity,
    pub to_hero: Entity,
    pub order_type: DelegationType,
    pub target: DelegationTarget,
    pub issued_at: f32,
    pub expires_at: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DelegationType {
    Move,
    Attack,
    Build,
    Research,
    Diplomacy,
    Recruit,
    Patrol,
    Gather,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub enum DelegationTarget {
    Position(GridPos),
    Entity(Entity),
    Region(RegionId),
    Building(BuildingType),
    Tech(String),
    Faction(FactionId),
}

// ─── Phase 4 Enhancements ───────────────────────────────────────────────────

/// Trust tier levels (GDD §20.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TrustTier {
    /// 0-20: 30-50% AI efficiency, 15% order refusal
    Rivalry,
    /// 21-40: 50-65% AI efficiency, 15% misinterpretation
    Tension,
    /// 41-60: 65-80% AI efficiency
    Neutral,
    /// 61-80: 80-90% AI efficiency, +20% synergy
    Cooperation,
    /// 81-100: 90-100% AI efficiency, +50% synergy, unlock ultimates
    Brotherhood,
}

/// Delegation order with full parameters (GDD §20.3).
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct DelegationOrderDetailed {
    pub order_type: DelegationType,
    pub target: DelegationTarget,
    pub priority: OrderPriority,
    pub aggression: AggressionLevel,
    pub duration_secs: Option<f32>,
    pub retreat_threshold: f32,
    pub issued_at: f32,
    pub expires_at: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum OrderPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum AggressionLevel {
    Cautious,
    Normal,
    Aggressive,
    Suicidal,
}

/// Brotherhood ability tiers (GDD §20.5).
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct BrotherhoodAbility {
    pub tier: u8,
    pub trust_required: f32,
    pub cooldown: f32,
    pub active: bool,
}
