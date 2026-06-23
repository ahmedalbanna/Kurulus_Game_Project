use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Tracks the detection chain stage: Suspicion → Confirmation → Alarm.
///
/// GDD spec:
/// - Suspicion phase: 1-3 seconds where the observer is uncertain
/// - Confirmation phase: 1 second to confirm the sighting
/// - Alarm phase: broadcasts alert to all enemies within 50m
///
/// This component is added to units that are actively detecting a hidden unit.
#[derive(Component, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Component)]
pub struct DetectionChain {
    /// Current suspicion level (0.0 = none, 1.0 = confirmed sighting).
    pub suspicion_level: f32,
    /// Timer for suspicion progression (seconds spent in current stage).
    pub timer: f32,
    /// Time required to advance from suspicion to confirmation (1-3s random).
    pub confirmation_time: f32,
    /// Whether the alarm has been raised.
    pub alarm_raised: bool,
    /// The entity being tracked (the hidden unit under suspicion).
    pub tracked_entity: Option<Entity>,
    /// Whether a silent kill has prevented the alarm.
    pub alarm_prevented: bool,
}

impl Default for DetectionChain {
    fn default() -> Self {
        Self {
            suspicion_level: 0.0,
            timer: 0.0,
            confirmation_time: 2.0, // default 2s
            alarm_raised: false,
            tracked_entity: None,
            alarm_prevented: false,
        }
    }
}

impl DetectionChain {
    /// Create a new detection chain targeting a specific entity.
    /// Confirmation time is randomized between 1.0 and 3.0 seconds.
    pub fn new(target: Entity, rng_seed: f32) -> Self {
        // Use the seed to produce a deterministic-ish random between 1.0 and 3.0
        let confirmation_time = 1.0 + (rng_seed.abs().fract() * 2.0);
        Self {
            suspicion_level: 0.3, // start with suspicion
            timer: 0.0,
            confirmation_time,
            alarm_raised: false,
            tracked_entity: Some(target),
            alarm_prevented: false,
        }
    }
}
