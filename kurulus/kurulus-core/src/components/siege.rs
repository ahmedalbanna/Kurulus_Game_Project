use crate::coord::HeightLevel;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct WallSegment {
    pub health: f32,
    pub max_health: f32,
    pub height_level: HeightLevel,
    pub is_breached: bool,
    pub breach_size: f32,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct SiegeEngine {
    pub engine_type: SiegeEngineType,
    pub crew: Vec<Entity>,
    pub reload_timer: f32,
    pub ammo: SiegeAmmoType,
    pub target: Option<Entity>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BreachPoint {
    pub wall_segment: Entity,
    pub size: f32,
    pub is_passable: bool,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SiegeEngineType {
    Trebuchet,
    Ballista,
    BatteringRam,
    SiegeTower,
    Catapult,
    Mantlet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SiegeAmmoType {
    Stone,
    Fire,
    Bolt,
    Rubbish,
}
