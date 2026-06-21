use bevy::prelude::*;
use kurulus_core::prelude::*;
use kurulus_systems::*;

/// Registers all plugins in the correct dependency order
pub fn register_plugins(app: &mut App) {
    // === CORE SETUP ===
    app
        // States (must be first)
        .init_state::<AppState>()
        .init_state::<GamePhase>()
        .init_state::<MissionState>()

        // Core resources
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>()
        .init_resource::<TickTimer>()
        .init_resource::<CoCommandState>()
        .init_resource::<TrustMatrix>()

        // Event channels
        .add_event::<DamageDealt>()
        .add_event::<UnitDied>()
        .add_event::<AttackTriggered>()
        .add_event::<MoraleBroken>()
        .add_event::<AmbushTriggered>()
        .add_event::<StealthRevealed>()
        .add_event::<MoveOrder>()
        .add_event::<AttackOrder>()
        .add_event::<BuildOrder>()
        .add_event::<RecruitOrder>()
        .add_event::<ResearchOrder>()
        .add_event::<AbilityUsed>()
        .add_event::<ObjectiveCompleted>()
        .add_event::<MissionEnded>()
        .add_event::<MissionStarted>()
        .add_event::<DialogueTriggered>()
        .add_event::<ObjectiveProgress>()
        .add_event::<FactionRelationChanged>()
        .add_event::<RegionCaptured>()
        .add_event::<WarDeclared>()
        .add_event::<TreatySigned>()
        .add_event::<SeasonChanged>()
        .add_event::<DayNightCycle>()
        .add_event::<SelectionChanged>()
        .add_event::<PanelOpened>()
        .add_event::<PanelClosed>()
        .add_event::<Notification>();

    // === LOGIC PLUGINS (Order matters for system dependencies) ===
    // 1. Core Loop — drives GamePhase transitions
    app.add_plugins(CoreLoopPlugin);

    // 2. World Map — provides region/faction context for all others
    app.add_plugins(WorldMapPlugin);

    // 3. AI — needs world context, provides threats
    app.add_plugins(AIPlugin);

    // 4. Economy — resource production/consumption
    app.add_plugins(EconomyPlugin);

    // 5. Unit Command — movement, formations, orders
    app.add_plugins(UnitCommandPlugin);

    // 6. Combat — needs units positioned, produces damage events
    app.add_plugins(CombatPlugin);

    // 7. Stealth — needs combat for ambush triggers
    app.add_plugins(StealthPlugin);

    // 8. Hero — abilities interact with combat/stealth
    app.add_plugins(HeroPlugin);

    // 9. Base Building — consumes economy, produces units
    app.add_plugins(BaseBuildingPlugin);

    // 10. Progression — XP, tech tree, sovereignty points
    app.add_plugins(ProgressionPlugin);

    // 11. Narrative — missions, objectives, dialogue triggers
    app.add_plugins(NarrativePlugin);

    // 12. Diplomacy — faction relations, treaties
    app.add_plugins(DiplomacyPlugin);

    // 13. Beylik Politics (Season 3+)
    app.add_plugins(BeylikPoliticsPlugin);

    // 14. Next-Gen Heroes (Season 5+)
    app.add_plugins(NextGenHeroesPlugin);

    // 15. Romance/Loyalty (Season 5+)
    app.add_plugins(RomanceLoyaltyPlugin);

    // 16. Succession Legacy (Season 5+)
    app.add_plugins(SuccessionLegacyPlugin);

    // 17. Siege Warfare (Season 6)
    app.add_plugins(SiegeWarfarePlugin);

    // 18. Imperial Admin (Season 6)
    app.add_plugins(ImperialAdminPlugin);

    // 19. Naval Warfare (Season 6)
    app.add_plugins(NavalWarfarePlugin);

    // 20. Heir Co-Rule (Season 6)
    app.add_plugins(CoRulePlugin);

    // 21. Audio Logic (event -> sound mapping)
    app.add_plugins(AudioLogicPlugin);

    // 22. UI Logic (data binding, no bevy_ui rendering)
    app.add_plugins(UILogicPlugin);

    // === RENDER PLUGIN (Conditional — Phase 1 = 2D, Phase 2 = 3D) ===
    #[cfg(feature = "render-2d")]
    app.add_plugins(kurulus_render_2d::Render2DPlugin);

    #[cfg(feature = "render-3d")]
    app.add_plugins(kurulus_render_3d::Render3DPlugin);

    // === HEADLESS TEST HARNESS ===
    #[cfg(feature = "headless-test")]
    app.add_plugins(headless_test::HeadlessTestPlugin);
}

