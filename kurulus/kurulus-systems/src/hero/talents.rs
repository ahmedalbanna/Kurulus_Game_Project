use bevy::prelude::*;
use kurulus_core::prelude::*;

use super::HeroLeveledUp;

// ─── Talent level thresholds ─────────────────────────────────────────────────

/// Level at which the first talent choice is offered (GDD §5).
const FIRST_TALENT_LEVEL: u8 = 6;

/// Level at which specialization path is chosen (GDD §5).
const SPECIALIZATION_LEVEL: u8 = 8;

/// Level at which the second talent choice is offered (GDD §5).
const SECOND_TALENT_LEVEL: u8 = 10;

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Watches for level-up events and requests talent/specialization choices
/// when the hero reaches the appropriate levels.
///
/// - Level 6: request first talent choice (OptionA vs OptionB)
/// - Level 8: request specialization path choice (3 paths per hero)
/// - Level 10: request second talent choice (OptionA vs OptionB)
pub fn hero_talent_request_system(
    mut level_up_events: EventReader<HeroLeveledUp>,
    mut talent_requests: EventWriter<TalentChoiceRequest>,
    mut spec_requests: EventWriter<SpecializationRequest>,
    heroes: Query<&Hero>,
) {
    for evt in level_up_events.read() {
        let Ok(hero) = heroes.get(evt.hero) else {
            continue;
        };
        let hero_id = hero.hero_id;
        let (opt_a, opt_b) = get_talent_option_descriptions(hero_id, evt.new_level);

        match evt.new_level {
            FIRST_TALENT_LEVEL => {
                talent_requests.send(TalentChoiceRequest {
                    hero: evt.hero,
                    hero_id,
                    level: evt.new_level,
                    option_a: opt_a,
                    option_b: opt_b,
                });
            }
            SPECIALIZATION_LEVEL => {
                let paths = hero_specialization_paths(hero_id);
                spec_requests.send(SpecializationRequest {
                    hero: evt.hero,
                    hero_id,
                    paths,
                });
            }
            SECOND_TALENT_LEVEL => {
                talent_requests.send(TalentChoiceRequest {
                    hero: evt.hero,
                    hero_id,
                    level: evt.new_level,
                    option_a: opt_a,
                    option_b: opt_b,
                });
            }
            _ => {}
        }
    }
}

/// Returns the available specialization paths for a given hero (GDD §5).
fn hero_specialization_paths(hero_id: HeroId) -> Vec<SpecializationPath> {
    match hero_id {
        HeroId::Osman => vec![
            SpecializationPath::Conquest,
            SpecializationPath::Sovereignty,
            SpecializationPath::Faith,
        ],
        HeroId::Turgut => vec![
            SpecializationPath::Mountain,
            SpecializationPath::Storm,
            SpecializationPath::Guardian,
        ],
        HeroId::Buran => vec![
            SpecializationPath::Fire,
            SpecializationPath::Shadow,
            SpecializationPath::Eye,
        ],
        _ => vec![
            SpecializationPath::Conquest,
            SpecializationPath::Sovereignty,
            SpecializationPath::Faith,
        ],
    }
}

/// Returns human-readable descriptions for each specialization path.
pub fn specialization_description(path: SpecializationPath) -> &'static str {
    match path {
        SpecializationPath::Conquest => "Offensive powerhouse — +25% damage, +15% attack speed",
        SpecializationPath::Sovereignty => {
            "Balanced leadership — +10% to all stats, +1 resource regen"
        }
        SpecializationPath::Faith => "Defensive resilience — -20% damage taken, +30% max HP",
        SpecializationPath::Mountain => "Unstoppable force — +30% defense, +40% max HP",
        SpecializationPath::Storm => "Relentless assault — +20% attack speed, chain attacks",
        SpecializationPath::Guardian => "Immovable protector — AoE damage reduction, taunt on hit",
        SpecializationPath::Fire => "Burning devastation — +50% fire DoT, spread on kill",
        SpecializationPath::Shadow => "Elusive predator — permanent stealth, +100% backstab",
        SpecializationPath::Eye => "Perfect accuracy — +50% range, +30% crit chance",
    }
}

/// Processes `TalentChoiceSelected` events from the UI and stores
/// the choice in the hero's component.
pub fn hero_talent_apply_system(
    mut talent_events: EventReader<TalentChoiceSelected>,
    mut heroes: Query<&mut Hero>,
) {
    for evt in talent_events.read() {
        if let Ok(mut hero) = heroes.get_mut(evt.hero) {
            hero.talent_choices.insert(evt.level, evt.choice);
        }
    }
}

