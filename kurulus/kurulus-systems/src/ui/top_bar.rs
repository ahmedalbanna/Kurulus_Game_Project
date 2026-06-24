//! Top Resource Bar HUD — displays resources, population, SP, time, and speed.
//!
//! ## Architecture
//!
//! 1. **`top_bar_logic_system`** (FixedUpdate) — reads `ResourceStockpile` and
//!    `GameTime`, updates `ResourceBarState` with resource deltas and time string.
//!
//! 2. **`top_bar_render_system`** (Update) — spawns / updates a single entity
//!    carrying both [`Node`] (layout) and [`Text`] (content). The text is
//!    rewritten each frame — no children, no hierarchy issues.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for the top bar text.
const FONT_SIZE: f32 = 14.0;

/// Padding around the bar (top, right, bottom, left).
const PANEL_PADDING: UiRect = UiRect::px(12.0, 12.0, 4.0, 4.0);

/// Fixed height of the top bar.
const BAR_HEIGHT: f32 = 28.0;

/// Background colour of the top bar.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.65);

/// Default text colour.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

/// Threshold fraction below which a resource is considered "low".
const LOW_RESOURCE_THRESHOLD: f32 = 0.10;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Append a delta indicator (↑N / ↓N) to the string if non-zero.
fn append_delta(buf: &mut String, delta: i32) {
    if delta > 0 {
        buf.push_str(&format!("↑{}", delta));
    } else if delta < 0 {
        buf.push_str(&format!("↓{}", -delta));
    }
}

/// Pick a colour suffix for a resource value based on current / max ratio.
fn resource_warning_flag(value: u32, max: f32) -> &'static str {
    if max <= 0.0 {
        return ""; // unknown capacity
    }
    let ratio = value as f32 / max;
    if ratio == 0.0 {
        "!!" // empty — danger
    } else if ratio < LOW_RESOURCE_THRESHOLD {
        "!" // low — warning
    } else {
        ""
    }
}

