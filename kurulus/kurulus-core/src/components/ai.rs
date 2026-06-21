use super::building::BuildingType;
use super::unit::UnitArchetype;
use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AIController {
    pub personality: AIPersonality,
    pub difficulty: AIDifficulty,
    pub decision_interval: f32,
    pub last_decision_time: f32,
    pub current_goal: Option<AIGoal>,
    pub strategy: AIStrategy,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorTree {
    pub root: BehaviorNode,
    pub current_node: Option<BehaviorNodeId>,
    pub blackboard: AIDataBlackboard,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Threat {
    pub target: Entity,
    pub threat_level: f32,
    pub last_seen: f32,
    pub threat_type: ThreatType,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIPersonality {
    Aggressive,
    Defensive,
    Balanced,
    Cautious,
    Reckless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIDifficulty {
    Easy,
    Normal,
    Hard,
    Expert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIGoal {
    ExpandTerritory,
    BuildEconomy,
    TrainArmy,
    AttackPlayer,
    DefendBase,
    CompleteObjective,
    GatherResources,
    ResearchTech,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIStrategy {
    Rush,
    Turtle,
    Boom,
    Balanced,
    Guerrilla,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Condition(BehaviorCondition),
    Action(BehaviorAction),
    Inverter(Box<BehaviorNode>),
    Repeater(Box<BehaviorNode>, u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BehaviorNodeId(pub usize);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BehaviorCondition {
    HasResources,
    HasArmy,
    IsUnderAttack,
    CanExpand,
    EnemyInRange,
    HasHero,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BehaviorAction {
    MoveTo(GridPos),
    Attack(Entity),
    Build(BuildingType),
    Recruit(UnitArchetype),
    Research(String),
    Retreat,
    Patrol(Vec<GridPos>),
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AIDataBlackboard {
    pub data: Vec<(String, f32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatType {
    Military,
    Economic,
    Diplomatic,
    Proximity,
}
