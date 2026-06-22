use bevy::prelude::*;
use crate::coord::GridPos;
use crate::components::{BuildingType, ZoneType};
use crate::resources::city::{CityLevel, CityEventType};

/// Fired when a new zone district is placed on the map.
#[derive(Event, Clone, Debug)]
pub struct ZonePlaced {
    pub zone_type: ZoneType,
    pub position: GridPos,
    pub entity: Option<Entity>,
}

/// Fired when an existing zone is upgraded to a higher level.
#[derive(Event, Clone, Debug)]
pub struct ZoneUpgraded {
    pub entity: Entity,
    pub zone_type: ZoneType,
    pub old_level: u8,
    pub new_level: u8,
}

/// Fired when the settlement advances to a higher city level
/// (Village → Town → City → Metropolis).
#[derive(Event, Clone, Debug)]
pub struct CityLevelUp {
    pub old_level: CityLevel,
    pub new_level: CityLevel,
    pub population: u32,
}

/// Fired when a building is upgraded to the next tier in its chain.
#[derive(Event, Clone, Debug)]
pub struct BuildingUpgraded {
    pub entity: Entity,
    pub building_type: BuildingType,
    pub old_level: u8,
    pub new_level: u8,
    pub old_name: String,
    pub new_name: String,
}

/// Fired when a city event (positive or negative) begins.
#[derive(Event, Clone, Debug)]
pub struct CityEventOccurred {
    pub event_type: CityEventType,
    pub magnitude: f32,
    pub duration: f32,
}

/// Fired when a city event ends.
#[derive(Event, Clone, Debug)]
pub struct CityEventEnded {
    pub event_type: CityEventType,
}

/// Fired to request a service coverage recalculation.
#[derive(Event, Clone, Debug)]
pub struct RecalculateCoverage;
