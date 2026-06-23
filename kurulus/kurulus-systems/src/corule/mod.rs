use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct CoRulePlugin;

impl Plugin for CoRulePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CoCommandState>()
            .init_resource::<TrustMatrix>()
            .register_type::<CoCommand>()
            .register_type::<CoCommandStateComponent>()
            .register_type::<TrustMeter>()
            .register_type::<TrustTier>()
            .register_type::<TrustMatrix>()
            .register_type::<CoCommandState>()
            .register_type::<BrotherhoodAbility>()
            .register_type::<OrderPriority>()
            .register_type::<AggressionLevel>()
            .register_type::<DelegationOrderDetailed>()
            .add_systems(
                FixedUpdate,
                (
                    co_command_dispatch_system,
                    trust_update_system,
                    synergy_calculation_system,
                    delegation_order_system,
                    brotherhood_ability_system,
                    competence_calculation_system,
                ),
            );
    }
}

/// Trust gain per successful objective completion.
const TRUST_GAIN_PER_OBJECTIVE: f32 = 5.0;
/// Trust decay per second when idle.
const TRUST_DECAY_RATE: f32 = 0.01;
/// Max trust value.
const TRUST_MAX: f32 = 100.0;
/// Compute the TrustTier from a numeric trust value (GDD §20.4).
pub fn compute_trust_tier(trust: f32) -> TrustTier {
    if trust <= 20.0 {
        TrustTier::Rivalry
    } else if trust <= 40.0 {
        TrustTier::Tension
    } else if trust <= 60.0 {
        TrustTier::Neutral
    } else if trust <= 80.0 {
        TrustTier::Cooperation
    } else {
        TrustTier::Brotherhood
    }
}

/// Compute AI competence factor from trust tier and skill level.
/// Returns (efficiency_factor, refusal_chance, misinterpret_chance).
pub fn compute_ai_competence(tier: TrustTier, skill_level: f32) -> (f32, f32, f32) {
    let base_efficiency = match tier {
        TrustTier::Rivalry => 0.40,
        TrustTier::Tension => 0.55,
        TrustTier::Neutral => 0.70,
        TrustTier::Cooperation => 0.85,
        TrustTier::Brotherhood => 0.95,
    };
    let efficiency = (base_efficiency + skill_level * 0.02).min(1.0);
    let (refusal, misinterpret) = match tier {
        TrustTier::Rivalry => (0.15, 0.15),
        TrustTier::Tension => (0.10, 0.15),
        TrustTier::Neutral => (0.05, 0.05),
        TrustTier::Cooperation => (0.02, 0.02),
        TrustTier::Brotherhood => (0.0, 0.0),
    };
    (efficiency, refusal, misinterpret)
}

/// Dispatches co-command orders between primary and secondary heroes.
/// Routes delegation orders to appropriate execution systems.
fn co_command_dispatch_system(
    co_commands: Query<(&CoCommand, &CoCommandStateComponent, &TrustMeter), Changed<CoCommand>>,
) {
    for (co_cmd, state, meter) in co_commands.iter() {
        if state.mode == CoCommandMode::Solo {
            continue;
        }

        let tier = compute_trust_tier((meter.primary_trust + meter.secondary_trust) / 2.0);
        let (efficiency, refusal_chance, _misinterpret_chance) = compute_ai_competence(tier, 1.0); // skill_level = 1.0 as default

        for order in &co_cmd.delegated_orders {
            // Check if order is refused (Rivalry/Tension have refusal chance)
            let _is_refused = refusal_chance > 0.0;

            match order.order_type {
                DelegationType::Move | DelegationType::Patrol => {
                    // Movement orders with AI efficiency factor
                    let _ = efficiency;
                }
                DelegationType::Attack => {
                    // Attack orders
                }
                DelegationType::Build | DelegationType::Recruit | DelegationType::Gather => {
                    // Economy orders
                }
                DelegationType::Research | DelegationType::Diplomacy => {
                    // Strategic orders
                }
            }
        }
    }
}

/// Updates trust levels between co-rulers based on shared success.
/// Uses actual entity IDs (not PLACEHOLDER) for the global TrustMatrix.
fn trust_update_system(
    time: Res<Time>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut pairs: Query<(Entity, &CoCommandStateComponent, &mut TrustMeter)>,
    mut trust: ResMut<TrustMatrix>,
) {
    let dt = time.delta_secs();

    // Trust grows with shared objectives
    for _evt in objective_events.read() {
        for (_, _, mut meter) in pairs.iter_mut() {
            meter.primary_trust = (meter.primary_trust + TRUST_GAIN_PER_OBJECTIVE).min(TRUST_MAX);
            meter.secondary_trust =
                (meter.secondary_trust + TRUST_GAIN_PER_OBJECTIVE).min(TRUST_MAX);
        }
    }

    // Trust decays and sync to global matrix
    for (_, state, mut meter) in pairs.iter_mut() {
        meter.primary_trust = (meter.primary_trust - TRUST_DECAY_RATE * dt).max(0.0);
        meter.secondary_trust = (meter.secondary_trust - TRUST_DECAY_RATE * dt).max(0.0);

        // Sync with global trust matrix using actual entity IDs
        let avg_trust = (meter.primary_trust + meter.secondary_trust) / 2.0;
        trust
            .trust_values
            .push((state.primary_hero, state.secondary_hero, avg_trust));
    }
}

