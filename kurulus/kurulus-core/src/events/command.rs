use bevy::prelude::*;
use crate::coord::GridPos;
use crate::components::{BuildingType, UnitArchetype, AbilityId};

#[derive(Event, Clone, Debug)]
pub struct MoveOrder {
    pub entity: Entity,
    pub target: GridPos,
    pub formation: Option<Entity>,
}

#[derive(Event, Clone, Debug)]
pub struct AttackOrder {
    pub entity: Entity,
    pub target: Entity,
    pub aggressive: bool,
}

#[derive(Event, Clone, Debug)]
pub struct BuildOrder {
    pub entity: Entity,
    pub building_type: BuildingType,
    pub position: GridPos,
}

#[derive(Event, Clone, Debug)]
pub struct RecruitOrder {
    pub building: Entity,
    pub archetype: UnitArchetype,
    pub count: u32,
}

#[derive(Event, Clone, Debug)]
pub struct ResearchOrder {
    pub entity: Entity,
    pub tech_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct AbilityUsed {
    pub caster: Entity,
    pub ability_id: AbilityId,
    pub target: AbilityTarget,
}

#[derive(Event, Clone, Debug)]
pub enum AbilityTarget {
    Entity(Entity),
    Position(GridPos),
    Direction(Vec2),
    None,
}
