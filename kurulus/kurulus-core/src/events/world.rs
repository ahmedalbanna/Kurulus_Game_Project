use bevy::prelude::*;
use bevy::math::IVec2;
use serde::{Deserialize, Serialize};
use crate::components::{Encounter, FactionId, PoiId, RegionId, TreatyType};

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

#[derive(Event, Clone, Debug)]
pub struct WorldMapTravelStarted {
    pub from_region: RegionId,
    pub to_region: RegionId,
    pub duration: f32,
}

#[derive(Event, Clone, Debug)]
pub struct WorldMapTravelCompleted {
    pub from_region: RegionId,
    pub to_region: RegionId,
    pub entity: Option<Entity>,
}

#[derive(Event, Clone, Debug)]
pub struct TileRevealed {
    pub position: IVec2,
    pub region: Option<RegionId>,
}

#[derive(Event, Clone, Debug)]
pub struct TileExplored {
    pub position: IVec2,
    pub region: Option<RegionId>,
}

#[derive(Event, Clone, Debug)]
pub struct PoiDiscovered {
    pub poi_id: PoiId,
    pub position: IVec2,
    pub poi_type: String,
    pub name: String,
}

#[derive(Event, Clone, Debug)]
pub struct PoiVisited {
    pub poi_id: PoiId,
    pub position: IVec2,
}

#[derive(Event, Clone, Debug)]
pub struct EncounterTriggered {
    pub encounter: Encounter,
    pub entity: Option<Entity>,
}

// --- Enums ---

#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeasonType {
    Spring,
    Summer,
    Autumn,
    Winter,
}
