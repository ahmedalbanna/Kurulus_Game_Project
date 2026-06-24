use bevy::prelude::*;
use kurulus_config::register_config_assets;
use kurulus_systems::*;

/// Registers all plugins in the correct dependency order
pub fn register_plugins(app: &mut App) {
    // === REMOTE CONTROL (BRP) — for MCP / external tooling ===
    #[cfg(feature = "remote")]
    {
        use bevy::remote::http::RemoteHttpPlugin;
        use bevy::remote::RemotePlugin;
        app.add_plugins(RemotePlugin::default())
            .add_plugins(RemoteHttpPlugin::default());
    }

    // === CORE INFRASTRUCTURE PLUGINS ===
    // App::new() starts with only MainSchedulePlugin. Add essential
    // infrastructure: task pools (IoTaskPool for asset I/O), time,
    // transforms, state management, and the asset server.
    app.add_plugins((
        bevy::core::TaskPoolPlugin::default(),
        bevy::core::TypeRegistrationPlugin,
        bevy::core::FrameCountPlugin,
        TransformPlugin,
        bevy::time::TimePlugin,
        // StatesPlugin is added later by register_logic_plugins
        AssetPlugin::default(),
    ));

    // === CONFIG ASSET LOADERS ===
    register_config_assets(app);

    // === ALL LOGIC (resources, events, plugins) via register_logic_plugins ===
    // This handles state initialization, resource setup, event registration,
    // and all 22 logic plugins in the correct order.
    register_logic_plugins(app);

    // === STARTUP: Spawn Mission 1 tutorial scenario ===
    app.add_systems(Startup, |mut commands: Commands| {
        narrative::setup_mission1_scenario(&mut commands);
    });

    // === RENDER PLUGIN (Conditional) ===
    // Phase 1 uses 2D sprite rendering; Phase 2 will use 3D PBR.
    // Each rendering path includes the necessary Bevy render infrastructure
    // (RenderPlugin, ImagePlugin, core render pipeline, text, UI) plus the
    // project-specific render plugins.
    #[cfg(feature = "render-2d")]
    app.add_plugins((
        // Window + input infrastructure
        bevy::window::WindowPlugin::default(),
        bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default(),
        bevy::input::InputPlugin,
        // Picking (required by sprite and UI picking backends)
        bevy::picking::PickingPlugin::default(),
        // Render pipeline
        bevy::render::RenderPlugin::default(),
        bevy::render::texture::ImagePlugin::default(),
        bevy::core_pipeline::CorePipelinePlugin,
        bevy::sprite::SpritePlugin::default(),
        bevy::text::TextPlugin,
        bevy::ui::UiPlugin::default(),
        bevy::gizmos::GizmoPlugin,
        // Project-specific 2D render
        kurulus_render_2d::Render2DPlugin,
    ));

    #[cfg(feature = "render-3d")]
    app.add_plugins((
        // Window + input infrastructure
        bevy::window::WindowPlugin::default(),
        bevy::winit::WinitPlugin::<bevy::winit::WakeUp>::default(),
        bevy::input::InputPlugin,
        // Picking (required by sprite and UI picking backends)
        bevy::picking::PickingPlugin::default(),
        // Render pipeline
        bevy::render::RenderPlugin::default(),
        bevy::render::texture::ImagePlugin::default(),
        bevy::core_pipeline::CorePipelinePlugin,
        bevy::pbr::PbrPlugin::default(),
        bevy::ui::UiPlugin::default(),
        // Project-specific 3D render
        kurulus_render_3d::Render3DPlugin,
    ));

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
            app.add_plugins(MinimalPlugins)
                .add_plugins(register_logic_plugins)
                .init_resource::<TestScenario>()
                .add_systems(Startup, setup_test_scenario)
                .add_systems(
                    FixedUpdate,
                    run_test_assertions.in_set(SimulationSet::ComponentCleanup),
                );
        }
    }

    #[derive(Resource, Default)]
    pub struct TestScenario {
        pub current_test: Option<TestCase>,
        pub results: Vec<TestResult>,
    }

    #[derive(Debug)]
    pub enum TestCase {
        MeleeCombat {
            attacker: Entity,
            defender: Entity,
            expected_damage: f32,
        },
        AmbushTrigger {
            ambusher: Entity,
            target: Entity,
            expected_bonus: f32,
        },
        FormationMovement {
            leader: Entity,
            followers: Vec<Entity>,
            target: Vec2,
        },
        SiegeBreach {
            engine: Entity,
            wall: Entity,
            hits_to_breach: u32,
        },
        ResourceProduction {
            building: Entity,
            expected_per_tick: f32,
        },
        HeroAbility {
            hero: Entity,
            ability: AbilityId,
            target: Entity,
        },
    }

    #[derive(Debug)]
    pub struct TestResult {
        pub name: String,
        pub passed: bool,
        pub message: String,
    }

    fn setup_test_scenario(mut commands: Commands) {
        // Spawn test entities programmatically
        let attacker = commands
            .spawn((
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
            ))
            .id();

        let defender = commands
            .spawn((
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
            ))
            .id();

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
                TestCase::MeleeCombat {
                    attacker,
                    defender,
                    expected_damage,
                } => {
                    for event in events.read() {
                        if event.attacker == *attacker && event.target == *defender {
                            let passed = (event.amount - expected_damage).abs() < 0.1;
                            results.push(TestResult {
                                name: "MeleeCombat".to_string(),
                                passed,
                                message: if passed {
                                    format!(
                                        "✅ MeleeCombat: {:.1} damage dealt (expected {:.1})",
                                        event.amount, expected_damage
                                    )
                                } else {
                                    format!(
                                        "❌ MeleeCombat: {:.1} damage dealt (expected {:.1})",
                                        event.amount, expected_damage
                                    )
                                },
                            });
                            for r in results.iter() {
                                info!("{}", r.message);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
