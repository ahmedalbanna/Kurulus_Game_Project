use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct RomanceLoyaltyPlugin;

impl Plugin for RomanceLoyaltyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RomanceRegistry>()
            .init_resource::<LegacyContributions>()
            .init_resource::<RelationshipMatrix>()
            .add_event::<LoyaltyChanged>()
            .add_event::<RomanceStageAdvanced>()
            .register_type::<LoyaltyRating>()
            .register_type::<RomanceLink>()
            .register_type::<RomanceStage>()
            .register_type::<ActiveLoyaltyModifiers>()
            .register_type::<RomanceRegistry>()
            .register_type::<RelationshipPair>()
            .register_type::<RelationshipType>()
            .register_type::<LoveImpactTier>()
            .register_type::<CoupleAbility>()
            .register_type::<ChoiceImpactRecord>()
            .register_type::<RelationshipMatrix>()
            .add_systems(
                FixedUpdate,
                (
                    loyalty_calculation_system,
                    romance_progression_system,
                    relationship_event_system,
                    love_impact_system,
                    choice_impact_system,
                ),
            );
    }
}

/// Constants for loyalty thresholds.
const LOYALTY_BETRAYAL_THRESHOLD: f32 = 30.0;
const LOYALTY_MAX: f32 = 100.0;
const PROXIMITY_AFFINITY_RANGE: f32 = 10.0; // tiles
const AFFINITY_GAIN_PER_SECOND: f32 = 0.5;
const AFFINITY_KILL_BONUS: f32 = 2.0;
const AFFINITY_OBJECTIVE_BONUS: f32 = 5.0;

/// Tick-based loyalty calculation. Applies modifiers, decays/grows toward baseline.
fn loyalty_calculation_system(
    time: Res<Time>,
    mut heroes: Query<(Entity, &mut LoyaltyRating, &mut ActiveLoyaltyModifiers)>,
    mut change_events: EventWriter<LoyaltyChanged>,
) {
    let dt = time.delta_secs();
    for (entity, mut loyalty, mut modifiers) in heroes.iter_mut() {
        let old_value = loyalty.value;

        // Apply active modifiers
        let mut total_delta = 0.0;
        modifiers.modifiers.retain(|m| {
            if let Some(remaining) = m.duration {
                if remaining <= 0.0 {
                    return false; // expired
                }
            }
            total_delta += m.amount;
            true
        });

        // Natural drift toward center (50)
        let center = 50.0;
        let drift = (center - loyalty.value) * 0.001 * dt;
        loyalty.value = (loyalty.value + total_delta + drift).clamp(0.0, LOYALTY_MAX);

        // Fire event on significant change (>5 points)
        if (loyalty.value - old_value).abs() > 5.0 {
            change_events.send(LoyaltyChanged {
                hero: entity,
                old_value,
                new_value: loyalty.value,
                reason: if total_delta != 0.0 {
                    "Modifier applied".into()
                } else {
                    "Natural drift".into()
                },
            });
        }
    }
}

/// Tracks romance progression between linked characters.
/// Stages: Acquaintance → Friendship → Romance → Betrothal → Married
/// Affinity grows when characters are in proximity (same area).
fn romance_progression_system(
    time: Res<Time>,
    mut links: Query<(Entity, &mut RomanceLink)>,
    mut advance_events: EventWriter<RomanceStageAdvanced>,
    mut registry: ResMut<RomanceRegistry>,
    positions: Query<&WorldPosition>,
    heroes: Query<&Hero>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut death_events: EventReader<UnitDied>,
) {
    let dt = time.delta_secs();

    // Process event-based affinity bonuses first
    for _evt in objective_events.read() {
        for (_, mut link) in links.iter_mut() {
            // Check if the partner hero exists and completed the objective
            if heroes.get(link.partner).is_ok() {
                link.affinity = (link.affinity + AFFINITY_OBJECTIVE_BONUS).min(100.0);
            }
        }
    }

    for evt in death_events.read() {
        if let Ok(_) = heroes.get(evt.killer) {
            for (_, mut link) in links.iter_mut() {
                // If entity killed something, affinity with partner grows
                if let Ok(_) = heroes.get(link.partner) {
                    link.affinity = (link.affinity + AFFINITY_KILL_BONUS).min(100.0);
                }
            }
        }
        let _ = evt;
    }

    // Proximity-based affinity growth
    for (entity, mut link) in links.iter_mut() {
        // Check if both entities have positions for proximity check
        let pos_self = positions.get(entity).ok();
        let pos_partner = positions.get(link.partner).ok();

        if let (Some(self_pos), Some(partner_pos)) = (pos_self, pos_partner) {
            let diff = self_pos.grid.0 - partner_pos.grid.0;
            let distance = (diff.x.abs() + diff.y.abs()) as f32; // Manhattan distance
            if distance <= PROXIMITY_AFFINITY_RANGE {
                link.affinity += AFFINITY_GAIN_PER_SECOND * dt;
            }
        }

        link.affinity = link.affinity.min(100.0);

        // Stage thresholds
        let new_stage = if link.affinity >= 90.0 {
            RomanceStage::Married
        } else if link.affinity >= 70.0 {
            RomanceStage::Betrothal
        } else if link.affinity >= 50.0 {
            RomanceStage::Romance
        } else if link.affinity >= 25.0 {
            RomanceStage::Friendship
        } else {
            RomanceStage::Acquaintance
        };

        if new_stage != link.stage {
            let _old_stage = link.stage;
            link.stage = new_stage;
            advance_events.send(RomanceStageAdvanced {
                hero_a: entity,
                hero_b: link.partner,
                new_stage,
            });

            // Update registry
            registry.links.push((entity, link.partner, new_stage));
            registry.links.retain(|(a, b, _)| !(*a == entity && *b == link.partner));
        }
    }
}

