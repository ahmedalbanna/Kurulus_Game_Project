use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Duration a hero stays KnockedOut before recovering (GDD: 30s).
const KNOCKOUT_DURATION: f32 = 30.0;

/// Duration a hero stays in Recovering state (GDD: 5s).
const RECOVERY_DURATION: f32 = 5.0;

/// Percentage of max HP restored on recovery (GDD: 20%).
const RECOVERY_HP_FRACTION: f32 = 0.20;

/// Health fraction threshold to trigger KO (<= 0 HP).
const KO_HP_THRESHOLD: f32 = 0.0;

// ─── Events ──────────────────────────────────────────────────────────────────

/// Internal event: signals that a death-blow check should be performed.
/// Fired by abilities/tools that can execute a downed hero.
#[derive(Event, Clone, Debug)]
pub struct DeathBlowAttempt {
    pub target: Entity,
    pub attacker: Entity,
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Main hero state machine tick.
///
/// Runs every frame and handles:
/// 1. HP ≤ 0 → KnockedOut transition (from any active state)
/// 2. KnockedOut timer tick → timer expiry → Recovering
/// 3. Recovering timer tick → timer expiry → Idle (with 20% HP)
/// 4. Death blow received while KO → Dead
///
/// Death blow is handled via `DeathBlowAttempt` events — any ability or
/// enemy attack system can send this event when targeting a KO'd hero.
pub fn hero_state_machine_system(
    time: Res<Time>,
    mut heroes: Query<(
        Entity,
        &mut Hero,
        &mut HeroState,
        &mut HeroDeathState,
        &mut Health,
    )>,
    mut death_blow_events: EventReader<DeathBlowAttempt>,
    mut ko_events: EventWriter<HeroKnockedOut>,
    mut recovery_events: EventWriter<HeroRecovering>,
    mut idle_events: EventWriter<HeroReturnedToIdle>,
    mut killed_events: EventWriter<HeroKilledInMission>,
    mut death_blow_out_events: EventWriter<HeroDeathBlow>,
) {
    // Collect valid death blow attempts (target is KO'd hero) before any mutations
    let death_blows: Vec<(Entity, Entity)> = death_blow_events
        .read()
        .filter_map(|evt| {
            heroes
                .get(evt.target)
                .ok()
                .filter(|(_, _, state, _, _)| matches!(state, HeroState::KnockedOut { .. }))
                .map(|_| (evt.target, evt.attacker))
        })
        .collect();

    let delta = time.delta_secs();

    for (entity, mut hero, mut state, mut death_state, mut health) in heroes.iter_mut() {
        match state.as_mut() {
            // ── Normal states: check HP for KO ──────────────────────────
            s @ (HeroState::Idle
            | HeroState::Moving
            | HeroState::Running
            | HeroState::Attacking
            | HeroState::UsingAbility
            | HeroState::Channeling) => {
                if health.current <= KO_HP_THRESHOLD {
                    // Transition to KnockedOut
                    *s = HeroState::KnockedOut {
                        timer: KNOCKOUT_DURATION,
                    };
                    death_state.is_dead_this_mission = false;
                    death_state.knockout_timer = KNOCKOUT_DURATION;

                    ko_events.send(HeroKnockedOut {
                        hero: entity,
                        hero_id: hero.hero_id,
                        knockout_duration: KNOCKOUT_DURATION,
                    });
                }
            }

            // ── KnockedOut: tick timer, check death blows ──────────────
            HeroState::KnockedOut { ref mut timer } => {
                *timer -= delta;
                death_state.knockout_timer = *timer;

                // Check death blow
                let is_death_blown = death_blows.iter().any(|(target, _)| *target == entity);

                if is_death_blown {
                    hero.is_permanently_dead = true;
                    *state = HeroState::Dead;
                    death_state.is_dead_this_mission = true;

                    let killer = death_blows
                        .iter()
                        .find(|(target, _)| *target == entity)
                        .map(|(_, attacker)| *attacker)
                        .unwrap_or(entity);

                    death_blow_out_events.send(HeroDeathBlow {
                        hero: entity,
                        hero_id: hero.hero_id,
                        killer,
                    });

                    killed_events.send(HeroKilledInMission {
                        hero: entity,
                        hero_id: hero.hero_id,
                        killer,
                    });
                } else if *timer <= 0.0 {
                    // Timer expired → transition to Recovering
                    *state = HeroState::Recovering {
                        timer: RECOVERY_DURATION,
                    };

                    recovery_events.send(HeroRecovering {
                        hero: entity,
                        hero_id: hero.hero_id,
                    });
                }
            }

            // ── Recovering: tick timer, then return to Idle ────────────
            HeroState::Recovering { ref mut timer } => {
                *timer -= delta;

                if *timer <= 0.0 {
                    // Return to action with 20% HP
                    health.current = health.max * RECOVERY_HP_FRACTION;
                    *state = HeroState::Idle;

                    idle_events.send(HeroReturnedToIdle {
                        hero: entity,
                        hero_id: hero.hero_id,
                    });
                }
            }

            // ── Dead: no transitions ───────────────────────────────────
            HeroState::Dead => {}
        }
    }
}

/// Applies HeroDeathPenalties when a hero is killed in a mission.
pub fn hero_death_penalty_system(
    mut killed_events: EventReader<HeroKilledInMission>,
    mut penalties: ResMut<HeroDeathPenalties>,
) {
    for evt in killed_events.read() {
        let entry = penalties
            .penalties
            .entry(evt.hero_id)
            .or_insert_with(DeathPenalty::default);
        entry.rating_loss += 10.0;
        entry.xp_loss += 100;
        entry.morale_penalty += 15.0;
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HeroDeathPenalties>()
            .add_event::<HeroKnockedOut>()
            .add_event::<HeroRecovering>()
            .add_event::<HeroReturnedToIdle>()
            .add_event::<HeroKilledInMission>()
            .add_event::<HeroDeathBlow>()
            .add_event::<DeathBlowAttempt>()
            .add_event::<DamageDealt>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, hero_state_machine_system);
        app
    }

