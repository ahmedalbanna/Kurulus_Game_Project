use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Global configuration for the Ambush & Stealth system.
/// All GDD magic numbers live here for easy balance tuning.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct AmbushConfig {
    /// Zone defaults
    pub zones: ZoneDefaults,
    /// Detection system configuration
    pub detection: DetectionConfig,
    /// Ambush effects configuration
    pub effects: AmbushEffectsConfig,
    /// Lure / decoy configuration
    pub lure: LureConfig,
    /// Scoring and rewards
    pub scoring: AmbushScoringConfig,
}

impl Default for AmbushConfig {
    fn default() -> Self {
        Self {
            zones: ZoneDefaults::default(),
            detection: DetectionConfig::default(),
            effects: AmbushEffectsConfig::default(),
            lure: LureConfig::default(),
            scoring: AmbushScoringConfig::default(),
        }
    }
}

/// Stealth zone default parameters (per type).
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDefaults {
    pub dense_forest_detection_static: f32,
    pub dense_forest_detection_moving: f32,
    pub medium_forest_detection_static: f32,
    pub medium_forest_detection_moving: f32,
    pub tall_grass_detection: f32,
    pub night_detection: f32,
    pub ruins_detection: f32,
    pub caves_detection: f32,
    pub crop_field_detection: f32,
    pub artificial_cover_detection: f32,
    /// Overcrowded zone detection penalty (override detection range).
    pub overcrowded_detection: f32,
}

impl Default for ZoneDefaults {
    fn default() -> Self {
        Self {
            dense_forest_detection_static: 5.0,
            dense_forest_detection_moving: 8.0,
            medium_forest_detection_static: 8.0,
            medium_forest_detection_moving: 12.0,
            tall_grass_detection: 4.0,
            night_detection: 3.0,
            ruins_detection: 6.0,
            caves_detection: 3.0,
            crop_field_detection: 5.0,
            artificial_cover_detection: 4.0,
            overcrowded_detection: 10.0,
        }
    }
}

/// Detection system parameters.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct DetectionConfig {
    // Vision cone angles (degrees)
    pub vision_cone_front_angle: f32,
    pub vision_cone_peripheral_angle: f32,
    pub vision_cone_side_angle: f32,
    // Vision cone multipliers
    pub front_mult: f32,
    pub peripheral_mult: f32,
    pub side_mult: f32,
    pub rear_mult: f32,
    // Noise
    pub noise_detection_mult: f32,
    pub noise_walk_range: f32,
    pub noise_run_range: f32,
    pub noise_combat_range: f32,
    // Alarm
    pub alarm_radius: f32,
    pub alarm_duration: f32,
    // Detection chain timings
    pub suspicion_time_min: f32,
    pub suspicion_time_max: f32,
    pub confirmation_time: f32,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            vision_cone_front_angle: 60.0,
            vision_cone_peripheral_angle: 90.0,
            vision_cone_side_angle: 80.0,
            front_mult: 1.0,
            peripheral_mult: 0.7,
            side_mult: 0.4,
            rear_mult: 0.0,
            noise_detection_mult: 0.3,
            noise_walk_range: 5.0,
            noise_run_range: 12.0,
            noise_combat_range: 15.0,
            alarm_radius: 50.0,
            alarm_duration: 30.0,
            suspicion_time_min: 1.0,
            suspicion_time_max: 3.0,
            confirmation_time: 1.0,
        }
    }
}

/// Ambush effects parameters.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct AmbushEffectsConfig {
    pub damage_multiplier: f32,
    pub damage_duration: f32,
    pub panic_morale_penalty: f32,
    pub panic_morale_check_mult: f32,
    pub formation_break_duration: f32,
    pub shock_duration: f32,
    pub failed_ambush_enemy_bonus: f32,
    pub placement_cost_gold: f32,
    pub default_zone_radius: f32,
    pub default_zone_duration: f32,
    pub default_bonus_damage: f32,
    pub default_bonus_morale: f32,
}

impl Default for AmbushEffectsConfig {
    fn default() -> Self {
        Self {
            damage_multiplier: 2.0,
            damage_duration: 10.0,
            panic_morale_penalty: 30.0,
            panic_morale_check_mult: 2.5,
            formation_break_duration: 3.0,
            shock_duration: 2.0,
            failed_ambush_enemy_bonus: 1.5,
            placement_cost_gold: 30.0,
            default_zone_radius: 4.0,
            default_zone_duration: 60.0,
            default_bonus_damage: 1.5,
            default_bonus_morale: 0.2,
        }
    }
}

/// Lure and decoy system parameters.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct LureConfig {
    pub bait_rider_speed: f32,
    pub bait_rider_hp: f32,
    pub bait_rider_damage: f32,
    pub bait_rider_cost_gold: f32,
    pub max_bait_riders_per_mission: u32,
    pub default_leash_range: f32,
    pub mongol_leash_range: f32,
    pub byzantine_leash_range: f32,
    pub taunt_cooldown: f32,
    pub sprint_cooldown: f32,
    pub sprint_speed_mult: f32,
    pub sprint_duration: f32,
    pub dodge_chance: f32,
    pub whistle_range: f32,
    pub whistle_cooldown: f32,
}

impl Default for LureConfig {
    fn default() -> Self {
        Self {
            bait_rider_speed: 6.0,
            bait_rider_hp: 100.0,
            bait_rider_damage: 8.0,
            bait_rider_cost_gold: 50.0,
            max_bait_riders_per_mission: 2,
            default_leash_range: 40.0,
            mongol_leash_range: 60.0,
            byzantine_leash_range: 40.0,
            taunt_cooldown: 20.0,
            sprint_cooldown: 30.0,
            sprint_speed_mult: 1.4,
            sprint_duration: 5.0,
            dodge_chance: 0.3,
            whistle_range: 15.0,
            whistle_cooldown: 15.0,
        }
    }
}

/// Ambush scoring and rewards.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct AmbushScoringConfig {
    pub points_per_kill: f32,
    pub points_per_friendly_loss: f32,
    pub bonus_no_detection: f32,
    pub bonus_staged_ambush: f32,
    pub bonus_all_heroes: f32,
    pub bonus_fast_execution: f32,
    pub bonus_time_threshold: f32,
    pub loot_bonus_percent: f32,
}

impl Default for AmbushScoringConfig {
    fn default() -> Self {
        Self {
            points_per_kill: 5.0,
            points_per_friendly_loss: -10.0,
            bonus_no_detection: 30.0,
            bonus_staged_ambush: 20.0,
            bonus_all_heroes: 25.0,
            bonus_fast_execution: 15.0,
            bonus_time_threshold: 30.0,
            loot_bonus_percent: 0.15,
        }
    }
}
