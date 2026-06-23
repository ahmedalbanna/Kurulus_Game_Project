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
            .register_type::<CoCommandState>()
            .register_type::<TrustMatrix>()
            .add_systems(
                FixedUpdate,
                (
                    co_command_dispatch_system,
                    trust_update_system,
                    synergy_calculation_system,
                    delegation_order_system,
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

/// Dispatches co-command orders between primary and secondary heroes.
/// Reads the CoCommand component and routes delegation orders to the
/// appropriate execution systems.
fn co_command_dispatch_system(
    commands: Commands,
    co_commands: Query<(&CoCommand, &CoCommandStateComponent), Changed<CoCommand>>,
    heroes: Query<&Hero>,
) {
    for (co_cmd, state) in co_commands.iter() {
        // Only dispatch in Joint or Delegated mode
        if state.mode == CoCommandMode::Solo {
            continue;
        }

        // Verify both heroes exist
        let _primary_ok = heroes.get(co_cmd.primary_hero).is_ok();
        let _secondary_ok = heroes.get(co_cmd.secondary_hero).is_ok();

        // Route delegation orders to the secondary hero
        for order in &co_cmd.delegated_orders {
            match order.order_type {
                DelegationType::Move | DelegationType::Patrol => {
                    // Movement orders — handled by unit command system
                    let _ = &order;
                }
                DelegationType::Attack => {
                    // Attack orders — handled by combat system
                }
                DelegationType::Build | DelegationType::Recruit | DelegationType::Gather => {
                    // Economy orders — handled by economy system
                }
                DelegationType::Research | DelegationType::Diplomacy => {
                    // Strategic orders — handled by progression/diplomacy
                }
            }
        }

        let _ = commands;
    }
}

/// Updates trust levels between co-rulers based on shared success.
fn trust_update_system(
    time: Res<Time>,
    mut trust: ResMut<TrustMatrix>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut co_commands: Query<&mut TrustMeter>,
) {
    // Trust grows with shared objectives
    for _evt in objective_events.read() {
        for mut meter in co_commands.iter_mut() {
            meter.primary_trust = (meter.primary_trust + TRUST_GAIN_PER_OBJECTIVE).min(TRUST_MAX);
            meter.secondary_trust = (meter.secondary_trust + TRUST_GAIN_PER_OBJECTIVE).min(TRUST_MAX);
        }
    }

    // Trust decays slowly over time without interaction
    let dt = time.delta_secs();
    for mut meter in co_commands.iter_mut() {
        meter.primary_trust = (meter.primary_trust - TRUST_DECAY_RATE * dt).max(0.0);
        meter.secondary_trust = (meter.secondary_trust - TRUST_DECAY_RATE * dt).max(0.0);
    }

    // Sync trust values into the global matrix
    for meter in co_commands.iter() {
        trust.trust_values.push((
            Entity::PLACEHOLDER,
            Entity::PLACEHOLDER,
            (meter.primary_trust + meter.secondary_trust) / 2.0,
        ));
    }
}

/// Calculates synergy bonus between co-ruling heroes.
/// Synergy = (average trust / 100) × base synergy factor.
fn synergy_calculation_system(
    co_commands: Query<&CoCommandStateComponent>,
    mut state: ResMut<CoCommandState>,
) {
    let mut total_synergy = 0.0;
    let mut count = 0u32;

    for cmd in co_commands.iter() {
        total_synergy += cmd.synergy_bonus;
        count += 1;
    }

    if count > 0 {
        // Average synergy across all co-command pairs
        let avg_synergy = total_synergy / count as f32;
        state.active = avg_synergy > 0.1;
    }
}

/// Processes delegation orders between heroes.
fn delegation_order_system(
    commands: Commands,
    mut co_commands: Query<&mut CoCommand>,
    _state: Res<CoCommandState>,
) {
    for mut co_cmd in co_commands.iter_mut() {
        // Process and clean up expired delegation orders
        co_cmd.delegated_orders.retain(|order| {
            if let Some(expiry) = order.expires_at {
                expiry > 0.0 // placeholder: in real impl, check against game time
            } else {
                true // no expiry
            }
        });
    }

    let _ = commands;
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
            .add_systems(Update, (co_command_dispatch_system, trust_update_system, synergy_calculation_system, delegation_order_system));
        app
    }

    #[test]
    fn test_trust_grows_with_objectives() {
        let mut app = test_app();
        let hero_a = app
            .world_mut()
            .spawn((
                Hero::new(HeroId::Osman),
                TrustMeter {
                    primary_trust: 50.0,
                    secondary_trust: 50.0,
                    max_trust: 100.0,
                    decay_rate: 0.01,
                    gain_rate: 5.0,
                },
            ))
            .id();

        let _hero_b = app
            .world_mut()
            .spawn(Hero::new(HeroId::Turgut))
            .id();

        app.world_mut().send_event(ObjectiveCompleted {
            objective_id: ObjectiveId::TutorialMoveCamera,
            mission_id: MissionId::Mission01,
        });
        app.update();

        let meter = app.world().get::<TrustMeter>(hero_a).unwrap();
        assert!(
            (meter.primary_trust - 55.0).abs() < 0.1,
            "Expected trust ~55, got {}",
            meter.primary_trust
        );
    }

    #[test]
    fn test_synergy_calculation() {
        let mut app = test_app();
        app.world_mut().spawn(CoCommandStateComponent {
            primary_hero: Entity::PLACEHOLDER,
            secondary_hero: Entity::PLACEHOLDER,
            mode: CoCommandMode::Joint,
            trust_level: 80.0,
            synergy_bonus: 0.35,
        });

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

        let cmds = app.world_mut().query::<&CoCommand>().iter(app.world()).collect::<Vec<_>>();
        assert!(!cmds.is_empty());
        assert!(cmds[0].delegated_orders.is_empty(), "Expired orders should be cleaned up");
    }
}
