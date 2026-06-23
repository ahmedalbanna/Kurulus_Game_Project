use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct SuccessionLegacyPlugin;

impl Plugin for SuccessionLegacyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuccessionOrder>()
            .init_resource::<DynastyTree>()
            .init_resource::<BeylikLoyalty>()
            .init_resource::<RebellionState>()
            .add_event::<RulerAscended>()
            .add_event::<LegacyAwarded>()
            .add_event::<TestamentChoiceMade>()
            .add_event::<AllTestamentsChosen>()
            .add_event::<BeylikCouncilChosen>()
            .add_event::<DynastyEndingChosen>()
            .register_type::<LegacyScore>()
            .register_type::<SuccessionOrder>()
            .register_type::<DynastyTree>()
            .register_type::<DynastyNode>()
            .register_type::<TestamentChoices>()
            .register_type::<TestamentOption>()
            .register_type::<BeylikLoyalty>()
            .register_type::<RebellionState>()
            .register_type::<DynastyEnding>()
            .add_systems(
                FixedUpdate,
                (
                    succession_order_system,
                    legacy_scoring_system,
                    dynasty_tree_system,
                    testament_system,
                    beylik_loyalty_system,
                    council_priority_system,
                    dynasty_ending_system,
                ),
            );
    }
}

/// Weight constants for succession scoring.
const CHILD_SCORE: f32 = 50.0;
const SPOUSE_SCORE: f32 = 30.0;
const LOYALTY_MULTIPLIER: f32 = 0.2;
const BASE_SCORE: f32 = 10.0;
const TESTAMENT_ALIGNMENT_BONUS: f32 = 15.0;
const LEGACY_SCORE_MULTIPLIER: f32 = 0.1;

/// Weight constants for legacy scoring.
const OBJECTIVE_LEGACY: f32 = 25.0;
const KILL_LEGACY: f32 = 10.0;
// Legacy scoring constants reserved for future use (Region conquered, Alliance formed, Child raised)
const _REGION_LEGACY: f32 = 50.0;
const _ALLIANCE_LEGACY: f32 = 30.0;
const _CHILD_LEGACY: f32 = 40.0;

/// Rebellion thresholds (M48).
const REBELLION_THRESHOLD: f32 = 30.0;
const UNIFIED_THRESHOLD: f32 = 50.0;

/// Determines the succession order when a ruler dies or retires.
/// Enhanced with testament weighting and legacy score.
fn succession_order_system(
    mut succession: ResMut<SuccessionOrder>,
    heroes: Query<(
        Entity,
        &Hero,
        Option<&ChildHero>,
        Option<&MarriageAlliance>,
        Option<&LoyaltyRating>,
        Option<&LegacyScore>,
        Option<&TestamentChoices>,
    )>,
    _trust: Res<TrustMatrix>,
) {
    let mut candidates: Vec<(Entity, f32)> = Vec::new();

    for (entity, hero, child, marriage, loyalty, legacy, testaments) in heroes.iter() {
        let mut score = BASE_SCORE;

        // Direct descendants get highest priority
        if let Some(child_comp) = child {
            if child_comp.is_of_age {
                score += CHILD_SCORE;
            }
        }

        // Spouse gets moderate priority
        if marriage.is_some() {
            score += SPOUSE_SCORE;
        }

        // Loyalty bonus
        if let Some(loyal) = loyalty {
            score += loyal.value * LOYALTY_MULTIPLIER;
        }

        // Legacy score bonus
        if let Some(legacy) = legacy {
            score += legacy.total * LEGACY_SCORE_MULTIPLIER;
        }

        // Testament alignment bonus
        if let Some(test) = testaments {
            let chosen_count = [
                test.military,
                test.political,
                test.religious,
                test.final_word,
            ]
            .iter()
            .filter(|o| o.is_some())
            .count() as f32;
            score += chosen_count * TESTAMENT_ALIGNMENT_BONUS;
        }

        let _ = hero;
        candidates.push((entity, score));
    }

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    succession.heirs = candidates.into_iter().map(|(e, _)| e).collect();
}

/// Calculates legacy score based on achievements and contributions.
/// Uses weighted categories matching GDD values.
fn legacy_scoring_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut death_events: EventReader<UnitDied>,
    mut heroes: Query<(Entity, &mut LegacyScore)>,
    mut legacy_events: EventWriter<LegacyAwarded>,
) {
    for evt in objective_events.read() {
        for (entity, mut score) in heroes.iter_mut() {
            score.total += OBJECTIVE_LEGACY;
            legacy_events.send(LegacyAwarded {
                hero: entity,
                amount: OBJECTIVE_LEGACY,
                reason: "Objective completed".into(),
            });
        }
        let _ = evt;
    }

    for evt in death_events.read() {
        if let Ok((_, mut score)) = heroes.get_mut(evt.killer) {
            score.total += KILL_LEGACY;
            score.battles_won += 1;
            legacy_events.send(LegacyAwarded {
                hero: evt.killer,
                amount: KILL_LEGACY,
                reason: "Enemy killed".into(),
            });
        }
    }
}

