use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::ai::AIDifficulty;
use crate::components::unit::FactionId;

// ─── Difficulty Scaling ─────────────────────────────────────────────────────

/// Per-difficulty scaling multipliers applied to AI opponents.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct AIDifficultyConfig {
    pub easy: DifficultyModifiers,
    pub normal: DifficultyModifiers,
    pub hard: DifficultyModifiers,
    pub expert: DifficultyModifiers,
}

impl Default for AIDifficultyConfig {
    fn default() -> Self {
        Self {
            easy: DifficultyModifiers {
                gather_rate_multiplier: 0.75,
                attack_frequency_multiplier: 0.5,
                decision_interval_multiplier: 1.5,
                attack_damage_multiplier: 0.85,
                build_speed_multiplier: 0.75,
                target_selection_quality: 0.3,
            },
            normal: DifficultyModifiers {
                gather_rate_multiplier: 1.0,
                attack_frequency_multiplier: 1.0,
                decision_interval_multiplier: 1.0,
                attack_damage_multiplier: 1.0,
                build_speed_multiplier: 1.0,
                target_selection_quality: 0.6,
            },
            hard: DifficultyModifiers {
                gather_rate_multiplier: 1.5,
                attack_frequency_multiplier: 1.3,
                decision_interval_multiplier: 0.7,
                attack_damage_multiplier: 1.1,
                build_speed_multiplier: 1.3,
                target_selection_quality: 0.85,
            },
            expert: DifficultyModifiers {
                gather_rate_multiplier: 2.0,
                attack_frequency_multiplier: 1.5,
                decision_interval_multiplier: 0.5,
                attack_damage_multiplier: 1.2,
                build_speed_multiplier: 1.5,
                target_selection_quality: 1.0,
            },
        }
    }
}

impl AIDifficultyConfig {
    pub fn for_difficulty(&self, difficulty: AIDifficulty) -> &DifficultyModifiers {
        match difficulty {
            AIDifficulty::Easy => &self.easy,
            AIDifficulty::Normal => &self.normal,
            AIDifficulty::Hard => &self.hard,
            AIDifficulty::Expert => &self.expert,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyModifiers {
    /// Multiplier for resource gathering rate.
    pub gather_rate_multiplier: f32,
    /// Multiplier for how often the AI attacks (higher = more aggressive).
    pub attack_frequency_multiplier: f32,
    /// Multiplier for decision interval (lower = faster decisions).
    pub decision_interval_multiplier: f32,
    /// Multiplier for outgoing attack damage.
    pub attack_damage_multiplier: f32,
    /// Multiplier for building construction speed.
    pub build_speed_multiplier: f32,
    /// How optimal target selection is (0.0 = random, 1.0 = perfect).
    pub target_selection_quality: f32,
}

// ─── Faction Config ─────────────────────────────────────────────────────────

/// Per-faction configuration including default AI strategy and unit bonuses.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct FactionConfig {
    /// Default AI strategy per faction.
    pub default_strategies: Vec<(FactionId, FactionStrategy)>,
    /// Unit archetype bonuses per faction.
    pub unit_bonuses: Vec<(FactionId, FactionUnitBonus)>,
}

impl Default for FactionConfig {
    fn default() -> Self {
        Self {
            default_strategies: vec![
                (
                    FactionId::Kayi,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Balanced,
                        transition_strategy: None,
                        transition_trigger: StrategyTransitionTrigger::Never,
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::LightInfantry,
                            crate::components::unit::UnitArchetype::Archer,
                        ],
                    },
                ),
                (
                    FactionId::Byzantine,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Turtle,
                        transition_strategy: Some(crate::components::ai::AIStrategy::Boom),
                        transition_trigger: StrategyTransitionTrigger::ArmyThreshold(2000.0),
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::HeavyInfantry,
                            crate::components::unit::UnitArchetype::Archer,
                        ],
                    },
                ),
                (
                    FactionId::Mongol,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Rush,
                        transition_strategy: Some(crate::components::ai::AIStrategy::Guerrilla),
                        transition_trigger: StrategyTransitionTrigger::GameTime(300.0), // 5 min
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::LightCavalry,
                            crate::components::unit::UnitArchetype::Archer,
                        ],
                    },
                ),
                (
                    FactionId::Dodurga,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Balanced,
                        transition_strategy: None,
                        transition_trigger: StrategyTransitionTrigger::Never,
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::LightInfantry,
                        ],
                    },
                ),
                (
                    FactionId::Candar,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Boom,
                        transition_strategy: Some(crate::components::ai::AIStrategy::Balanced),
                        transition_trigger: StrategyTransitionTrigger::ArmyThreshold(1500.0),
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::HeavyInfantry,
                            crate::components::unit::UnitArchetype::HeavyCavalry,
                        ],
                    },
                ),
                (
                    FactionId::Karaman,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Turtle,
                        transition_strategy: None,
                        transition_trigger: StrategyTransitionTrigger::Never,
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::HeavyInfantry,
                        ],
                    },
                ),
                (
                    FactionId::Germiyan,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Balanced,
                        transition_strategy: None,
                        transition_trigger: StrategyTransitionTrigger::Never,
                        preferred_archetypes: vec![
                            crate::components::unit::UnitArchetype::LightCavalry,
                            crate::components::unit::UnitArchetype::Archer,
                        ],
                    },
                ),
                (
                    FactionId::Neutral,
                    FactionStrategy {
                        default_strategy: crate::components::ai::AIStrategy::Balanced,
                        transition_strategy: None,
                        transition_trigger: StrategyTransitionTrigger::Never,
                        preferred_archetypes: vec![],
                    },
                ),
            ],
            unit_bonuses: vec![
                // Kayi: Light Infantry excel, high morale
                (
                    FactionId::Kayi,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.10, // +10% attack
                        armor_bonus: 0.0,
                        speed_bonus: 0.0,
                        morale_bonus: 0.15, // +15% morale
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![(
                            crate::components::unit::UnitArchetype::LightInfantry,
                            ArchetypeBonus {
                                attack_bonus: 0.10,
                                defense_bonus: 0.05,
                                speed_bonus: 0.05,
                            },
                        )],
                    },
                ),
                // Byzantine: Heavy Infantry and Archers, tough defense
                (
                    FactionId::Byzantine,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.15,  // +15% armor
                        speed_bonus: -0.10, // -10% speed (slow but tough)
                        morale_bonus: 0.05,
                        gather_rate_bonus: 0.10, // +10% gather (economy focus)
                        archetype_specific: vec![
                            (
                                crate::components::unit::UnitArchetype::HeavyInfantry,
                                ArchetypeBonus {
                                    attack_bonus: 0.0,
                                    defense_bonus: 0.15,
                                    speed_bonus: -0.10,
                                },
                            ),
                            (
                                crate::components::unit::UnitArchetype::Archer,
                                ArchetypeBonus {
                                    attack_bonus: 0.10, // +10% range damage
                                    defense_bonus: 0.0,
                                    speed_bonus: 0.0,
                                },
                            ),
                        ],
                    },
                ),
                // Mongol: Cavalry and Archers, fast and deadly
                (
                    FactionId::Mongol,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.05,
                        armor_bonus: -0.10, // -10% armor (fragile)
                        speed_bonus: 0.20,  // +20% speed
                        morale_bonus: 0.0,
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![
                            (
                                crate::components::unit::UnitArchetype::LightCavalry,
                                ArchetypeBonus {
                                    attack_bonus: 0.10,
                                    defense_bonus: -0.10,
                                    speed_bonus: 0.20,
                                },
                            ),
                            (
                                crate::components::unit::UnitArchetype::Archer,
                                ArchetypeBonus {
                                    attack_bonus: 0.15, // +15% archer damage
                                    defense_bonus: 0.0,
                                    speed_bonus: 0.0,
                                },
                            ),
                        ],
                    },
                ),
                // Other factions: default bonuses (minimal)
                (
                    FactionId::Dodurga,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.0,
                        speed_bonus: 0.0,
                        morale_bonus: 0.0,
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![],
                    },
                ),
                (
                    FactionId::Candar,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.05,
                        speed_bonus: 0.0,
                        morale_bonus: 0.0,
                        gather_rate_bonus: -0.10,
                        archetype_specific: vec![],
                    },
                ),
                (
                    FactionId::Karaman,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.10,
                        speed_bonus: -0.05,
                        morale_bonus: 0.0,
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![],
                    },
                ),
                (
                    FactionId::Germiyan,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.0,
                        speed_bonus: 0.10,
                        morale_bonus: 0.0,
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![],
                    },
                ),
                (
                    FactionId::Neutral,
                    FactionUnitBonus {
                        attack_damage_bonus: 0.0,
                        armor_bonus: 0.0,
                        speed_bonus: 0.0,
                        morale_bonus: 0.0,
                        gather_rate_bonus: 0.0,
                        archetype_specific: vec![],
                    },
                ),
            ],
        }
    }
}

