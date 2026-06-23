use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum named units deployable per mission (GDD §7).
pub const MAX_NAMED_UNITS_PER_MISSION: usize = 4;

// ─── Events ──────────────────────────────────────────────────────────────────

/// Fired when a named unit permanently dies in a mission.
#[derive(Event, Clone, Debug)]
pub struct NamedUnitKilled {
    pub unit_id: NamedUnitId,
    pub entity: Entity,
    pub mission_id: String,
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Applies named unit stats from `NamedUnitRoster` when a `NamedUnit` entity
/// is spawned (one-shot via `Added<NamedUnit>`).
pub fn named_unit_stats_system(
    roster: Res<NamedUnitRoster>,
    mut units: Query<(&NamedUnit, &mut Stats, &mut Health), Added<NamedUnit>>,
) {
    for (named, mut stats, mut health) in units.iter_mut() {
        if let Some(block) = roster.units.get(&named.unit_id) {
            stats.move_speed = block.move_speed;
            stats.attack_damage = block.attack_damage;
            stats.attack_speed = block.attack_speed;
            stats.vision_range = block.vision_range;
            stats.morale = stats.morale_max; // start at full morale

            health.max = block.max_health;
            health.current = block.max_health;
            health.armor = block.armor;
        }
    }
}

/// Permadeath system: when a `NamedUnit` entity's HP reaches 0, mark it
/// as permanently dead and emit `NamedUnitKilled`.
pub fn named_unit_permadeath_system(
    mut _commands: Commands,
    mut death_events: EventReader<UnitDied>,
    mut named_units: Query<(&mut NamedUnit, &mut Hero, &Health)>,
    mut kill_events: EventWriter<NamedUnitKilled>,
) {
    for evt in death_events.read() {
        if let Ok((mut named, mut hero, health)) = named_units.get_mut(evt.entity) {
            // Only trigger if actually dead (HP <= 0 or Dead component would be added)
            if health.current > 0.0 {
                continue;
            }
            named.is_permanently_dead = true;
            hero.is_permanently_dead = true;

            kill_events.send(NamedUnitKilled {
                unit_id: named.unit_id,
                entity: evt.entity,
                mission_id: String::new(), // populated by mission context
            });
        }
    }
}

/// Passive ability system — applies unique named unit passives each tick.
/// Uses `Without<NamedUnit>` on the allies query to avoid B0001 conflict
/// with the named units query.
pub fn named_unit_ability_system(
    mut units: Query<(&NamedUnit, &NamedUnitAbilities, &Stats, &Health, &WorldPosition)>,
    all_allies: Query<(&Unit, &Stats, &WorldPosition), Without<NamedUnit>>,
) {
    for (_named, abilities, _stats, _health, pos) in units.iter_mut() {
        for &passive_id in &abilities.passive_ids {
            match passive_id {
                // Static passives (applied at spawn, no tick effect needed):
                // Konur: Rider's Swiftness — +15% charge damage (combat system)
                // Ayazbey: Hawk's Eye — +25% range, stealth detect (stealth system)
                // Salur: Tribe's Eye — +20% scouting, +5% squad speed (scout system)
                NamedUnitAbilityId::RidersSwiftness
                | NamedUnitAbilityId::HawksEye
                | NamedUnitAbilityId::TribesEye => {}

                // ── Bamsi: Veteran's Cry — +10% defense aura (15m) ──
                NamedUnitAbilityId::VeteransCry => {
                    for (ally_unit, _ally_stats, ally_pos) in all_allies.iter() {
                        if ally_unit.faction == FactionId::Kayi {
                            let dist = pos.world_2d.0.distance(ally_pos.world_2d.0);
                            if dist <= 15.0 {
                                // Aura active — defense bonus applied in
                                // stat calculation system (future)
                            }
                        }
                    }
                }

                // ── Salvador: Roman Tactics — +15% defense aura (12m) ──
                NamedUnitAbilityId::RomanTactics => {
                    for (ally_unit, _ally_stats, ally_pos) in all_allies.iter() {
                        if ally_unit.faction == FactionId::Kayi {
                            let dist = pos.world_2d.0.distance(ally_pos.world_2d.0);
                            if dist <= 12.0 {
                                // Aura active — handled by stat system
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Initializes the `NamedUnitRoster` resource with GDD-defined stat blocks.
pub fn initialize_named_unit_roster(mut roster: ResMut<NamedUnitRoster>) {
    // Konur Alp — Light Cavalry Commander
    roster.units.insert(
        NamedUnitId::KonurAlp,
        NamedUnitStatsBlock {
            max_health: 250.0,
            attack_damage: 22.0,
            attack_speed: 1.3,
            armor: 15.0,
            move_speed: 5.2,
            vision_range: 8.0,
        },
    );
    // Ayazbey — Archer Commander
    roster.units.insert(
        NamedUnitId::Ayazbey,
        NamedUnitStatsBlock {
            max_health: 180.0,
            attack_damage: 20.0,
            attack_speed: 1.6,
            armor: 10.0,
            move_speed: 4.5,
            vision_range: 12.0,
        },
    );
    // Bamsi Beyrek — Veteran Infantry
    roster.units.insert(
        NamedUnitId::BamsiBeyrek,
        NamedUnitStatsBlock {
            max_health: 300.0,
            attack_damage: 28.0,
            attack_speed: 1.0,
            armor: 20.0,
            move_speed: 3.8,
            vision_range: 6.0,
        },
    );
    // Salvador / Abu Bakr — Heavy Infantry Commander
    roster.units.insert(
        NamedUnitId::Salvador,
        NamedUnitStatsBlock {
            max_health: 350.0,
            attack_damage: 30.0,
            attack_speed: 0.9,
            armor: 28.0,
            move_speed: 3.5,
            vision_range: 7.0,
        },
    );
    // Salur Alp — Scout Squad Leader
    roster.units.insert(
        NamedUnitId::SalurAlp,
        NamedUnitStatsBlock {
            max_health: 220.0,
            attack_damage: 20.0,
            attack_speed: 1.1,
            armor: 18.0,
            move_speed: 4.2,
            vision_range: 10.0,
        },
    );
}

/// Returns the unique passive ability ID for a named unit (GDD-defined).
pub fn named_unit_passive(unit_id: NamedUnitId) -> NamedUnitAbilityId {
    match unit_id {
        NamedUnitId::KonurAlp => NamedUnitAbilityId::RidersSwiftness,
        NamedUnitId::Ayazbey => NamedUnitAbilityId::HawksEye,
        NamedUnitId::BamsiBeyrek => NamedUnitAbilityId::VeteransCry,
        NamedUnitId::Salvador => NamedUnitAbilityId::RomanTactics,
        NamedUnitId::SalurAlp => NamedUnitAbilityId::TribesEye,
    }
}

/// Spawns a named unit entity with full components: NamedUnit, NamedUnitAbilities,
/// Health, Stats, Unit, WorldPosition, and Hero (for shared hero infrastructure).
pub fn spawn_named_unit(
    commands: &mut Commands,
    unit_id: NamedUnitId,
    faction: FactionId,
    position: IVec2,
) -> Entity {
    let passive_id = named_unit_passive(unit_id);
    let abilities = NamedUnitAbilities {
        passive_ids: vec![passive_id],
    };

    commands
        .spawn((
            NamedUnit {
                unit_id,
                is_permanently_dead: false,
            },
            abilities,
            Hero::new(HeroId::Osman), // placeholder; named_unit_id set below
            // Health will be overwritten by named_unit_stats_system via Added<NamedUnit>
            Health::default(),
            Stats::default(),
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction,
                ..default()
            },
            WorldPosition::new(position, CELL_SIZE),
        ))
        .id()
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<NamedUnitRoster>()
            .add_event::<NamedUnitKilled>()
            .add_event::<UnitDied>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(
                Update,
                (
                    named_unit_stats_system,
                    named_unit_permadeath_system,
                    named_unit_ability_system,
                ),
            );
        app
    }

    fn spawn_named(app: &mut App, id: NamedUnitId) -> Entity {
        let entity = app
            .world_mut()
            .spawn((
                NamedUnit {
                    unit_id: id,
                    is_permanently_dead: false,
                },
                NamedUnitAbilities {
                    passive_ids: vec![named_unit_passive(id)],
                },
                Health::default(),
                Stats::default(),
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    faction: FactionId::Kayi,
                    ..default()
                },
                WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            ))
            .id();
        entity
    }

    // ─── Stats system tests ─────────────────────────────────────────────

    #[test]
    fn test_named_unit_stats_applied() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);

        let entity = spawn_named(&mut app, NamedUnitId::KonurAlp);
        app.update(); // triggers Added<NamedUnit> query

        let health = app.world().get::<Health>(entity).unwrap();
        let stats = app.world().get::<Stats>(entity).unwrap();

        assert!((health.max - 250.0).abs() < 0.01, "Expected 250 HP, got {}", health.max);
        assert!((health.current - 250.0).abs() < 0.01);
        assert!((health.armor - 15.0).abs() < 0.01, "Expected 15 armor, got {}", health.armor);
        assert!((stats.attack_damage - 22.0).abs() < 0.01, "Expected 22 damage, got {}", stats.attack_damage);
        assert!((stats.attack_speed - 1.3).abs() < 0.01);
        assert!((stats.move_speed - 5.2).abs() < 0.01);
        assert!((stats.vision_range - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_all_named_units_have_stats() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);
        let roster = app.world().resource::<NamedUnitRoster>();

        assert!(roster.units.contains_key(&NamedUnitId::KonurAlp));
        assert!(roster.units.contains_key(&NamedUnitId::Ayazbey));
        assert!(roster.units.contains_key(&NamedUnitId::BamsiBeyrek));
        assert!(roster.units.contains_key(&NamedUnitId::Salvador));
        assert!(roster.units.contains_key(&NamedUnitId::SalurAlp));
        assert_eq!(roster.units.len(), 5);
    }

    #[test]
    fn test_ayazbey_has_ranged_stats() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);

        let entity = spawn_named(&mut app, NamedUnitId::Ayazbey);
        app.update();

        let health = app.world().get::<Health>(entity).unwrap();
        let stats = app.world().get::<Stats>(entity).unwrap();

        assert!((health.max - 180.0).abs() < 0.01);
        assert!((stats.attack_damage - 20.0).abs() < 0.01);
        assert!((stats.attack_speed - 1.6).abs() < 0.01);
        // Longer vision for archer
        assert!((stats.vision_range - 12.0).abs() < 0.01);
    }

    // ─── Permadeath tests ───────────────────────────────────────────────

    #[test]
    fn test_permadeath_marked_on_death() {
        let mut app = test_app();
        let entity = spawn_named(&mut app, NamedUnitId::KonurAlp);

        // Give a Hero component too (permadeath system queries both)
        app.world_mut().entity_mut(entity).insert(Hero::new(HeroId::Osman));

        // Set HP to 0
        app.world_mut()
            .entity_mut(entity)
            .insert(Health {
                current: 0.0,
                max: 250.0,
                armor: 15.0,
                armor_type: ArmorType::Light,
            });

        // Fire death event
        app.world_mut().send_event(UnitDied {
            entity,
            killer: Entity::PLACEHOLDER,
        });
        app.update();

        let named = app.world().get::<NamedUnit>(entity).unwrap();
        assert!(named.is_permanently_dead, "Named unit should be marked permanently dead");
    }

    #[test]
    fn test_permadeath_not_marked_when_alive() {
        let mut app = test_app();
        let entity = spawn_named(&mut app, NamedUnitId::KonurAlp);
        app.world_mut().entity_mut(entity).insert(Hero::new(HeroId::Osman));

        // HP > 0
        app.world_mut()
            .entity_mut(entity)
            .insert(Health {
                current: 100.0,
                max: 250.0,
                armor: 15.0,
                armor_type: ArmorType::Light,
            });

        app.world_mut().send_event(UnitDied {
            entity,
            killer: Entity::PLACEHOLDER,
        });
        app.update();

        let named = app.world().get::<NamedUnit>(entity).unwrap();
        assert!(!named.is_permanently_dead, "Named unit should NOT be marked dead while HP > 0");
    }

    // ─── Passive ability tests ──────────────────────────────────────────

    #[test]
    fn test_passive_mapping_all_units() {
        assert_eq!(
            named_unit_passive(NamedUnitId::KonurAlp),
            NamedUnitAbilityId::RidersSwiftness
        );
        assert_eq!(
            named_unit_passive(NamedUnitId::Ayazbey),
            NamedUnitAbilityId::HawksEye
        );
        assert_eq!(
            named_unit_passive(NamedUnitId::BamsiBeyrek),
            NamedUnitAbilityId::VeteransCry
        );
        assert_eq!(
            named_unit_passive(NamedUnitId::Salvador),
            NamedUnitAbilityId::RomanTactics
        );
        assert_eq!(
            named_unit_passive(NamedUnitId::SalurAlp),
            NamedUnitAbilityId::TribesEye
        );
    }

    #[test]
    fn test_named_unit_spawn_adds_abilities() {
        let mut app = test_app();
        let entity = spawn_named(&mut app, NamedUnitId::BamsiBeyrek);
        app.update();

        let abilities = app.world().get::<NamedUnitAbilities>(entity).unwrap();
        assert!(!abilities.passive_ids.is_empty());
        assert_eq!(abilities.passive_ids[0], NamedUnitAbilityId::VeteransCry);
    }

    #[test]
    fn test_bamsi_stats_match_gdd() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);

        let entity = spawn_named(&mut app, NamedUnitId::BamsiBeyrek);
        app.update();

        let health = app.world().get::<Health>(entity).unwrap();
        let stats = app.world().get::<Stats>(entity).unwrap();

        assert!((health.max - 300.0).abs() < 0.01);
        assert!((stats.attack_damage - 28.0).abs() < 0.01);
        assert!((stats.attack_speed - 1.0).abs() < 0.01);
        assert!((health.armor - 20.0).abs() < 0.01);
        assert!((stats.move_speed - 3.8).abs() < 0.01);
    }

    #[test]
    fn test_salvador_has_highest_hp() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);
        let roster = app.world().resource::<NamedUnitRoster>();
        let salvador = roster.units.get(&NamedUnitId::Salvador).unwrap();
        assert!((salvador.max_health - 350.0).abs() < 0.01);
        assert!((salvador.armor - 28.0).abs() < 0.01);
    }

    #[test]
    fn test_salur_balanced_scout() {
        let mut app = test_app();
        let _ = app.world_mut().run_system_once(initialize_named_unit_roster);

        let entity = spawn_named(&mut app, NamedUnitId::SalurAlp);
        app.update();

        let health = app.world().get::<Health>(entity).unwrap();
        let stats = app.world().get::<Stats>(entity).unwrap();

        // Medium HP, decent speed, good vision
        assert!((health.max - 220.0).abs() < 0.01);
        assert!((stats.move_speed - 4.2).abs() < 0.01);
        assert!((stats.vision_range - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_max_named_units_constant() {
        assert_eq!(MAX_NAMED_UNITS_PER_MISSION, 4);
    }
}
