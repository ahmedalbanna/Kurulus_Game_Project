//! Hero Bar HUD — displays hero portraits with health / resource bars.
//!
//! ## Architecture
//!
//! 1. **`hero_bar_logic_system`** (FixedUpdate) — reads `ActiveHero` and queries
//!    hero entities for `Hero`, `Health`, `Stats`. Populates `HeroBarState`.
//!
//! 2. **`hero_bar_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`HeroBarRoot`].

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for hero bar text.
const FONT_SIZE: f32 = 13.0;

/// Padding inside the bar.
const PANEL_PADDING: UiRect = UiRect::px(8.0, 8.0, 4.0, 4.0);

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);

/// Text colour for hero names.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

/// Number of hero slots to display (Season 1 core 3 + 1 reserve).
const MAX_HERO_SLOTS: usize = 4;

/// Health bar segments.
const BAR_WIDTH: usize = 6;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a visual bar string (e.g. "██████░░").
fn bar_string(current: f32, max: f32, width: usize) -> String {
    if max <= 0.0 {
        return "░".repeat(width);
    }
    let ratio = (current / max).clamp(0.0, 1.0);
    let filled = (ratio * width as f32).ceil() as usize;
    let filled = filled.min(width);
    let empties = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empties))
}

fn hero_hotkey(index: usize) -> &'static str {
    match index {
        0 => "1",
        1 => "2",
        2 => "3",
        3 => "4",
        _ => "",
    }
}

fn hero_name(hero_id: HeroId) -> &'static str {
    match hero_id {
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

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: discovers heroes and populates `HeroBarState`.
pub fn hero_bar_logic_system(
    heroes: Query<(Entity, &Hero, &Health, &Stats)>,
    active_hero: Res<ActiveHero>,
    mut bar_state: ResMut<HeroBarState>,
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
        let index = slots.len();
        slots.push(HeroSlot {
            entity: Some(entity),
            name: hero_name(hero.hero_id).to_string(),
            health_pct: hp_pct,
            mana_pct,
            available: is_active || stats.morale > 0.0,
            hotkey: hero_hotkey(index).to_string(),
        });
    }

    bar_state.heroes = slots;
}

/// Render system: spawns / updates the hero bar UI entity.
pub fn hero_bar_render_system(
    mut commands: Commands,
    bar_state: Res<HeroBarState>,
    root_query: Query<Entity, With<HeroBarRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(72.0),
                left: Val::Px(240.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
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
            HeroBarRoot,
            Name::new("HeroBar"),
        ));
        return;
    };

    let body = build_hero_bar_text(&bar_state);
    commands.entity(entity).insert(Text::new(&body));
}

fn build_hero_bar_text(state: &HeroBarState) -> String {
    if state.heroes.is_empty() {
        return String::from("No heroes available");
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for slot in &state.heroes {
        if !current_line.is_empty() {
            current_line.push_str("  ");
        }

        let hp_bar = bar_string(slot.health_pct * 100.0, 100.0, BAR_WIDTH);
        let mana_bar = bar_string(slot.mana_pct * 100.0, 100.0, BAR_WIDTH);

        let star = if slot.available { "★" } else { "☆" };
        let name_display = &slot.name;

        current_line.push_str(&format!(
            "[{}] {} {}  HP {}  RS {}",
            slot.hotkey, star, name_display, hp_bar, mana_bar,
        ));

        // Start new line every 2 heroes
        if lines.len() % 2 == 1 {
            lines.push(current_line.clone());
            current_line.clear();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines.join("\n")
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
            .init_resource::<ActiveHero>()
            .add_systems(FixedUpdate, hero_bar_logic_system)
            .add_systems(Update, hero_bar_render_system);
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
        assert_eq!(bar_string(100.0, 100.0, 6), "██████");
    }

    #[test]
    fn test_bar_string_empty() {
        assert_eq!(bar_string(0.0, 100.0, 6), "░░░░░░");
    }

    #[test]
    fn test_bar_string_half() {
        assert_eq!(bar_string(50.0, 100.0, 6), "███░░░");
    }

    #[test]
    fn test_hero_name_lookup() {
        assert_eq!(hero_name(HeroId::Osman), "Osman");
        assert_eq!(hero_name(HeroId::Turgut), "Turgut");
        assert_eq!(hero_name(HeroId::Buran), "Buran");
    }

    #[test]
    fn test_hero_hotkey() {
        assert_eq!(hero_hotkey(0), "1");
        assert_eq!(hero_hotkey(1), "2");
        assert_eq!(hero_hotkey(3), "4");
        assert_eq!(hero_hotkey(4), "");
    }

    #[test]
    fn test_no_heroes_shows_empty_message() {
        let state = HeroBarState::default();
        let text = build_hero_bar_text(&state);
        assert_eq!(text, "No heroes available");
    }

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<HeroBarRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn HeroBarRoot entity");
    }

    #[test]
    fn test_render_shows_hero_info() {
        let mut app = test_app();

        spawn_hero(app.world_mut(), HeroId::Osman, 800.0, 1000.0, 80.0);
        spawn_hero(app.world_mut(), HeroId::Turgut, 600.0, 1200.0, 90.0);

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<HeroBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("[1]"),
            "Should show hero 1 hotkey: {}",
            text.0
        );
        assert!(text.0.contains("Osman"), "Should show Osman: {}", text.0);
        assert!(text.0.contains("HP"), "Should show HP bar: {}", text.0);
    }

    #[test]
    fn test_logic_populates_bar_state() {
        let mut app = test_app();

        let e = spawn_hero(app.world_mut(), HeroId::Osman, 800.0, 1000.0, 80.0);
        app.world_mut().resource_mut::<ActiveHero>().entity = Some(e);

        run_frames(&mut app, 10);

        let state = app.world().resource::<HeroBarState>();
        assert_eq!(state.heroes.len(), 1, "Should have 1 hero slot");
        assert_eq!(state.heroes[0].name, "Osman");
        assert!(state.heroes[0].available, "Active hero should be available");
        assert_eq!(state.heroes[0].hotkey, "1");
    }

    #[test]
    fn test_logic_health_fraction() {
        let mut app = test_app();

        spawn_hero(app.world_mut(), HeroId::Osman, 500.0, 1000.0, 80.0);
        run_frames(&mut app, 10);

        let state = app.world().resource::<HeroBarState>();
        assert_eq!(state.heroes.len(), 1);
        assert!(
            (state.heroes[0].health_pct - 0.5).abs() < 0.01,
            "HP fraction should be 0.5"
        );
    }

    #[test]
    fn test_logic_max_heroes_capped() {
        let mut app = test_app();

        // Spawn more than MAX_HERO_SLOTS
        for hero_id in &[
            HeroId::Osman,
            HeroId::Turgut,
            HeroId::Buran,
            HeroId::Orhan,
            HeroId::Alaeddin,
        ] {
            spawn_hero(app.world_mut(), *hero_id, 1000.0, 1000.0, 100.0);
        }

        run_frames(&mut app, 10);

        let state = app.world().resource::<HeroBarState>();
        assert!(
            state.heroes.len() <= MAX_HERO_SLOTS,
            "Should cap at {} heroes",
            MAX_HERO_SLOTS
        );
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<HeroBarRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(node.bottom, Val::Px(72.0), "Bar should be 72px from bottom");
        assert_eq!(node.left, Val::Px(240.0), "Bar should be 240px from left");
    }
}
