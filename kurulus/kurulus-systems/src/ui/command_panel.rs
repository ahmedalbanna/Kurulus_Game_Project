//! Command Panel HUD — displays the four main commands (Attack, Patrol, Guard,
//! Stop) on the right side of the screen, above the minimap.
//!
//! ## Architecture
//!
//! 1. **`command_panel_logic_system`** (FixedUpdate) — reads selection state,
//!    determines which commands are available.
//!
//! 2. **`command_panel_render_system`** (Update) — spawns / updates a single
//!    entity tagged with [`CommandPanelRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for command panel text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 8.0, 8.0);

/// Background colour (dark brown).
const BG_COLOR: Color = Color::srgba(0.15, 0.12, 0.10, 0.88);

/// Text colour for available commands.
const AVAILABLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.92);

// ─── Command definitions ─────────────────────────────────────────────────────

/// The four main command buttons displayed in the command panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandButton {
    Attack,
    Patrol,
    Guard,
    Stop,
}

impl CommandButton {
    fn hotkey(self) -> &'static str {
        match self {
            CommandButton::Attack => "A",
            CommandButton::Patrol => "P",
            CommandButton::Guard => "G",
            CommandButton::Stop => "S",
        }
    }

    fn label(self) -> &'static str {
        match self {
            CommandButton::Attack => "Attack",
            CommandButton::Patrol => "Patrol",
            CommandButton::Guard => "Guard",
            CommandButton::Stop => "Stop",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            CommandButton::Attack => "⚔",
            CommandButton::Patrol => "🔄",
            CommandButton::Guard => "🛡",
            CommandButton::Stop => "⏹",
        }
    }
}

const ALL_COMMANDS: [CommandButton; 4] = [
    CommandButton::Attack,
    CommandButton::Patrol,
    CommandButton::Guard,
    CommandButton::Stop,
];

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reserved for future command state management
/// (e.g., cooldowns, morale requirements).
pub fn command_panel_logic_system() {
    // Placeholder for future command state tracking
}

/// Render system: spawns / updates the command panel UI entity.
pub fn command_panel_render_system(
    mut commands: Commands,
    selection: Res<SelectionState>,
    root_query: Query<Entity, With<CommandPanelRoot>>,
    layout: Res<MainScreenLayout>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(layout.padding + layout.minimap_size + layout.padding),
                right: Val::Px(layout.padding),
                width: Val::Px(layout.command_panel_width),
                height: Val::Px(layout.command_panel_height),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                justify_content: JustifyContent::SpaceEvenly,
                padding: PANEL_PADDING,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Text::new(""),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(AVAILABLE_COLOR),
            CommandPanelRoot,
            Name::new("CommandPanel"),
        ));
        return;
    };

    let body = build_command_panel_text(!selection.selected.is_empty());
    commands.entity(entity).insert(Text::new(&body));
}

fn build_command_panel_text(any_selected: bool) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(ALL_COMMANDS.len());

    for cmd in &ALL_COMMANDS {
        lines.push(format!("[{}] {} {}", cmd.hotkey(), cmd.icon(), cmd.label()));
    }

    if !any_selected {
        lines.push(String::new());
        lines.push(String::from("No selection"));
    }

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
            .init_resource::<MainScreenLayout>()
            .add_systems(FixedUpdate, command_panel_logic_system)
            .add_systems(Update, command_panel_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    fn spawn_combat_unit(world: &mut World) -> Entity {
        world
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    ..Default::default()
                },
                Name::new("CombatUnit"),
            ))
            .id()
    }

    fn spawn_worker(world: &mut World) -> Entity {
        world
            .spawn((
                Unit {
                    archetype: UnitArchetype::Worker,
                    ..Default::default()
                },
                Name::new("Worker"),
            ))
            .id()
    }

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<CommandPanelRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn CommandPanelRoot entity");
    }

    #[test]
    fn test_render_shows_all_commands() {
        let mut app = test_app();

        let unit = spawn_combat_unit(app.world_mut());
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandPanelRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(roots, 1);
    }

    #[test]
    fn test_empty_selection_shows_no_selection() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("No selection"),
            "Empty selection should show message: {}",
            text.0
        );
    }

    #[test]
    fn test_command_availability_labels() {
        assert_eq!(CommandButton::Attack.label(), "Attack");
        assert_eq!(CommandButton::Patrol.label(), "Patrol");
        assert_eq!(CommandButton::Guard.label(), "Guard");
        assert_eq!(CommandButton::Stop.label(), "Stop");
    }

    #[test]
    fn test_command_hotkeys() {
        assert_eq!(CommandButton::Attack.hotkey(), "A");
        assert_eq!(CommandButton::Patrol.hotkey(), "P");
        assert_eq!(CommandButton::Guard.hotkey(), "G");
        assert_eq!(CommandButton::Stop.hotkey(), "S");
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.right, Val::Px(8.0), "Panel should be 8px from right");
        assert_eq!(node.width, Val::Px(130.0), "Panel should be 130px wide");
        assert_eq!(node.height, Val::Px(170.0), "Panel should be 170px high");
    }
}