impl FactionConfig {
    pub fn strategy_for(&self, faction: FactionId) -> Option<&FactionStrategy> {
        self.default_strategies
            .iter()
            .find(|(f, _)| *f == faction)
            .map(|(_, s)| s)
    }

    pub fn bonus_for(&self, faction: FactionId) -> Option<&FactionUnitBonus> {
        self.unit_bonuses
            .iter()
            .find(|(f, _)| *f == faction)
            .map(|(_, b)| b)
    }
}

// ─── Faction Strategy ───────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionStrategy {
    /// The default AI strategy for this faction.
    pub default_strategy: crate::components::ai::AIStrategy,
    /// Optional strategy to transition to when trigger condition is met.
    pub transition_strategy: Option<crate::components::ai::AIStrategy>,
    /// What triggers the strategy transition.
    pub transition_trigger: StrategyTransitionTrigger,
    /// Preferred unit archetypes for recruitment.
    pub preferred_archetypes: Vec<crate::components::unit::UnitArchetype>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StrategyTransitionTrigger {
    Never,
    /// Transition after a certain total army strength is reached.
    ArmyThreshold(f32),
    /// Transition after a certain game time in seconds.
    GameTime(f32),
}

// ─── Faction Unit Bonus ─────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FactionUnitBonus {
    /// Global faction-wide bonuses applied to all units.
    pub attack_damage_bonus: f32,
    pub armor_bonus: f32,
    pub speed_bonus: f32,
    pub morale_bonus: f32,
    pub gather_rate_bonus: f32,
    /// Archetype-specific bonuses.
    pub archetype_specific: Vec<(crate::components::unit::UnitArchetype, ArchetypeBonus)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeBonus {
    pub attack_bonus: f32,
    pub defense_bonus: f32,
    pub speed_bonus: f32,
}
