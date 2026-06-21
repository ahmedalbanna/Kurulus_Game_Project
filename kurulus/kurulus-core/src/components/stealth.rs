use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Stealth {
    pub level: f32,
    pub is_hidden: bool,
    pub hidden_timer: f32,
    pub ambush_zone: Option<Entity>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Detection {
    pub radius: f32,
    pub height_bonus: f32,
    pub active: bool,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AmbushZone {
    pub owner: Entity,
    pub radius: f32,
    pub bonus_damage: f32,
    pub bonus_morale: f32,
    pub duration: f32,
    pub remaining: f32,
}
