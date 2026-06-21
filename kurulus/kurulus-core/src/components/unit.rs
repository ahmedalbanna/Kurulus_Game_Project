use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub archetype: UnitArchetype,
    pub faction: FactionId,
    pub formation: FormationType,
    pub stance: CombatStance,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub armor: f32,
    pub armor_type: ArmorType,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            armor: 5.0,
            armor_type: ArmorType::Light,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub move_speed: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub vision_range: f32,
    pub morale: f32,
    pub morale_max: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            move_speed: 3.0,
            attack_range: 1.0,
            attack_damage: 10.0,
            attack_speed: 1.0,
            vision_range: 6.0,
            morale: 50.0,
            morale_max: 100.0,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Formation {
    pub leader: Entity,
    pub offset: Vec2,
    pub slot_index: usize,
}

#[derive(Component, Clone, Debug, Default)]
pub struct AttackTimer(pub Timer);

impl AttackTimer {
    pub fn new(duration: f32, mode: TimerMode) -> Self {
        Self(Timer::from_seconds(duration, mode))
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dead;

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitArchetype {
    LightInfantry,
    HeavyInfantry,
    Archer,
    LightCavalry,
    HeavyCavalry,
    Scout,
    SiegeCrew,
    Worker,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FactionId {
    Kayi,
    Dodurga,
    Candar,
    Germiyan,
    Karaman,
    Byzantine,
    Mongol,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Column,
    Square,
    Wedge,
    Skirmish,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatStance {
    Aggressive,
    Defensive,
    Hold,
    Retreat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorType {
    Light,
    Medium,
    Heavy,
    Fortification,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FormationFollower;