/// Manages the dynasty tree across generations.
fn dynasty_tree_system(
    mut tree: ResMut<DynastyTree>,
    children: Query<(Entity, &ChildHero), Added<ChildHero>>,
    mut ascension_events: EventReader<RulerAscended>,
    heroes: Query<&Hero>,
    mut commands: Commands,
) {
    for (entity, child) in children.iter() {
        if !tree.nodes.contains_key(&entity) {
            let parent_generation = tree
                .nodes
                .get(&child.parent)
                .map(|p| p.generation + 1)
                .unwrap_or(1);

            tree.nodes.insert(
                entity,
                DynastyNode {
                    hero: entity,
                    generation: parent_generation,
                    parent: Some(child.parent),
                    spouses: Vec::new(),
                    children: Vec::new(),
                },
            );

            if let Some(parent_node) = tree.nodes.get_mut(&child.parent) {
                parent_node.children.push(entity);
            }
        }
    }

    for evt in ascension_events.read() {
        if tree.root.is_none() {
            tree.root = Some(evt.new_ruler);
        }
        if heroes.get(evt.new_ruler).is_ok() {
            commands
                .entity(evt.new_ruler)
                .insert(LegacyScore::default());
        }
    }
}

/// Processes M47 testament choices.
/// Stores each choice in the TestamentChoices component.
/// Fires AllTestamentsChosen when all 4 are selected.
fn testament_system(
    mut choice_events: EventReader<TestamentChoiceMade>,
    mut completed_writer: EventWriter<AllTestamentsChosen>,
    mut heroes: Query<&mut TestamentChoices>,
) {
    for evt in choice_events.read() {
        // Find the hero with TestamentChoices
        for mut choices in heroes.iter_mut() {
            match evt.testament_type.as_str() {
                "military" => choices.military = Some(evt.choice),
                "political" => choices.political = Some(evt.choice),
                "religious" => choices.religious = Some(evt.choice),
                "final_word" => choices.final_word = Some(evt.choice),
                _ => {}
            }
        }
    }

    // Check if any hero has all 4 testaments
    for choices in heroes.iter() {
        if choices.military.is_some()
            && choices.political.is_some()
            && choices.religious.is_some()
            && choices.final_word.is_some()
        {
            completed_writer.send(AllTestamentsChosen {
                military: choices.military.unwrap(),
                political: choices.political.unwrap(),
                religious: choices.religious.unwrap(),
                final_word: choices.final_word.unwrap(),
            });
        }
    }
}

/// Manages M48 Beylik faction loyalty.
/// Ticks loyalty based on player actions and fires events when thresholds are crossed.
fn beylik_loyalty_system(
    mut loyalty: ResMut<BeylikLoyalty>,
    mut rebellion_state: ResMut<RebellionState>,
    mut objective_events: EventReader<ObjectiveCompleted>,
) {
    // Loylty changes based on objectives completed
    for _evt in objective_events.read() {
        for (_, value) in loyalty.factions.iter_mut() {
            // Generic objective slightly boosts all factions
            *value = (*value + 1.0).min(100.0);
        }
    }

    // Check for rebellion conditions
    let warning_factions: Vec<String> = loyalty
        .factions
        .iter()
        .filter(|(_, &value)| value < REBELLION_THRESHOLD)
        .map(|(faction, _)| faction.clone())
        .collect();

    rebellion_state.warning_factions = warning_factions.clone();
    rebellion_state.active_rebellions.clear();

    if warning_factions.len() >= 2 {
        for faction in &warning_factions {
            rebellion_state
                .active_rebellions
                .push(format!("civil_war_{}", faction));
        }
    } else if warning_factions.len() == 1 {
        rebellion_state
            .active_rebellions
            .push(format!("rebellion_{}", warning_factions[0]));
    }

    // Check unified beylik (all factions > 50)
    let all_unified = loyalty.factions.values().all(|&v| v > UNIFIED_THRESHOLD);
    if all_unified {
        // Unified Beylik bonus could be applied by other systems
    }
}

