use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::MissionId;
use crate::resources::ResourceStockpile;

/// Tracks the overall campaign state: which missions are completed/unlocked
/// and which special items have been collected.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct CampaignState {
    pub completed_missions: Vec<MissionId>,
    pub unlocked_missions: Vec<MissionId>,
    pub current_mission: Option<MissionId>,
    /// Special narrative items unlocked during the campaign
    /// (e.g. "Byzantine_Standard", "Mongol_Horse_Breed").
    pub unlocked_items: Vec<String>,
}

impl Default for CampaignState {
    fn default() -> Self {
        Self {
            completed_missions: Vec::new(),
            // Mission01 is always unlocked by default
            unlocked_missions: vec![MissionId::Mission01],
            current_mission: None,
            unlocked_items: Vec::new(),
        }
    }
}

impl CampaignState {
    /// Returns the next mission in sequence after the given one.
    pub fn next_mission(current: MissionId) -> Option<MissionId> {
        match current {
            MissionId::Mission01 => Some(MissionId::Mission02),
            MissionId::Mission02 => Some(MissionId::Mission03),
            MissionId::Mission03 => Some(MissionId::Mission04),
            MissionId::Mission04 => Some(MissionId::Mission05),
            MissionId::Mission05 => Some(MissionId::Mission06),
            MissionId::Mission06 => None,
        }
    }

    /// Marks a mission as completed and unlocks the next in sequence.
    pub fn complete_mission(&mut self, mission_id: MissionId) {
        if !self.completed_missions.contains(&mission_id) {
            self.completed_missions.push(mission_id);
        }
        if let Some(next) = Self::next_mission(mission_id) {
            if !self.unlocked_missions.contains(&next) {
                self.unlocked_missions.push(next);
            }
        }
    }

    /// Returns whether a mission has been completed.
    pub fn is_completed(&self, mission_id: MissionId) -> bool {
        self.completed_missions.contains(&mission_id)
    }

    /// Returns whether a mission is unlocked (playable).
    pub fn is_unlocked(&self, mission_id: MissionId) -> bool {
        self.unlocked_missions.contains(&mission_id)
    }
}

/// Carries over ResourceStockpile between missions in campaign mode.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ResourcePersistence {
    pub saved_stockpile: Option<ResourceStockpile>,
}

impl Default for ResourcePersistence {
    fn default() -> Self {
        Self { saved_stockpile: None }
    }
}

/// Event fired when the campaign advances (mission completed, next unlocked).
#[derive(Event, Clone, Debug)]
pub struct CampaignAdvanced {
    pub completed_mission: MissionId,
    pub next_unlocked: Option<MissionId>,
}
