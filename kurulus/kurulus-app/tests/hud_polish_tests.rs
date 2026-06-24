//! Integration tests for Phase 4 HUD Polish features:
//! font scaling, visibility toggles, notification fade effects,
//! and colourblind mode support.

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a minimal app with `register_logic_plugins` (registers all 22
/// logic plugins including UILogicPlugin) for full HUD integration tests.
fn hud_polish_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
    register_logic_plugins(&mut app);
    app
}

fn run_frames(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

// ─── HudSettings Initialization ──────────────────────────────────────────────

#[test]
fn test_hud_settings_defaults() {
    let settings = HudSettings::default();
    assert!((settings.font_scale - 1.0).abs() < f32::EPSILON);
    assert_eq!(settings.colorblind_mode, ColorblindMode::Normal);
    assert_eq!(settings.panel_visibility.len(), 7);
    for (_, visible) in &settings.panel_visibility {
        assert!(*visible, "All panels should be visible by default");
    }
}

#[test]
fn test_hud_settings_is_visible_unknown_panel() {
    let settings = HudSettings {
        panel_visibility: vec![],
        ..Default::default()
    };
    assert!(settings.is_visible(HudPanel::TopBar));
}

#[test]
fn test_hud_settings_set_panel_by_name() {
    let mut settings = HudSettings::default();
    assert!(settings.is_visible(HudPanel::Notifications));

    settings.set_panel_by_name("notifications", false);
    assert!(!settings.is_visible(HudPanel::Notifications));

    settings.set_panel_by_name("notifications", true);
    assert!(settings.is_visible(HudPanel::Notifications));
}

#[test]
fn test_hud_panel_name_mapping() {
    assert_eq!(HudPanel::TopBar.name(), "top_bar");
    assert_eq!(HudPanel::Minimap.name(), "minimap");
    assert_eq!(HudPanel::Notifications.name(), "notifications");
    assert_eq!(HudPanel::Objectives.name(), "objectives");
    assert_eq!(HudPanel::HeroBar.name(), "hero_bar");
    assert_eq!(HudPanel::CommandBar.name(), "command_bar");
    assert_eq!(HudPanel::SelectionPanel.name(), "selection_panel");
}

// ─── Scaling Integration ─────────────────────────────────────────────────────

#[test]
fn test_scaling_applied_to_top_bar() {
    let mut app = hud_polish_test_app();

    // Run frames to spawn the top bar entity
    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist after render");
    let initial_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Double the scale
    app.world_mut().resource_mut::<HudSettings>().font_scale = 2.0;
    run_frames(&mut app, 6);

    let scaled_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;
    let expected = initial_font * 2.0;
    assert!(
        (scaled_font - expected).abs() < 0.1,
        "Font should approximately double: initial={} scaled={} expected={}",
        initial_font,
        scaled_font,
        expected
    );
}

#[test]
fn test_scaling_without_change_preserves_font() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let font1 = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Run more frames without changing scale
    run_frames(&mut app, 10);

    let font2 = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;
    assert!(
        (font1 - font2).abs() < 0.01,
        "Font should not change without scaling: {} → {}",
        font1,
        font2
    );
}

#[test]
fn test_scaling_down_then_up() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let original = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Scale up to 1.5
    app.world_mut().resource_mut::<HudSettings>().font_scale = 1.5;
    run_frames(&mut app, 6);
    let font_up = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Scale back to 1.0
    app.world_mut().resource_mut::<HudSettings>().font_scale = 1.0;
    run_frames(&mut app, 6);
    let font_back = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    assert!(
        (font_back - original).abs() < 0.1,
        "Font should return close to original after scaling back: original={} back={} up={}",
        original,
        font_back,
        font_up
    );
}

#[test]
fn test_scaling_clamps_to_minimum() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let initial_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Scale to 0.1 — should clamp to 0.5
    app.world_mut().resource_mut::<HudSettings>().font_scale = 0.1;
    run_frames(&mut app, 6);

    let min_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;
    let expected_min = initial_font * 0.5;
    assert!(
        (min_font - expected_min).abs() < 0.1,
        "Font should be at minimum ({}), got {}",
        expected_min,
        min_font
    );
}