/// Processes `SpecializationSelected` events from the UI and stores
/// the chosen path in the hero's component.
/// Applies immediate stat bonuses based on the chosen specialization.
pub fn hero_specialization_apply_system(
    mut spec_events: EventReader<SpecializationSelected>,
    mut heroes: Query<(&mut Hero, &mut Stats, &mut Health)>,
) {
    for evt in spec_events.read() {
        if let Ok((mut hero, mut stats, mut health)) = heroes.get_mut(evt.hero) {
            hero.specialization = Some(evt.path);
            apply_specialization_bonus(evt.path, &mut stats, &mut health);
        }
    }
}

/// Applies the stat bonuses for a specialization path immediately.
fn apply_specialization_bonus(path: SpecializationPath, stats: &mut Stats, health: &mut Health) {
    match path {
        SpecializationPath::Conquest => {
            stats.attack_damage *= 1.25;
            stats.attack_speed *= 1.15;
        }
        SpecializationPath::Sovereignty => {
            stats.attack_damage *= 1.10;
            stats.attack_speed *= 1.10;
            stats.move_speed *= 1.10;
            stats.vision_range *= 1.10;
        }
        SpecializationPath::Faith => {
            health.max *= 1.30;
            health.current = health.current.max(health.max * 0.3);
        }
        SpecializationPath::Mountain => {
            health.max *= 1.40;
            health.current = health.current.max(health.max * 0.3);
            health.armor *= 1.30;
        }
        SpecializationPath::Storm => {
            stats.attack_speed *= 1.20;
        }
        SpecializationPath::Guardian => {
            health.max *= 1.20;
            health.armor *= 1.15;
        }
        SpecializationPath::Fire => {
            stats.attack_damage *= 1.15;
        }
        SpecializationPath::Shadow => {
            stats.move_speed *= 1.15;
        }
        SpecializationPath::Eye => {
            stats.vision_range *= 1.50;
            stats.attack_range *= 1.50;
        }
    }
}

