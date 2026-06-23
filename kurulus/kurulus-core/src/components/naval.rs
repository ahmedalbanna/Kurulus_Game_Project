use super::diplomacy::GoodType;
use super::unit::FactionId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Ship {
    pub ship_type: ShipType,
    pub faction: FactionId,
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub cargo_capacity: u32,
    pub cargo: Vec<ShipCargo>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Harbor {
    pub faction: FactionId,
    pub level: u8,
    pub docked_ships: Vec<Entity>,
    pub max_ships: u8,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BlockadeZone {
    pub owner: FactionId,
    pub radius: f32,
    pub ships: Vec<Entity>,
    pub is_active: bool,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShipType {
    FishingBoat,
    Transport,
    Galley,
    WarGalley,
    FireShip,
    Cog,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShipCargo {
    pub good_type: GoodType,
    pub quantity: u32,
}
