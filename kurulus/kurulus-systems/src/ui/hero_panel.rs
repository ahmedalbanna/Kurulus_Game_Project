//! Hero Panel HUD — displays hero portraits with HP and Stamina bars.
//!
//! ## Architecture
//!
//! 1. **`hero_panel_logic_system`** (FixedUpdate) — reads `HeroBarState` and
//!    selection state, updates the hero panel state.
//!
//! 2. **`hero_panel_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`HeroPanelRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for hero panel text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 8.0, 8.0);

/// Background colour (dark brown for a rustic feel).
const BG_COLOR: Color = Color::srgba(0.15, 0.12, 0.10, 0.88);

/// Text colour for hero names.
const TEXT_COLOR: Color = Color::srgb(0.95, 0.95, 0.92);

/// Health bar filled block colour (red tint).
const HP_FILL: char = '█';

/// Health bar empty block colour.
const HP_EMPTY: char = '░';

/// Stamina bar filled block.
const STA_FILL: char = '█';

/// Stamina bar empty block.
const STA_EMPTY: char = '░';

/// Number of segments in HP/Stamina bars.
const BAR_WIDTH: usize = 6;

/// Number of hero portrait slots to display.
const MAX_HERO_SLOTS: usize = 3;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a visual bar string (e.g. "██████░░").
fn bar_string(current: f32, max: f32, fill: char, empty: char, width: usize) -> String {
    if max <= 0.0 {
        return empty.to_string().repeat(width);
    }
    let ratio = (current / max).clamp(0.0, 1.0);
    let filled = (ratio * width as f32).ceil() as usize;
    let filled = filled.min(width);
    let empties = width.saturating_sub(filled);
    format!(
        "{}{}",
        fill.to_string().repeat(filled),
        empty.to_string().repeat(empties)
    )
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads `HeroBarState` and updates panel-specific data.
pub fn hero_panel_logic_system(
    mut bar_state: ResMut<HeroBarState>,
    heroes: Query<(Entity, &Hero, &Health, &Stats)>,
    active_hero: Res<ActiveHero>,
) {
    let mut slots: Vec<HeroSlot> = Vec::with_capacity(MAX_HERO_SLOTS);

    for (entity, hero, health, stats) in heroes.iter().take(MAX_HERO_SLOTS) {
        let is_active = active_hero.entity == Some(entity);
        let hp_pct = if health.max > 0.0 {
            health.current / health.max
        } else {
            1.0
        };
        let mana_pct = if hero.resource_max > 0.0 {
            hero.resource_current / hero.resource_max
        } else {
            1.0
        };
        slots.push(HeroSlot {
            entity: Some(entity),
            name: hero_id_name(hero.hero_id).to_string(),
            health_pct: hp_pct,
            mana_pct,
            available: is_active || stats.morale > 0.0,
            hotkey: String::new(),
        });
    }

    bar_state.heroes = slots;
}

/// Render system: spawns / updates the hero panel UI entity.
pub fn hero_panel_render_system(
    mut commands: Commands,
    bar_state: Res<HeroBarState>,
    root_query: Query<Entity, With<HeroPanelRoot>>,
    layout: Res<MainScreenLayout>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(
                    layout.padding
                        + layout.unit_stats_height
                        + layout.action_bar_height
                        + layout.padding * 2.0,
                ),
                left: Val::Px(layout.padding),
                width: Val::Px(layout.hero_panel_width),
                height: Val::Px(layout.hero_panel_height),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                padding: PANEL_PADDING,
                ..default()
            },
            BackgroundColor(BG_COLOR),
            Text::new(""),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(TEXT_COLOR),
            HeroPanelRoot,
            Name::new("HeroPanel"),
        ));
        return;
    };

    let body = build_hero_panel_text(&bar_state);
    commands.entity(entity).insert(Text::new(&body));
}

