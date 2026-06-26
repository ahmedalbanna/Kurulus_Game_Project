//! Action Bar HUD — displays building/unit action buttons (Move, Flag, Heroes,
//! Blacksmith, Oba, Barracks, Alpa, Horse Archer).
//!
//! ## Architecture
//!
//! 1. **`action_bar_logic_system`** (FixedUpdate) — reads selection state and
//!    available buildings/units, populates `ActionBarState`.
//!
//! 2. **`action_bar_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`ActionBarRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for action bar text.
const FONT_SIZE: f32 = 12.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 6.0, 6.0);

/// Background colour (dark brown).
const BG_COLOR: Color = Color::srgba(0.15, 0.12, 0.10, 0.85);

/// Text colour for available actions.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: populates available actions based on selection and context.
pub fn action_bar_logic_system(
    selection: Res<SelectionState>,
    unit_query: Query<&Unit>,
    building_query: Query<&Building>,
    mut action_state: ResMut<ActionBarState>,
) {
    // Build default action rows based on what's selected
    let has_selection = !selection.selected.is_empty();

    // Row 1: Move, Flag (always available with selection)
    action_state.row1 = vec![
        ActionSlot {
            action_type: ActionType::Move,
            label: ActionType::Move.label().to_string(),
            available: has_selection,
            hotkey: Some("M".to_string()),
            icon: ActionType::Move.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::Flag,
            label: ActionType::Flag.label().to_string(),
            available: has_selection,
            hotkey: Some("F".to_string()),
            icon: ActionType::Flag.icon().to_string(),
        },
    ];

    // Row 2: Building/training actions (always show, availability varies)
    action_state.row2 = vec![
        ActionSlot {
            action_type: ActionType::Heroes,
            label: ActionType::Heroes.label().to_string(),
            available: true,
            hotkey: Some("H".to_string()),
            icon: ActionType::Heroes.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::Blacksmith,
            label: ActionType::Blacksmith.label().to_string(),
            available: true,
            hotkey: Some("B".to_string()),
            icon: ActionType::Blacksmith.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::Oba,
            label: ActionType::Oba.label().to_string(),
            available: true,
            hotkey: Some("O".to_string()),
            icon: ActionType::Oba.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::Barracks,
            label: ActionType::Barracks.label().to_string(),
            available: true,
            hotkey: Some("R".to_string()),
            icon: ActionType::Barracks.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::Alpa,
            label: ActionType::Alpa.label().to_string(),
            available: true,
            hotkey: Some("A".to_string()),
            icon: ActionType::Alpa.icon().to_string(),
        },
        ActionSlot {
            action_type: ActionType::HorseArcher,
            label: ActionType::HorseArcher.label().to_string(),
            available: true,
            hotkey: Some("C".to_string()),
            icon: ActionType::HorseArcher.icon().to_string(),
        },
    ];

    // If we have specific units selected, adjust availability
    if has_selection {
        for &entity in &selection.selected {
            if let Ok(unit) = unit_query.get(entity) {
                // Workers can build, combat units can move/attack
                match unit.archetype {
                    UnitArchetype::Worker => {
                        // Workers can build structures
                        action_state.row1[0].available = true; // Move
                        action_state.row1[1].available = true; // Flag
                    }
                    _ => {
                        // Combat units
                        action_state.row1[0].available = true; // Move
                        action_state.row1[1].available = true; // Flag
                    }
                }
            }
            if let Ok(_building) = building_query.get(entity) {
                // Buildings have their own action sets
                action_state.row1[0].available = true;
            }
        }
    }
}

/// Render system: spawns / updates the action bar UI entity.
pub fn action_bar_render_system(
    mut commands: Commands,
    action_state: Res<ActionBarState>,
    root_query: Query<Entity, With<ActionBarRoot>>,
    layout: Res<MainScreenLayout>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(layout.padding),
                left: Val::Px(layout.padding + layout.hero_panel_width + layout.padding),
                width: Val::Px(layout.action_bar_width),
                height: Val::Px(layout.action_bar_height),
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
            ActionBarRoot,
            Name::new("ActionBar"),
        ));
        return;
    };

    let body = build_action_bar_text(&action_state);
    commands.entity(entity).insert(Text::new(&body));
}

