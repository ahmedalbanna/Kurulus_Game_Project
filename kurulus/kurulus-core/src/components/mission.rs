use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Objective {
    pub objective_id: ObjectiveId,
    pub objective_type: ObjectiveType,
    pub description: String,
    pub progress: f32,
    pub required_progress: f32,
    pub completed: bool,
    pub rewards: MissionRewards,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct MissionStateComponent {
    pub mission_id: MissionId,
    pub phase: MissionPhase,
    pub objectives: Vec<Objective>,
    pub current_objective_index: usize,
    pub timer: Option<f32>,
    pub time_limit: Option<f32>,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct DialogueRef {
    pub dialogue_id: String,
    pub trigger_type: DialogueTrigger,
    pub triggered: bool,
    pub cooldown: f32,
}

// --- Enums ---

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionId {
    Mission01,
    Mission02,
    Mission03,
    Mission04,
    Mission05,
    Mission06,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectiveId {
    TutorialMoveCamera,
    TutorialBuildOba,
    TutorialRecruitUnit,
    TutorialAttackEnemy,
    BuildBarracks,
    RecruitTenUnits,
    CaptureRegion,
    KillEnemyHero,
    SurviveWave,
    GatherResources,
    EscortCaravan,
    DefendPosition,
    DestroyWall,
    RescueAlly,
    Custom(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ObjectiveType {
    Build,
    Recruit,
    Kill,
    Capture,
    Survive,
    Gather,
    Escort,
    Defend,
    Destroy,
    Rescue,
    ReachLocation,
    ResearchTech,
    Diplomacy,
    Timed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MissionPhase {
    Intro,
    Active,
    Complete,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionRewards {
    pub gold: u32,
    pub xp: u32,
    pub sovereignty_points: u32,
    pub item_unlock: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DialogueTrigger {
    OnMissionStart,
    OnObjectiveComplete,
    OnEnemySpotted,
    OnBuildingBuilt,
    OnHeroLevelUp,
    OnTimer,
    OnCondition(String),
}

/// Marker component for a unit that needs to be rescued in a mission.
/// The objective progresses when this entity is escorted to safety (despawned or reaches the extraction zone).
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct RescueTarget {
    pub extracted: bool,
}
