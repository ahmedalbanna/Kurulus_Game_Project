use bevy::prelude::*;
use crate::components::{DamageType, WeaponType};

#[derive(Event, Clone, Debug)]
pub struct DamageDealt {
    pub attacker: Entity,
    pub target: Entity,
    pub amount: f32,
    pub damage_type: DamageType,
}

#[derive(Event, Clone, Debug)]
pub struct UnitDied {
    pub entity: Entity,
    pub killer: Entity,
}

#[derive(Event, Clone, Debug)]
pub struct AttackTriggered {
    pub attacker: Entity,
    pub target: Entity,
    pub weapon_type: WeaponType,
}

#[derive(Event, Clone, Debug)]
pub struct MoraleBroken {
    pub entity: Entity,
    pub morale_value: f32,
}

#[derive(Event, Clone, Debug)]
pub struct AmbushTriggered {
    pub ambusher: Entity,
    pub target: Entity,
    pub damage_bonus: f32,
    pub zone: Entity,
}

#[derive(Event, Clone, Debug)]
pub struct StealthRevealed {
    pub entity: Entity,
    pub revealer: Entity,
}
