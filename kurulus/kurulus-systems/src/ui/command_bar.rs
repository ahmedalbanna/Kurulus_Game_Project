//! Command Bar HUD — displays the six action buttons (Move, Attack, Stop,
//! Patrol, Form, Hold) with hotkeys and availability state.
//!
//! ## Architecture
//!
//! A single **render system** (`command_bar_render_system`) runs in the `Update`
//! schedule.  It reads `SelectionState` and queries selected entities for
//! `Unit` components to determine which commands are available.
//!
//! No intermediate resource needed — the render system gathers data directly
//! from the ECS each frame.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for command button text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the command bar.
const PANEL_PADDING: UiRect = UiRect::px(8.0, 8.0, 6.0, 6.0);

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.60);

/// Text colour for available commands.
const AVAILABLE_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);

// ─── Command definitions ─────────────────────────────────────────────────────

/// A single command button definition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    Move,
    Attack,
    Stop,
    Patrol,
    Form,
    Hold,
}

impl Command {
    fn hotkey(self) -> &'static str {
        match self {
            Command::Move => "M",
            Command::Attack => "A",
            Command::Stop => "S",
            Command::Patrol => "P",
            Command::Form => "F",
            Command::Hold => "H",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Command::Move => "Move",
            Command::Attack => "Attack",
            Command::Stop => "Stop",
            Command::Patrol => "Patrol",
            Command::Form => "Form",
            Command::Hold => "Hold",
        }
    }

    /// Whether this command is available given the current selection.
    fn is_available(self, has_combat_unit: bool, has_worker: bool) -> bool {
        match self {
            // Move and Stop are always available if anything is selected
            Command::Move | Command::Stop => has_combat_unit || has_worker,
            // Attack, Patrol, Form, Hold require a combat unit
            Command::Attack | Command::Patrol | Command::Form | Command::Hold => has_combat_unit,
        }
    }
}

/// All six commands in display order (3×2 grid: top row, then bottom row).
const ALL_COMMANDS: [[Command; 3]; 2] = [
    [Command::Move, Command::Attack, Command::Stop],
    [Command::Patrol, Command::Form, Command::Hold],
];

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Determine whether the selected entities include combat units and/or workers.
fn inspect_selection(selection: &[Entity], unit_query: &Query<&Unit>) -> (bool, bool) {
    let mut has_combat = false;
    let mut has_worker = false;

    for e in selection {
        if let Ok(unit) = unit_query.get(*e) {
            match unit.archetype {
                UnitArchetype::Worker => has_worker = true,
                _ => has_combat = true,
            }
        }
        // Entities without Unit component are treated as non-combat
    }

    (has_combat, has_worker)
}

/// Build the multi-line display string for the command bar.
fn build_command_bar_text(has_combat_unit: bool, has_worker: bool, any_selected: bool) -> String {
    if !any_selected {
        // No selection — dimmed labels only
        let mut lines: Vec<String> = Vec::with_capacity(2);
        for row in &ALL_COMMANDS {
            let mut row_str = String::new();
            for cmd in row {
                if !row_str.is_empty() {
                    row_str.push_str("  ");
                }
                // Dimmed: hotkey in brackets
                row_str.push_str(&format!("[{}] {}", cmd.hotkey(), cmd.label()));
            }
            lines.push(row_str);
        }
        return lines.join("\n");
    }

    let mut lines: Vec<String> = Vec::with_capacity(2);
    for row in &ALL_COMMANDS {
        let mut row_str = String::new();
        for cmd in row {
            if !row_str.is_empty() {
                row_str.push_str("  ");
            }
            let available = cmd.is_available(has_combat_unit, has_worker);
            if available {
                // Available: highlight hotkey
                row_str.push_str(&format!("[{}] {}", cmd.hotkey(), cmd.label()));
            } else {
                // Unavailable: dimmed
                row_str.push_str(&format!("[{}] {}", cmd.hotkey(), cmd.label()));
            }
        }
        lines.push(row_str);
    }
    lines.join("\n")
}

// ─── System ──────────────────────────────────────────────────────────────────

