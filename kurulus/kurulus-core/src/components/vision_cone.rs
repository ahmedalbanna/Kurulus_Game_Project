use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Defines the angular vision cone for detection calculations.
/// Units detect other entities based on their facing direction and angle.
///
/// GDD spec: 120° front, 180° peripheral, 0° rear (blind behind).
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct VisionCone {
    /// Front-facing angle in degrees (centered on facing direction).
    /// Units within this angle get full detection range.
    pub front_angle: f32,
    /// Peripheral angle in degrees.
    /// Units between front_angle and side_angle get peripheral_mult detection.
    pub peripheral_angle: f32,
    /// Side angle in degrees — detection drops significantly here.
    pub side_angle: f32,
    /// Multiplier applied to detection range for targets in front arc.
    pub front_mult: f32,
    /// Multiplier for targets in peripheral arc.
    pub peripheral_mult: f32,
    /// Multiplier for targets in side arc (partial).
    pub side_mult: f32,
    /// Multiplier for targets behind the unit (typically 0.0 = blind).
    pub rear_mult: f32,
    /// Current facing direction (unit vector).
    pub facing: Vec2,
}

impl Default for VisionCone {
    fn default() -> Self {
        Self {
            front_angle: 60.0,      // 60° each side of facing = 120° total front
            peripheral_angle: 90.0, // 90° each side = 180° peripheral
            side_angle: 80.0,       // 80° from left/right = 160° total side
            front_mult: 1.0,
            peripheral_mult: 0.7,
            side_mult: 0.4,
            rear_mult: 0.0,
            facing: Vec2::new(0.0, -1.0), // default: facing "down"
        }
    }
}

impl VisionCone {
    /// Compute the detection multiplier based on the angle from facing direction
    /// to the target position.
    ///
    /// `angle_to_target` is the absolute angle in degrees between the detector's
    /// facing direction and the vector to the target (0° = directly in front).
    pub fn detection_mult(&self, angle_to_target: f32) -> f32 {
        let angle = angle_to_target.abs();
        if angle <= self.front_angle {
            self.front_mult
        } else if angle <= self.peripheral_angle {
            self.peripheral_mult
        } else if angle <= self.side_angle {
            self.side_mult
        } else {
            self.rear_mult
        }
    }
}