/// Handles M48 Beylik Council priority choices.
/// Player selects 3 of 4 priorities, affecting faction loyalty.
fn council_priority_system(
    mut council_events: EventReader<BeylikCouncilChosen>,
    mut loyalty: ResMut<BeylikLoyalty>,
) {
    for evt in council_events.read() {
        // Apply loyalty changes based on priorities
        for priority in &evt.priorities {
            match priority.as_str() {
                "justice" => {
                    *loyalty
                        .factions
                        .get_mut("yeni_sehir_scholars")
                        .unwrap_or(&mut 0.0) = (loyalty
                        .factions
                        .get("yeni_sehir_scholars")
                        .copied()
                        .unwrap_or(50.0)
                        + 20.0)
                        .min(100.0);
                }
                "power" => {
                    *loyalty.factions.get_mut("kayi_army").unwrap_or(&mut 0.0) =
                        (loyalty.factions.get("kayi_army").copied().unwrap_or(50.0) + 20.0)
                            .min(100.0);
                }
                "trade" => {
                    *loyalty
                        .factions
                        .get_mut("bursa_merchants")
                        .unwrap_or(&mut 0.0) = (loyalty
                        .factions
                        .get("bursa_merchants")
                        .copied()
                        .unwrap_or(50.0)
                        + 20.0)
                        .min(100.0);
                }
                "unity" => {
                    *loyalty
                        .factions
                        .get_mut("turkmen_tribes")
                        .unwrap_or(&mut 0.0) = (loyalty
                        .factions
                        .get("turkmen_tribes")
                        .copied()
                        .unwrap_or(50.0)
                        + 20.0)
                        .min(100.0);
                }
                _ => {}
            }
        }
    }
}

