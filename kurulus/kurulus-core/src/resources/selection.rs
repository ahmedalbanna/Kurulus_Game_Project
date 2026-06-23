use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks the player's current selection state and control groups.
#[derive(Resource, Clone, Debug)]
pub struct SelectionState {
    /// Currently selected entities in order (first = most recently selected).
    /// Order matters for UI display (e.g. showing info for the most recently
    /// selected entity when only one unit's details are shown).
    pub selected: Vec<Entity>,
    /// Control groups: key 1-9 → ordered list of entities.
    pub control_groups: HashMap<u8, Vec<Entity>>,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            selected: Vec::new(),
            control_groups: HashMap::new(),
        }
    }
}

/// Configuration for auto-engagement (stances).
/// Used by the auto-engagement system (Phase C) and the stance-change
/// system (Phase E).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct EngagementConfig {
    /// Distance at which aggressive units auto-engage (meters).
    pub aggro_range: f32,
    /// Distance at which defensive units auto-engage.
    pub defensive_range: f32,
    /// Chase distance for aggressive stance.
    pub chase_range: f32,
    /// HP percentage threshold for flee behavior.
    pub flee_hp_pct: f32,
    /// Distance to retreat toward rally point.
    pub retreat_distance: f32,
}

impl Default for EngagementConfig {
    fn default() -> Self {
        Self {
            aggro_range: 6.0,
            defensive_range: 15.0,
            chase_range: 50.0,
            flee_hp_pct: 0.30,
            retreat_distance: 100.0,
        }
    }
}
