use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── Romance/Loyalty System ──────────────────────────────────────────────────

/// Tracks a hero's loyalty rating (0-100). Below 30 risks betrayal/desertion.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct LoyaltyRating {
    pub value: f32,
    pub max: f32,
}

impl Default for LoyaltyRating {
    fn default() -> Self {
        Self {
            value: 50.0,
            max: 100.0,
        }
    }
}

/// Tracks a romantic relationship between two characters.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct RomanceLink {
    pub partner: Entity,
    pub stage: RomanceStage,
    pub affinity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum RomanceStage {
    Acquaintance,
    Friendship,
    Romance,
    Betrothal,
    Married,
}

impl Default for RomanceStage {
    fn default() -> Self {
        Self::Acquaintance
    }
}

/// Modifies loyalty based on events (used to compute loyalty changes).
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct LoyaltyModifier {
    pub reason: String,
    pub amount: f32,
    pub duration: Option<f32>,
}

/// Tracks active loyalty modifiers on a hero.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ActiveLoyaltyModifiers {
    pub modifiers: Vec<LoyaltyModifier>,
}

/// Resource storing all active romance links across the campaign.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct RomanceRegistry {
    pub links: Vec<(Entity, Entity, RomanceStage)>,
}

/// Event: loyalty changed significantly.
#[derive(Event, Clone, Debug)]
pub struct LoyaltyChanged {
    pub hero: Entity,
    pub old_value: f32,
    pub new_value: f32,
    pub reason: String,
}

/// Event: romance stage advanced.
#[derive(Event, Clone, Debug)]
pub struct RomanceStageAdvanced {
    pub hero_a: Entity,
    pub hero_b: Entity,
    pub new_stage: RomanceStage,
}

// ─── Phase 4 Enhancements ───────────────────────────────────────────────────

/// Predefined relationship between two characters (GDD §15).
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct RelationshipPair {
    pub hero_a: Entity,
    pub hero_b: Entity,
    pub relationship_type: RelationshipType,
    pub starting_value: f32,
    pub current_value: f32,
    pub max_value: f32,
    pub events_completed: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum RelationshipType {
    Romantic,
    Brotherhood,
    Rivalry,
    Mentorship,
    Political,
}

/// Love impact tier (GDD §15.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum LoveImpactTier {
    /// >80: +20% morale, special abilities
    Inspired,
    /// 50-80: no effect
    Normal,
    /// <50: -10% morale, -5% damage
    Distracted,
}

/// Special couple ability (unlocked at high love).
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct CoupleAbility {
    pub ability_id: String,
    pub required_tier: LoveImpactTier,
    pub uses_per_mission: u32,
    pub remaining_uses: u32,
}

/// Tracks choice impacts on relationships (mission decisions).
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ChoiceImpactRecord {
    pub mission_id: String,
    pub choice_id: String,
    pub affected_pairs: Vec<(Entity, f32)>,
}

/// Resource storing the pre-defined relationship matrix for the campaign.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct RelationshipMatrix {
    pub pairs: Vec<(Entity, Entity, RelationshipType, f32)>,
}