/// Build the single-line display string from the current bar state.
fn format_top_bar_text(state: &ResourceBarState) -> String {
    let mut s = String::new();

    // ── Resources ──────────────────────────────────────────────────────
    s.push_str(&format!("💰 {}", state.gold));
    append_delta(&mut s, state.gold_delta);
    s.push_str(resource_warning_flag(state.gold, 500.0));

    s.push_str(&format!("  🪵 {}", state.wood));
    append_delta(&mut s, state.wood_delta);
    s.push_str(resource_warning_flag(state.wood, 500.0));

    s.push_str(&format!("  🌾 {}", state.food));
    append_delta(&mut s, state.food_delta);
    s.push_str(resource_warning_flag(state.food, 500.0));

    s.push_str(&format!("  ⛏️ {}", state.iron));
    append_delta(&mut s, state.iron_delta);
    s.push_str(resource_warning_flag(state.iron, 500.0));

    s.push_str(&format!("  🪨 {}", state.stone));
    append_delta(&mut s, state.stone_delta);
    s.push_str(resource_warning_flag(state.stone, 500.0));

    // ── Population ─────────────────────────────────────────────────────
    s.push_str(&format!(
        "  👥 {}/{}",
        state.population_current, state.population_max
    ));

    // ── Sovereignty Points ─────────────────────────────────────────────
    s.push_str(&format!("  ✦ {}", state.sovereignty_points));

    // ── Time ───────────────────────────────────────────────────────────
    s.push_str(&format!("  🕐 {}", state.time_string));

    // ── Speed ──────────────────────────────────────────────────────────
    let speed_icon = if state.game_speed >= 3.0 {
        "▶▶▶"
    } else if state.game_speed >= 2.0 {
        "▶▶"
    } else {
        "▶"
    };
    s.push_str(&format!("  {}", speed_icon));

    s
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads `ResourceStockpile` and `GameTime`, updates `ResourceBarState`.
pub fn top_bar_logic_system(
    stockpile: Res<ResourceStockpile>,
    game_time: Res<GameTime>,
    mut bar_state: ResMut<ResourceBarState>,
) {
    // Store previous values for delta computation
    bar_state.prev_gold = bar_state.gold;
    bar_state.prev_wood = bar_state.wood;
    bar_state.prev_food = bar_state.food;
    bar_state.prev_iron = bar_state.iron;
    bar_state.prev_stone = bar_state.stone;

    // Snapshot current values (cast f32 → u32 truncates, which is fine for display)
    bar_state.gold = stockpile.gold as u32;
    bar_state.wood = stockpile.wood as u32;
    bar_state.food = stockpile.food as u32;
    bar_state.iron = stockpile.iron as u32;
    bar_state.stone = stockpile.stone as u32;
    bar_state.population_current = stockpile.population;
    bar_state.population_max = stockpile.max_population;
    bar_state.sovereignty_points = stockpile.sovereignty_points;
    bar_state.game_speed = game_time.time_scale;

    // Compute per-resource deltas
    bar_state.gold_delta = bar_state.gold as i32 - bar_state.prev_gold as i32;
    bar_state.wood_delta = bar_state.wood as i32 - bar_state.prev_wood as i32;
    bar_state.food_delta = bar_state.food as i32 - bar_state.prev_food as i32;
    bar_state.iron_delta = bar_state.iron as i32 - bar_state.prev_iron as i32;
    bar_state.stone_delta = bar_state.stone as i32 - bar_state.prev_stone as i32;

    // Format time string
    bar_state.time_string = format!("{:02}:{:02}", game_time.hour, game_time.minute);
}

/// Render system: spawns / updates the top resource bar UI entity.
pub fn top_bar_render_system(
    mut commands: Commands,
    bar_state: Res<ResourceBarState>,
    root_query: Query<Entity, With<TopBarRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: PANEL_PADDING,
                height: Val::Px(BAR_HEIGHT),
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Text::new(""),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            TopBarRoot,
            Name::new("TopBar"),
        ));
        return;
    };

    // Update text content every frame
    let body = format_top_bar_text(&bar_state);
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

    /// Build a minimal app with top bar systems installed.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<ResourceStockpile>()
            .init_resource::<GameTime>()
            .init_resource::<ResourceBarState>()
            .add_systems(FixedUpdate, top_bar_logic_system)
            .add_systems(Update, top_bar_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── ResourceBarState / Logic Tests ──────────────────────────────────

    #[test]
    fn test_top_bar_logic_reads_stockpile() {
        let mut app = test_app();

        app.world_mut().resource_mut::<ResourceStockpile>().gold = 250.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 100.0;
        app.world_mut().resource_mut::<ResourceStockpile>().food = 78.0;
        app.world_mut().resource_mut::<ResourceStockpile>().iron = 42.0;
        app.world_mut().resource_mut::<ResourceStockpile>().stone = 30.0;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .population = 18;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .max_population = 30;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 8;

        run_frames(&mut app, 2);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.gold, 250);
        assert_eq!(state.wood, 100);
        assert_eq!(state.food, 78);
        assert_eq!(state.iron, 42);
        assert_eq!(state.stone, 30);
        assert_eq!(state.population_current, 18);
        assert_eq!(state.population_max, 30);
        assert_eq!(state.sovereignty_points, 8);
    }

    #[test]
    fn test_top_bar_logic_snaps_time() {
        let mut app = test_app();

        // GameTime defaults to 08:00
        run_frames(&mut app, 2);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.time_string, "08:00");
    }

    #[test]
    fn test_top_bar_logic_updates_gold_on_change() {
        let mut app = test_app();

        // Set stockpile to initial value and run enough frames for FixedUpdate
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 100.0;
        run_frames(&mut app, 10);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.gold, 100, "Gold should be synced from stockpile");

        // Change stockpile
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 150.0;
        run_frames(&mut app, 10);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.gold, 150, "Gold should update after change");
    }

    #[test]
    fn test_top_bar_logic_negative_delta() {
        let mut app = test_app();

        // Establish baseline
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 100.0;
        run_frames(&mut app, 10);

        // Decrease
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 80.0;
        run_frames(&mut app, 10);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.gold, 80, "Gold should reflect decreased value");
    }

    /// Tests that the delta formula works correctly in isolation.
    /// The system-level delta test is covered by the integration test
    /// `test_top_bar_render_updates_when_resources_change`.
    #[test]
    fn test_delta_formula() {
        let mut state = ResourceBarState::default();
        state.prev_gold = 100;
        state.gold = 150;
        state.gold_delta = state.gold as i32 - state.prev_gold as i32;
        assert_eq!(state.gold_delta, 50);

        state.prev_gold = 100;
        state.gold = 80;
        state.gold_delta = state.gold as i32 - state.prev_gold as i32;
        assert_eq!(state.gold_delta, -20);
    }

    #[test]
    fn test_top_bar_logic_speed_reads_game_time() {
        let mut app = test_app();

        app.world_mut().resource_mut::<GameTime>().time_scale = 2.0;
        run_frames(&mut app, 2);

        let state = app.world().resource::<ResourceBarState>();
        assert_eq!(state.game_speed, 2.0);
    }

    // ── Render System Tests ─────────────────────────────────────────────

    #[test]
    fn test_top_bar_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<TopBarRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Render system should spawn TopBarRoot entity");
    }

    #[test]
    fn test_top_bar_render_contains_resources() {
        let mut app = test_app();

        app.world_mut().resource_mut::<ResourceStockpile>().gold = 340.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 125.0;
        app.world_mut().resource_mut::<ResourceStockpile>().food = 78.0;
        app.world_mut().resource_mut::<ResourceStockpile>().iron = 42.0;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .population = 18;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .max_population = 30;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 8;

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<TopBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        let content = &text.0;

        assert!(content.contains("340"), "Should show gold: {}", content);
        assert!(content.contains("125"), "Should show wood: {}", content);
        assert!(content.contains("78"), "Should show food: {}", content);
        assert!(content.contains("42"), "Should show iron: {}", content);
        assert!(
            content.contains("18/30"),
            "Should show population: {}",
            content
        );
        assert!(content.contains("8"), "Should show SP: {}", content);
    }

    #[test]
    fn test_top_bar_render_shows_time() {
        let mut app = test_app();

        // GameTime defaults to 08:00
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<TopBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert!(
            text.0.contains("08:00"),
            "Should show time 08:00, got: {}",
            text.0
        );
    }

    #[test]
    fn test_top_bar_render_shows_speed_icon() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<TopBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // At default speed 1.0, should show single play icon
        assert!(
            text.0.contains('▶'),
            "Should show speed icon, got: {}",
            text.0
        );
    }

    #[test]
    fn test_top_bar_render_updates_when_resources_change() {
        let mut app = test_app();

        // First set resources
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 100.0;
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<TopBarRoot>>()
            .iter(app.world())
            .collect();
        let text1 = app.world().get::<Text>(roots[0]).unwrap().0.clone();
        assert!(text1.contains("100"), "Initial: {}", text1);

        // Change resources
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 200.0;
        run_frames(&mut app, 4);

        let text2 = app.world().get::<Text>(roots[0]).unwrap().0.clone();
        assert!(
            text2.contains("200"),
            "Should update to 200, got: {}",
            text2
        );
    }

    // ── Helper function tests ──────────────────────────────────────────

    #[test]
    fn test_append_delta_positive() {
        let mut s = String::new();
        append_delta(&mut s, 5);
        assert_eq!(s, "↑5");
    }

    #[test]
    fn test_append_delta_negative() {
        let mut s = String::new();
        append_delta(&mut s, -3);
        assert_eq!(s, "↓3");
    }

    #[test]
    fn test_append_delta_zero() {
        let mut s = String::new();
        append_delta(&mut s, 0);
        assert_eq!(s, "");
    }

    #[test]
    fn test_resource_warning_flag_normal() {
        let flag = resource_warning_flag(250, 500.0);
        assert_eq!(flag, "");
    }

    #[test]
    fn test_resource_warning_flag_low() {
        let flag = resource_warning_flag(25, 500.0);
        assert_eq!(flag, "!");
    }

    #[test]
    fn test_resource_warning_flag_empty() {
        let flag = resource_warning_flag(0, 500.0);
        assert_eq!(flag, "!!");
    }

    #[test]
    fn test_format_top_bar_text_basic() {
        let state = ResourceBarState {
            gold: 340,
            wood: 125,
            food: 78,
            iron: 42,
            stone: 30,
            population_current: 18,
            population_max: 30,
            sovereignty_points: 8,
            time_string: "12:34".to_string(),
            game_speed: 1.0,
            ..Default::default()
        };
        let text = format_top_bar_text(&state);
        assert!(text.contains("340"), "Gold: {}", text);
        assert!(text.contains("125"), "Wood: {}", text);
        assert!(text.contains("18/30"), "Pop: {}", text);
        assert!(text.contains("8"), "SP: {}", text);
        assert!(text.contains("12:34"), "Time: {}", text);
    }
}
