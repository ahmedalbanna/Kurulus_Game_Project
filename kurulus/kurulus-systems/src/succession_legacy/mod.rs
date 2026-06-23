use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct SuccessionLegacyPlugin;

impl Plugin for SuccessionLegacyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SuccessionOrder>()
            .init_resource::<DynastyTree>()
            .add_event::<RulerAscended>()
            .add_event::<LegacyAwarded>()
            .register_type::<LegacyScore>()
            .register_type::<SuccessionOrder>()
            .register_type::<DynastyTree>()
            .register_type::<DynastyNode>()
            .add_systems(
                FixedUpdate,
                (
                    succession_order_system,
                    legacy_scoring_system,
                    dynasty_tree_system,
                ),
            );
    }
}

/// Determines the succession order when a ruler dies or retires.
/// Priority: eldest child → spouse → sibling → most loyal hero.
fn succession_order_system(
    mut succession: ResMut<SuccessionOrder>,
    heroes: Query<(Entity, &Hero, Option<&ChildHero>, Option<&MarriageAlliance>, Option<&LoyaltyRating>)>,
    _trust: Res<TrustMatrix>,
) {
    // Score each hero for succession priority
    let mut candidates: Vec<(Entity, f32)> = Vec::new();

    for (entity, hero, child, marriage, loyalty) in heroes.iter() {
        let mut score = 0.0;

        // Direct descendants get highest priority
        if let Some(child_comp) = child {
            if child_comp.is_of_age {
                score += 50.0;
            }
        }

        // Spouse gets moderate priority
        if let Some(_marriage) = marriage {
            score += 30.0;
        }

        // Loyalty bonus
        if let Some(loyal) = loyalty {
            score += loyal.value * 0.2;
        }

        // Base score from rank/importance (placeholder: all heroes get base)
        score += 10.0;

        let _ = hero; // ensure usage

        candidates.push((entity, score));
    }

    // Sort by score descending
    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    succession.heirs = candidates.into_iter().map(|(e, _)| e).collect();
}

/// Calculates legacy score based on achievements and contributions.
fn legacy_scoring_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut death_events: EventReader<UnitDied>,
    mut heroes: Query<(Entity, &mut LegacyScore)>,
    mut legacy_events: EventWriter<LegacyAwarded>,
) {
    for evt in objective_events.read() {
        // Grant legacy for each hero (objective benefits all)
        for (entity, mut score) in heroes.iter_mut() {
            score.total += 25.0;
            legacy_events.send(LegacyAwarded {
                hero: entity,
                amount: 25.0,
                reason: "Objective completed".into(),
            });
        }
        let _ = evt;
    }

    for evt in death_events.read() {
        if let Ok((_, mut score)) = heroes.get_mut(evt.killer) {
            score.total += 10.0;
            score.battles_won += 1;
            legacy_events.send(LegacyAwarded {
                hero: evt.killer,
                amount: 10.0,
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
    // Handle new children being added
    for (entity, child) in children.iter() {
        if !tree.nodes.contains_key(&entity) {
            // Compute generation before mutable borrow to avoid borrow checker conflict
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

            // Add child to parent's node
            if let Some(parent_node) = tree.nodes.get_mut(&child.parent) {
                parent_node.children.push(entity);
            }
        }
    }

    // Handle ruler ascension
    for evt in ascension_events.read() {
        if tree.root.is_none() {
            tree.root = Some(evt.new_ruler);
        }
        if let Ok(_) = heroes.get(evt.new_ruler) {
            commands.entity(evt.new_ruler).insert(LegacyScore::default());
        }
    }
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
            .add_event::<RulerAscended>()
            .add_event::<LegacyAwarded>()
            .add_event::<ObjectiveCompleted>()
            .add_event::<UnitDied>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, (succession_order_system, legacy_scoring_system, dynasty_tree_system));
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
        // Hero with higher loyalty should rank higher
        let pos_high = succession.heirs.iter().position(|e| *e == high_loyalty);
        let pos_low = succession.heirs.iter().position(|e| *e == low_loyalty);
        assert!(pos_high < pos_low, "High-loyalty hero should rank above low-loyalty");
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

        // Need to trigger Added<ChildHero> query via spawn
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
        assert!(tree.nodes.contains_key(&child), "Child should be in dynasty tree");
    }
}
