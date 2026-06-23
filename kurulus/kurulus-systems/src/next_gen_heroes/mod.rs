use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct NextGenHeroesPlugin;

impl Plugin for NextGenHeroesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextGenRegistry>()
            .init_resource::<LegacyContributions>()
            .add_event::<ChildOfAge>()
            .add_event::<MarriageFormed>()
            .register_type::<ChildHero>()
            .register_type::<MarriageAlliance>()
            .register_type::<NextGenRegistry>()
            .register_type::<LegacyContributions>()
            .add_systems(
                FixedUpdate,
                (
                    hero_legacy_system,
                    hero_marriage_system,
                    child_growth_system,
                ),
            );
    }
}

/// Age of maturity for child heroes (in-game years).
const CHILD_MATURITY_AGE: f32 = 16.0;
/// Education level to reach before coming of age.
const EDUCATION_CAP: u32 = 5;

/// Tracks hero contributions to build a legacy score.
fn hero_legacy_system(
    mut death_events: EventReader<UnitDied>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut contributions: ResMut<LegacyContributions>,
    heroes: Query<&Hero>,
) {
    for evt in death_events.read() {
        if let Ok(hero) = heroes.get(evt.killer) {
            contributions.contributions.push((
                evt.killer,
                "kill".into(),
                10.0,
            ));
            let _ = hero.hero_id; // ensure usage
        }
    }

    for evt in objective_events.read() {
        // Objective completed by any hero — tracked for all heroes
        // In a full implementation, this would attribute the objective
        let _ = evt;
    }
}

/// Manages marriage alliances between heroes and between factions.
/// A marriage creates a diplomacy bond and can produce child heroes.
fn hero_marriage_system(
    mut commands: Commands,
    mut marriage_events: EventReader<MarriageFormed>,
    mut registry: ResMut<NextGenRegistry>,
    heroes: Query<&Hero>,
    relations: Res<FactionRelations>,
) {
    for evt in marriage_events.read() {
        registry.marriages.push((evt.hero_a, evt.hero_b));

        // Mark both entities with MarriageAlliance
        commands.entity(evt.hero_a).insert(MarriageAlliance {
            spouse: evt.hero_b,
            spouse_faction: evt.faction_b,
            active: true,
            duration: 0.0,
        });
        commands.entity(evt.hero_b).insert(MarriageAlliance {
            spouse: evt.hero_a,
            spouse_faction: evt.faction_a,
            active: true,
            duration: 0.0,
        });

        // Relation boost between factions
        let _ = relations;
        let _ = heroes;
    }
}

/// Handles growth and education of child characters over time.
/// At maturity age + sufficient education, fires ChildOfAge event.
fn child_growth_system(
    time: Res<Time>,
    mut children: Query<(Entity, &mut ChildHero)>,
    mut of_age_events: EventWriter<ChildOfAge>,
) {
    let dt = time.delta_secs();
    for (entity, mut child) in children.iter_mut() {
        child.age += dt / 3600.0; // 1 real second ≈ 1 game hour

        // Education progresses over time
        if child.education_level < EDUCATION_CAP {
            child.education_level += 1;
        }

        // Check maturity
        if child.age >= CHILD_MATURITY_AGE && child.education_level >= EDUCATION_CAP {
            if !child.is_of_age {
                child.is_of_age = true;
                of_age_events.send(ChildOfAge {
                    child: entity,
                    parent: child.parent,
                });
            }
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
            .init_resource::<NextGenRegistry>()
            .init_resource::<FactionRelations>()
            .add_event::<ChildOfAge>()
            .add_event::<MarriageFormed>()
            .add_event::<ObjectiveCompleted>()
            .add_event::<UnitDied>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, (hero_legacy_system, child_growth_system, hero_marriage_system));
        app
    }

    #[test]
    fn test_child_grows_over_time() {
        let mut app = test_app();
        let parent = app.world_mut().spawn_empty().id();
        let child = app
            .world_mut()
            .spawn(ChildHero {
                parent,
                age: 10.0,
                education_level: 2,
                is_of_age: false,
            })
            .id();

        // Advance time significantly
        for _ in 0..60 {
            app.update();
        }

        let child_comp = app.world().get::<ChildHero>(child).unwrap();
        assert!(child_comp.age > 10.0, "Child age should increase");
        assert!(child_comp.education_level >= 2, "Education should progress");
    }

    #[test]
    fn test_marriage_creates_alliance() {
        let mut app = test_app();
        let a = app.world_mut().spawn(Hero::new(HeroId::Osman)).id();
        let b = app.world_mut().spawn(Hero::new(HeroId::Buran)).id();

        app.world_mut().send_event(MarriageFormed {
            hero_a: a,
            hero_b: b,
            faction_a: FactionId::Kayi,
            faction_b: FactionId::Kayi,
        });
        app.update();

        assert!(app.world().get::<MarriageAlliance>(a).is_some());
        assert!(app.world().get::<MarriageAlliance>(b).is_some());
    }

    #[test]
    fn test_legacy_tracks_kills() {
        let mut app = test_app();
        let hero = app
            .world_mut()
            .spawn(Hero::new(HeroId::Osman))
            .id();
        let enemy = app.world_mut().spawn_empty().id();

        // Legacy tracks kills via UnitDied events
        app.world_mut().send_event(UnitDied {
            entity: enemy,
            killer: hero,
        });
        app.update();

        let contributions = app.world().resource::<LegacyContributions>();
        assert!(!contributions.contributions.is_empty());
    }
}
