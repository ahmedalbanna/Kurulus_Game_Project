//! Unit Stats Panel — displays Damage, Defense, Speed for the selected unit
//! along with a battle cry / flavour text.
//!
//! ## Architecture
//!
//! 1. **`unit_stats_logic_system`** (FixedUpdate) — reads selected entity's
//!    `Stats` and `Health` components, populates `UnitStatsState`.
//!
//! 2. **`unit_stats_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`UnitStatsRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for stats panel text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 6.0, 6.0);

/// Background colour (dark brown).
const BG_COLOR: Color = Color::srgba(0.15, 0.12, 0.10, 0.85);

/// Text colour.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads selected entity's Stats + Health, updates UnitStatsState.
pub fn unit_stats_logic_system(
    selection: Res<SelectionState>,
    stats_query: Query<&Stats>,
    health_query: Query<&Health>,
    unit_query: Query<&Unit>,
    mut stats_state: ResMut<UnitStatsState>,
) {
    stats_state.damage = 0;
    stats_state.defense = 0;
    stats_state.speed = 0;
    stats_state.selected_unit_name = None;

    if let Some(&entity) = selection.selected.first() {
        // Read stats for damage and speed
        if let Ok(stats) = stats_query.get(entity) {
            stats_state.damage = stats.attack_damage as u32;
            stats_state.speed = stats.move_speed as u32;
        }
        // Read health for armor/defense
        if let Ok(health) = health_query.get(entity) {
            stats_state.defense = health.armor as u32;
        }
        // Read unit archetype for display name
        if let Ok(unit) = unit_query.get(entity) {
            stats_state.selected_unit_name = Some(archetype_label(&unit.archetype).to_string());
        }
    }
}

/// Display name for each unit archetype.
fn archetype_label(archetype: &UnitArchetype) -> &'static str {
    match archetype {
        UnitArchetype::LightInfantry => "Light Infantry",
        UnitArchetype::HeavyInfantry => "Heavy Infantry",
        UnitArchetype::Archer => "Archer",
        UnitArchetype::LightCavalry => "Light Cavalry",
        UnitArchetype::HeavyCavalry => "Heavy Cavalry",
        UnitArchetype::Scout => "Scout",
        UnitArchetype::BaitRider => "Bait Rider",
        UnitArchetype::SiegeCrew => "Siege Crew",
        UnitArchetype::Worker => "Worker",
    }
}

/// Render system: spawns / updates the unit stats panel UI entity.
pub fn unit_stats_render_system(
    mut commands: Commands,
    stats_state: Res<UnitStatsState>,
    root_query: Query<Entity, With<UnitStatsRoot>>,
    layout: Res<MainScreenLayout>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(layout.padding + layout.action_bar_height + layout.padding),
                left: Val::Px(layout.padding),
                width: Val::Px(layout.unit_stats_width),
                height: Val::Px(layout.unit_stats_height),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: PANEL_PADDING,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Text::new(""),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            UnitStatsRoot,
            Name::new("UnitStats"),
        ));
        return;
    };

    let body = build_unit_stats_text(&stats_state);
    commands.entity(entity).insert(Text::new(&body));
}

fn build_unit_stats_text(state: &UnitStatsState) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Battle cry / flavour text
    lines.push(state.battle_cry.clone());

    // Unit name if available
    if let Some(name) = &state.selected_unit_name {
        lines.push(format!("[ {} ]", name));
    }

    // Stats line
    lines.push(format!(
        "Damage: {}  Defense: {}  Speed: {}",
        state.damage, state.defense, state.speed
    ));

    lines.join("\n")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<SelectionState>()
            .init_resource::<UnitStatsState>()
            .init_resource::<MainScreenLayout>()
            .add_systems(FixedUpdate, unit_stats_logic_system)
            .add_systems(Update, unit_stats_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    fn spawn_unit(
        world: &mut World,
        archetype: UnitArchetype,
        attack: f32,
        speed: f32,
        armor: f32,
    ) -> Entity {
        world
            .spawn((
                Unit {
                    archetype,
                    ..Default::default()
                },
                Stats {
                    attack_damage: attack,
                    move_speed: speed,
                    ..Default::default()
                },
                Health {
                    armor,
                    ..Default::default()
                },
                Name::new("TestUnit"),
            ))
            .id()
    }

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<UnitStatsRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn UnitStatsRoot entity");
    }

    #[test]
    fn test_stats_display_empty_selection() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<UnitStatsRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Damage: 0"),
            "Should show zero damage: {}",
            text.0
        );
        assert!(
            text.0.contains("God is Great"),
            "Should show battle cry: {}",
            text.0
        );
    }

    #[test]
    fn test_stats_updates_on_selection() {
        let mut app = test_app();

        let unit = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            45.0,
            20.0,
            30.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        run_frames(&mut app, 4);

        let stats = app.world().resource::<UnitStatsState>();
        assert_eq!(stats.damage, 45, "Damage should be 45");
        assert_eq!(stats.speed, 20, "Speed should be 20");
        assert_eq!(stats.defense, 30, "Defense should be 30");

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<UnitStatsRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Damage: 45"),
            "Should show damage 45: {}",
            text.0
        );
        assert!(
            text.0.contains("Defense: 30"),
            "Should show defense 30: {}",
            text.0
        );
        assert!(
            text.0.contains("Speed: 20"),
            "Should show speed 20: {}",
            text.0
        );
    }

    #[test]
    fn test_battle_cry_display() {
        let mut state = UnitStatsState::default();
        state.battle_cry = String::from("God is Great! Cry");
        let text = build_unit_stats_text(&state);
        assert!(
            text.contains("God is Great"),
            "Should show battle cry: {}",
            text
        );
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<UnitStatsRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.left, Val::Px(8.0), "Panel should be 8px from left");
        assert_eq!(node.width, Val::Px(300.0), "Panel should be 300px wide");
        assert_eq!(node.height, Val::Px(70.0), "Panel should be 70px high");
    }
}