fn build_action_bar_text(state: &ActionBarState) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Row 1
    let mut row1_str = String::new();
    for (i, slot) in state.row1.iter().enumerate() {
        if i > 0 {
            row1_str.push_str("  ");
        }
        let hotkey = slot.hotkey.as_deref().unwrap_or("");
        row1_str.push_str(&format!("[{}] {} {}", hotkey, slot.icon, slot.label));
    }
    lines.push(row1_str);

    // Row 2
    let mut row2_str = String::new();
    for (i, slot) in state.row2.iter().enumerate() {
        if i > 0 {
            row2_str.push_str("  ");
        }
        let hotkey = slot.hotkey.as_deref().unwrap_or("");
        row2_str.push_str(&format!("[{}] {} {}", hotkey, slot.icon, slot.label));
    }
    lines.push(row2_str);

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
            .init_resource::<ActionBarState>()
            .init_resource::<MainScreenLayout>()
            .add_systems(FixedUpdate, action_bar_logic_system)
            .add_systems(Update, action_bar_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn ActionBarRoot entity");
    }

    #[test]
    fn test_action_bar_shows_actions() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(text.0.contains("Move"), "Should show Move: {}", text.0);
        assert!(text.0.contains("Flag"), "Should show Flag: {}", text.0);
        assert!(text.0.contains("Heroes"), "Should show Heroes: {}", text.0);
        assert!(
            text.0.contains("Blacksmith"),
            "Should show Blacksmith: {}",
            text.0
        );
        assert!(text.0.contains("Oba"), "Should show Oba: {}", text.0);
        assert!(
            text.0.contains("Barracks"),
            "Should show Barracks: {}",
            text.0
        );
        assert!(text.0.contains("Alpa"), "Should show Alpa: {}", text.0);
    }

    #[test]
    fn test_render_has_two_rows() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        let lines: Vec<&str> = text.0.lines().collect();
        assert_eq!(lines.len(), 2, "Should have 2 rows: {:?}", lines);
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ActionBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.bottom, Val::Px(8.0), "Bar should be 8px from bottom");
        assert_eq!(node.width, Val::Px(500.0), "Bar should be 500px wide");
        assert_eq!(node.height, Val::Px(80.0), "Bar should be 80px high");
    }

    #[test]
    fn test_action_availability_with_selection() {
        let mut app = test_app();

        // Add a combat unit to selection
        let unit = app
            .world_mut()
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    ..Default::default()
                },
                Name::new("CombatUnit"),
            ))
            .id();
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        run_frames(&mut app, 4);

        let state = app.world().resource::<ActionBarState>();
        assert!(state.row1[0].available, "Move should be available");
        assert!(state.row1[1].available, "Flag should be available");
    }

    #[test]
    fn test_action_type_labels() {
        assert_eq!(ActionType::Move.label(), "Move");
        assert_eq!(ActionType::Blacksmith.label(), "Blacksmith");
        assert_eq!(ActionType::Oba.label(), "Oba");
        assert_eq!(ActionType::Barracks.label(), "Barracks");
        assert_eq!(ActionType::Alpa.label(), "Alpa");
        assert_eq!(ActionType::HorseArcher.label(), "H.Archer");
        assert_eq!(ActionType::Heroes.label(), "Heroes");
    }

    #[test]
    fn test_action_type_icons() {
        assert_eq!(ActionType::Move.icon(), "🚩");
        assert_eq!(ActionType::Heroes.icon(), "👤");
        assert_eq!(ActionType::Blacksmith.icon(), "⚒");
        assert_eq!(ActionType::Oba.icon(), "🏠");
        assert_eq!(ActionType::Barracks.icon(), "🏰");
        assert_eq!(ActionType::Alpa.icon(), "🐴");
        assert_eq!(ActionType::HorseArcher.icon(), "🏹");
    }
}
