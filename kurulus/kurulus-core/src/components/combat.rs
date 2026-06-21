use super::unit::ArmorType;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Sword,
    Axe,
    Spear,
    Bow,
    Crossbow,
    Siege,
    Unarmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Melee,
    Ranged,
    Siege,
    Fire,
    Magic,
    True,
}
