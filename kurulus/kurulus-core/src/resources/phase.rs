use crate::components::{HeroId, MissionId};
use crate::resources::ResourceStockpile;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// Difficulty Configuration
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum DifficultyLevel {
    Story,
    Normal,
    Hard,
    Epic,
}

impl Default for DifficultyLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl DifficultyLevel {
    /// Multiplier applied to incoming damage (higher = enemies deal more).
    pub fn incoming_damage_mult(&self) -> f32 {
        match self {
            DifficultyLevel::Story => 0.6,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.3,
            DifficultyLevel::Epic => 1.6,
        }
    }

    /// Multiplier applied to outgoing damage (higher = player deals more).
    pub fn outgoing_damage_mult(&self) -> f32 {
        match self {
            DifficultyLevel::Story => 1.4,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 0.8,
            DifficultyLevel::Epic => 0.65,
        }
    }

    /// Multiplier applied to enemy resource production.
    pub fn enemy_resource_mult(&self) -> f32 {
        match self {
            DifficultyLevel::Story => 0.7,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.3,
            DifficultyLevel::Epic => 1.6,
        }
    }

    /// AI reaction speed multiplier (higher = faster decisions).
    pub fn ai_reaction_mult(&self) -> f32 {
        match self {
            DifficultyLevel::Story => 0.6,
            DifficultyLevel::Normal => 1.0,
            DifficultyLevel::Hard => 1.3,
            DifficultyLevel::Epic => 1.6,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DifficultyLevel::Story => "Story",
            DifficultyLevel::Normal => "Normal",
            DifficultyLevel::Hard => "Hard",
            DifficultyLevel::Epic => "Epic",
        }
    }
}

/// Global difficulty configuration, set before a mission starts.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct DifficultyConfig {
    pub level: DifficultyLevel,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            level: DifficultyLevel::Normal,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Harvest State
// ═══════════════════════════════════════════════════════════════════════════

/// Tracks the harvest calculation process for the current mission.
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct HarvestState {
    /// Whether harvest calculation is in progress.
    pub is_calculating: bool,
    /// Whether the harvest has been calculated for this cycle.
    pub calculated: bool,
    /// Base bonus multiplier for resource yields (default 1.0).
    pub resource_bonus_mult: f32,
    /// SP multiplier from difficulty and hero bonuses.
    pub sp_multiplier: f32,
    /// Accumulated resource loot from the execution phase.
    pub accumulated_loot: ResourceStockpile,
    /// Accumulated XP for distribution during harvest.
    pub accumulated_xp: f32,
    /// Number of primary objectives completed this mission.
    pub primary_objectives_completed: u32,
    /// Number of secondary objectives completed.
    pub secondary_objectives_completed: u32,
    /// Number of hidden objectives discovered and completed.
    pub hidden_objectives_completed: u32,
    /// Total enemy kills during the Execution phase (used for loot calc).
    pub total_kills: u32,
}

impl Default for HarvestState {
    fn default() -> Self {
        Self {
            is_calculating: false,
            calculated: false,
            resource_bonus_mult: 1.0,
            sp_multiplier: 1.0,
            accumulated_loot: ResourceStockpile::default(),
            accumulated_xp: 0.0,
            primary_objectives_completed: 0,
            secondary_objectives_completed: 0,
            hidden_objectives_completed: 0,
            total_kills: 0,
        }
    }
}

/// Data-driven loot table for harvest calculations.
/// These values can be overridden per-mission via RON config.
#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub struct LootTable {
    /// Base food loot from clearing a standard enemy unit.
    pub food_per_enemy_kill: f32,
    /// Base gold loot from clearing a standard enemy unit.
    pub gold_per_enemy_kill: f32,
    /// Base wood loot from clearing a standard enemy unit.
    pub wood_per_enemy_kill: f32,
    /// Base stone loot from clearing a standard enemy unit.
    pub stone_per_enemy_kill: f32,
    /// Bonus food per friendly unit that survived.
    pub food_per_survivor: f32,
    /// Bonus gold per friendly unit that survived.
    pub gold_per_survivor: f32,
    /// SP awarded per primary objective completed.
    pub sp_per_primary_objective: u32,
    /// SP awarded per secondary objective completed.
    pub sp_per_secondary_objective: u32,
    /// SP awarded per hidden objective completed.
    pub sp_per_hidden_objective: u32,
    /// Base XP awarded per enemy kill.
    pub xp_per_enemy_kill: f32,
    /// XP bonus for hero moments completed.
    pub xp_per_hero_moment: f32,
    /// Harvest duration multiplier (1.0 = default harvest duration).
    pub duration_multiplier: f32,
}