/// Returns the talent option descriptions for a given hero+level combination.
/// These should be displayed by the UI when `TalentChoiceRequest` fires.
pub fn get_talent_option_descriptions(hero_id: HeroId, level: u8) -> (String, String) {
    match (hero_id, level) {
        // Osman
        (HeroId::Osman, 6) => (
            "Wider WarCry: +50% radius, +25% duration".into(),
            "CallOfTheTribe summons 8 alps instead of 5, +30s duration".into(),
        ),
        (HeroId::Osman, 10) => (
            "Lionheart triggers at 40% HP instead of 30%, +10% damage reduction".into(),
            "DreamOfTheTree also reveals invisible units for the duration".into(),
        ),
        // Turgut
        (HeroId::Turgut, 6) => (
            "GroundStrike: +50% damage, knocks back 3 cells".into(),
            "EarthquakeChallenge: +1 cell radius, -2s cooldown".into(),
        ),
        (HeroId::Turgut, 10) => (
            "IronWall: +10% damage reduction per nearby ally (max +50%)".into(),
            "BullsWrath enrage duration +6s, attack speed bonus +50%".into(),
        ),
        // Buran
        (HeroId::Buran, 6) => (
            "FlamingArrow: +50% fire DoT, spreads to nearby enemies".into(),
            "BlindingShot: also reveals target for 10s".into(),
        ),
        (HeroId::Buran, 10) => (
            "ShadowLightness: +10% move speed, stealth after 2s still".into(),
            "DeathVolley: 30 arrows instead of 20, +20% damage per arrow".into(),
        ),
        // Fallback
        _ => ("Option A".into(), "Option B".into()),
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
            .add_event::<HeroLeveledUp>()
            .add_event::<TalentChoiceRequest>()
            .add_event::<SpecializationRequest>()
            .add_event::<TalentChoiceSelected>()
            .add_event::<SpecializationSelected>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(
                Update,
                (
                    hero_talent_request_system,
                    hero_talent_apply_system,
                    hero_specialization_apply_system,
                ),
            );
        app
    }

    #[test]
    fn test_talent_request_fires_at_level_6() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();

        app.world_mut()
            .send_event(HeroLeveledUp { hero, new_level: 6 });
        app.update();

        let events = app.world().resource::<Events<TalentChoiceRequest>>();
        let mut reader = events.get_cursor();
        let count = reader.read(&events).count();
        assert_eq!(count, 1, "Expected 1 talent choice request at level 6");
    }

    #[test]
    fn test_specialization_request_fires_at_level_8() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();

        app.world_mut()
            .send_event(HeroLeveledUp { hero, new_level: 8 });
        app.update();

        let events = app.world().resource::<Events<SpecializationRequest>>();
        let mut reader = events.get_cursor();
        let count = reader.read(&events).count();
        assert_eq!(count, 1, "Expected 1 specialization request at level 8");
    }

    #[test]
    fn test_talent_request_fires_at_level_10() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();

        app.world_mut().send_event(HeroLeveledUp {
            hero,
            new_level: 10,
        });
        app.update();

        let events = app.world().resource::<Events<TalentChoiceRequest>>();
        let mut reader = events.get_cursor();
        let count = reader.read(&events).count();
        assert_eq!(count, 1, "Expected 1 talent choice request at level 10");
    }

    #[test]
    fn test_no_talent_request_at_other_levels() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();

        app.world_mut()
            .send_event(HeroLeveledUp { hero, new_level: 4 });
        app.update();

        let talent_events = app.world().resource::<Events<TalentChoiceRequest>>();
        let spec_events = app.world().resource::<Events<SpecializationRequest>>();
        assert!(talent_events
            .get_cursor()
            .read(&talent_events)
            .next()
            .is_none());
        assert!(spec_events.get_cursor().read(&spec_events).next().is_none());
    }

    #[test]
    fn test_talent_choice_stored_on_hero() {
        let mut app = test_app();
        let hero = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();

        app.world_mut().send_event(TalentChoiceSelected {
            hero,
            level: 6,
            choice: TalentChoice::OptionA,
        });
        app.update();

        let hero_comp = app.world().get::<Hero>(hero).unwrap();
        assert_eq!(
            hero_comp.talent_choices.get(&6),
            Some(&TalentChoice::OptionA)
        );
    }

    #[test]
    fn test_specialization_stored_on_hero() {
        let mut app = test_app();
        let hero = app
            .world_mut()
            .spawn((
                Hero::new(HeroId::Osman),
                Stats::default(),
                Health::default(),
            ))
            .id();

        app.world_mut().send_event(SpecializationSelected {
            hero,
            path: SpecializationPath::Conquest,
        });
        app.update();

        let hero_comp = app.world().get::<Hero>(hero).unwrap();
        assert_eq!(hero_comp.specialization, Some(SpecializationPath::Conquest));
    }

    #[test]
    fn test_specialization_boost_applied() {
        let mut app = test_app();
        let hero = app
            .world_mut()
            .spawn((
                Hero::new(HeroId::Osman),
                Stats::default(),
                Health::default(),
            ))
            .id();

        let base_damage = app.world().get::<Stats>(hero).unwrap().attack_damage;

        app.world_mut().send_event(SpecializationSelected {
            hero,
            path: SpecializationPath::Conquest,
        });
        app.update();

        let stats = app.world().get::<Stats>(hero).unwrap();
        assert!(
            (stats.attack_damage - base_damage * 1.25).abs() < 0.001,
            "Expected damage {} * 1.25 = {}, got {}",
            base_damage,
            base_damage * 1.25,
            stats.attack_damage
        );
    }

    #[test]
    fn test_osman_talent_descriptions() {
        let (a1, b1) = get_talent_option_descriptions(HeroId::Osman, 6);
        assert!(a1.contains("WarCry"), "Expected WarCry talent, got: {}", a1);
        assert!(
            b1.contains("CallOfTheTribe"),
            "Expected CallOfTheTribe talent, got: {}",
            b1
        );

        let (a2, b2) = get_talent_option_descriptions(HeroId::Osman, 10);
        assert!(
            a2.contains("Lionheart"),
            "Expected Lionheart talent, got: {}",
            a2
        );
        assert!(
            b2.contains("DreamOfTheTree"),
            "Expected DreamOfTheTree talent, got: {}",
            b2
        );
    }

    #[test]
    fn test_turgut_specialization_paths() {
        let paths = hero_specialization_paths(HeroId::Turgut);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&SpecializationPath::Mountain));
        assert!(paths.contains(&SpecializationPath::Storm));
        assert!(paths.contains(&SpecializationPath::Guardian));
    }

    #[test]
    fn test_buran_specialization_paths() {
        let paths = hero_specialization_paths(HeroId::Buran);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&SpecializationPath::Fire));
        assert!(paths.contains(&SpecializationPath::Shadow));
        assert!(paths.contains(&SpecializationPath::Eye));
    }

    #[test]
    fn test_osman_specialization_paths() {
        let paths = hero_specialization_paths(HeroId::Osman);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&SpecializationPath::Conquest));
        assert!(paths.contains(&SpecializationPath::Sovereignty));
        assert!(paths.contains(&SpecializationPath::Faith));
    }

    #[test]
    fn test_specialization_descriptions_exist() {
        let all_paths = [
            SpecializationPath::Conquest,
            SpecializationPath::Sovereignty,
            SpecializationPath::Faith,
            SpecializationPath::Mountain,
            SpecializationPath::Storm,
            SpecializationPath::Guardian,
            SpecializationPath::Fire,
            SpecializationPath::Shadow,
            SpecializationPath::Eye,
        ];
        for path in &all_paths {
            let desc = specialization_description(*path);
            assert!(!desc.is_empty(), "Description missing for {:?}", path);
        }
    }
}
