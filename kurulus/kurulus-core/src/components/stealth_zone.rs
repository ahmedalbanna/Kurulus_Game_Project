use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Types of stealth zones as defined in GDD 5.4.
/// Each zone type provides different detection ranges and unit capacities.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StealthZoneType {
    DenseForest,
    MediumForest,
    TallGrass,
    NightDarkness,
    Ruins,
    Caves,
    CropField,
    ArtificialCover,
}

impl StealthZoneType {
    /// Detection range for a stationary (hidden) unit inside this zone.
    pub fn detection_range_static(&self) -> f32 {
        match self {
            StealthZoneType::DenseForest => 5.0,
            StealthZoneType::MediumForest => 8.0,
            StealthZoneType::TallGrass => 4.0,
            StealthZoneType::NightDarkness => 3.0,
            StealthZoneType::Ruins => 6.0,
            StealthZoneType::Caves => 3.0,
            StealthZoneType::CropField => 5.0,
            StealthZoneType::ArtificialCover => 4.0,
        }
    }

    /// Detection range for a moving unit inside this zone.
    pub fn detection_range_moving(&self) -> f32 {
        match self {
            StealthZoneType::DenseForest => 8.0,
            StealthZoneType::MediumForest => 12.0,
            StealthZoneType::TallGrass => 4.0, // must stay still
            StealthZoneType::NightDarkness => 3.0,
            StealthZoneType::Ruins => 6.0,
            StealthZoneType::Caves => 3.0,
            StealthZoneType::CropField => 5.0,
            StealthZoneType::ArtificialCover => 4.0, // must stay still
        }
    }

    /// Maximum number of units this zone can conceal.
    /// None = unlimited (practically).
    pub fn capacity(&self) -> Option<u32> {
        match self {
            StealthZoneType::DenseForest => None,
            StealthZoneType::MediumForest => Some(15),
            StealthZoneType::TallGrass => Some(1),
            StealthZoneType::NightDarkness => None,
            StealthZoneType::Ruins => Some(10),
            StealthZoneType::Caves => Some(10),
            StealthZoneType::CropField => Some(3),
            StealthZoneType::ArtificialCover => Some(5),
        }
    }

    /// Whether this zone can be destroyed (e.g. forest fire, siege).
    pub fn destructible(&self) -> bool {
        matches!(
            self,
            StealthZoneType::DenseForest
                | StealthZoneType::MediumForest
                | StealthZoneType::CropField
                | StealthZoneType::Ruins
        )
    }

    /// Stealth level bonus granted by this zone (0.0 to 1.0).
    pub fn stealth_bonus(&self) -> f32 {
        match self {
            StealthZoneType::DenseForest => 0.35,
            StealthZoneType::MediumForest => 0.25,
            StealthZoneType::TallGrass => 0.30,
            StealthZoneType::NightDarkness => 0.40,
            StealthZoneType::Ruins => 0.20,
            StealthZoneType::Caves => 0.45,
            StealthZoneType::CropField => 0.25,
            StealthZoneType::ArtificialCover => 0.30,
        }
    }
}

/// Component attached to terrain tiles that provide stealth coverage.
/// This is a parallel overlay to `TerrainType` — a tile may have both.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct StealthZone {
    pub zone_type: StealthZoneType,
    /// Current number of units occupying this zone.
    pub current_occupancy: u32,
    /// Whether this zone has been destroyed (e.g. forest fire).
    pub destroyed: bool,
}

impl StealthZone {
    pub fn new(zone_type: StealthZoneType) -> Self {
        Self {
            zone_type,
            current_occupancy: 0,
            destroyed: false,
        }
    }

    /// Returns true if this zone is at or over capacity.
    pub fn is_overcrowded(&self) -> bool {
        if self.destroyed {
            return true;
        }
        match self.zone_type.capacity() {
            None => false,
            Some(cap) => self.current_occupancy > cap,
        }
    }

    /// Returns the effective detection range modifier for a unit in this zone.
    /// Overcrowded zones reduce stealth effectiveness.
    pub fn effective_detection_range(&self, moving: bool) -> f32 {
        let base = if moving {
            self.zone_type.detection_range_moving()
        } else {
            self.zone_type.detection_range_static()
        };
        if self.is_overcrowded() {
            // Overcrowded: detection range increases dramatically
            10.0
        } else {
            base
        }
    }
}
