use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct RomanceLoyaltyPlugin;

impl Plugin for RomanceLoyaltyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RomanceRegistry>()
            .init_resource::<LegacyContributions>()
            .add_event::<LoyaltyChanged>()
            .add_event::<RomanceStageAdvanced>()
            .register_type::<LoyaltyRating>()
            .register_type::<RomanceLink>()
            .register_type::<RomanceStage>()
            .register_type::<ActiveLoyaltyModifiers>()
            .register_type::<RomanceRegistry>()
            .add_systems(
                FixedUpdate,
                (
                    loyalty_calculation_system,
                    romance_progression_system,
                    relationship_event_system,
                ),
            );
    }
}

/// Constants for loyalty thresholds.
const LOYALTY_BETRAYAL_THRESHOLD: f32 = 30.0;
const LOYALTY_MAX: f32 = 100.0;

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
fn romance_progression_system(
    time: Res<Time>,
    mut links: Query<(Entity, &mut RomanceLink)>,
    mut advance_events: EventWriter<RomanceStageAdvanced>,
    mut registry: ResMut<RomanceRegistry>,
) {
    let dt = time.delta_secs();
    for (entity, mut link) in links.iter_mut() {
        // Affinity grows slowly over time when together
        link.affinity += 0.1 * dt;
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

            // Remove old entry if exists
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
            if let (Ok(_a), Ok(_b)) = (heroes.get(evt.hero_a), heroes.get(evt.hero_b)) {
                // Marriage bonus: both heroes get +10 max loyalty
                // This is a placeholder; actual bonuses applied by other systems
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
            .add_event::<LoyaltyChanged>()
            .add_event::<RomanceStageAdvanced>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, (loyalty_calculation_system, romance_progression_system, relationship_event_system));
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
}