/// Headless test harness — logic validation without any rendering
#[cfg(feature = "headless-test")]
pub mod headless_test {
    use bevy::prelude::*;
    use kurulus_core::prelude::*;
    use kurulus_systems::register_logic_plugins;

    pub struct HeadlessTestPlugin;

    impl Plugin for HeadlessTestPlugin {
        fn build(&self, app: &mut App) {
            app
                .add_plugins(MinimalPlugins)
                .add_plugins(register_logic_plugins)
                .init_resource::<TestScenario>()
                .add_systems(Startup, setup_test_scenario)
                .add_systems(FixedUpdate, run_test_assertions.in_set(SimulationSet::ComponentCleanup));
        }
    }

    #[derive(Resource, Default)]
    pub struct TestScenario {
        pub current_test: Option<TestCase>,
        pub results: Vec<TestResult>,
    }

    #[derive(Debug)]
    pub enum TestCase {
        MeleeCombat { attacker: Entity, defender: Entity, expected_damage: f32 },
        AmbushTrigger { ambusher: Entity, target: Entity, expected_bonus: f32 },
        FormationMovement { leader: Entity, followers: Vec<Entity>, target: Vec2 },
        SiegeBreach { engine: Entity, wall: Entity, hits_to_breach: u32 },
        ResourceProduction { building: Entity, expected_per_tick: f32 },
        HeroAbility { hero: Entity, ability: AbilityId, target: Entity },
    }

    #[derive(Debug)]
    pub struct TestResult {
        pub name: String,
        pub passed: bool,
        pub message: String,
    }

    fn setup_test_scenario(
        mut commands: Commands,
    ) {
        // Spawn test entities programmatically
        let attacker = commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            Stats {
                move_speed: 4.0,
                attack_range: 2.0,
                attack_damage: 25.0,
                attack_speed: 1.0,
                vision_range: 8.0,
                morale: 70.0,
                morale_max: 100.0,
            },
            Health {
                current: 150.0,
                max: 150.0,
                armor: 10.0,
                armor_type: ArmorType::Light,
            },
            MeleeWeapon {
                damage: 25.0,
                weapon_type: WeaponType::Sword,
                range: 1.5,
                armor_penetration: 0.2,
            },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        )).id();

        let defender = commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
            Stats {
                move_speed: 3.0,
                attack_range: 1.0,
                attack_damage: 15.0,
                attack_speed: 0.8,
                vision_range: 6.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 5.0,
                armor_type: ArmorType::Light,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(1.25, TimerMode::Repeating)),
        )).id();

        commands.insert_resource(TestScenario {
            current_test: Some(TestCase::MeleeCombat {
                attacker,
                defender,
                expected_damage: 20.0, // 25 - (5 * 0.5) = 22.5, but calc uses weapon.damage - armor*0.5
            }),
            results: Vec::new(),
        });
    }

    fn run_test_assertions(
        scenario: Res<TestScenario>,
        _query: Query<(&Health, &Stats, &WorldPosition)>,
        mut events: EventReader<DamageDealt>,
        mut results: Local<Vec<TestResult>>,
    ) {
        if let Some(test) = &scenario.current_test {
            match test {
                TestCase::MeleeCombat { attacker, defender, expected_damage } => {
                    for event in events.read() {
                        if event.attacker == *attacker && event.target == *defender {
                            let passed = (event.amount - expected_damage).abs() < 0.1;
                            results.push(TestResult {
                                name: "MeleeCombat".to_string(),
                                passed,
                                message: if passed {
                                    format!("✅ MeleeCombat: {:.1} damage dealt (expected {:.1})", event.amount, expected_damage)
                                } else {
                                    format!("❌ MeleeCombat: {:.1} damage dealt (expected {:.1})", event.amount, expected_damage)
                                },
                            });
                            for r in results.iter() {
                                bevy::utils::info!("{}", r.message);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
