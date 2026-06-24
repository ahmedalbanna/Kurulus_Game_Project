//! HUD Polish Module — scaling, visibility toggles, accessibility, fade effects.
//!
//! ## Systems
//!
//! | System | Schedule | Purpose |
//! |--------|----------|---------|
//! | `hud_scaling_system` | Update | Adjusts `TextFont.font_size` by `HudSettings.font_scale` ratio |
//! | `hud_visibility_system` | Update | Shows / hides panels via `Visibility` |
//!
//! ## Helpers (exported for other HUD modules)
//!
//! - `colorblind_icon` — colourblind-safe notification icon text
//! - `fade_prefix` — fade-level visual indicator for notifications
//! - `colorblind_bool_text` — yes/no with text (not just emoji)

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// Scaling System
// ═══════════════════════════════════════════════════════════════════════════════

/// Scales `TextFont.font_size` on all HUD root entities whenever
/// `HudSettings.font_scale` changes.
///
/// Uses a ratio-based approach (`new_scale / old_scale`) so that the
/// base font sizes set by each render system remain the reference.
pub fn hud_scaling_system(
    settings: Res<HudSettings>,
    mut query: Query<
        &mut TextFont,
        Or<(
            With<TopBarRoot>,
            With<CommandBarRoot>,
            With<SelectionPanelRoot>,
            With<HeroBarRoot>,
            With<NotificationStackRoot>,
            With<ObjectivesPanelRoot>,
            With<MinimapRoot>,
        )>,
    >,
    mut initialized: Local<bool>,
    mut previous_scale: Local<f32>,
) {
    let scale = settings.font_scale.clamp(0.5, 2.0);

    // First frame: store baseline and skip.
    if !*initialized {
        *previous_scale = scale;
        *initialized = true;
        return;
    }

    // No change — skip.
    if (scale - *previous_scale).abs() < f32::EPSILON {
        return;
    }

    let ratio = scale / *previous_scale;
    if ratio.is_finite() && ratio > 0.0 {
        for mut font in query.iter_mut() {
            font.font_size *= ratio;
        }
    }

    *previous_scale = scale;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Visibility System
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies `HudSettings` visibility flags to each HUD panel root entity.
///
/// Each panel is queried separately and its `Visibility` component is set to
/// `Visible` or `Hidden` according to `HudSettings.is_visible(panel)`.
pub fn hud_visibility_system(
    settings: Res<HudSettings>,
    top_bar: Query<Entity, With<TopBarRoot>>,
    cmd_bar: Query<Entity, With<CommandBarRoot>>,
    sel_panel: Query<Entity, With<SelectionPanelRoot>>,
    hero_bar: Query<Entity, With<HeroBarRoot>>,
    notifications: Query<Entity, With<NotificationStackRoot>>,
    objectives: Query<Entity, With<ObjectivesPanelRoot>>,
    minimap: Query<Entity, With<MinimapRoot>>,
    mut visibility: Query<&mut Visibility>,
) {
    fn apply(
        settings: &HudSettings,
        panel: HudPanel,
        entity: Entity,
        vis_q: &mut Query<&mut Visibility>,
    ) {
        if let Ok(mut v) = vis_q.get_mut(entity) {
            *v = if settings.is_visible(panel) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }

    if let Ok(entity) = top_bar.get_single() {
        apply(&settings, HudPanel::TopBar, entity, &mut visibility);
    }
    if let Ok(entity) = cmd_bar.get_single() {
        apply(&settings, HudPanel::CommandBar, entity, &mut visibility);
    }
    if let Ok(entity) = sel_panel.get_single() {
        apply(&settings, HudPanel::SelectionPanel, entity, &mut visibility);
    }
    if let Ok(entity) = hero_bar.get_single() {
        apply(&settings, HudPanel::HeroBar, entity, &mut visibility);
    }
    if let Ok(entity) = notifications.get_single() {
        apply(&settings, HudPanel::Notifications, entity, &mut visibility);
    }
    if let Ok(entity) = objectives.get_single() {
        apply(&settings, HudPanel::Objectives, entity, &mut visibility);
    }
    if let Ok(entity) = minimap.get_single() {
        apply(&settings, HudPanel::Minimap, entity, &mut visibility);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Colourblind Helpers (exported for other HUD modules)
// ═══════════════════════════════════════════════════════════════════════════════

/// Return a colourblind-friendly icon for a notification type.
///
/// When `mode` is `ColorblindMode::Normal`, standard emoji are returned.
/// In any colourblind mode, descriptive text labels are used instead
/// so that the notification type is distinguishable without colour vision.
pub fn colorblind_icon(nt: NotificationType, mode: ColorblindMode) -> &'static str {
    if mode == ColorblindMode::Normal {
        return match nt {
            NotificationType::Info => "ℹ️",
            NotificationType::Warning => "⚠️",
            NotificationType::Error => "‼️",
            NotificationType::Success => "✅",
            NotificationType::Achievement => "🏆",
        };
    }

    // Colourblind-safe text labels
    match nt {
        NotificationType::Info => "[I]",
        NotificationType::Warning => "[!]",
        NotificationType::Error => "[X]",
        NotificationType::Success => "[OK]",
        NotificationType::Achievement => "[★]",
    }
}

/// Return a fade-indicator prefix based on remaining lifetime ratio.
///
/// Used by the notification render system to visually hint at
/// imminent expiry without relying on animation.
///
/// | `fade_pct` | Prefix | Meaning |
/// |------------|--------|---------|
/// | > 0.50     | `""`   | Full visibility |
/// | 0.25 – 0.50 | `"· "` | Dimming |
/// | 0.0 – 0.25 | `"  · "` | Nearly gone |
pub fn fade_prefix(fade_pct: f32) -> &'static str {
    if fade_pct > 0.50 {
        "" // full
    } else if fade_pct > 0.25 {
        "· " // dimming
    } else if fade_pct > 0.0 {
        "  · " // nearly gone
    } else {
        "" // expired (shouldn't be displayed)
    }
}

/// Return colourblind-safe boolean display text.
/// In normal mode, standard check/cross emoji are used.
/// In colourblind mode, text labels provide the same information.
pub fn colorblind_bool_text(completed: bool, mode: ColorblindMode) -> &'static str {
    if mode == ColorblindMode::Normal {
        if completed {
            "✅"
        } else {
            "☐"
        }
    } else {
        if completed {
            "[DONE]"
        } else {
            "[TODO]"
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
    use std::time::Duration;

    // ─── Scaling System Tests ───────────────────────────────────────────

    fn scaling_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<HudSettings>()
            .add_systems(Update, hud_scaling_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    #[test]
    fn test_scaling_initializes_without_changing_font() {
        let mut app = scaling_test_app();

        // Spawn a HUD entity with a known font size
        let e = app
            .world_mut()
            .spawn((
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TopBarRoot,
            ))
            .id();

        run_frames(&mut app, 2);

        // First frame initialises, second frame should have no change
        let font = app.world().get::<TextFont>(e).unwrap();
        assert_eq!(
            font.font_size, 14.0,
            "Font size should remain unchanged when scale is 1.0"
        );
    }

    #[test]
    fn test_scaling_updates_when_scale_changes() {
        let mut app = scaling_test_app();

        let e = app
            .world_mut()
            .spawn((
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TopBarRoot,
            ))
            .id();

        // Run first frame to initialise
        run_frames(&mut app, 2);

        // Change scale
        app.world_mut().resource_mut::<HudSettings>().font_scale = 1.5;
        run_frames(&mut app, 2);

        let font = app.world().get::<TextFont>(e).unwrap();
        assert!(
            (font.font_size - 21.0).abs() < 0.01,
            "Font should be 21.0 (14 * 1.5), got {}",
            font.font_size
        );
    }

    #[test]
    fn test_scaling_down() {
        let mut app = scaling_test_app();

        let e = app
            .world_mut()
            .spawn((
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TopBarRoot,
            ))
            .id();

        run_frames(&mut app, 2);

        app.world_mut().resource_mut::<HudSettings>().font_scale = 0.75;
        run_frames(&mut app, 2);

        let font = app.world().get::<TextFont>(e).unwrap();
        assert!(
            (font.font_size - 10.5).abs() < 0.01,
            "Font should be 10.5 (14 * 0.75), got {}",
            font.font_size
        );
    }

    #[test]
    fn test_scaling_multiple_changes() {
        let mut app = scaling_test_app();

        let e = app
            .world_mut()
            .spawn((
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TopBarRoot,
            ))
            .id();

        // First frame: initialise
        run_frames(&mut app, 2);

        // Change to 1.5
        app.world_mut().resource_mut::<HudSettings>().font_scale = 1.5;
        run_frames(&mut app, 2);
        let font = app.world().get::<TextFont>(e).unwrap();
        assert!((font.font_size - 21.0).abs() < 0.01);

        // Change to 2.0 (ratio = 2.0 / 1.5 ≈ 1.333)
        app.world_mut().resource_mut::<HudSettings>().font_scale = 2.0;
        run_frames(&mut app, 2);
        let font = app.world().get::<TextFont>(e).unwrap();
        assert!(
            (font.font_size - 28.0).abs() < 0.01,
            "Font should be 28.0, got {}",
            font.font_size
        );
    }

    #[test]
    fn test_scaling_clamps_scale_range() {
        let mut app = scaling_test_app();

        let e = app
            .world_mut()
            .spawn((
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TopBarRoot,
            ))
            .id();

        run_frames(&mut app, 2);

        // Set to a value below 0.5 — should clamp
        app.world_mut().resource_mut::<HudSettings>().font_scale = 0.1;
        run_frames(&mut app, 2);

        let font = app.world().get::<TextFont>(e).unwrap();
        // 0.5 / 1.0 = 0.5 ratio; 14 * 0.5 = 7.0
        assert!(
            (font.font_size - 7.0).abs() < 0.01,
            "Font should be 7.0, got {}",
            font.font_size
        );
    }

    #[test]
    fn test_scaling_affects_all_panel_types() {
        let mut app = scaling_test_app();
        run_frames(&mut app, 2); // initialise

        // Spawn one entity for each marker type
        let entities = [
            (
                "TopBarRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TopBarRoot,
                    ))
                    .id(),
            ),
            (
                "CommandBarRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        CommandBarRoot,
                    ))
                    .id(),
            ),
            (
                "SelectionPanelRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        SelectionPanelRoot,
                    ))
                    .id(),
            ),
            (
                "HeroBarRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        HeroBarRoot,
                    ))
                    .id(),
            ),
            (
                "NotificationStackRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        NotificationStackRoot,
                    ))
                    .id(),
            ),
            (
                "ObjectivesPanelRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        ObjectivesPanelRoot,
                    ))
                    .id(),
            ),
            (
                "MinimapRoot",
                app.world_mut()
                    .spawn((
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        MinimapRoot,
                    ))
                    .id(),
            ),
        ];

        // Scale up
        app.world_mut().resource_mut::<HudSettings>().font_scale = 2.0;
        run_frames(&mut app, 2);

        for (name, entity) in &entities {
            let font = app.world().get::<TextFont>(*entity).unwrap();
            assert!(
                (font.font_size - 28.0).abs() < 0.01,
                "{} font should be scaled to 28.0 (14 * 2), got {}",
                name,
                font.font_size
            );
        }
    }

    #[test]
    fn test_scaling_skips_non_hud_entities() {
        let mut app = scaling_test_app();

        let e = app
            .world_mut()
            .spawn((TextFont {
                font_size: 14.0,
                ..default()
            },))
            .id(); // No HUD marker

        run_frames(&mut app, 2); // initialise
        app.world_mut().resource_mut::<HudSettings>().font_scale = 2.0;
        run_frames(&mut app, 2);

        let font = app.world().get::<TextFont>(e).unwrap();
        assert!(
            (font.font_size - 14.0).abs() < 0.01,
            "Entities without HUD markers should not be scaled"
        );
    }

    // ─── Visibility System Tests ────────────────────────────────────────

    fn visibility_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<HudSettings>()
            .add_systems(Update, hud_visibility_system);
        app
    }

    #[test]
    fn test_visibility_shows_all_by_default() {
        let mut app = visibility_test_app();

        let e1 = app
            .world_mut()
            .spawn((TopBarRoot, Visibility::Visible))
            .id();
        let e2 = app
            .world_mut()
            .spawn((MinimapRoot, Visibility::Visible))
            .id();

        run_frames(&mut app, 2);

        assert_eq!(
            *app.world().get::<Visibility>(e1).unwrap(),
            Visibility::Visible,
            "TopBar should be visible by default"
        );
        assert_eq!(
            *app.world().get::<Visibility>(e2).unwrap(),
            Visibility::Visible,
            "Minimap should be visible by default"
        );
    }

    #[test]
    fn test_visibility_hides_panel() {
        let mut app = visibility_test_app();

        let e = app
            .world_mut()
            .spawn((TopBarRoot, Visibility::Visible))
            .id();
        run_frames(&mut app, 2);

        // Hide TopBar
        app.world_mut()
            .resource_mut::<HudSettings>()
            .set_panel_by_name("top_bar", false);
        run_frames(&mut app, 2);

        assert_eq!(
            *app.world().get::<Visibility>(e).unwrap(),
            Visibility::Hidden,
            "TopBar should be hidden after toggle"
        );
    }

    #[test]
    fn test_visibility_toggle_panel() {
        let mut app = visibility_test_app();

        let e = app
            .world_mut()
            .spawn((TopBarRoot, Visibility::Visible))
            .id();
        run_frames(&mut app, 2);

        // Hide
        app.world_mut()
            .resource_mut::<HudSettings>()
            .set_panel_by_name("top_bar", false);
        run_frames(&mut app, 2);
        assert_eq!(
            *app.world().get::<Visibility>(e).unwrap(),
            Visibility::Hidden
        );

        // Show again
        app.world_mut()
            .resource_mut::<HudSettings>()
            .set_panel_by_name("top_bar", true);
        run_frames(&mut app, 2);
        assert_eq!(
            *app.world().get::<Visibility>(e).unwrap(),
            Visibility::Visible,
            "TopBar should be visible after re-show"
        );
    }

    #[test]
    fn test_visibility_individual_panels() {
        let mut app = visibility_test_app();

        let top = app
            .world_mut()
            .spawn((TopBarRoot, Visibility::Visible))
            .id();
        let minimap = app
            .world_mut()
            .spawn((MinimapRoot, Visibility::Visible))
            .id();
        run_frames(&mut app, 2);

        // Hide only minimap
        app.world_mut()
            .resource_mut::<HudSettings>()
            .set_panel_by_name("minimap", false);
        run_frames(&mut app, 2);

        assert_eq!(
            *app.world().get::<Visibility>(top).unwrap(),
            Visibility::Visible,
            "TopBar should remain visible"
        );
        assert_eq!(
            *app.world().get::<Visibility>(minimap).unwrap(),
            Visibility::Hidden,
            "Minimap should be hidden"
        );
    }

    // ─── Colourblind Helper Tests ────────────────────────────────────────

    #[test]
    fn test_colorblind_icon_normal_uses_emoji() {
        assert_eq!(
            colorblind_icon(NotificationType::Info, ColorblindMode::Normal),
            "ℹ️"
        );
        assert_eq!(
            colorblind_icon(NotificationType::Warning, ColorblindMode::Normal),
            "⚠️"
        );
        assert_eq!(
            colorblind_icon(NotificationType::Error, ColorblindMode::Normal),
            "‼️"
        );
        assert_eq!(
            colorblind_icon(NotificationType::Success, ColorblindMode::Normal),
            "✅"
        );
        assert_eq!(
            colorblind_icon(NotificationType::Achievement, ColorblindMode::Normal),
            "🏆"
        );
    }

    #[test]
    fn test_colorblind_icon_protanopia_uses_text() {
        let mode = ColorblindMode::Protanopia;
        assert_eq!(colorblind_icon(NotificationType::Info, mode), "[I]");
        assert_eq!(colorblind_icon(NotificationType::Warning, mode), "[!]");
        assert_eq!(colorblind_icon(NotificationType::Error, mode), "[X]");
        assert_eq!(colorblind_icon(NotificationType::Success, mode), "[OK]");
        assert_eq!(colorblind_icon(NotificationType::Achievement, mode), "[★]");
    }

    #[test]
    fn test_colorblind_icon_deuteranopia() {
        assert_eq!(
            colorblind_icon(NotificationType::Info, ColorblindMode::Deuteranopia),
            "[I]"
        );
        assert_eq!(
            colorblind_icon(NotificationType::Warning, ColorblindMode::Deuteranopia),
            "[!]"
        );
    }

    #[test]
    fn test_colorblind_icon_tritanopia() {
        assert_eq!(
            colorblind_icon(NotificationType::Error, ColorblindMode::Tritanopia),
            "[X]"
        );
    }

    #[test]
    fn test_colorblind_icon_achromatopsia() {
        for nt in &[
            NotificationType::Info,
            NotificationType::Warning,
            NotificationType::Error,
            NotificationType::Success,
            NotificationType::Achievement,
        ] {
            let icon = colorblind_icon(*nt, ColorblindMode::Achromatopsia);
            assert!(
                icon.starts_with('['),
                "Achromatopsia icon should be text: {}",
                icon
            );
        }
    }

    // ─── Fade Prefix Tests ───────────────────────────────────────────────

    #[test]
    fn test_fade_prefix_full() {
        assert_eq!(fade_prefix(0.75), "");
        assert_eq!(fade_prefix(1.0), "");
    }

    #[test]
    fn test_fade_prefix_dimming() {
        assert_eq!(fade_prefix(0.50), "· ");
        assert_eq!(fade_prefix(0.30), "· ");
    }

    #[test]
    fn test_fade_prefix_nearly_gone() {
        assert_eq!(fade_prefix(0.25), "  · ");
        assert_eq!(fade_prefix(0.10), "  · ");
    }

    #[test]
    fn test_fade_prefix_boundary_50() {
        assert_eq!(fade_prefix(0.50), "· ");
        assert_eq!(fade_prefix(0.5001), "");
    }

    #[test]
    fn test_fade_prefix_boundary_25() {
        assert_eq!(fade_prefix(0.25), "  · ");
        assert_eq!(fade_prefix(0.2501), "· ");
    }

    #[test]
    fn test_fade_prefix_zero() {
        assert_eq!(fade_prefix(0.0), "");
    }

    // ─── Colorblind Bool Text Tests ──────────────────────────────────────

    #[test]
    fn test_colorblind_bool_text_normal() {
        assert_eq!(colorblind_bool_text(true, ColorblindMode::Normal), "✅");
        assert_eq!(colorblind_bool_text(false, ColorblindMode::Normal), "☐");
    }

    #[test]
    fn test_colorblind_bool_text_protanopia() {
        assert_eq!(
            colorblind_bool_text(true, ColorblindMode::Protanopia),
            "[DONE]"
        );
        assert_eq!(
            colorblind_bool_text(false, ColorblindMode::Protanopia),
            "[TODO]"
        );
    }

    #[test]
    fn test_colorblind_bool_text_achromatopsia() {
        assert_eq!(
            colorblind_bool_text(true, ColorblindMode::Achromatopsia),
            "[DONE]"
        );
        assert_eq!(
            colorblind_bool_text(false, ColorblindMode::Achromatopsia),
            "[TODO]"
        );
    }

    // ─── HudPanel Tests ──────────────────────────────────────────────────

    #[test]
    fn test_hud_panel_name_mapping() {
        assert_eq!(HudPanel::TopBar.name(), "top_bar");
        assert_eq!(HudPanel::SelectionPanel.name(), "selection_panel");
        assert_eq!(HudPanel::CommandBar.name(), "command_bar");
        assert_eq!(HudPanel::HeroBar.name(), "hero_bar");
        assert_eq!(HudPanel::Notifications.name(), "notifications");
        assert_eq!(HudPanel::Objectives.name(), "objectives");
        assert_eq!(HudPanel::Minimap.name(), "minimap");
    }

    #[test]
    fn test_hud_settings_default_all_visible() {
        let settings = HudSettings::default();
        assert!(settings.is_visible(HudPanel::TopBar));
        assert!(settings.is_visible(HudPanel::Minimap));
        assert!(settings.is_visible(HudPanel::Notifications));
        assert!(settings.is_visible(HudPanel::Objectives));
        assert!(settings.is_visible(HudPanel::HeroBar));
        assert!(settings.is_visible(HudPanel::CommandBar));
        assert!(settings.is_visible(HudPanel::SelectionPanel));
    }

    #[test]
    fn test_hud_settings_toggle_by_name() {
        let mut settings = HudSettings::default();
        assert!(settings.is_visible(HudPanel::Minimap));

        settings.set_panel_by_name("minimap", false);
        assert!(!settings.is_visible(HudPanel::Minimap));

        settings.set_panel_by_name("minimap", true);
        assert!(settings.is_visible(HudPanel::Minimap));
    }

    #[test]
    fn test_hud_settings_unknown_name_noop() {
        let mut settings = HudSettings::default();
        let before = settings.is_visible(HudPanel::TopBar);
        settings.set_panel_by_name("nonexistent", false);
        assert_eq!(settings.is_visible(HudPanel::TopBar), before);
    }

    #[test]
    fn test_hud_settings_default_scale() {
        let settings = HudSettings::default();
        assert!((settings.font_scale - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_hud_settings_default_colorblind() {
        let settings = HudSettings::default();
        assert_eq!(settings.colorblind_mode, ColorblindMode::Normal);
    }
}
