//! Notification Stack HUD — displays stacked notifications with priorities,
//! auto-dismiss, and type-based colouring.
//!
//! ## Architecture
//!
//! 1. **`notification_logic_system`** (FixedUpdate) — reads `Notification` events,
//!    pushes entries onto `NotificationStack`, ticks lifetimes, removes expired.
//!
//! 2. **`notification_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`NotificationStackRoot`].

use super::hud_polish::{colorblind_icon, fade_prefix};
use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Maximum visible notifications.
const MAX_VISIBLE: usize = 5;

/// Default duration for notifications that don't specify one.
const DEFAULT_DURATION: f32 = 5.0;

/// Font size for notification text.
const FONT_SIZE: f32 = 14.0;

/// Padding inside the stack.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 6.0, 6.0);

/// Width of the notification panel.
const PANEL_WIDTH: f32 = 320.0;

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.60);

/// Tick interval in FixedUpdate seconds (matches 1/60 * time_scale).
const TICK_DELTA: f32 = 0.1;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build the multi-line text for the notification stack.
///
/// Includes colourblind-safe icon text when `cm` is not `Normal`, and
/// fade-indicator prefixes for entries approaching their expiry time.
fn build_notification_text(entries: &[NotificationEntry], cm: ColorblindMode) -> String {
    entries
        .iter()
        .take(MAX_VISIBLE)
        .map(|e| {
            let fade_pct = if e.duration > 0.0 {
                (e.time_remaining / e.duration).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let icon = colorblind_icon(e.notification_type, cm);
            let fade = fade_prefix(fade_pct);
            format!("{}{} {}", fade, icon, e.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads `Notification` events, manages stack lifetimes.
pub fn notification_logic_system(
    mut stack: ResMut<NotificationStack>,
    mut events: EventReader<Notification>,
) {
    // Push new notifications
    for ev in events.read() {
        let duration = ev.duration.unwrap_or(DEFAULT_DURATION);
        stack.entries.push(NotificationEntry::new(
            ev.notification_type,
            ev.message.clone(),
            duration,
        ));
    }

    // Tick all entries
    let mut i = 0;
    while i < stack.entries.len() {
        let entry = &mut stack.entries[i];
        entry.time_remaining -= TICK_DELTA;
        if entry.time_remaining <= 0.0 {
            stack.entries.swap_remove(i);
        } else {
            i += 1;
        }
    }
}

/// Render system: spawns / updates the notification stack UI entity.
///
/// Reads `HudSettings` for colourblind mode and applies fade prefixes to
/// entries nearing their auto-dismiss time.
pub fn notification_render_system(
    mut commands: Commands,
    stack: Res<NotificationStack>,
    hud_settings: Res<HudSettings>,
    root_query: Query<Entity, With<NotificationStackRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                top: Val::Px(40.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
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
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            NotificationStackRoot,
            Name::new("NotificationStack"),
        ));
        return;
    };

    let body = build_notification_text(&stack.entries, hud_settings.colorblind_mode);
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
            .init_resource::<NotificationStack>()
            .init_resource::<HudSettings>()
            .add_event::<Notification>()
            .add_systems(FixedUpdate, notification_logic_system)
            .add_systems(Update, notification_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── Helper tests ───────────────────────────────────────────────────

    #[test]
    fn test_colorblind_icon_info() {
        assert_eq!(
            colorblind_icon(NotificationType::Info, ColorblindMode::Normal),
            "ℹ️"
        );
    }

    #[test]
    fn test_colorblind_icon_warning() {
        assert_eq!(
            colorblind_icon(NotificationType::Warning, ColorblindMode::Normal),
            "⚠️"
        );
    }

    #[test]
    fn test_colorblind_icon_error() {
        assert_eq!(
            colorblind_icon(NotificationType::Error, ColorblindMode::Normal),
            "‼️"
        );
    }

    #[test]
    fn test_colorblind_icon_success() {
        assert_eq!(
            colorblind_icon(NotificationType::Success, ColorblindMode::Normal),
            "✅"
        );
    }

    #[test]
    fn test_colorblind_icon_achievement() {
        assert_eq!(
            colorblind_icon(NotificationType::Achievement, ColorblindMode::Normal),
            "🏆"
        );
    }

    #[test]
    fn test_build_text_empty() {
        let text = build_notification_text(&[], ColorblindMode::Normal);
        assert_eq!(text, "");
    }

    #[test]
    fn test_build_text_single() {
        let entries = vec![NotificationEntry::new(
            NotificationType::Info,
            "Test message".into(),
            5.0,
        )];
        let text = build_notification_text(&entries, ColorblindMode::Normal);
        assert!(text.contains("Test message"));
        assert!(text.contains("ℹ️"));
    }

    #[test]
    fn test_build_text_multiple() {
        let entries = vec![
            NotificationEntry::new(NotificationType::Info, "First".into(), 5.0),
            NotificationEntry::new(NotificationType::Warning, "Second".into(), 5.0),
        ];
        let text = build_notification_text(&entries, ColorblindMode::Normal);
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(
            text.contains('\n'),
            "Multiple entries should be newline-separated"
        );
    }

    // ── Logic system tests ─────────────────────────────────────────────

    #[test]
    fn test_logic_pushes_event() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "Unit damaged!".into(),
                notification_type: NotificationType::Warning,
                duration: Some(5.0),
            });

        run_frames(&mut app, 2);

        let stack = app.world().resource::<NotificationStack>();
        assert_eq!(stack.entries.len(), 1);
        assert_eq!(stack.entries[0].message, "Unit damaged!");
        assert_eq!(
            stack.entries[0].notification_type,
            NotificationType::Warning
        );
    }

    #[test]
    fn test_logic_uses_default_duration_when_none() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "test".into(),
                notification_type: NotificationType::Info,
                duration: None,
            });

        run_frames(&mut app, 2);

        let stack = app.world().resource::<NotificationStack>();
        assert_eq!(stack.entries.len(), 1);
        assert!((stack.entries[0].duration - DEFAULT_DURATION).abs() < 0.01);
    }

    #[test]
    fn test_logic_removes_expired() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "short".into(),
                notification_type: NotificationType::Info,
                duration: Some(0.05), // very short
            });

        // Run enough frames for the entry to expire
        run_frames(&mut app, 20);

        let stack = app.world().resource::<NotificationStack>();
        assert!(stack.entries.is_empty(), "Short notification should expire");
    }

    #[test]
    fn test_logic_keeps_longer_entries() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "long".into(),
                notification_type: NotificationType::Info,
                duration: Some(60.0),
            });

        run_frames(&mut app, 10);

        let stack = app.world().resource::<NotificationStack>();
        assert_eq!(stack.entries.len(), 1, "Long notification should persist");
    }

    #[test]
    fn test_logic_multiple_events() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "A".into(),
                notification_type: NotificationType::Info,
                duration: Some(10.0),
            });
        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "B".into(),
                notification_type: NotificationType::Warning,
                duration: Some(10.0),
            });

        run_frames(&mut app, 2);

        let stack = app.world().resource::<NotificationStack>();
        assert_eq!(stack.entries.len(), 2);
    }

    #[test]
    fn test_logic_respects_max_visible() {
        let mut app = test_app();

        // Push more than MAX_VISIBLE
        for i in 0..MAX_VISIBLE + 3 {
            app.world_mut()
                .resource_mut::<Events<Notification>>()
                .send(Notification {
                    message: format!("Event {}", i),
                    notification_type: NotificationType::Info,
                    duration: Some(10.0),
                });
        }

        run_frames(&mut app, 2);

        let stack = app.world().resource::<NotificationStack>();
        // Logic doesn't cap — the render system takes MAX_VISIBLE
        // But the stack should still have all entries
        assert_eq!(stack.entries.len(), MAX_VISIBLE + 3);
        // Build text should only show MAX_VISIBLE
        let text = build_notification_text(&stack.entries, ColorblindMode::Normal);
        assert_eq!(text.lines().count(), MAX_VISIBLE);
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<NotificationStackRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn NotificationStackRoot entity");
    }

    #[test]
    fn test_render_shows_notification_text() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "Test notification".into(),
                notification_type: NotificationType::Info,
                duration: Some(10.0),
            });

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<NotificationStackRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Test notification"),
            "Should show message: {}",
            text.0
        );
    }

    #[test]
    fn test_render_empty_stack_shows_no_text() {
        let mut app = test_app();
        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<NotificationStackRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert!(text.0.is_empty(), "Empty stack should have no text");
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<NotificationStackRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.right, Val::Px(12.0), "Panel should be 12px from right");
        assert_eq!(node.top, Val::Px(40.0), "Panel should be 40px from top");
    }

    // ── Fade effect tests ─────────────────────────────────────────────

    #[test]
    fn test_fade_prefix_full_visibility() {
        let entry = NotificationEntry::new(NotificationType::Info, "Test".into(), 10.0);
        // Entry is freshly created so time_remaining == duration
        let entries = vec![entry];
        let text = build_notification_text(&entries, ColorblindMode::Normal);
        // No fade prefix for full visibility
        assert!(text.contains("ℹ️"), "Should contain info icon: {}", text);
        assert!(
            !text.starts_with('·'),
            "Should not start with fade prefix: {}",
            text
        );
    }

    #[test]
    fn test_fade_prefix_dimming() {
        let mut entry = NotificationEntry::new(NotificationType::Warning, "Dim".into(), 10.0);
        entry.time_remaining = 3.0; // 30% remaining → dimming range (0.25–0.50)
        let text = build_notification_text(&[entry], ColorblindMode::Normal);
        assert!(
            text.contains("· "),
            "Dimming notification should have '· ' prefix: {}",
            text
        );
    }

    #[test]
    fn test_fade_prefix_nearly_gone() {
        let mut entry = NotificationEntry::new(NotificationType::Error, "Gone".into(), 10.0);
        entry.time_remaining = 1.0; // 10% remaining → nearly gone
        let text = build_notification_text(&[entry], ColorblindMode::Normal);
        assert!(
            text.contains("  · "),
            "Nearly-gone notification should have '  · ' prefix: {}",
            text
        );
    }

    #[test]
    fn test_colorblind_mode_uses_text_icons() {
        let entries = vec![
            NotificationEntry::new(NotificationType::Info, "Info".into(), 10.0),
            NotificationEntry::new(NotificationType::Warning, "Warn".into(), 10.0),
            NotificationEntry::new(NotificationType::Error, "Err".into(), 10.0),
            NotificationEntry::new(NotificationType::Success, "OK".into(), 10.0),
            NotificationEntry::new(NotificationType::Achievement, "Ach".into(), 10.0),
        ];

        // Normal mode uses emoji
        let normal_text = build_notification_text(&entries, ColorblindMode::Normal);
        assert!(
            normal_text.contains("ℹ️"),
            "Normal should use emoji: {}",
            normal_text
        );

        // Protanopia mode uses text labels
        let cb_text = build_notification_text(&entries, ColorblindMode::Protanopia);
        assert!(
            cb_text.contains("[I]"),
            "Protanopia should use [I]: {}",
            cb_text
        );
        assert!(
            cb_text.contains("[!]"),
            "Protanopia should use [!]: {}",
            cb_text
        );
        assert!(
            cb_text.contains("[X]"),
            "Protanopia should use [X]: {}",
            cb_text
        );
        assert!(
            cb_text.contains("[OK]"),
            "Protanopia should use [OK]: {}",
            cb_text
        );
        assert!(
            cb_text.contains("[★]"),
            "Protanopia should use [★]: {}",
            cb_text
        );
    }

    #[test]
    fn test_colorblind_mode_fade_still_applied() {
        let mut entry = NotificationEntry::new(NotificationType::Info, "FadeCB".into(), 10.0);
        entry.time_remaining = 2.0; // 20% → nearly gone
        let text = build_notification_text(&[entry], ColorblindMode::Deuteranopia);
        assert!(
            text.contains("  · "),
            "Fade prefix should still apply in colorblind mode: {}",
            text
        );
        assert!(
            text.contains("[I]"),
            "Colorblind icon should be used: {}",
            text
        );
    }

    #[test]
    fn test_colorblind_mode_all_variants_use_text() {
        let entry = NotificationEntry::new(NotificationType::Info, "Test".into(), 10.0);
        for mode in &[
            ColorblindMode::Protanopia,
            ColorblindMode::Deuteranopia,
            ColorblindMode::Tritanopia,
            ColorblindMode::Achromatopsia,
        ] {
            let text = build_notification_text(&[entry.clone()], *mode);
            assert!(text.contains("[I]"), "{:?} should use [I]: {}", mode, text);
        }
    }

    #[test]
    fn test_render_respects_colorblind_mode() {
        let mut app = test_app();

        // Switch to protanopia
        app.world_mut()
            .resource_mut::<HudSettings>()
            .colorblind_mode = ColorblindMode::Protanopia;

        app.world_mut()
            .resource_mut::<Events<Notification>>()
            .send(Notification {
                message: "Colorblind test".into(),
                notification_type: NotificationType::Success,
                duration: Some(10.0),
            });

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<NotificationStackRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("[OK]"),
            "Render should use colorblind-safe [OK], got: {}",
            text.0
        );
    }
}
