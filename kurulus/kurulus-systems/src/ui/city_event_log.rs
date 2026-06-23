//! City Event Log HUD — displays recent city events as a scrolling text feed.
//!
//! ## Architecture
//!
//! Two systems:
//!
//! 1. **`city_event_log_logic_system`** (FixedUpdate) — reads `CityEventOccurred` /
//!    `CityEventEnded` events and drives the `CityEventLog` resource (adding/expiring
//!    entries). Runs unconditionally (works in headless tests).
//!
//! 2. **`city_event_log_render_system`** (Update) — spawns a single entity carrying
//!    both [`Node`] (layout) and [`Text`] (content). The text is updated in-place
//!    each frame — no child entities, no hierarchy issues. In headless test mode
//!    (no `UiPlugin`) the entity is inert; harmless.
//!
//! The entity is tagged with [`CityEventLogRoot`] so we can find it each frame.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of visible lines in the log (font-size × line-height approximation).
const MAX_VISIBLE_LINES: usize = 8;

/// Font size for log entry text.
const FONT_SIZE: f32 = 14.0;

/// Padding around the whole panel (top, right, bottom, left).
const PANEL_PADDING: UiRect = UiRect::px(8.0, 8.0, 6.0, 6.0);

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build the multi-line display string from the current log entries.
fn format_event_log(entries: &[CityEventLogEntry]) -> String {
    entries
        .iter()
        .take(MAX_VISIBLE_LINES)
        .map(|e| &e.message[..])
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads city events, maintains the [`CityEventLog`] resource.
pub fn city_event_log_logic_system(
    mut event_log: ResMut<CityEventLog>,
    mut event_occured: EventReader<CityEventOccurred>,
    mut event_ended: EventReader<CityEventEnded>,
    game_time: Res<GameTime>,
) {
    let delta = game_time.time_scale as f32 * 0.1;

    for ev in event_occured.read() {
        let mut entry = CityEventLogEntry::new(ev.event_type);
        if ev.magnitude > 0.7 {
            entry.message = format!("‼️ {} (severe)", ev.event_type.description());
        } else if ev.magnitude < 0.3 {
            entry.message = format!("ℹ️ {} (minor)", ev.event_type.description());
        } else {
            entry.message = format!("📢 {}", ev.event_type.description());
        }
        event_log.push(entry);
    }

    for ev in event_ended.read() {
        let entry = CityEventLogEntry {
            event_type: ev.event_type,
            message: format!("✅ {} resolved", ev.event_type.description()),
            time_remaining: entry_lifetime_default() * 0.6,
        };
        event_log.push(entry);
    }

    event_log.tick(delta);
}

/// Render system: spawns / updates a single entity with [`Node`] + [`Text`].
///
/// In Bevy 0.15, `Text` requires `Node`, so a single entity carries both
/// layout and content.  We update the text every frame; no child entities
/// are created or destroyed, so there are no stale-reference panics.
pub fn city_event_log_render_system(
    mut commands: Commands,
    event_log: Res<CityEventLog>,
    root_query: Query<Entity, With<CityEventLogRoot>>,
) {
    // ── Find or spawn the single UI entity ──────────────────────────────
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _ = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(12.0),
                    top: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    padding: PANEL_PADDING,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                Text::new(""),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                CityEventLogRoot,
                Name::new("CityEventLog"),
            ));

        // First frame — text is empty, nothing more to do.
        return;
    };

    // ── Rewrite the text every frame ──────────────────────────────────
    let body = format_event_log(&event_log.entries);

    let dominant_colour = event_log
        .entries
        .first()
        .map(|e| {
            if e.event_type.is_positive() {
                Color::srgb(0.3, 0.9, 0.3)
            } else {
                Color::srgb(0.9, 0.3, 0.3)
            }
        })
        .unwrap_or(Color::srgb(0.9, 0.9, 0.9));

    commands.entity(entity).insert((
        Text::new(&body),
        TextColor(dominant_colour),
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use kurulus_core::resources::city::CityEventLog;
    use kurulus_core::resources::city::CityEventType;
    use std::time::Duration;

    /// Helper: build a minimal app with the two event-log systems installed.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<CityEventLog>()
            .init_resource::<GameTime>()
            .add_event::<CityEventOccurred>()
            .add_event::<CityEventEnded>()
            .add_systems(Update, city_event_log_render_system)
            .add_systems(FixedUpdate, city_event_log_logic_system);
        app
    }

    /// Run N frames (Update + FixedUpdate if applicable).
    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── CityEventLog resource tests ───────────────────────────────────

    #[test]
    fn test_log_starts_empty() {
        let log = CityEventLog::default();
        assert!(log.entries.is_empty());
    }

    #[test]
    fn test_push_adds_front() {
        let mut log = CityEventLog::default();
        log.push(CityEventLogEntry::new(CityEventType::Festival));
        log.push(CityEventLogEntry::new(CityEventType::Plague));
        assert_eq!(log.entries.len(), 2);
        // Newest is first
        assert_eq!(log.entries[0].event_type, CityEventType::Plague);
        assert_eq!(log.entries[1].event_type, CityEventType::Festival);
    }

    #[test]
    fn test_tick_removes_expired() {
        let mut log = CityEventLog::default();
        log.push(CityEventLogEntry {
            event_type: CityEventType::Festival,
            message: "test".into(),
            time_remaining: 5.0,
        });
        log.push(CityEventLogEntry {
            event_type: CityEventType::Plague,
            message: "test".into(),
            time_remaining: 1.0,
        });

        // Tick 0.5 seconds → both still alive
        log.tick(0.5);
        assert_eq!(log.entries.len(), 2);

        // Tick another 0.6 → plague (0.5+0.6=1.1) expires
        let removed = log.tick(0.6);
        assert_eq!(removed, 1);
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].event_type, CityEventType::Festival);
    }

    #[test]
    fn test_log_capped_at_max() {
        let mut log = CityEventLog::default();
        // Push more than the cap
        for _ in 0..20 {
            log.push(CityEventLogEntry::new(CityEventType::Festival));
        }
        // Max visible + buffer = 8 + 4 = 12
        assert!(log.entries.len() <= 12);
    }

    #[test]
    fn test_entry_description_positive() {
        let entry = CityEventLogEntry::new(CityEventType::Festival);
        assert!(entry.event_type.is_positive());
        assert!(!entry.message.is_empty());
        assert!(entry.time_remaining > 0.0);
    }

    #[test]
    fn test_entry_description_negative() {
        let entry = CityEventLogEntry::new(CityEventType::Plague);
        assert!(!entry.event_type.is_positive());
        assert!(!entry.message.is_empty());
    }

    // ── Logic system tests ────────────────────────────────────────────

    #[test]
    fn test_logic_system_adds_event_entry() {
        let mut app = test_app();

        // Fire a CityEventOccurred event
        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Festival,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 2); // 1 frame to dispatch event + 1 for logic system to read

        let log = app.world().resource::<CityEventLog>();
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].event_type, CityEventType::Festival);
    }

    #[test]
    fn test_logic_system_adds_resolved_entry() {
        let mut app = test_app();

        // Fire a CityEventOccurred first
        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Festival,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 2);

        // Now fire CityEventEnded
        app.world_mut()
            .resource_mut::<Events<CityEventEnded>>()
            .send(CityEventEnded {
                event_type: CityEventType::Festival,
            });

        run_frames(&mut app, 2);

        let log = app.world().resource::<CityEventLog>();
        assert_eq!(log.entries.len(), 2);
        // Newest entry is the "resolved" one
        assert!(log.entries[0].message.contains("resolved"));
    }

    // ── Render system tests ───────────────────────────────────────────

    #[test]
    fn test_render_system_spawns_root_entity() {
        let mut app = test_app();

        run_frames(&mut app, 2);

        // After 2 frames, the render system should have spawned the root
        let count = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Render system should spawn CityEventLogRoot entity");
    }

    #[test]
    fn test_render_system_updates_text() {
        let mut app = test_app();

        // Fire a festival event
        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Festival,
                magnitude: 0.5,
                duration: 30.0,
            });

        // Run enough frames for event to be processed and UI updated
        run_frames(&mut app, 10);

        // Find the root entity (single(), not exactly_one, to avoid itertools dep)
        let root: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(root.len(), 1, "Exactly one CityEventLogRoot entity");
        let root = root[0];

        // Should have Text component with non-empty content
        let text = app.world().get::<Text>(root).expect("Root should have Text");
        assert!(!text.0.is_empty(), "Text should contain event description");
        // Should mention "festival" (case-insensitive check)
        assert!(
            text.0.to_lowercase().contains("festival"),
            "Text should mention festival, got: {}",
            text.0
        );
    }

    #[test]
    fn test_render_system_colours_positive_green() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Festival,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1, "Exactly one CityEventLogRoot entity");
        let root = roots[0];

        let color = app.world().get::<TextColor>(root).expect("Should have TextColor");
        let rgba = color.0.to_srgba();
        // Positive events should have a green-ish tint (G component > R & B)
        assert!(
            rgba.green > rgba.red && rgba.green > rgba.blue,
            "Positive events should have green tint, got: {:?}",
            color.0
        );
    }

    #[test]
    fn test_render_system_colours_negative_red() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Plague,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1, "Exactly one CityEventLogRoot entity");
        let root = roots[0];

        let color = app.world().get::<TextColor>(root).expect("Should have TextColor");
        let rgba = color.0.to_srgba();
        // Negative events should have a red-ish tint (R component > G)
        assert!(
            rgba.red > rgba.green,
            "Negative events should have red tint, got: {:?}",
            color.0
        );
    }

    #[test]
    fn test_empty_log_shows_no_text() {
        let mut app = test_app();

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1, "Exactly one CityEventLogRoot entity");
        let root = roots[0];

        let text = app.world().get::<Text>(root).expect("Root should have Text");
        assert!(text.0.is_empty(), "Empty log should have empty text");
    }

    #[test]
    fn test_multiple_events_in_log() {
        let mut app = test_app();

        // Fire two events
        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Festival,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 3);

        app.world_mut()
            .resource_mut::<Events<CityEventOccurred>>()
            .send(CityEventOccurred {
                event_type: CityEventType::Plague,
                magnitude: 0.5,
                duration: 30.0,
            });

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<CityEventLogRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1, "Exactly one CityEventLogRoot entity");
        let root = roots[0];

        let text = app.world().get::<Text>(root).expect("Root should have Text");
        // Text should contain both events (they appear multi-line)
        assert!(
            text.0.to_lowercase().contains("festival"),
            "Text should mention festival, got: {}",
            text.0
        );
        assert!(
            text.0.to_lowercase().contains("plague") || text.0.to_lowercase().contains("disease"),
            "Text should mention plague/disease, got: {}",
            text.0
        );
    }
}
