use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Alertness level of a unit — directly affects detection capability.
#[derive(Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Normal patrol state — baseline detection.
    Unaware,
    /// Heard a suspicious sound — slightly improved detection.
    Suspicious,
    /// After an alarm or combat — heightened detection.
    Alert,
    /// In active combat — maximum detection but distracted.
    Combat,
}

impl AlertLevel {
    /// Multiplier applied to detection range based on alertness.
    pub fn detection_multiplier(&self) -> f32 {
        match self {
            AlertLevel::Unaware => 0.8,
            AlertLevel::Suspicious => 1.0,
            AlertLevel::Alert => 1.2,
            AlertLevel::Combat => 1.3,
        }
    }
}

/// Observer state component — tracks factors that affect a unit's detection ability.
/// Modelled after GDD Observer Factors table.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ObserverState {
    /// Current alertness level.
    pub alert_level: AlertLevel,
    /// Fatigue: 0.0 = fresh, 1.0 = exhausted.
    /// Fatigue multiplies detection range by (1.0 - fatigue * 0.3).
    pub fatigue: f32,
    /// Whether this unit is part of a formation (formations increase vigilance).
    pub in_formation: bool,
    /// Whether this unit is alone (alone units are less vigilant).
    pub is_alone: bool,
    /// Whether this unit is sleeping (only at night in camps).
    pub is_sleeping: bool,
}

impl Default for ObserverState {
    fn default() -> Self {
        Self {
            alert_level: AlertLevel::Unaware,
            fatigue: 0.0,
            in_formation: false,
            is_alone: true,
            is_sleeping: false,
        }
    }
}

impl ObserverState {
    /// Compute the combined observer factor multiplier.
    /// This is the product of all active observer factors.
    pub fn effective_detection_mult(&self) -> f32 {
        let alert_mult = self.alert_level.detection_multiplier();
        let fatigue_mult = (1.0 - self.fatigue * 0.3).max(0.1);
        let formation_mult = if self.in_formation { 1.1 } else { 1.0 };
        let alone_mult = if self.is_alone { 0.9 } else { 1.0 };
        let sleep_mult = if self.is_sleeping { 0.1 } else { 1.0 };

        alert_mult * fatigue_mult * formation_mult * alone_mult * sleep_mult
    }
}
