use super::building::BuildingType;
use super::unit::UnitArchetype;
use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── Utility AI (big-brain pattern) Components ──────────────────────────────
//
// This implements a Utility AI system where entities have a `UtilityThinker`
// that evaluates multiple `Scorer` + `Action` pairs (considerations) to select
// the highest-scoring action dynamically. Designed for complex AI that needs
// to react fluidly to changing game state (enemy commanders, heroes).
//
// Simpler AI (workers, civilians) should continue to use Behavior Trees.

/// Marker component for entities using Utility AI instead of Behavior Trees.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct UtilityThinker {
    /// The list of considerations (scorer + action pairs) to evaluate.
    pub considerations: Vec<Consideration>,
    /// Seconds between full re-evaluations.
    pub evaluation_interval: f32,
    /// Game time of last evaluation.
    pub last_evaluation: f32,
    /// The currently selected action, if any.
    pub current_action: Option<UtilityAction>,
    /// Progress toward completion of the current action (0.0–1.0).
    pub action_progress: f32,
    /// Whether the current action is still running.
    pub action_running: bool,
    /// If true, the current action runs to completion before re-evaluating.
    pub sticky: bool,
}

impl Default for UtilityThinker {
    fn default() -> Self {
        Self {
            considerations: Vec::new(),
            evaluation_interval: 0.5,
            last_evaluation: 0.0,
            current_action: None,
            action_progress: 0.0,
            action_running: false,
            sticky: true,
        }
    }
}

/// A single consideration pairing a scorer with an action and a weight.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Consideration {
    /// Name for debugging / logging.
    pub name: String,
    /// The scorer used to compute a utility score (0.0–1.0).
    pub scorer: ScorerConfig,
    /// The action to take when this consideration wins.
    pub action: UtilityAction,
    /// Multiplier on the raw score (can bias toward specific actions).
    pub weight: f32,
}

/// Configuration for a utility scorer that evaluates a specific aspect
/// of the game world and returns a normalised score (0.0–1.0).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScorerConfig {
    /// Score based on the proximity/strength of nearby enemies.
    /// Parameter: range (grid units), falloff (0=linear, 1=exponential).
    TargetThreat { range: f32, falloff: f32 },
    /// Score increases as HP ratio drops below threshold.
    LowHealth { threshold: f32 },
    /// Score based on resource shortage (low stockpile → high score).
    ResourceNeed {
        resource: AiUtilityResource,
        threshold: f32,
    },
    /// Score based on distance to nearest enemy.
    DistanceToEnemy { max_range: f32 },
    /// Score from current morale level (low morale → high score for defensive actions).
    MoraleState,
    /// Score based on our army count vs enemy army count.
    ForceRatio { advantage_threshold: f32 },
    /// Score is 1.0 if faction has a building of the given type, else 0.0.
    HasBuilding { building: BuildingType },
    /// Score is 1.0 if the entity is currently under attack.
    IsUnderAttack,
    /// Score is 1.0 if population is at or above cap.
    PopulationCapReached,
    /// Constant score (for always-available fallback actions).
    Constant(f32),
    /// Custom named scorer resolved at runtime (extensible).
    Custom(String),
}

/// Actions a Utility AI entity can take.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UtilityAction {
    /// Attack the nearest enemy.
    AttackNearest,
    /// Attack a specific target entity (filled in at runtime).
    AttackEntity(Entity),
    /// Retreat toward a safe position.
    Retreat(GridPos),
    /// Gather a specific resource.
    Gather(AiUtilityResource),
    /// Build a building at a position.
    Build(BuildingType, GridPos),
    /// Patrol between two positions.
    Patrol(GridPos, GridPos),
    /// Defend a position (move to and hold).
    Defend(GridPos),
    /// Recruit units of an archetype.
    Recruit(UnitArchetype, u32),
    /// Research a technology (string key).
    Research(String),
    /// Do nothing for the given duration (in seconds).
    Wait(f32),
    /// Flank a target entity.
    Flank(Entity),
    /// Custom action string (for extensibility).
    Custom(String),
}

/// Resource types the Utility AI can reason about gathering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiUtilityResource {
    Food,
    Wood,
    Stone,
    Gold,
}

impl std::fmt::Display for AiUtilityResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Food => write!(f, "Food"),
            Self::Wood => write!(f, "Wood"),
            Self::Stone => write!(f, "Stone"),
            Self::Gold => write!(f, "Gold"),
        }
    }
}