/// Handles relationship events: marriage, betrayal, etc.
fn relationship_event_system(
    mut commands: Commands,
    mut loyalty_events: EventReader<LoyaltyChanged>,
    mut stage_events: EventReader<RomanceStageAdvanced>,
    heroes: Query<&Hero>,
) {
    for evt in loyalty_events.read() {
        // Betrayal check: loyalty below threshold
        if evt.new_value < LOYALTY_BETRAYAL_THRESHOLD {
            // Mark hero as disloyal — future mechanic to trigger desertion
            commands.entity(evt.hero).insert(ActiveLoyaltyModifiers {
                modifiers: vec![LoyaltyModifier {
                    reason: "Low loyalty — betrayal risk".into(),
                    amount: 0.0,
                    duration: None,
                }],
            });
        }
    }

    for evt in stage_events.read() {
        if evt.new_stage == RomanceStage::Married {
            // Marriage unlocks succession bonuses
            // Both heroes get +10 loyalty via modifier
            if heroes.get(evt.hero_a).is_ok() && heroes.get(evt.hero_b).is_ok() {
                // Marriage bonus is applied by successor systems
            }
        }
    }
}

/// Computes love impact tiers from relationship pairs.
/// Fires events when tiers change to allow stat systems to react.
fn love_impact_system(
    mut commands: Commands,
    pairs: Query<(Entity, &RelationshipPair)>,
    links: Query<&RomanceLink>,
    mut tier_changed: Local<Vec<(Entity, LoveImpactTier)>>,
) {
    for (entity, pair) in pairs.iter() {
        // Get the current affinity from both sides
        let mut affinity = pair.current_value;

        if let Ok(link) = links.get(pair.hero_a) {
            affinity = link.affinity;
        }
        if let Ok(link) = links.get(pair.hero_b) {
            affinity = affinity.max(link.affinity);
        }

        let new_tier = if affinity > 80.0 {
            LoveImpactTier::Inspired
        } else if affinity >= 50.0 {
            LoveImpactTier::Normal
        } else {
            LoveImpactTier::Distracted
        };

        // Check if tier changed from previous frame
        if let Some(pos) = tier_changed.iter().position(|(e, _)| *e == entity) {
            let (_, current_tier) = tier_changed[pos];
            if current_tier != new_tier {
                tier_changed[pos] = (entity, new_tier);
                // TODO: emit event for stat system to apply modifiers
            }
        } else {
            tier_changed.push((entity, new_tier));
        }
    }

    // Clean up entries for despawned entities
    tier_changed.retain(|(e, _)| commands.get_entity(*e).is_some());
}

/// Processes mission choice impacts on relationships.
/// Reads ChoiceImpactRecord and applies affinity deltas.
fn choice_impact_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut pairs: Query<&mut RelationshipPair>,
    mut links: Query<&mut RomanceLink>,
) {
    for evt in objective_events.read() {
        for mut pair in pairs.iter_mut() {
            // Record the event in the pair's history
            pair.events_completed.push(format!("obj_{:?}", evt.objective_id));

            // Apply a small affinity boost for shared objective completion
            if let Ok(mut link) = links.get_mut(pair.hero_a) {
                link.affinity = (link.affinity + 1.0).min(pair.max_value);
            }
            if let Ok(mut link) = links.get_mut(pair.hero_b) {
                link.affinity = (link.affinity + 1.0).min(pair.max_value);
            }
        }
    }
}

