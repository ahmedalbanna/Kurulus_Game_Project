use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── NextGen Heroes System ───────────────────────────────────────────────────

/// Marks a hero as a "child" of another hero (next generation).
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct ChildHero {
    pub parent: Entity,
    pub age: f32,
    pub education_level: u32,
    pub is_of_age: bool,
}

/// Tracks a marriage alliance between two factions/heroes.
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct MarriageAlliance {
    pub spouse: Entity,
    pub spouse_faction: crate::components::FactionId,
    pub active: bool,
    pub duration: f32,
}

/// Resource tracking all next-gen heroes across the campaign.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct NextGenRegistry {
    pub children: Vec<(Entity, Entity)>, // (parent, child)
    pub marriages: Vec<(Entity, Entity)>, // (hero_a, hero_b)
}

/// Resource tracking the legacy impact of each hero.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct LegacyContributions {
    pub contributions: Vec<(Entity, String, f32)>, // (entity, category, value)
}

/// Event: a child hero comes of age and becomes playable.
#[derive(Event, Clone, Debug)]
pub struct ChildOfAge {
    pub child: Entity,
    pub parent: Entity,
}

/// Event: a marriage alliance is formed.
#[derive(Event, Clone, Debug)]
pub struct MarriageFormed {
    pub hero_a: Entity,
    pub hero_b: Entity,
    pub faction_a: crate::components::FactionId,
    pub faction_b: crate::components::FactionId,
}
