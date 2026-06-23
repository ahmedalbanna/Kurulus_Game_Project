use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Succession/Legacy System ────────────────────────────────────────────────

/// Calculated legacy score for a hero (accumulated over the campaign).
#[derive(Component, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct LegacyScore {
    pub total: f32,
    pub battles_won: u32,
    pub regions_conquered: u32,
    pub alliances_formed: u32,
    pub children_raised: u32,
    pub years_ruled: f32,
}

impl Default for LegacyScore {
    fn default() -> Self {
        Self {
            total: 0.0,
            battles_won: 0,
            regions_conquered: 0,
            alliances_formed: 0,
            children_raised: 0,
            years_ruled: 0.0,
        }
    }
}

/// Resource ordering heroes by succession priority.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct SuccessionOrder {
    /// Ordered list of heirs (index 0 = next ruler).
    pub heirs: Vec<Entity>,
}

/// A node in the dynasty tree.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct DynastyNode {
    pub hero: Entity,
    pub generation: u32,
    pub parent: Option<Entity>,
    pub spouses: Vec<Entity>,
    pub children: Vec<Entity>,
}

/// Resource managing the dynasty tree across generations.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct DynastyTree {
    pub nodes: HashMap<Entity, DynastyNode>,
    pub root: Option<Entity>,
}

/// Event: a new ruler ascends to leadership.
#[derive(Event, Clone, Debug)]
pub struct RulerAscended {
    pub new_ruler: Entity,
    pub previous_ruler: Option<Entity>,
    pub generation: u32,
}

/// Event: legacy points were awarded.
#[derive(Event, Clone, Debug)]
pub struct LegacyAwarded {
    pub hero: Entity,
    pub amount: f32,
    pub reason: String,
}

// ─── Phase 4 Enhancements ───────────────────────────────────────────────────

/// Osman's testament choice (M47).
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct TestamentChoices {
    pub military: Option<TestamentOption>,
    pub political: Option<TestamentOption>,
    pub religious: Option<TestamentOption>,
    pub final_word: Option<TestamentOption>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum TestamentOption {
    A,
    B,
}

/// Beylik faction loyalty tracker (M48).
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct BeylikLoyalty {
    pub factions: HashMap<String, f32>,
}

impl Default for BeylikLoyalty {
    fn default() -> Self {
        Self {
            factions: HashMap::from([
                ("kayi_army".into(), 90.0),
                ("yeni_sehir_scholars".into(), 85.0),
                ("bursa_merchants".into(), 70.0),
                ("frontier_princes".into(), 50.0),
                ("turkmen_tribes".into(), 40.0),
            ]),
        }
    }
}

/// Rebellion state tracker (M48).
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct RebellionState {
    pub active_rebellions: Vec<String>,
    pub warning_factions: Vec<String>,
}

/// Dynasty ending type (M49).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum DynastyEnding {
    EmpireOfJustice,
    HolyEmpire,
    EmpireOfScience,
    EmpireOfPeoples,
    SecretLegendary,
}

/// Event: a testament choice was made.
#[derive(Event, Clone, Debug)]
pub struct TestamentChoiceMade {
    pub testament_type: String,
    pub choice: TestamentOption,
}

/// Event: all four testaments have been chosen (M47 complete).
#[derive(Event, Clone, Debug)]
pub struct AllTestamentsChosen {
    pub military: TestamentOption,
    pub political: TestamentOption,
    pub religious: TestamentOption,
    pub final_word: TestamentOption,
}

/// Event: Beylik council priority chosen (M48).
#[derive(Event, Clone, Debug)]
pub struct BeylikCouncilChosen {
    pub priorities: Vec<String>,
}

/// Event: the dynasty ending has been determined (M49).
#[derive(Event, Clone, Debug)]
pub struct DynastyEndingChosen {
    pub ending: DynastyEnding,
}
