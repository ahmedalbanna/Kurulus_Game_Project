//! Objectives Panel HUD — displays primary and secondary mission objectives.
//!
//! ## Architecture
//!
//! A single **render system** (`objectives_render_system`) runs in the `Update`
//! schedule.  It reads `ObjectiveTracker` populated by the mission system and
//! spawns / updates a single entity tagged with [`ObjectivesPanelRoot`].
//!
//! No logic system needed — the `ObjectiveTracker` state is managed by the
//! mission / narrative systems (Phase 4+).

use super::hud_polish::colorblind_bool_text;
use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for objective text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 8.0, 8.0);

/// Panel width.
const PANEL_WIDTH: f32 = 280.0;

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.60);

/// Text colour for incomplete objectives.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

/// Max display length for objective descriptions.
const MAX_DESC_LEN: usize = 60;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Truncate a description to a reasonable display length (char-wise).
fn truncate(desc: &str) -> String {
    let char_count = desc.chars().count();
    if char_count > MAX_DESC_LEN {
        let truncated: String = desc.chars().take(MAX_DESC_LEN - 1).collect();
        format!("{}…", truncated)
    } else {
        desc.to_string()
    }
}

/// Build a display line for a single objective entry.
fn format_objective(entry: &ObjectiveEntry, indent: bool, cm: ColorblindMode) -> String {
    let prefix = colorblind_bool_text(entry.completed, cm);
    let desc = truncate(&entry.description);
    let progress = entry
        .progress
        .map(|(current, total)| format!(" ({}/{})", current, total))
        .unwrap_or_default();

    if indent {
        format!("  {} {}{}", prefix, desc, progress)
    } else {
        format!("{} {}{}", prefix, desc, progress)
    }
}

/// Build the full multi-line text for the objectives panel.
fn build_objectives_text(objectives: &ObjectiveTracker, cm: ColorblindMode) -> String {
    let mut lines: Vec<String> = Vec::new();

    // ── Primary objectives ─────────────────────────────────────────────
    if !objectives.primary.is_empty() {
        lines.push("═══ PRIMARY ═══".to_string());
        for entry in &objectives.primary {
            lines.push(format_objective(entry, false, cm));
        }
    }

    // ── Secondary objectives ───────────────────────────────────────────
    if !objectives.secondary.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("── SECONDARY ──".to_string());
        for entry in &objectives.secondary {
            lines.push(format_objective(entry, true, cm));
        }
    }

    if lines.is_empty() {
        lines.push("No active objectives".to_string());
    }

    lines.join("\n")
}

// ─── System ──────────────────────────────────────────────────────────────────

