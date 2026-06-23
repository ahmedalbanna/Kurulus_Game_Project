//! Overlay Legend HUD — explains what gizmo colours mean on screen.
//!
//! ## Architecture
//!
//! A single render system (`zone_legend_render_system`) runs in the [`Update`]
//! schedule.  It queries [`OverlaySettings`] for the current mode and maintains
//! one UI entity tagged with [`ZoneLegendRoot`].
//!
//! The entity is spawned or despawned each frame depending on whether an overlay
//! is active — there's no persistent entity when the overlay is off, so there's
//! no stale UI in headless test mode.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for legend text.
const FONT_SIZE: f32 = 14.0;

/// Padding around the whole panel.
const PANEL_PADDING: UiRect = UiRect::px(8.0, 8.0, 6.0, 6.0);

/// Minimum gap between the label and the colour hint.
const INDENT: &str = "  ";

// ─── Legend builders ─────────────────────────────────────────────────────────

/// Build the zone-colour legend text.
fn zone_legend_body() -> String {
    vec![
        format!("═══ {} ═══", OverlayMode::Zones.label()),
        format!("{}■ Residential  (green)", INDENT),
        format!("{}■ Military     (red)", INDENT),
        format!("{}■ Economic     (yellow)", INDENT),
        format!("{}■ Religious    (blue)", INDENT),
        format!("{}■ Admin        (purple)", INDENT),
        format!("{}■ No Zone      (grey)", INDENT),
    ]
    .join("\n")
}

/// Build the service-coverage legend text.
fn coverage_legend_body() -> String {
    vec![
        format!("═══ {} ═══", OverlayMode::ServiceCoverage.label()),
        format!("{}■ Full Coverage    (green)", INDENT),
        format!("{}■ Partial Coverage  (yellow)", INDENT),
        format!("{}■ No Coverage      (red)", INDENT),
    ]
    .join("\n")
}

/// Build the happiness-map legend text.
fn happiness_legend_body() -> String {
    vec![
        format!("═══ {} ═══", OverlayMode::Happiness.label()),
        format!("{}■ Happy     (green)", INDENT),
        format!("{}■ Neutral   (yellow)", INDENT),
        format!("{}■ Unhappy   (orange)", INDENT),
        format!("{}■ Miserable (red)", INDENT),
    ]
    .join("\n")
}

/// Choose the legend body based on the active overlay mode.
fn legend_for_mode(mode: OverlayMode) -> Option<String> {
    match mode {
        OverlayMode::None => None,
        OverlayMode::Zones => Some(zone_legend_body()),
        OverlayMode::ServiceCoverage => Some(coverage_legend_body()),
        OverlayMode::Happiness => Some(happiness_legend_body()),
    }
}

// ─── System ──────────────────────────────────────────────────────────────────

/// Render system: spawns, updates, or despawns the legend UI entity each frame.
///
/// When the overlay mode is [`OverlayMode::None`] the entity is despawned so
/// the legend does not take up visual space.  Otherwise a single entity with
/// [`Node`] + [`Text`] is maintained in the bottom-left corner.
pub fn zone_legend_render_system(
    mut commands: Commands,
    overlay: Res<OverlaySettings>,
    root_query: Query<Entity, With<ZoneLegendRoot>>,
) {
    let body = legend_for_mode(overlay.mode);

    // ── Find or manage the UI entity ────────────────────────────────────
    let current_root = root_query.get_single().ok();

    match (body, current_root) {
        (Some(text), None) => {
            // Overlay is on but root is missing → spawn
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(8.0),
                    left: Val::Px(12.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    padding: PANEL_PADDING,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                Text::new(&text),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ZoneLegendRoot,
                Name::new("ZoneLegend"),
            ));
        }
        (Some(text), Some(entity)) => {
            // Overlay is on and root exists → update text in-place
            commands.entity(entity).insert(Text::new(&text));
        }
        (None, Some(entity)) => {
            // Overlay turned off → despawn the legend
            commands.entity(entity).despawn_recursive();
        }
        (None, None) => {
            // Nothing to do — no overlay, no entity.
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use kurulus_core::resources::city::OverlaySettings;
    use std::time::Duration;

    /// Helper: build a minimal app with the legend system installed.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<OverlaySettings>()
            .add_systems(Update, zone_legend_render_system);
        app
    }

    /// Run N frames.
    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── Legend content tests ───────────────────────────────────────────

    #[test]
    fn test_zone_legend_contains_all_types() {
        let body = zone_legend_body();
        assert!(body.contains("Residential"));
        assert!(body.contains("Military"));
        assert!(body.contains("Economic"));
        assert!(body.contains("Religious"));
        assert!(body.contains("Admin"));
        assert!(body.contains("No Zone"));
        assert!(body.contains("Zone View"));
    }

    #[test]
    fn test_coverage_legend_contains_all_types() {
        let body = coverage_legend_body();
        assert!(body.contains("Full Coverage"));
        assert!(body.contains("Partial Coverage"));
        assert!(body.contains("No Coverage"));
        assert!(body.contains("Service Coverage"));
    }

    #[test]
    fn test_happiness_legend_contains_all_types() {
        let body = happiness_legend_body();
        assert!(body.contains("Happy"));
        assert!(body.contains("Neutral"));
        assert!(body.contains("Unhappy"));
        assert!(body.contains("Miserable"));
        assert!(body.contains("Happiness Map"));
    }

    #[test]
    fn test_none_mode_returns_none() {
        assert!(legend_for_mode(OverlayMode::None).is_none());
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_no_entity_when_overlay_none() {
        let mut app = test_app();
        run_frames(&mut app, 3);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ZoneLegendRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "No legend entity when mode is None");
    }

    #[test]
    fn test_spawns_entity_when_overlay_active() {
        let mut app = test_app();

        // Enable Zones overlay
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;

        run_frames(&mut app, 3);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ZoneLegendRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(
            count, 1,
            "Legend entity should exist when overlay is active"
        );
    }

    #[test]
    fn test_text_depends_on_mode() {
        let mut app = test_app();

        // Set to Coverage mode
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::ServiceCoverage;

        run_frames(&mut app, 3);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ZoneLegendRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert!(
            text.0.contains("Service Coverage"),
            "Should show coverage legend, got: {}",
            text.0
        );
        assert!(
            text.0.contains("Full Coverage"),
            "Should mention Full Coverage"
        );
    }

    #[test]
    fn test_despawns_when_overlay_turned_off() {
        let mut app = test_app();

        // Enable overlay
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;
        run_frames(&mut app, 3);

        // Then turn off
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::None;
        run_frames(&mut app, 3);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<ZoneLegendRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "Legend should despawn when overlay turns off");
    }

    #[test]
    fn test_switching_mode_updates_text() {
        let mut app = test_app();

        // Start with Zones
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;
        run_frames(&mut app, 3);

        // Switch to Happiness
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Happiness;
        run_frames(&mut app, 3);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<ZoneLegendRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert!(
            text.0.contains("Happiness Map"),
            "Text should update to Happiness mode, got: {}",
            text.0
        );
        assert!(text.0.contains("Unhappy"), "Should mention Unhappy");
        // Should not contain old zone content
        assert!(
            !text.0.contains("Residential"),
            "Should not contain old zone content"
        );
    }
}