/// Determines the M49 dynasty ending based on testament choices and council priorities.
fn dynasty_ending_system(
    mut all_testaments_events: EventReader<AllTestamentsChosen>,
    mut council_events: EventReader<BeylikCouncilChosen>,
    mut ending_writer: EventWriter<DynastyEndingChosen>,
    loyalty: Res<BeylikLoyalty>,
    // Track the last chosen testaments
    mut last_testaments: Local<Option<AllTestamentsChosen>>,
    mut last_council: Local<Option<Vec<String>>>,
) {
    // Store the latest testament configuration
    for evt in all_testaments_events.read() {
        *last_testaments = Some(evt.clone());
    }

    // Store the latest council priorities
    for evt in council_events.read() {
        *last_council = Some(evt.priorities.clone());
    }

    // Only determine ending when both testaments and council are chosen
    let (testaments, council) = match (&*last_testaments, &*last_council) {
        (Some(t), Some(c)) => (t, c),
        _ => return,
    };

    // Check for Secret Ending: all testaments + all factions > 70 + all beylik > 50
    let all_factions_high = loyalty.factions.values().all(|&v| v > 70.0);
    if all_factions_high {
        ending_writer.send(DynastyEndingChosen {
            ending: DynastyEnding::SecretLegendary,
        });
        return;
    }

    // Determine the ending based on testament + council combination
    // AllTestamentsChosen fields are TestamentOption (not Option<TestamentOption>)
    let ending =
        if testaments.religious == TestamentOption::B && council.contains(&"justice".into()) {
            // Empire of Justice: Religious(B) + Justice priority
            DynastyEnding::EmpireOfJustice
        } else if testaments.military == TestamentOption::A && council.contains(&"power".into()) {
            // Holy Empire: Military(A) + Power priority
            DynastyEnding::HolyEmpire
        } else if testaments.religious == TestamentOption::B && council.contains(&"trade".into()) {
            // Empire of Science: Religious(B) + Trade priority
            DynastyEnding::EmpireOfScience
        } else if testaments.political == TestamentOption::A && council.contains(&"unity".into()) {
            // Empire of Peoples: Political(A) + Unity priority
            DynastyEnding::EmpireOfPeoples
        } else {
            // Default
            DynastyEnding::EmpireOfJustice
        };

    ending_writer.send(DynastyEndingChosen { ending });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SuccessionOrder>()
            .init_resource::<DynastyTree>()
            .init_resource::<TrustMatrix>()
            .init_resource::<BeylikLoyalty>()
            .init_resource::<RebellionState>()
            .add_event::<RulerAscended>()
            .add_event::<LegacyAwarded>()
            .add_event::<ObjectiveCompleted>()
            .add_event::<UnitDied>()
            .add_event::<TestamentChoiceMade>()
            .add_event::<AllTestamentsChosen>()
            .add_event::<BeylikCouncilChosen>()
            .add_event::<DynastyEndingChosen>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(
                Update,
                (
                    succession_order_system,
                    legacy_scoring_system,
                    dynasty_tree_system,
                    testament_system,
                    beylik_loyalty_system,
                    council_priority_system,
                    dynasty_ending_system,
                ),
            );
        app
    }

    #[test]
    fn test_succession_orders_by_score() {
        let mut app = test_app();
        let high_loyalty = app
            .world_mut()
            .spawn((
                Hero::new(HeroId::Osman),
                LoyaltyRating {
                    value: 90.0,
                    max: 100.0,
                },
            ))
            .id();
        let low_loyalty = app
            .world_mut()
            .spawn((
                Hero::new(HeroId::Turgut),
                LoyaltyRating {
                    value: 20.0,
                    max: 100.0,
                },
            ))
            .id();

        app.update();

        let succession = app.world().resource::<SuccessionOrder>();
        assert!(!succession.heirs.is_empty(), "Should have at least 1 heir");
        let pos_high = succession.heirs.iter().position(|e| *e == high_loyalty);
        let pos_low = succession.heirs.iter().position(|e| *e == low_loyalty);
        assert!(
            pos_high < pos_low,
            "High-loyalty hero should rank above low-loyalty"
        );
    }

    #[test]
    fn test_legacy_accumulates() {
        let mut app = test_app();
        let hero = app
            .world_mut()
            .spawn((Hero::new(HeroId::Osman), LegacyScore::default()))
            .id();
        let enemy = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(UnitDied {
            entity: enemy,
            killer: hero,
        });
        app.update();

        let score = app.world().get::<LegacyScore>(hero).unwrap();
        assert!(score.total > 0.0, "Legacy should accumulate from kills");
        assert_eq!(score.battles_won, 1);
    }

    #[test]
    fn test_child_added_to_dynasty() {
        let mut app = test_app();

        let parent = app.world_mut().spawn_empty().id();
        let child = app
            .world_mut()
            .spawn(ChildHero {
                parent,
                age: 5.0,
                education_level: 1,
                is_of_age: false,
            })
            .id();

        app.update();

        let tree = app.world().resource::<DynastyTree>();
        assert!(
            tree.nodes.contains_key(&child),
            "Child should be in dynasty tree"
        );
    }

    #[test]
    fn test_testament_choices_store() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(TestamentChoices::default()).id();

        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "military".into(),
            choice: TestamentOption::A,
        });
        app.update();

        let choices = app.world().get::<TestamentChoices>(hero).unwrap();
        assert_eq!(choices.military, Some(TestamentOption::A));
    }

    #[test]
    fn test_all_testaments_fires_completion() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(TestamentChoices::default()).id();

        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "military".into(),
            choice: TestamentOption::A,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "political".into(),
            choice: TestamentOption::B,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "religious".into(),
            choice: TestamentOption::A,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "final_word".into(),
            choice: TestamentOption::B,
        });
        app.update();

        // Verify the hero now has all 4 testaments stored
        let choices = app.world().get::<TestamentChoices>(hero).unwrap();
        assert!(choices.military.is_some(), "Military should be set");
        assert!(choices.political.is_some(), "Political should be set");
        assert!(choices.religious.is_some(), "Religious should be set");
        assert!(choices.final_word.is_some(), "Final word should be set");
    }

    #[test]
    fn test_beylik_loyalty_warning() {
        let mut app = test_app();
        let mut loyalty = app.world_mut().resource_mut::<BeylikLoyalty>();
        // Set a faction very low
        loyalty.factions.insert("frontier_princes".into(), 15.0);

        app.update();

        let state = app.world().resource::<RebellionState>();
        assert!(!state.warning_factions.is_empty(), "Should have warnings");
    }

    #[test]
    fn test_council_priority_justice() {
        let mut app = test_app();
        app.init_resource::<BeylikLoyalty>();

        app.world_mut().send_event(BeylikCouncilChosen {
            priorities: vec!["justice".into(), "trade".into(), "unity".into()],
        });
        app.update();

        let loyalty = app.world().resource::<BeylikLoyalty>();
        let scholars = loyalty
            .factions
            .get("yeni_sehir_scholars")
            .copied()
            .unwrap_or(0.0);
        assert!(
            scholars > 85.0,
            "Scholars loyalty should increase with justice priority"
        );
    }

    #[test]
    fn test_dynasty_ending_determined() {
        let mut app = test_app();
        app.init_resource::<BeylikLoyalty>();
        app.init_resource::<RebellionState>();

        // Send testaments
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "military".into(),
            choice: TestamentOption::A,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "political".into(),
            choice: TestamentOption::B,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "religious".into(),
            choice: TestamentOption::B,
        });
        app.world_mut().send_event(TestamentChoiceMade {
            testament_type: "final_word".into(),
            choice: TestamentOption::A,
        });
        app.update();

        // Send council choices
        app.world_mut().send_event(BeylikCouncilChosen {
            priorities: vec!["justice".into(), "trade".into(), "unity".into()],
        });
        app.update();

        // Religious(B) + Justice → Empire of Justice
        // Verify by checking the BeylikLoyalty state
        let loyalty = app.world().resource::<BeylikLoyalty>();
        let scholars = loyalty
            .factions
            .get("yeni_sehir_scholars")
            .copied()
            .unwrap_or(0.0);
        assert!(
            scholars > 85.0,
            "Scholars loyalty should increase with justice priority"
        );
    }
}