/// Render system: spawns / updates the objectives panel UI entity.
/// Reads `HudSettings` for colourblind-friendly display.
pub fn objectives_render_system(
    mut commands: Commands,
    objectives: Res<ObjectiveTracker>,
    hud_settings: Res<HudSettings>,
    root_query: Query<Entity, With<ObjectivesPanelRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Px(45.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: PANEL_PADDING,
                width: Val::Px(PANEL_WIDTH),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Text::new(""),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            ObjectivesPanelRoot,
            Name::new("ObjectivesPanel"),
        ));
        return;
    };

    let body = build_objectives_text(&objectives, hud_settings.colorblind_mode);
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<ObjectiveTracker>()
            .init_resource::<HudSettings>()
            .add_systems(Update, objectives_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    fn make_entry(id: &str, desc: &str, completed: bool) -> ObjectiveEntry {
        ObjectiveEntry {
            id: id.to_string(),
            description: desc.to_string(),
            completed,
            progress: None,
        }
    }

    // ── Helper tests ───────────────────────────────────────────────────

    #[test]
    fn test_truncate_short() {
        assert_eq!(truncate("Hello"), "Hello");
    }

    #[test]
    fn test_truncate_long() {
        let long = "A".repeat(MAX_DESC_LEN + 10);
        let result = truncate(&long);
        // Result = (MAX_DESC_LEN - 1) chars + 1 ellipsis char = MAX_DESC_LEN chars
        assert_eq!(
            result.chars().count(),
            MAX_DESC_LEN,
            "Should truncate to MAX_DESC_LEN chars"
        );
        assert!(result.ends_with('…'), "Should end with ellipsis");
    }

    #[test]
    fn test_format_objective_incomplete() {
        let entry = make_entry("obj1", "Build an Oba", false);
        let text = format_objective(&entry, false, ColorblindMode::Normal);
        assert!(text.starts_with("☐"), "Incomplete should start with ☐");
        assert!(text.contains("Build an Oba"));
    }

    #[test]
    fn test_format_objective_completed() {
        let entry = make_entry("obj1", "Build an Oba", true);
        let text = format_objective(&entry, false, ColorblindMode::Normal);
        assert!(text.starts_with("✅"), "Completed should start with ✅");
    }

    #[test]
    fn test_format_objective_with_progress() {
        let mut entry = make_entry("obj1", "Kill enemies", false);
        entry.progress = Some((3, 10));
        let text = format_objective(&entry, false, ColorblindMode::Normal);
        assert!(text.contains("(3/10)"), "Should show progress: {}", text);
    }

    #[test]
    fn test_format_objective_indented() {
        let entry = make_entry("obj1", "Secondary task", false);
        let text = format_objective(&entry, true, ColorblindMode::Normal);
        assert!(
            text.starts_with("  "),
            "Indented should have leading spaces"
        );
    }

    #[test]
    fn test_build_text_empty() {
        let tracker = ObjectiveTracker::default();
        let text = build_objectives_text(&tracker, ColorblindMode::Normal);
        assert_eq!(text, "No active objectives");
    }

    #[test]
    fn test_build_text_primary_only() {
        let tracker = ObjectiveTracker {
            primary: vec![make_entry("p1", "Capture the flag", false)],
            secondary: vec![],
        };
        let text = build_objectives_text(&tracker, ColorblindMode::Normal);
        assert!(text.contains("PRIMARY"));
        assert!(text.contains("Capture the flag"));
        assert!(!text.contains("SECONDARY"));
    }

    #[test]
    fn test_build_text_secondary_only() {
        let tracker = ObjectiveTracker {
            primary: vec![],
            secondary: vec![make_entry("s1", "Gather wood", false)],
        };
        let text = build_objectives_text(&tracker, ColorblindMode::Normal);
        assert!(text.contains("SECONDARY"));
        assert!(text.contains("Gather wood"));
    }

    #[test]
    fn test_build_text_both_sections() {
        let tracker = ObjectiveTracker {
            primary: vec![make_entry("p1", "Win the battle", false)],
            secondary: vec![make_entry("s1", "Save villagers", false)],
        };
        let text = build_objectives_text(&tracker, ColorblindMode::Normal);
        assert!(
            text.contains("PRIMARY"),
            "Should show primary header: {}",
            text
        );
        assert!(
            text.contains("SECONDARY"),
            "Should show secondary header: {}",
            text
        );
        assert!(text.contains("Win the battle"));
        assert!(text.contains("Save villagers"));
    }

    #[test]
    fn test_build_text_shows_completion() {
        let tracker = ObjectiveTracker {
            primary: vec![make_entry("p1", "Build mosque", true)],
            secondary: vec![],
        };
        let text = build_objectives_text(&tracker, ColorblindMode::Normal);
        assert!(
            text.contains("✅"),
            "Completed objective should show checkmark"
        );
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn ObjectivesPanelRoot entity");
    }

    #[test]
    fn test_render_shows_objectives() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<ObjectiveTracker>()
            .primary
            .push(make_entry("p1", "Win the mission", false));

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Win the mission"),
            "Should show objective: {}",
            text.0
        );
        assert!(text.0.contains("PRIMARY"), "Should show header: {}", text.0);
    }

    #[test]
    fn test_render_empty_tracker_shows_no_active() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert_eq!(text.0, "No active objectives");
    }

    #[test]
    fn test_render_updates_when_objectives_change() {
        let mut app = test_app();

        run_frames(&mut app, 4);
        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .collect();
        let text1 = app.world().get::<Text>(roots[0]).unwrap().0.clone();
        assert_eq!(text1, "No active objectives");

        // Add an objective
        app.world_mut()
            .resource_mut::<ObjectiveTracker>()
            .primary
            .push(make_entry("p1", "New objective", false));
        run_frames(&mut app, 4);

        let text2 = app.world().get::<Text>(roots[0]).unwrap().0.clone();
        assert!(
            text2.contains("New objective"),
            "Should update text: {}",
            text2
        );
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.right, Val::Px(12.0), "Panel should be 12px from right");
        assert_eq!(node.top, Val::Px(45.0), "Panel should be 45px from top");
    }

    // ── Colourblind mode tests ─────────────────────────────────────────

    #[test]
    fn test_format_objective_colorblind_mode() {
        let entry = make_entry("obj1", "Build an Oba", false);

        let normal = format_objective(&entry, false, ColorblindMode::Normal);
        assert!(normal.starts_with("☐"), "Normal mode should use ☐");

        let cb = format_objective(&entry, false, ColorblindMode::Protanopia);
        assert!(
            cb.contains("[TODO]"),
            "Colourblind mode should use [TODO]: {}",
            cb
        );
    }

    #[test]
    fn test_format_objective_completed_colorblind() {
        let entry = make_entry("obj1", "Build an Oba", true);

        let normal = format_objective(&entry, false, ColorblindMode::Normal);
        assert!(normal.starts_with("✅"), "Normal completed should use ✅");

        let cb = format_objective(&entry, false, ColorblindMode::Deuteranopia);
        assert!(
            cb.contains("[DONE]"),
            "Colourblind completed should use [DONE]: {}",
            cb
        );
    }

    #[test]
    fn test_build_text_colorblind_mode() {
        let tracker = ObjectiveTracker {
            primary: vec![make_entry("p1", "Win", true)],
            secondary: vec![make_entry("s1", "Gather", false)],
        };

        let cb_text = build_objectives_text(&tracker, ColorblindMode::Tritanopia);
        assert!(
            cb_text.contains("[DONE]"),
            "Primary completed should use [DONE]: {}",
            cb_text
        );
        assert!(
            cb_text.contains("[TODO]"),
            "Secondary incomplete should use [TODO]: {}",
            cb_text
        );
    }

    #[test]
    fn test_render_colorblind_mode() {
        let mut app = test_app();

        // Switch to colourblind mode
        app.world_mut()
            .resource_mut::<HudSettings>()
            .colorblind_mode = ColorblindMode::Achromatopsia;

        app.world_mut()
            .resource_mut::<ObjectiveTracker>()
            .primary
            .push(make_entry("p1", "Win the mission", true));

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("[DONE]"),
            "Render should use [DONE] in achromatopsia mode: {}",
            text.0
        );
    }
}
