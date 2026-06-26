//! Settings Button HUD — displays a gear icon in the top-right corner that
//! opens the settings menu when clicked.
//!
//! ## Architecture
//!
//! A single **render system** (`settings_button_render_system`) runs in the
//! `Update` schedule.  It spawns or updates a single entity tagged with
//! [`SettingsButtonRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for the settings button.
const FONT_SIZE: f32 = 22.0;

/// Background colour (semi-transparent dark).
const BG_COLOR: Color = Color::srgba(0.15, 0.12, 0.10, 0.70);

/// Text colour for the gear icon.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.85, 0.70);

/// The settings gear icon character.
const GEAR_ICON: &str = "⚙";

// ─── System ──────────────────────────────────────────────────────────────────

/// Render system: spawns / updates the settings button UI entity.
pub fn settings_button_render_system(
    mut commands: Commands,
    root_query: Query<Entity, With<SettingsButtonRoot>>,
    layout: Res<MainScreenLayout>,
) {
    if let Ok(_entity) = root_query.get_single() {
        // Already spawned — no per-frame updates needed for a static button
        return;
    }

    // First frame — spawn the root entity
    let _ = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(4.0),
            right: Val::Px(8.0),
            width: Val::Px(layout.settings_button_size),
            height: Val::Px(layout.settings_button_size),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BG_COLOR),
        Text::new(GEAR_ICON),
        TextFont {
            font_size: FONT_SIZE,
            ..default()
        },
        TextColor(TEXT_COLOR),
        SettingsButtonRoot,
        Name::new("SettingsButton"),
    ));
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
            .init_resource::<MainScreenLayout>()
            .add_systems(Update, settings_button_render_system);
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
            .query_filtered::<Entity, With<SettingsButtonRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn SettingsButtonRoot entity");
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SettingsButtonRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.top, Val::Px(4.0), "Button should be 4px from top");
        assert_eq!(node.right, Val::Px(8.0), "Button should be 8px from right");
        assert_eq!(node.width, Val::Px(36.0), "Button should be 36px wide");
        assert_eq!(node.height, Val::Px(36.0), "Button should be 36px high");
    }

    #[test]
    fn test_entity_is_stable() {
        let mut app = test_app();
        run_frames(&mut app, 10);

        // Should still have exactly one entity after many frames
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SettingsButtonRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "Should maintain exactly one SettingsButtonRoot entity"
        );
    }
}
