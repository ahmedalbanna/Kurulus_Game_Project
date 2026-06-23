use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Indicates whether a building currently has road access (an adjacent Road
/// entity) and whether the production penalty has been applied.
///
/// Buildings without road access suffer a production penalty (see
/// `road_access_production_penalty_system` in the road_infrastructure plugin).
///
/// The `penalty_applied` flag tracks whether the production rates have already
/// been halved so the penalty is only applied/removed when access *changes*,
/// avoiding floating-point drift from repeated re-scaling.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct RoadAccess {
    pub has_access: bool,
    /// `true` after the 50 % production penalty has been applied.
    pub penalty_applied: bool,
}

impl Default for RoadAccess {
    fn default() -> Self {
        Self {
            has_access: false,
            penalty_applied: false,
        }
    }
}

/// Marker component for buildings that require road access to operate at
/// full efficiency.  Attached by [`requires_road_access`] at construction
/// completion time.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct RequiresRoadAccess;

/// Returns `true` if the given building type should require road access.
pub fn requires_road_access(bt: crate::components::building::BuildingType) -> bool {
    use crate::components::building::BuildingType;
    match bt {
        // Roads themselves do not need road access.
        BuildingType::Road => false,
        // Defensive structures operate independently.
        BuildingType::Palisade | BuildingType::StoneWall | BuildingType::Gate => false,
        // Watchtowers sit on the perimeter — no road needed.
        BuildingType::Watchtower => false,
        // All other buildings (production, military, special, services)
        // need road access for supply chains and mobility.
        _ => true,
    }
}