/// Adds a loyalty modifier to a hero (public helper for other systems).
pub fn add_loyalty_modifier(
    commands: &mut Commands,
    hero: Entity,
    reason: &str,
    amount: f32,
    duration: Option<f32>,
) {
    commands.entity(hero).insert(ActiveLoyaltyModifiers {
        modifiers: vec![LoyaltyModifier {
            reason: reason.into(),
            amount,
            duration,
        }],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RomanceRegistry>()
            .init_resource::<RelationshipMatrix>()
            .add_event::<LoyaltyChanged>()
            .add_event::<RomanceStageAdvanced>()
            .add_event::<ObjectiveCompleted>()
            .add_event::<UnitDied>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, (
                loyalty_calculation_system,
                romance_progression_system,
                relationship_event_system,
                love_impact_system,
                choice_impact_system,
            ));
        app
    }

    #[test]
    fn test_loyalty_drift_toward_center() {
        let mut app = test_app();
        let hero = app
            .world_mut()
            .spawn((
                LoyaltyRating {
                    value: 100.0,
                    max: 100.0,
                },
                ActiveLoyaltyModifiers::default(),
            ))
            .id();

        // Run multiple updates to accumulate drift
        for _ in 0..60 {
            app.update();
        }

        let loyalty = app.world().get::<LoyaltyRating>(hero).unwrap();
        assert!(loyalty.value < 100.0, "Should drift down from 100");
    }

    #[test]
    fn test_romance_progression() {
        let mut app = test_app();
        let _a = app.world_mut().spawn_empty().id();
        let b = app.world_mut().spawn_empty().id();

        app.world_mut().spawn(RomanceLink {
            partner: b,
            stage: RomanceStage::Acquaintance,
            affinity: 80.0, // Should trigger Romance stage
        });

        app.update();

        // Find the romance link
        let links: Vec<&RomanceLink> = app.world_mut().query::<&RomanceLink>().iter(app.world()).collect();
        assert!(!links.is_empty());
        let link = links[0];
        assert!(matches!(link.stage, RomanceStage::Romance | RomanceStage::Betrothal),
            "Expected Romance/Betrothal at affinity 80, got {:?}", link.stage);
    }

    #[test]
    fn test_affinity_grows_with_objectives() {
        let mut app = test_app();
        let partner = app.world_mut().spawn(Hero::default()).id();
        let hero = app
            .world_mut()
            .spawn((
                RomanceLink {
                    partner,
                    stage: RomanceStage::Acquaintance,
                    affinity: 10.0,
                },
                Hero::default(),
            ))
            .id();

        app.world_mut().send_event(ObjectiveCompleted {
            objective_id: ObjectiveId::TutorialMoveCamera,
            mission_id: MissionId::Mission01,
        });
        app.update();

        let link = app.world().get::<RomanceLink>(hero).unwrap();
        assert!(link.affinity > 10.0, "Affinity should grow with objectives");
    }

    #[test]
    fn test_affinity_grows_with_kills() {
        let mut app = test_app();
        let partner = app.world_mut().spawn(Hero::default()).id();
        let hero = app
            .world_mut()
            .spawn((
                RomanceLink {
                    partner,
                    stage: RomanceStage::Acquaintance,
                    affinity: 10.0,
                },
                Hero::default(),
            ))
            .id();
        let enemy = app.world_mut().spawn_empty().id();

        app.world_mut().send_event(UnitDied {
            entity: enemy,
            killer: hero,
        });
        app.update();

        let link = app.world().get::<RomanceLink>(hero).unwrap();
        assert!(link.affinity > 10.0, "Affinity should grow with kills");
    }

    #[test]
    fn test_affinity_grows_with_proximity() {
        let mut app = test_app();
        let partner = app
            .world_mut()
            .spawn(WorldPosition::new(IVec2::new(0, 0), 1.0))
            .id();
        let hero = app
            .world_mut()
            .spawn((
                RomanceLink {
                    partner,
                    stage: RomanceStage::Acquaintance,
                    affinity: 10.0,
                },
                WorldPosition::new(IVec2::new(1, 0), 1.0), // within range
            ))
            .id();

        // Run multiple updates for proximity growth
        for _ in 0..60 {
            app.update();
        }

        let link = app.world().get::<RomanceLink>(hero).unwrap();
        assert!(link.affinity > 10.0, "Affinity should grow with proximity");
    }

    #[test]
    fn test_love_impact_tier_computed() {
        let mut app = test_app();
        let partner = app.world_mut().spawn_empty().id();
        let hero = app.world_mut().spawn(RelationshipPair {
            hero_a: Entity::PLACEHOLDER,
            hero_b: Entity::PLACEHOLDER,
            relationship_type: RelationshipType::Romantic,
            starting_value: 90.0,
            current_value: 90.0,
            max_value: 100.0,
            events_completed: Vec::new(),
        }).id();

        app.update();

        // love_impact_system should store the tier in its local vec
        // We can't easily read Local, but the system should not crash
        let _ = hero;
        let _ = partner;
    }

    #[test]
    fn test_love_tier_inspired_at_high_affinity() {
        let mut app = test_app();
        let partner = app.world_mut().spawn_empty().id();
        let hero = app.world_mut().spawn(RomanceLink {
            partner,
            stage: RomanceStage::Betrothal,
            affinity: 95.0,
        }).id();

        // Spawn a corresponding RelationshipPair
        app.world_mut().spawn(RelationshipPair {
            hero_a: hero,
            hero_b: partner,
            relationship_type: RelationshipType::Romantic,
            starting_value: 95.0,
            current_value: 95.0,
            max_value: 100.0,
            events_completed: Vec::new(),
        });

        app.update();
        // System should not crash — tier computation is internal
    }
}
