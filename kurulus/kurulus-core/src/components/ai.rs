use super::building::BuildingType;
use super::unit::UnitArchetype;
use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── Core AI Components ─────────────────────────────────────────────────────

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
    /// The path of child indices from root to the currently running node.
    /// Empty means the tree is finished (Success/Failure) and should restart.
    pub execution_path: Vec<usize>,
    pub blackboard: AIDataBlackboard,
    /// Whether the tree evaluation should pause in Running state.
    pub last_state: BehaviorState,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Threat {
    pub target: Entity,
    pub threat_level: f32,
    pub last_seen: f32,
    pub threat_type: ThreatType,
}

/// Marker component for AI-controlled worker units that auto-gather/construct.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Default)]
pub struct AIWorker {
    pub assigned_task: Option<AIWorkerTask>,
    pub idle_time: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIWorkerTask {
    GatherFood,
    GatherWood,
    GatherStone,
    GatherGold,
    Construct(BuildingType),
}

/// Marker component for AI-controlled combat units.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AICombatUnit {
    pub target: Option<Entity>,
    pub flanking: bool,
    pub retreat_timer: f32,
}

// ─── Behavior Tree State ────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorState {
    Success,
    Failure,
    Running,
}

// ─── Enums ──────────────────────────────────────────────────────────────────

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
    /// Issues a command / takes an action and returns Running until completion.
    Action(BehaviorAction),
    Inverter(Box<BehaviorNode>),
    /// Repeats child N times or forever if count is 0.
    Repeater(Box<BehaviorNode>, u32),
    /// Always succeeds; useful as a no-op fallback.
    Succeeder(Box<BehaviorNode>),
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
    /// Check if population is at or above cap.
    PopulationCapReached,
    /// Check if current army strength is below threshold.
    ArmyStrengthBelow(f32),
    /// Check if resource stockpile is below threshold for a given resource.
    ResourceLow(AiResourceType),
    /// Check if a specific building type exists.
    HasBuilding(BuildingType),
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiResourceType {
    Food,
    Gold,
    Wood,
    Stone,
}

impl std::fmt::Display for AiResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiResourceType::Food => write!(f, "Food"),
            AiResourceType::Gold => write!(f, "Gold"),
            AiResourceType::Wood => write!(f, "Wood"),
            AiResourceType::Stone => write!(f, "Stone"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BehaviorAction {
    /// Move entity to a grid position.
    MoveTo(GridPos),
    /// Attack a specific entity.
    Attack(Entity),
    /// Build a building at a position.
    Build(BuildingType, GridPos),
    /// Recruit units of a given archetype.
    Recruit(UnitArchetype, u32),
    /// Research a technology.
    Research(String),
    /// Retreat to a safe position.
    Retreat(GridPos),
    /// Patrol between two positions.
    Patrol(GridPos, GridPos),
    /// Idle for a number of ticks.
    Wait(u32),
    /// Flank around target.
    Flank(Entity),
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AIDataBlackboard {
    pub data: Vec<(String, f32)>,
}

impl AIDataBlackboard {
    pub fn set(&mut self, key: &str, value: f32) {
        for entry in self.data.iter_mut() {
            if entry.0 == key {
                entry.1 = value;
                return;
            }
        }
        self.data.push((key.to_string(), value));
    }

    pub fn get(&self, key: &str) -> Option<f32> {
        self.data.iter().find(|e| e.0 == key).map(|e| e.1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatType {
    Military,
    Economic,
    Diplomatic,
    Proximity,
}