impl Default for LootTable {
    fn default() -> Self {
        Self {
            food_per_enemy_kill: 15.0,
            gold_per_enemy_kill: 8.0,
            wood_per_enemy_kill: 5.0,
            stone_per_enemy_kill: 3.0,
            food_per_survivor: 5.0,
            gold_per_survivor: 3.0,
            sp_per_primary_objective: 3,
            sp_per_secondary_objective: 1,
            sp_per_hidden_objective: 1,
            xp_per_enemy_kill: 25.0,
            xp_per_hero_moment: 75.0,
            duration_multiplier: 1.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Pre-Mission Configuration
// ═══════════════════════════════════════════════════════════════════════════

/// Configuration set before a mission begins (hero selection, difficulty, etc.).
#[derive(Resource, Clone, Debug)]
pub struct PreMissionConfig {
    pub selected_heroes: Vec<HeroId>,
    pub difficulty: DifficultyLevel,
    pub mission_id: MissionId,
}

impl Default for PreMissionConfig {
    fn default() -> Self {
        Self {
            selected_heroes: Vec::new(),
            difficulty: DifficultyLevel::Normal,
            mission_id: MissionId::Mission01,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Save System
// ═══════════════════════════════════════════════════════════════════════════

/// Manages save slots and auto-save/checkpoint state.
#[derive(Resource, Clone, Debug)]
pub struct SaveSystem {
    pub auto_save_enabled: bool,
    pub manual_save_slots: Vec<Option<SaveSlot>>,
    pub checkpoint_data: Option<SaveSlot>,
    /// Game-time seconds of the last auto-save.
    pub last_auto_save_time: f64,
    /// Minimum interval between auto-saves in game-seconds.
    pub auto_save_interval: f64,
}

impl Default for SaveSystem {
    fn default() -> Self {
        Self {
            auto_save_enabled: true,
            // 10 manual save slots + 1 auto
            manual_save_slots: vec![None; 10],
            checkpoint_data: None,
            last_auto_save_time: 0.0,
            auto_save_interval: 300.0, // every 5 game-minutes
        }
    }
}

/// A single save slot containing all data needed to restore state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveSlot {
    /// Serialized campaign state (JSON string).
    pub campaign_state_json: String,
    pub hero_states: Vec<HeroSaveState>,
    pub resource_stockpile: ResourceStockpile,
    pub tech_unlocks: Vec<String>,
    pub narrative_choices: Vec<String>,
    pub timestamp: f64,
    pub mission_id: MissionId,
}

/// Minimal hero state for save/load.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroSaveState {
    pub hero_id: HeroId,
    pub level: u32,
    pub experience: f32,
    pub unlocked_abilities: Vec<String>,
    pub equipment: Vec<String>,
    pub health: f32,
    pub position_x: i32,
    pub position_y: i32,
}

// ═══════════════════════════════════════════════════════════════════════════
// Hero Moment Definition (stored as a config resource)
// ═══════════════════════════════════════════════════════════════════════════

/// Defines a triggered Hero Moment for a specific mission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroMomentConfig {
    pub hero_id: HeroId,
    pub ability_id: crate::components::AbilityId,
    pub mission_id: MissionId,
    pub trigger_phase: Option<crate::states::TacticalPhase>,
    pub description: String,
    pub bonus_xp: f32,
}

/// Resource containing all Hero Moment definitions for the current mission.
#[derive(Resource, Clone, Debug, Default)]
pub struct HeroMomentsForMission {
    pub moments: Vec<HeroMomentConfig>,
}
