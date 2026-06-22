//! City Event Log HUD — displays recent city events as a scrolling text feed.
//!
//! ## Architecture
//!
//! Two systems run in this module:
//!
//! 1. **`city_event_log_logic_system`** (FixedUpdate) — reads `CityEventOccurred` /
//!    `CityEventEnded` events and drives the `CityEventLog` resource (adding/expiring
//!    entries). Runs unconditionally (works in headless tests).
//!
//! 2. **`city_event_log_render_system`** (Update) — spawns & manages a `bevy_ui`
//!    node tree representing the event log. In headless test mode (no `UiPlugin`)
//!    the entities are spawned but never rendered — harmless.
//!
//! The root entity is tagged with [`CityEventLogRoot`] so we can find it each
//! frame. The UI tree is **rebuilt every frame** for simplicity — at ≤ 12 entries
//! the cost is negligible.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Spacing (px) between consecutive log entries.
const ENTRY_GAP: f32 = 4.0;

/// Font size for log entry text.
const FONT_SIZE: f32 = 14.0;

/// Padding around the log container (top, bottom, left, right).
const CONTAINER_PADDING: UiRect = UiRect::px(8.0, 8.0, 6.0, 6.0);

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads city events, maintains the [`CityEventLog`] resource.
///
/// Runs in `FixedUpdate` after city events have been generated (in
/// `SimulationSet::CityProgression`) so that this system sees the events
/// from the current tick.
pub fn city_event_log_logic_system(
    mut event_log: ResMut<CityEventLog>,
    mut event_occured: EventReader<CityEventOccurred>,
    mut event_ended: EventReader<CityEventEnded>,
    game_time: Res<GameTime>,
) {
    // Compute per-fixed-tick delta the same way city_event_processing_system does.
    let delta = game_time.time_scale as f32 * 0.1;

    // ── Process new events ────────────────────────────────────────────
    for ev in event_occured.read() {
        let mut entry = CityEventLogEntry::new(ev.event_type);
        // Blend magnitude into a shorter message so the player sees the severity
        if ev.magnitude > 0.7 {
            entry.message = format!("‼️ {} (severe)", ev.event_type.description());
        } else if ev.magnitude < 0.3 {
            entry.message = format!("ℹ️ {} (minor)", ev.event_type.description());
        } else {
            entry.message = format!("📢 {}", ev.event_type.description());
        }
        event_log.push(entry);
    }

    // ── Process ended events → push a "resolved" entry ────────────────
    for ev in event_ended.read() {
        let entry = CityEventLogEntry {
            event_type: ev.event_type,
            message: format!("✅ {} resolved", ev.event_type.description()),
            time_remaining: entry_lifetime_default() * 0.6, // shorter display
        };
        event_log.push(entry);
    }

    // ── Tick entry lifetimes ──────────────────────────────────────────
    event_log.tick(delta);
}

/// Render system: spawns / updates the bevy_ui node tree for the event log.
///
/// Runs in `Update` so that UI changes are smooth even when the simulation
/// is running at a different rate.  Rebuilds child text nodes every frame.
pub fn city_event_log_render_system(
    mut commands: Commands,
    event_log: Res<CityEventLog>,
    root_query: Query<Entity, With<CityEventLogRoot>>,
    child_query: Query<&Children>,
) {
    // ── Find or spawn root container ──────────────────────────────────
    let root_entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(12.0),
                    top: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::End,
                    padding: CONTAINER_PADDING,
                    row_gap: Val::Px(ENTRY_GAP),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                CityEventLogRoot,
                Name::new("CityEventLogRoot"),
            ))
            .id();

        // On the very first frame after spawning we have no entries, so
        // return early — next frame will add the children.
        return;
    };

    // ── Despawn existing child text nodes ──────────────────────────────
    // We rebuild children every frame for simplicity.
    if let Ok(children) = child_query.get(root_entity) {
        for &child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    // ── Spawn text nodes for current entries ──────────────────────────
    for entry in event_log.entries.iter() {
        let text_color = if entry.event_type.is_positive() {
            Color::srgb(0.3, 0.9, 0.3) // green
        } else {
            Color::srgb(0.9, 0.3, 0.3) // red
        };

        commands.entity(root_entity).with_children(|parent| {
            parent.spawn((
                Text::new(&entry.message),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    max_width: Val::Px(320.0),
                    ..default()
                },
            ));
        });
    }
}