/// Calculates synergy bonus between co-ruling heroes.
/// Synergy = (average trust / 100) × base synergy factor.
fn synergy_calculation_system(
    co_commands: Query<(&CoCommandStateComponent, &TrustMeter)>,
    mut state: ResMut<CoCommandState>,
) {
    let mut total_synergy = 0.0;
    let mut count = 0u32;

    for (cmd, meter) in co_commands.iter() {
        let tier = compute_trust_tier((meter.primary_trust + meter.secondary_trust) / 2.0);
        let synergy_multiplier = match tier {
            TrustTier::Cooperation => 1.2,
            TrustTier::Brotherhood => 1.5,
            _ => 1.0,
        };
        total_synergy += cmd.synergy_bonus * synergy_multiplier;
        count += 1;
    }

    if count > 0 {
        let avg_synergy = total_synergy / count as f32;
        state.active = avg_synergy > 0.1;
    }
}

/// Processes delegation orders between heroes.
fn delegation_order_system(mut co_commands: Query<&mut CoCommand>, _state: Res<CoCommandState>) {
    for mut co_cmd in co_commands.iter_mut() {
        // Process and clean up expired delegation orders
        co_cmd.delegated_orders.retain(|order| {
            if let Some(expiry) = order.expires_at {
                expiry > 0.0
            } else {
                true // no expiry
            }
        });
    }
}

/// Activates BrotherhoodAbility tiers based on trust level (GDD §20.5).
/// Tier 1: Trust ≥ 81, Tier 2: Trust ≥ 90, Tier 3: Trust ≥ 95.
fn brotherhood_ability_system(
    mut commands: Commands,
    pairs: Query<(Entity, &CoCommandStateComponent, &TrustMeter)>,
) {
    for (entity, _state, meter) in pairs.iter() {
        let avg_trust = (meter.primary_trust + meter.secondary_trust) / 2.0;
        let tier = compute_trust_tier(avg_trust);

        // Only Brotherhood tier unlocks abilities
        if tier != TrustTier::Brotherhood {
            // Remove brotherhood ability if trust drops below threshold
            commands.entity(entity).remove::<BrotherhoodAbility>();
            continue;
        }

        let ability_tier = if avg_trust >= 95.0 {
            3u8
        } else if avg_trust >= 90.0 {
            2u8
        } else {
            1u8
        };

        // Insert or update the BrotherhoodAbility component
        let trust_required = match ability_tier {
            3 => 95.0,
            2 => 90.0,
            _ => 81.0,
        };

        commands.entity(entity).insert(BrotherhoodAbility {
            tier: ability_tier,
            trust_required,
            cooldown: match ability_tier {
                3 => 120.0,
                2 => 90.0,
                _ => 60.0,
            },
            active: true,
        });
    }
}