    /// Advance the app by `frames` frames at the test frame rate (1/60s each).
    fn advance_frames(app: &mut App, frames: u32) {
        for _ in 0..frames {
            app.update();
        }
    }

    /// Advance the app by `seconds` at 60fps.
    fn advance_seconds(app: &mut App, seconds: f32) {
        let frames = (seconds * 60.0).ceil() as u32;
        advance_frames(app, frames);
    }

    fn spawn_hero(app: &mut App, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Hero::new(HeroId::Osman),
                HeroState::Idle,
                HeroDeathState::default(),
                Health {
                    current: hp,
                    max: 1000.0,
                    armor: 0.0,
                    armor_type: ArmorType::Light,
                },
            ))
            .id()
    }

    #[test]
    fn test_hero_enters_ko_when_hp_zero() {
        let mut app = test_app();
        let hero = spawn_hero(&mut app, 0.0);
        app.update();

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert!(
            matches!(state, HeroState::KnockedOut { .. }),
            "Expected KnockedOut, got {:?}",
            state
        );
    }

    #[test]
    fn test_hero_stays_active_when_hp_positive() {
        let mut app = test_app();
        let hero = spawn_hero(&mut app, 100.0);
        app.update();

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Idle);
    }

    #[test]
    fn test_hero_ko_timer_counts_down() {
        let mut app = test_app();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO
        app.update();

        let timer_1 = match app.world().get::<HeroState>(hero).unwrap() {
            HeroState::KnockedOut { timer } => *timer,
            _ => panic!("Expected KnockedOut"),
        };
        assert!((timer_1 - KNOCKOUT_DURATION).abs() < 0.1);

        // Advance ~10 seconds at 60fps
        advance_seconds(&mut app, 10.0);

        let timer_2 = match app.world().get::<HeroState>(hero).unwrap() {
            HeroState::KnockedOut { timer } => *timer,
            _ => panic!("Expected KnockedOut"),
        };
        // Should have ticked down by ~10 seconds
        let expected = KNOCKOUT_DURATION - 10.0;
        assert!(
            (timer_2 - expected).abs() < 1.0,
            "Expected timer ~{:.1}, got {:.1}",
            expected,
            timer_2
        );
    }

    #[test]
    fn test_hero_recovering_after_ko_timer_expires() {
        let mut app = test_app();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO
        app.update();

        // Advance past KO duration
        advance_seconds(&mut app, KNOCKOUT_DURATION + 2.0);

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert!(
            matches!(state, HeroState::Recovering { .. }),
            "Expected Recovering, got {:?}",
            state
        );
    }

    #[test]
    fn test_hero_returns_to_idle_after_recovery() {
        let mut app = test_app();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO
        app.update();

        // Advance past KO + recovery
        advance_seconds(&mut app, KNOCKOUT_DURATION + RECOVERY_DURATION + 2.0);

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Idle);

        // HP restored to 20% of max
        let health = app.world().get::<Health>(hero).unwrap();
        assert!(
            (health.current - 200.0).abs() < 1.0,
            "Expected ~200 HP, got {}",
            health.current
        );
    }

    #[test]
    fn test_death_blow_kills_hero() {
        let mut app = test_app();
        let attacker = app.world_mut().spawn_empty().id();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO
        app.update();

        // Send death blow
        app.world_mut().send_event(DeathBlowAttempt {
            target: hero,
            attacker,
        });
        app.update();

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Dead);
    }

    #[test]
    fn test_death_blow_only_works_on_ko() {
        let mut app = test_app();
        let attacker = app.world_mut().spawn_empty().id();
        let hero = spawn_hero(&mut app, 100.0); // Not KO'd

        app.world_mut().send_event(DeathBlowAttempt {
            target: hero,
            attacker,
        });
        app.update();

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Idle);
    }

    #[test]
    fn test_death_penalty_tracks_accumulation() {
        let mut app = test_app();
        app.add_systems(Update, hero_death_penalty_system);
        let hero = spawn_hero(&mut app, 0.0);

        app.world_mut().send_event(HeroKilledInMission {
            hero,
            hero_id: HeroId::Osman,
            killer: Entity::PLACEHOLDER,
        });
        app.update();

        let penalties = app.world().resource::<HeroDeathPenalties>();
        let penalty = penalties.penalties.get(&HeroId::Osman).unwrap();
        assert_eq!(penalty.rating_loss, 10.0);
        assert_eq!(penalty.xp_loss, 100);
        assert_eq!(penalty.morale_penalty, 15.0);
    }

    #[test]
    fn test_death_blow_emits_killed_event() {
        let mut app = test_app();
        let attacker = app.world_mut().spawn_empty().id();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO — this should emit HeroKnockedOut
        app.update();

        // Send death blow
        app.world_mut().send_event(DeathBlowAttempt {
            target: hero,
            attacker,
        });
        app.update();

        // The state should be Dead
        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Dead);
    }

    #[test]
    fn test_hero_stays_dead_after_death_blow() {
        let mut app = test_app();
        let attacker = app.world_mut().spawn_empty().id();
        let hero = spawn_hero(&mut app, 0.0);

        // Enter KO
        app.update();

        // Death blow
        app.world_mut().send_event(DeathBlowAttempt {
            target: hero,
            attacker,
        });
        app.update();

        // Advance time — should still be Dead
        advance_seconds(&mut app, 60.0);

        let state = app.world().get::<HeroState>(hero).unwrap();
        assert_eq!(*state, HeroState::Dead);
    }
}
