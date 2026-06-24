use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marks that a unit should use flow‑field movement instead of individual
/// A* pathfinding.  The unit reads its movement direction from the
/// `FlowFieldCache` resource each tick.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowFieldMovement {
    /// The destination grid cell the unit is flowing toward.
    pub destination: GridPos,
}
