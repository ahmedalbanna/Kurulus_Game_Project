use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Component for bait rider / lure target entities.
/// These units are designed to attract enemy attention and lead them into ambushes.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct LureTarget {
    /// Maximum distance this unit can lead enemies before they return (leash range).
    pub leash_range: f32,
    /// The origin point (where the pursuer started following from).
    pub origin_point: Vec2,
    /// Whether this entity is a bait rider (special light cavalry).
    pub is_bait_rider: bool,
    /// Whether this unit can taunt enemies (draw aggro).
    pub can_taunt: bool,
    /// Cooldown remaining for taunt ability (seconds).
    pub taunt_cooldown_remaining: f32,
    /// Duration of taunt cooldown.
    pub taunt_cooldown_duration: f32,
    /// Cooldown remaining for sprint ability (seconds).
    pub sprint_cooldown_remaining: f32,
    /// Duration of sprint cooldown.
    pub sprint_cooldown_duration: f32,
    /// Whether the lure is currently being pursued.
    pub being_pursued: bool,
    /// Entities currently pursuing this lure target.
    pub pursuers: Vec<Entity>,
}

impl LureTarget {
    /// Create a new bait rider with default settings.
    pub fn bait_rider() -> Self {
        Self {
            leash_range: 40.0,
            origin_point: Vec2::ZERO,
            is_bait_rider: true,
            can_taunt: true,
            taunt_cooldown_remaining: 0.0,
            taunt_cooldown_duration: 20.0,
            sprint_cooldown_remaining: 0.0,
            sprint_cooldown_duration: 30.0,
            being_pursued: false,
            pursuers: Vec::new(),
        }
    }

    /// Create a regular lure target (non-bait-rider).
    pub fn lure(leash_range: f32) -> Self {
        Self {
            leash_range,
            origin_point: Vec2::ZERO,
            is_bait_rider: false,
            can_taunt: false,
            taunt_cooldown_remaining: 0.0,
            taunt_cooldown_duration: 20.0,
            sprint_cooldown_remaining: 0.0,
            sprint_cooldown_duration: 30.0,
            being_pursued: false,
            pursuers: Vec::new(),
        }
    }

    /// Returns true if taunt is ready to use.
    pub fn taunt_ready(&self) -> bool {
        self.can_taunt && self.taunt_cooldown_remaining <= 0.0
    }

    /// Returns true if sprint is ready to use.
    pub fn sprint_ready(&self) -> bool {
        self.sprint_cooldown_remaining <= 0.0
    }
}
