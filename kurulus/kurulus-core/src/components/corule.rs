use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::coord::GridPos;
use super::unit::FactionId;
use super::world::RegionId;
use super::building::BuildingType;

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