fn build_hero_panel_text(state: &HeroBarState) -> String {
    if state.heroes.is_empty() {
        return String::from("No heroes available");
    }

    let mut lines: Vec<String> = Vec::new();

    // First line: hero names
    let mut name_line = String::new();
    for (i, slot) in state.heroes.iter().enumerate() {
        if i > 0 {
            name_line.push_str("  ");
        }
        let star = if slot.available { "★" } else { "☆" };
        name_line.push_str(&format!("{} {}", star, slot.name));
    }
    lines.push(name_line);

    // Second line: HP bars
    let mut hp_line = String::from("HP");
    for slot in &state.heroes {
        let hp_bar = bar_string(slot.health_pct * 100.0, 100.0, HP_FILL, HP_EMPTY, BAR_WIDTH);
        hp_line.push_str(&format!("  {}", hp_bar));
    }
    lines.push(hp_line);

    // Third line: Stamina bars
    let mut sta_line = String::from("ST");
    for slot in &state.heroes {
        let sta_bar = bar_string(slot.mana_pct * 100.0, 100.0, STA_FILL, STA_EMPTY, BAR_WIDTH);
        sta_line.push_str(&format!("  {}", sta_bar));
    }
    lines.push(sta_line);

    lines.join("\n")
}

fn hero_id_name(id: HeroId) -> &'static str {
    match id {
        HeroId::Osman => "Osman",
        HeroId::Turgut => "Turgut",
        HeroId::Buran => "Buran",
        HeroId::Orhan => "Orhan",
        HeroId::Alaeddin => "Alaeddin",
        HeroId::Malhun => "Malhun",
        HeroId::Bala => "Bala",
        HeroId::Bamsi => "Bamsi",
        HeroId::Abdurrahman => "Abdurrahman",
        HeroId::Gunduz => "Gündüz",
        HeroId::Savci => "Savcı",
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

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<HeroBarState>()
            .init_resource::<MainScreenLayout>()
            .init_resource::<ActiveHero>()
            .add_systems(FixedUpdate, hero_panel_logic_system)
            .add_systems(Update, hero_panel_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    fn spawn_hero(
        world: &mut World,
        hero_id: HeroId,
        hp_current: f32,
        hp_max: f32,
        morale: f32,
    ) -> Entity {
        let hero = Hero::new(hero_id);
        world
            .spawn((
                hero,
                Health {
                    current: hp_current,
                    max: hp_max,
                    ..Default::default()
                },
                Stats {
                    morale,
                    morale_max: 100.0,
                    ..Default::default()
                },
                Name::new("Hero"),
            ))
            .id()
    }

    #[test]
    fn test_bar_string_full() {
        assert_eq!(bar_string(100.0, 100.0, '█', '░', 6), "██████");
    }

    #[test]
    fn test_bar_string_empty() {
        assert_eq!(bar_string(0.0, 100.0, '█', '░', 6), "░░░░░░");
    }

    #[test]
    fn test_bar_string_half() {
        assert_eq!(bar_string(50.0, 100.0, '█', '░', 6), "███░░░");
    }

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<HeroPanelRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn HeroPanelRoot entity");
    }

    #[test]
    fn test_render_shows_heroes() {
        let mut app = test_app();

        spawn_hero(app.world_mut(), HeroId::Osman, 800.0, 1000.0, 80.0);
        spawn_hero(app.world_mut(), HeroId::Turgut, 600.0, 1200.0, 90.0);
        spawn_hero(app.world_mut(), HeroId::Buran, 500.0, 800.0, 70.0);

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<HeroPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(text.0.contains("Osman"), "Should show Osman: {}", text.0);
        assert!(text.0.contains("Turgut"), "Should show Turgut: {}", text.0);
        assert!(text.0.contains("Buran"), "Should show Buran: {}", text.0);
        assert!(text.0.contains("HP"), "Should show HP bar: {}", text.0);
        assert!(text.0.contains("ST"), "Should show ST bar: {}", text.0);
    }

    #[test]
    fn test_logic_populates_hero_state() {
        let mut app = test_app();

        spawn_hero(app.world_mut(), HeroId::Osman, 500.0, 1000.0, 80.0);
        spawn_hero(app.world_mut(), HeroId::Turgut, 800.0, 1000.0, 90.0);

        run_frames(&mut app, 10);

        let state = app.world().resource::<HeroBarState>();
        assert_eq!(state.heroes.len(), 2, "Should have 2 hero slots");
        assert_eq!(state.heroes[0].name, "Osman");
        assert_eq!(state.heroes[1].name, "Turgut");
    }

    #[test]
    fn test_empty_state_shows_message() {
        let state = HeroBarState::default();
        let text = build_hero_panel_text(&state);
        assert_eq!(text, "No heroes available");
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<HeroPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.left, Val::Px(8.0), "Panel should be 8px from left");
        assert_eq!(node.width, Val::Px(300.0), "Panel should be 300px wide");
        assert_eq!(node.height, Val::Px(130.0), "Panel should be 130px high");
    }
}