#[test]
fn test_scaling_clamps_to_maximum() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let initial_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;

    // Scale to 3.0 — should clamp to 2.0
    app.world_mut().resource_mut::<HudSettings>().font_scale = 3.0;
    run_frames(&mut app, 6);

    let max_font = app
        .world()
        .get::<TextFont>(top_bar_entity)
        .unwrap()
        .font_size;
    let expected_max = initial_font * 2.0;
    assert!(
        (max_font - expected_max).abs() < 0.1,
        "Font should be at maximum ({}), got {}",
        expected_max,
        max_font
    );
}

// ─── Visibility Integration ──────────────────────────────────────────────────

#[test]
fn test_visibility_hides_top_bar() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");

    app.world_mut()
        .resource_mut::<HudSettings>()
        .set_panel_by_name("top_bar", false);
    run_frames(&mut app, 6);

    let vis = app.world().get::<Visibility>(top_bar_entity).unwrap();
    assert_eq!(
        *vis,
        Visibility::Hidden,
        "TopBar should be hidden after toggle"
    );
}

#[test]
fn test_visibility_individual_toggle() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let notif_entity = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    // Hide only notifications
    app.world_mut()
        .resource_mut::<HudSettings>()
        .set_panel_by_name("notifications", false);
    run_frames(&mut app, 6);

    assert_eq!(
        *app.world().get::<Visibility>(top_bar_entity).unwrap(),
        Visibility::Visible,
        "TopBar should remain visible"
    );
    assert_eq!(
        *app.world().get::<Visibility>(notif_entity).unwrap(),
        Visibility::Hidden,
        "Notifications should be hidden"
    );
}

#[test]
fn test_visibility_toggle_reapply() {
    let mut app = hud_polish_test_app();

    run_frames(&mut app, 6);

    let top_bar_entity = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");

    // Hide
    app.world_mut()
        .resource_mut::<HudSettings>()
        .set_panel_by_name("top_bar", false);
    run_frames(&mut app, 6);
    assert_eq!(
        *app.world().get::<Visibility>(top_bar_entity).unwrap(),
        Visibility::Hidden
    );

    // Show again
    app.world_mut()
        .resource_mut::<HudSettings>()
        .set_panel_by_name("top_bar", true);
    run_frames(&mut app, 6);
    assert_eq!(
        *app.world().get::<Visibility>(top_bar_entity).unwrap(),
        Visibility::Visible,
        "TopBar should be re-shown"
    );
}

// ─── Notification Fade Integration ───────────────────────────────────────────

#[test]
fn test_notification_fade_shows_prefix() {
    let mut app = hud_polish_test_app();

    // Push a nearly-expired entry directly into the stack
    let mut entry =
        NotificationEntry::new(NotificationType::Warning, "Nearly expired".into(), 10.0);
    entry.time_remaining = 2.0; // 20% — nearly gone
    app.world_mut()
        .resource_mut::<NotificationStack>()
        .entries
        .push(entry);

    run_frames(&mut app, 6);

    let notif_entity = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    let text = app.world().get::<Text>(notif_entity).unwrap();
    assert!(
        text.0.contains("  · "),
        "Nearly-expired notification should have '  · ' prefix, got: {}",
        text.0
    );
}

#[test]
fn test_notification_colorblind_mode_in_render() {
    let mut app = hud_polish_test_app();

    // Switch to colourblind mode
    app.world_mut()
        .resource_mut::<HudSettings>()
        .colorblind_mode = ColorblindMode::Protanopia;

    // Push a notification
    app.world_mut()
        .resource_mut::<Events<Notification>>()
        .send(Notification {
            message: "CB test".into(),
            notification_type: NotificationType::Success,
            duration: Some(10.0),
        });

    run_frames(&mut app, 10);

    let notif_entity = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    let text = app.world().get::<Text>(notif_entity).unwrap();
    assert!(
        text.0.contains("[OK]"),
        "Colourblind mode should show [OK] instead of ✅, got: {}",
        text.0
    );
}

#[test]
fn test_notification_normal_mode_uses_emoji() {
    let mut app = hud_polish_test_app();

    // Ensure normal mode (default)
    app.world_mut()
        .resource_mut::<HudSettings>()
        .colorblind_mode = ColorblindMode::Normal;

    app.world_mut()
        .resource_mut::<Events<Notification>>()
        .send(Notification {
            message: "Normal test".into(),
            notification_type: NotificationType::Info,
            duration: Some(10.0),
        });

    run_frames(&mut app, 10);

    let notif_entity = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    let text = app.world().get::<Text>(notif_entity).unwrap();
    assert!(
        text.0.contains("ℹ️"),
        "Normal mode should show ℹ️, got: {}",
        text.0
    );
}