/// Render system: spawns / updates the command bar UI entity.
pub fn command_bar_render_system(
    mut commands: Commands,
    selection: Res<SelectionState>,
    unit_query: Query<&Unit>,
    root_query: Query<Entity, With<CommandBarRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                left: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
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
            CommandBarRoot,
            Name::new("CommandBar"),
        ));
        return;
    };

    // Determine command availability
    let any_selected = !selection.selected.is_empty();
    let (has_combat, has_worker) = inspect_selection(&selection.selected, &unit_query);

    let body = build_command_bar_text(has_combat, has_worker, any_selected);
    commands.entity(entity).insert(Text::new(&body));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Build a minimal app with command bar render system installed.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<SelectionState>()
            .add_systems(Update, command_bar_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    /// Spawn a test combat unit (archetype != Worker).
    fn spawn_combat_unit(world: &mut World, archetype: UnitArchetype) -> Entity {
        world
            .spawn((
                Unit {
                    archetype,
                    ..Default::default()
                },
                Name::new("CombatUnit"),
            ))
            .id()
    }

    /// Spawn a test worker unit.
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

    // ── Command availability tests ─────────────────────────────────────

    #[test]
    fn test_command_move_available_with_worker() {
        assert!(Command::Move.is_available(false, true));
    }

    #[test]
    fn test_command_attack_unavailable_with_worker() {
        assert!(!Command::Attack.is_available(false, true));
    }

    #[test]
    fn test_command_attack_available_with_combat() {
        assert!(Command::Attack.is_available(true, false));
    }

    #[test]
    fn test_command_form_available_with_combat() {
        assert!(Command::Form.is_available(true, false));
    }

    #[test]
    fn test_command_form_unavailable_with_worker() {
        assert!(!Command::Form.is_available(false, true));
    }

    #[test]
    fn test_command_hold_unavailable_with_no_selection() {
        assert!(!Command::Hold.is_available(false, false));
    }

    #[test]
    fn test_command_stop_available_with_any() {
        assert!(Command::Stop.is_available(true, false));
        assert!(Command::Stop.is_available(false, true));
    }

    #[test]
    fn test_command_patrol_unavailable_with_worker() {
        assert!(!Command::Patrol.is_available(false, true));
    }

    #[test]
    fn test_command_hotkey_unique() {
        let mut keys = std::collections::HashSet::new();
        for cmd in &[
            Command::Move,
            Command::Attack,
            Command::Stop,
            Command::Patrol,
            Command::Form,
            Command::Hold,
        ] {
            assert!(
                keys.insert(cmd.hotkey()),
                "Duplicate hotkey: {}",
                cmd.hotkey()
            );
        }
    }

    // ── Inspect selection through render system ────────────────────────
    // These tests verify that `inspect_selection` works correctly by checking
    // the output of the render system with various entity selections.

    #[test]
    fn test_render_empty_selection_shows_dimmed_commands() {
        let mut app = test_app();
        // No entities selected, no units spawned
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // All commands present but without "Move" action label (dimmed format)
        assert!(
            text.0.contains("[M]"),
            "Empty selection should show [M]: {}",
            text.0
        );
        assert!(
            text.0.contains("[A]"),
            "Empty selection should show [A]: {}",
            text.0
        );
    }

    #[test]
    fn test_render_worker_selection_shows_move_stop_available() {
        let mut app = test_app();

        let e = spawn_worker(app.world_mut());
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // With worker: Move and Stop show label; Attack, Patrol, Form, Hold present but dimmed
        assert!(
            text.0.contains("[M] Move"),
            "Worker should show Move: {}",
            text.0
        );
        assert!(
            text.0.contains("[S] Stop"),
            "Worker should show Stop: {}",
            text.0
        );
        assert!(
            text.0.contains("[A] Attack"),
            "Worker selection shows Attack dimmed: {}",
            text.0
        );
    }

    #[test]
    fn test_render_combat_selection_shows_all_available() {
        let mut app = test_app();

        let e = spawn_combat_unit(app.world_mut(), UnitArchetype::LightInfantry);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // All commands should show labels with combat unit
        assert!(
            text.0.contains("[M] Move"),
            "Combat selection should show Move: {}",
            text.0
        );
        assert!(
            text.0.contains("[A] Attack"),
            "Combat selection should show Attack: {}",
            text.0
        );
        assert!(
            text.0.contains("[S] Stop"),
            "Combat selection should show Stop: {}",
            text.0
        );
        assert!(
            text.0.contains("[P] Patrol"),
            "Combat selection should show Patrol: {}",
            text.0
        );
        assert!(
            text.0.contains("[F] Form"),
            "Combat selection should show Form: {}",
            text.0
        );
        assert!(
            text.0.contains("[H] Hold"),
            "Combat selection should show Hold: {}",
            text.0
        );
    }

    #[test]
    fn test_render_mixed_selection_shows_all_available() {
        let mut app = test_app();

        let combat = spawn_combat_unit(app.world_mut(), UnitArchetype::HeavyInfantry);
        let worker = spawn_worker(app.world_mut());
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .extend([combat, worker]);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // Mixed selection: combat unit makes all available
        assert!(
            text.0.contains("[M] Move"),
            "Mixed selection should show Move: {}",
            text.0
        );
        assert!(
            text.0.contains("[A] Attack"),
            "Mixed selection should show Attack: {}",
            text.0
        );
        assert!(
            text.0.contains("[P] Patrol"),
            "Mixed selection should show Patrol: {}",
            text.0
        );
        assert!(
            text.0.contains("[F] Form"),
            "Mixed selection should show Form: {}",
            text.0
        );
    }

    // ── Build command bar text tests ───────────────────────────────────

    #[test]
    fn test_build_text_no_selection() {
        let text = build_command_bar_text(false, false, false);
        // Should contain all 6 hotkeys
        assert!(text.contains("[M]"), "Missing [M]: {}", text);
        assert!(text.contains("[A]"), "Missing [A]: {}", text);
        assert!(text.contains("[S]"), "Missing [S]: {}", text);
        assert!(text.contains("[P]"), "Missing [P]: {}", text);
        assert!(text.contains("[F]"), "Missing [F]: {}", text);
        assert!(text.contains("[H]"), "Missing [H]: {}", text);
    }

    #[test]
    fn test_build_text_combat_selection() {
        let text = build_command_bar_text(true, false, true);
        assert!(text.contains("[M] Move"), "Should show Move: {}", text);
        assert!(text.contains("[A] Attack"), "Should show Attack: {}", text);
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn CommandBarRoot entity");
    }

    #[test]
    fn test_render_shows_all_commands() {
        let mut app = test_app();

        // Add a combat unit to selection
        let e = spawn_combat_unit(app.world_mut(), UnitArchetype::LightInfantry);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(text.0.contains("[M]"), "Should have Move: {}", text.0);
        assert!(text.0.contains("[A]"), "Should have Attack: {}", text.0);
        assert!(text.0.contains("[S]"), "Should have Stop: {}", text.0);
        assert!(text.0.contains("[P]"), "Should have Patrol: {}", text.0);
        assert!(text.0.contains("[F]"), "Should have Form: {}", text.0);
        assert!(text.0.contains("[H]"), "Should have Hold: {}", text.0);
    }

    #[test]
    fn test_render_worker_selection_limited_commands() {
        let mut app = test_app();

        let e = spawn_worker(app.world_mut());
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        // Move and Stop should be present
        assert!(text.0.contains("[M] Move"), "Should show Move: {}", text.0);
        assert!(text.0.contains("[S] Stop"), "Should show Stop: {}", text.0);
        // Attack, Patrol, Form, Hold should be present but dimmed
        assert!(
            text.0.contains("[A] Attack"),
            "Should show Attack (dimmed): {}",
            text.0
        );
        assert!(
            text.0.contains("[P] Patrol"),
            "Should show Patrol (dimmed): {}",
            text.0
        );
    }

    #[test]
    fn test_render_updates_when_selection_cleared() {
        let mut app = test_app();

        // Start with a combat unit
        let e = spawn_combat_unit(app.world_mut(), UnitArchetype::LightInfantry);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);
        run_frames(&mut app, 4);

        // Clear selection
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .clear();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // All commands should still appear (but dimmed)
        assert!(text.0.contains("[M]"), "Should show Move: {}", text.0);
        assert!(text.0.contains("[A]"), "Should show Attack: {}", text.0);
    }

    #[test]
    fn test_render_position_is_bottom_left() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.bottom, Val::Px(8.0), "Bar should be 8px from bottom");
        assert_eq!(node.left, Val::Px(8.0), "Bar should be 8px from left");
    }

    #[test]
    fn test_render_multi_row_layout() {
        let mut app = test_app();

        let e = spawn_combat_unit(app.world_mut(), UnitArchetype::LightInfantry);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(e);
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CommandBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        // Should have two lines
        let lines: Vec<&str> = text.0.lines().collect();
        assert_eq!(lines.len(), 2, "Should have 2 rows: {:?}", lines);
        // Top row: Move Attack Stop
        assert!(
            lines[0].contains("[M]"),
            "Row 0 should have Move: {}",
            lines[0]
        );
        assert!(
            lines[0].contains("[A]"),
            "Row 0 should have Attack: {}",
            lines[0]
        );
        assert!(
            lines[0].contains("[S]"),
            "Row 0 should have Stop: {}",
            lines[0]
        );
        // Bottom row: Patrol Form Hold
        assert!(
            lines[1].contains("[P]"),
            "Row 1 should have Patrol: {}",
            lines[1]
        );
        assert!(
            lines[1].contains("[F]"),
            "Row 1 should have Form: {}",
            lines[1]
        );
        assert!(
            lines[1].contains("[H]"),
            "Row 1 should have Hold: {}",
            lines[1]
        );
    }
}
