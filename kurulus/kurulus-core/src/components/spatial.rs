use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for entities tracked in the `SpatialGrid`.
///
/// Attach this to any entity that should participate in efficient spatial
/// queries (threat evaluation, flanking calculation, stealth detection, etc.).
///
/// The `SpatialPlugin` will automatically rebuild the grid each frame from
/// all entities bearing this component + `WorldPosition`.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Default, Reflect)]
#[reflect(Component)]
pub struct TrackedBySpatial;