// ─── Notification Fade + Colorblind Combined ───────────────────────────────

#[test]
fn test_notification_fade_and_colorblind_combined() {
    let mut app = hud_polish_test_app();

    // Switch to colourblind
    app.world_mut()
        .resource_mut::<HudSettings>()
        .colorblind_mode = ColorblindMode::Achromatopsia;

    // Add a nearly-expired entry
    let mut entry = NotificationEntry::new(NotificationType::Error, "Combined".into(), 10.0);
    entry.time_remaining = 1.5; // 15% — nearly gone
    app.world_mut()
        .resource_mut::<NotificationStack>()
        .entries
        .push(entry);

    run_frames(&mut app, 6);

    let notif_entity = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    let text = app.world().get::<Text>(notif_entity).unwrap();
    assert!(
        text.0.contains("  · "),
        "Fade prefix should be present: {}",
        text.0
    );
    assert!(
        text.0.contains("[X]"),
        "Colourblind icon should be used: {}",
        text.0
    );
}

// ─── Multiple Panels Scaling ─────────────────────────────────────────────────

#[test]
fn test_multiple_panels_scaled_together() {
    let mut app = hud_polish_test_app();

    // Run to spawn panels
    run_frames(&mut app, 8);

    let top_e = app
        .world_mut()
        .query_filtered::<Entity, With<TopBarRoot>>()
        .iter(app.world())
        .next()
        .expect("TopBar should exist");
    let notif_e = app
        .world_mut()
        .query_filtered::<Entity, With<NotificationStackRoot>>()
        .iter(app.world())
        .next()
        .expect("NotificationStack should exist");

    let font_top_initial = app.world().get::<TextFont>(top_e).unwrap().font_size;
    let font_notif_initial = app.world().get::<TextFont>(notif_e).unwrap().font_size;

    // Scale up
    app.world_mut().resource_mut::<HudSettings>().font_scale = 1.5;
    run_frames(&mut app, 6);

    let font_top_scaled = app.world().get::<TextFont>(top_e).unwrap().font_size;
    let font_notif_scaled = app.world().get::<TextFont>(notif_e).unwrap().font_size;

    assert!(
        (font_top_scaled - font_top_initial * 1.5).abs() < 0.1,
        "TopBar font should scale: {} vs {}",
        font_top_scaled,
        font_top_initial * 1.5
    );
    assert!(
        (font_notif_scaled - font_notif_initial * 1.5).abs() < 0.1,
        "Notification font should scale: {} vs {}",
        font_notif_scaled,
        font_notif_initial * 1.5
    );
}

// ─── Objectives Colourblind Integration ─────────────────────────────────────

#[test]
fn test_objectives_colorblind_mode_in_render() {
    let mut app = hud_polish_test_app();

    // Switch to colourblind mode
    app.world_mut()
        .resource_mut::<HudSettings>()
        .colorblind_mode = ColorblindMode::Protanopia;

    // Add a completed objective
    app.world_mut()
        .resource_mut::<ObjectiveTracker>()
        .primary
        .push(ObjectiveEntry {
            id: "p1".into(),
            description: "Win the mission".into(),
            completed: true,
            progress: None,
        });

    run_frames(&mut app, 8);

    let roots: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
        .iter(app.world())
        .collect();
    assert_eq!(roots.len(), 1, "ObjectivesPanel should exist");
    let text = app.world().get::<Text>(roots[0]).unwrap();

    assert!(
        text.0.contains("[DONE]"),
        "Objectives should use [DONE] in colourblind mode, got: {}",
        text.0
    );
}

#[test]
fn test_objectives_normal_mode_uses_emoji() {
    let mut app = hud_polish_test_app();

    // Ensure normal mode (default)
    app.world_mut()
        .resource_mut::<HudSettings>()
        .colorblind_mode = ColorblindMode::Normal;

    // Add a completed objective
    app.world_mut()
        .resource_mut::<ObjectiveTracker>()
        .primary
        .push(ObjectiveEntry {
            id: "p1".into(),
            description: "Win".into(),
            completed: true,
            progress: None,
        });

    run_frames(&mut app, 8);

    let roots: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<ObjectivesPanelRoot>>()
        .iter(app.world())
        .collect();
    assert_eq!(roots.len(), 1);
    let text = app.world().get::<Text>(roots[0]).unwrap();

    assert!(
        text.0.contains("✅"),
        "Objectives should use ✅ in normal mode, got: {}",
        text.0
    );
}