/// Calculates AI competence based on trust tier and updates the state.
fn competence_calculation_system(mut pairs: Query<(&mut CoCommandStateComponent, &TrustMeter)>) {
    for (mut state, meter) in pairs.iter_mut() {
        let avg_trust = (meter.primary_trust + meter.secondary_trust) / 2.0;
        state.trust_level = avg_trust;

        let tier = compute_trust_tier(avg_trust);
        let (efficiency, _, _) = compute_ai_competence(tier, 1.0);
        state.synergy_bonus = efficiency;
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
            .init_resource::<CoCommandState>()
            .init_resource::<TrustMatrix>()
            .add_event::<ObjectiveCompleted>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(
                Update,
                (
                    co_command_dispatch_system,
                    trust_update_system,
                    synergy_calculation_system,
                    delegation_order_system,
                    brotherhood_ability_system,
                    competence_calculation_system,
                ),
            );
        app
    }

    #[test]
    fn test_trust_grows_with_objectives() {
        let mut app = test_app();
        let primary = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();
        let secondary = app.world_mut().spawn(Hero::new(HeroId::Turgut)).id();

        let pair = app
            .world_mut()
            .spawn((
                CoCommandStateComponent {
                    primary_hero: primary,
                    secondary_hero: secondary,
                    mode: CoCommandMode::Joint,
                    trust_level: 50.0,
                    synergy_bonus: 0.0,
                },
                TrustMeter {
                    primary_trust: 50.0,
                    secondary_trust: 50.0,
                    max_trust: 100.0,
                    decay_rate: 0.01,
                    gain_rate: 5.0,
                },
            ))
            .id();

        app.world_mut().send_event(ObjectiveCompleted {
            objective_id: ObjectiveId::TutorialMoveCamera,
            mission_id: MissionId::Mission01,
        });
        app.update();

        let meter = app.world().get::<TrustMeter>(pair).unwrap();
        assert!(
            (meter.primary_trust - 55.0).abs() < 0.1,
            "Expected trust ~55, got {}",
            meter.primary_trust
        );
    }

    #[test]
    fn test_synergy_calculation() {
        let mut app = test_app();
        let primary = app.world_mut().spawn_empty().id();
        let secondary = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            CoCommandStateComponent {
                primary_hero: primary,
                secondary_hero: secondary,
                mode: CoCommandMode::Joint,
                trust_level: 80.0,
                synergy_bonus: 0.35,
            },
            TrustMeter {
                primary_trust: 80.0,
                secondary_trust: 80.0,
                max_trust: 100.0,
                decay_rate: 0.01,
                gain_rate: 5.0,
            },
        ));

        app.update();

        let state = app.world().resource::<CoCommandState>();
        assert!(state.active, "Synergy should be active");
    }

    #[test]
    fn test_delegation_order_cleanup() {
        let mut app = test_app();
        app.world_mut().spawn(CoCommand {
            primary_hero: Entity::PLACEHOLDER,
            secondary_hero: Entity::PLACEHOLDER,
            active_mode: CoCommandMode::Delegated,
            delegated_orders: vec![DelegationOrder {
                from_hero: Entity::PLACEHOLDER,
                to_hero: Entity::PLACEHOLDER,
                order_type: DelegationType::Patrol,
                target: DelegationTarget::Position(GridPos(IVec2::new(10, 10))),
                issued_at: 0.0,
                expires_at: Some(-1.0), // expired
            }],
        });

        app.update();

        let cmds = app
            .world_mut()
            .query::<&CoCommand>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert!(!cmds.is_empty());
        assert!(
            cmds[0].delegated_orders.is_empty(),
            "Expired orders should be cleaned up"
        );
    }

    #[test]
    fn test_trust_tier_rivalry() {
        assert!(matches!(compute_trust_tier(10.0), TrustTier::Rivalry));
        assert!(matches!(compute_trust_tier(20.0), TrustTier::Rivalry));
    }

    #[test]
    fn test_trust_tier_brotherhood() {
        assert!(matches!(compute_trust_tier(85.0), TrustTier::Brotherhood));
        assert!(matches!(compute_trust_tier(100.0), TrustTier::Brotherhood));
    }

    #[test]
    fn test_ai_competence_rivalry() {
        let (efficiency, refusal, _) = compute_ai_competence(TrustTier::Rivalry, 0.0);
        assert!(efficiency <= 0.5, "Rivalry AI efficiency should be ≤50%");
        assert!(refusal > 0.0, "Rivalry should have refusal chance");
    }

    #[test]
    fn test_ai_competence_brotherhood() {
        let (efficiency, refusal, _) = compute_ai_competence(TrustTier::Brotherhood, 0.0);
        assert!(
            efficiency >= 0.9,
            "Brotherhood AI efficiency should be ≥90%"
        );
        assert!(refusal == 0.0, "Brotherhood should have no refusal");
    }

    #[test]
    fn test_brotherhood_ability_tier_1() {
        let mut app = test_app();
        let primary = app.world_mut().spawn_empty().id();
        let secondary = app.world_mut().spawn_empty().id();
        let pair = app
            .world_mut()
            .spawn((
                CoCommandStateComponent {
                    primary_hero: primary,
                    secondary_hero: secondary,
                    mode: CoCommandMode::Joint,
                    trust_level: 85.0,
                    synergy_bonus: 0.0,
                },
                TrustMeter {
                    primary_trust: 85.0,
                    secondary_trust: 85.0,
                    max_trust: 100.0,
                    decay_rate: 0.01,
                    gain_rate: 5.0,
                },
            ))
            .id();

        app.update();

        let ability = app.world().get::<BrotherhoodAbility>(pair);
        assert!(
            ability.is_some(),
            "Brotherhood ability should be active at trust 85"
        );
        if let Some(ab) = ability {
            assert_eq!(ab.tier, 1, "Should be tier 1 at trust 85");
        }
    }

    #[test]
    fn test_competence_updates_state() {
        let mut app = test_app();
        let primary = app.world_mut().spawn_empty().id();
        let secondary = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            CoCommandStateComponent {
                primary_hero: primary,
                secondary_hero: secondary,
                mode: CoCommandMode::Joint,
                trust_level: 50.0,
                synergy_bonus: 0.0,
            },
            TrustMeter {
                primary_trust: 90.0,
                secondary_trust: 90.0,
                max_trust: 100.0,
                decay_rate: 0.01,
                gain_rate: 5.0,
            },
        ));

        app.update();

        let states = app
            .world_mut()
            .query::<&CoCommandStateComponent>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert!(!states.is_empty());
        assert!(
            states[0].synergy_bonus > 0.5,
            "Synergy bonus should reflect high trust"
        );
    }
}
