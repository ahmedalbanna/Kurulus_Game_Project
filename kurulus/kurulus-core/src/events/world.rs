use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{FactionId, RegionId, TreatyType};

#[derive(Event, Clone, Debug)]
pub struct FactionRelationChanged {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub old_value: f32,
    pub new_value: f32,
}

#[derive(Event, Clone, Debug)]
pub struct RegionCaptured {
    pub region: RegionId,
    pub old_owner: FactionId,
    pub new_owner: FactionId,
}

#[derive(Event, Clone, Debug)]
pub struct WarDeclared {
    pub aggressor: FactionId,
    pub defender: FactionId,
    pub casus_belli: String,
}

#[derive(Event, Clone, Debug)]
pub struct TreatySigned {
    pub treaty_type: TreatyType,
    pub parties: Vec<FactionId>,
}

#[derive(Event, Clone, Debug)]
pub struct SeasonChanged {
    pub season: SeasonType,
}

#[derive(Event, Clone, Debug)]
pub struct DayNightCycle {
    pub is_night: bool,
    pub hour: f32,
}

// --- Enums ---

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeasonType {
    Spring,
    Summer,
    Autumn,
    Winter,
}
