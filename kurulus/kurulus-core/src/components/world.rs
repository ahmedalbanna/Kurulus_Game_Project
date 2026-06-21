use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use super::unit::FactionId;
use crate::coord::GridPos;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub region_id: RegionId,
    pub name: String,
    pub owner: FactionId,
    pub resource_bonus: RegionResources,
    pub control_points: Vec<GridPos>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Faction {
    pub faction_id: FactionId,
    pub name: String,
    pub color: Color,
    pub is_player: bool,
    pub is_alive: bool,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FogOfWar {
    pub explored: bool,
    pub visible: bool,
    pub last_seen: f64,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct SupplyLine {
    pub source: Entity,
    pub destination: Entity,
    pub capacity: f32,
    pub usage: f32,
    pub is_blocked: bool,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegionId {
    Sogut,
    Domanic,
    ErmeniBeli,
    Karacahisar,
    Bilecik,
    Yarhisar,
    Inegol,
    Kopruhisar,
    Bursa,
    Iznik,
    Izmit,
    Sakarya,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionResources {
    pub food: f32,
    pub gold: f32,
    pub wood: f32,
    pub stone: f32,
    pub population_capacity: u32,
}
