use bevy::prelude::*;
use crate::components::{ObjectiveId, MissionId, DialogueTrigger};

#[derive(Event, Clone, Debug)]
pub struct ObjectiveCompleted {
    pub objective_id: ObjectiveId,
    pub mission_id: MissionId,
}

#[derive(Event, Clone, Debug)]
pub struct MissionEnded {
    pub mission_id: MissionId,
    pub success: bool,
}

#[derive(Event, Clone, Debug)]
pub struct MissionStarted {
    pub mission_id: MissionId,
}

#[derive(Event, Clone, Debug)]
pub struct DialogueTriggered {
    pub dialogue_id: String,
    pub trigger_type: DialogueTrigger,
}

#[derive(Event, Clone, Debug)]
pub struct ObjectiveProgress {
    pub objective_id: ObjectiveId,
    pub progress: f32,
    pub required: f32,
}
