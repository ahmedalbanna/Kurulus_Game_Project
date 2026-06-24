//! Selection Panel HUD — displays information about the currently selected
//! entity / entities: unit archetype, health bar, morale bar, hero details.
//!
//! ## Architecture
//!
//! A single **render system** (`selection_panel_render_system`) runs in the
//! `Update` schedule.  It reads `SelectionState` and queries the ECS world for
//! components on selected entities (`Unit`, `Health`, `Stats`, `Hero`).  It then
//! spawns or updates a single entity tagged with [`SelectionPanelRoot`].
//!
//! No intermediate resource is needed — the render system gathers data directly
//! from the ECS each frame.

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Font size for panel text.
const FONT_SIZE: f32 = 14.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(10.0, 10.0, 8.0, 8.0);

/// Panel width.
const PANEL_WIDTH: f32 = 220.0;

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.65);

/// Default text colour.
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

/// Health bar filled block.
const HP_FILL: char = '█';

/// Health bar empty block.
const HP_EMPTY: char = '░';

/// Morale bar filled block.
const MORALE_FILL: char = '▓';

/// Morale bar empty block.
const MORALE_EMPTY: char = '░';

/// Number of segments in the HP / morale bar visualisation.
const BAR_WIDTH: usize = 10;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Build a visual bar string (e.g. "███████░░░").
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

/// Format a faction display name (from `Debug`).
fn faction_name(faction: &FactionId) -> &'static str {
    match faction {
        FactionId::Kayi => "Kayı",
        FactionId::Dodurga => "Dodurga",
        FactionId::Candar => "Candar",
        FactionId::Germiyan => "Germiyan",
        FactionId::Karaman => "Karaman",
        FactionId::Byzantine => "Byzantine",
        FactionId::Mongol => "Mongol",
        FactionId::Neutral => "Neutral",
    }
}

/// Convert `UnitArchetype` debug name to a user-friendly label.
fn archetype_label(archetype: &UnitArchetype) -> &'static str {
    match archetype {
        UnitArchetype::LightInfantry => "Light Infantry",
        UnitArchetype::HeavyInfantry => "Heavy Infantry",
        UnitArchetype::Archer => "Archer",
        UnitArchetype::LightCavalry => "Light Cavalry",
        UnitArchetype::HeavyCavalry => "Heavy Cavalry",
        UnitArchetype::Scout => "Scout",
        UnitArchetype::BaitRider => "Bait Rider",
        UnitArchetype::SiegeCrew => "Siege Crew",
        UnitArchetype::Worker => "Worker",
    }
}

/// Format the hero's resource type name.
fn hero_resource_label(resource_type: &HeroResource) -> &'static str {
    match resource_type {
        HeroResource::Inspiration => "IN",
        HeroResource::Fury => "FY",
        HeroResource::Focus => "FC",
    }
}

// ─── Multi-line text builders ────────────────────────────────────────────────

/// Build the selection panel text for a *single* selected entity.
fn build_single_selection_text(
    entity: Entity,
    unit: Option<&Unit>,
    health: Option<&Health>,
    stats: Option<&Stats>,
    hero: Option<&Hero>,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    // ── Header: hero star + name / archetype ──────────────────────────
    if let Some(hero) = hero {
        lines.push(format!("★ {}", hero_id_name(hero.hero_id)));
    }

    if let Some(unit) = unit {
        lines.push(format!(
            "{} ({})",
            archetype_label(&unit.archetype),
            faction_name(&unit.faction)
        ));
        lines.push(format!(
            "Stance: {:?}  Form: {:?}",
            unit.stance, unit.formation
        ));
    } else {
        lines.push(format!("Entity #{}", entity.index()));
    }

    // ── Health bar ────────────────────────────────────────────────────
    if let Some(health) = health {
        let bar = bar_string(health.current, health.max, HP_FILL, HP_EMPTY, BAR_WIDTH);
        lines.push(format!(
            "HP  {} {:>4.0}/{:>4.0}",
            bar, health.current, health.max
        ));
        if health.armor > 0.0 {
            lines.push(format!(
                "Armor: {:.0} ({:?})",
                health.armor, health.armor_type
            ));
        }
    }

    // ── Morale bar ────────────────────────────────────────────────────
    if let Some(stats) = stats {
        let bar = bar_string(
            stats.morale,
            stats.morale_max,
            MORALE_FILL,
            MORALE_EMPTY,
            BAR_WIDTH,
        );
        lines.push(format!(
            "Mor {} {:>4.0}/{:>4.0}",
            bar, stats.morale, stats.morale_max
        ));
    }

    // ── Hero-specific info ────────────────────────────────────────────
    if let Some(hero) = hero {
        let res_label = hero_resource_label(&hero.resource_type);
        let res_bar = bar_string(
            hero.resource_current,
            hero.resource_max,
            MORALE_FILL,
            MORALE_EMPTY,
            BAR_WIDTH,
        );
        lines.push(format!(
            "{} {} {:>4.0}/{:>4.0}",
            res_label, res_bar, hero.resource_current, hero.resource_max
        ));
        lines.push(format!("Level {}  XP: {}", hero.level, hero.xp));
        if let Some(spec) = &hero.specialization {
            lines.push(format!("Specialization: {:?}", spec));
        }
    }

    lines.join("\n")
}

/// Build the selection panel text for *multiple* selected entities.
fn build_multi_selection_text(
    entities: &[Entity],
    unit_query: &Query<&Unit>,
    health_query: &Query<&Health>,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("{} Units Selected", entities.len()));
    lines.push(String::new());

    // Count archetypes among selected units
    let mut archetype_counts: Vec<(&'static str, usize)> = Vec::new();
    for e in entities {
        if let Ok(unit) = unit_query.get(*e) {
            let label = archetype_label(&unit.archetype);
            if let Some((_, count)) = archetype_counts.iter_mut().find(|(l, _)| *l == label) {
                *count += 1;
            } else {
                archetype_counts.push((label, 1));
            }
        }
    }

    for (label, count) in &archetype_counts {
        if *count > 1 {
            lines.push(format!("○ {} ×{}", label, count));
        } else {
            lines.push(format!("○ {}", label));
        }
    }

    // Show summary health
    let total_hp: f32 = entities
        .iter()
        .filter_map(|e| health_query.get(*e).ok())
        .map(|h| h.current)
        .sum();
    let total_max_hp: f32 = entities
        .iter()
        .filter_map(|e| health_query.get(*e).ok())
        .map(|h| h.max)
        .sum();

    let pct = if total_max_hp > 0.0 {
        (total_hp / total_max_hp) * 100.0
    } else {
        0.0
    };

    lines.push(String::new());
    lines.push(format!("Avg HP: {:.0}%", pct));

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

// ─── System ──────────────────────────────────────────────────────────────────

/// Render system: spawns / updates the selection panel UI entity.
pub fn selection_panel_render_system(
    mut commands: Commands,
    selection: Res<SelectionState>,
    unit_query: Query<&Unit>,
    health_query: Query<&Health>,
    stats_query: Query<&Stats>,
    hero_query: Query<&Hero>,
    root_query: Query<Entity, With<SelectionPanelRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        // First frame — spawn the root entity
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
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
            TextColor(TEXT_COLOR),
            SelectionPanelRoot,
            Name::new("SelectionPanel"),
        ));
        return;
    };

    // Build text content
    let body = if selection.selected.is_empty() {
        String::from("No Selection")
    } else if selection.selected.len() == 1 {
        let e = selection.selected[0];
        build_single_selection_text(
            e,
            unit_query.get(e).ok(),
            health_query.get(e).ok(),
            stats_query.get(e).ok(),
            hero_query.get(e).ok(),
        )
    } else {
        build_multi_selection_text(&selection.selected, &unit_query, &health_query)
    };

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

    /// Build a minimal app with the selection panel render system installed.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<SelectionState>()
            .add_systems(Update, selection_panel_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    /// Spawn a test unit entity with optional components and return its Entity.
    fn spawn_unit(
        world: &mut World,
        archetype: UnitArchetype,
        hp_current: f32,
        hp_max: f32,
        morale: f32,
    ) -> Entity {
        world
            .spawn((
                Unit {
                    archetype,
                    ..Default::default()
                },
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
                Name::new("TestUnit"),
            ))
            .id()
    }

    /// Spawn a test hero entity.
    fn spawn_hero(
        world: &mut World,
        hero_id: HeroId,
        hp_current: f32,
        hp_max: f32,
        level: u8,
    ) -> Entity {
        let hero = Hero {
            level,
            ..Hero::new(hero_id)
        };
        world
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    ..Default::default()
                },
                Health {
                    current: hp_current,
                    max: hp_max,
                    ..Default::default()
                },
                Stats::default(),
                hero,
                Name::new("Hero"),
            ))
            .id()
    }

    // ── Helper tests ───────────────────────────────────────────────────

    #[test]
    fn test_bar_string_full() {
        let s = bar_string(100.0, 100.0, '█', '░', 10);
        assert_eq!(s, "██████████", "Full bar");
    }

    #[test]
    fn test_bar_string_empty() {
        let s = bar_string(0.0, 100.0, '█', '░', 10);
        assert_eq!(s, "░░░░░░░░░░", "Empty bar");
    }

    #[test]
    fn test_bar_string_half() {
        let s = bar_string(50.0, 100.0, '█', '░', 10);
        assert_eq!(s, "█████░░░░░", "Half bar");
    }

    #[test]
    fn test_bar_string_partial() {
        let s = bar_string(75.0, 100.0, '█', '░', 10);
        // 7.5 -> ceil = 8 filled
        assert_eq!(s, "████████░░", "75% bar");
    }

    #[test]
    fn test_bar_string_zero_max() {
        let s = bar_string(50.0, 0.0, '█', '░', 10);
        assert_eq!(s, "░░░░░░░░░░", "Zero max = empty bar");
    }

    #[test]
    fn test_archetype_label_light_infantry() {
        assert_eq!(
            archetype_label(&UnitArchetype::LightInfantry),
            "Light Infantry"
        );
    }

    #[test]
    fn test_archetype_label_heavy_cavalry() {
        assert_eq!(
            archetype_label(&UnitArchetype::HeavyCavalry),
            "Heavy Cavalry"
        );
    }

    #[test]
    fn test_archetype_label_archer() {
        assert_eq!(archetype_label(&UnitArchetype::Archer), "Archer");
    }

    #[test]
    fn test_hero_resource_label() {
        assert_eq!(hero_resource_label(&HeroResource::Inspiration), "IN");
        assert_eq!(hero_resource_label(&HeroResource::Fury), "FY");
        assert_eq!(hero_resource_label(&HeroResource::Focus), "FC");
    }

    #[test]
    fn test_hero_id_name() {
        assert_eq!(hero_id_name(HeroId::Osman), "Osman");
        assert_eq!(hero_id_name(HeroId::Turgut), "Turgut");
        assert_eq!(hero_id_name(HeroId::Buran), "Buran");
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn SelectionPanelRoot entity");
    }

    #[test]
    fn test_empty_selection_shows_no_selection() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert_eq!(text.0, "No Selection");
    }

    #[test]
    fn test_single_unit_selection_shows_info() {
        let mut app = test_app();

        let unit = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            80.0,
            100.0,
            60.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Light Infantry"),
            "Should show archetype: {}",
            text.0
        );
        assert!(text.0.contains("HP"), "Should show HP: {}", text.0);
        assert!(text.0.contains("Mor"), "Should show morale: {}", text.0);
        assert!(text.0.contains("80"), "Should show HP value: {}", text.0);
        assert!(
            text.0.contains("60"),
            "Should show morale value: {}",
            text.0
        );
    }

    #[test]
    fn test_hero_selection_shows_hero_info() {
        let mut app = test_app();

        let hero = spawn_hero(app.world_mut(), HeroId::Osman, 950.0, 1000.0, 3);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(hero);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(text.0.contains('★'), "Hero should have star: {}", text.0);
        assert!(
            text.0.contains("Osman"),
            "Should show hero name: {}",
            text.0
        );
        assert!(text.0.contains("Level 3"), "Should show level: {}", text.0);
        assert!(
            text.0.contains("IN"),
            "Should show resource type: {}",
            text.0
        );
    }

    #[test]
    fn test_multi_selection_shows_count() {
        let mut app = test_app();

        let u1 = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            100.0,
            100.0,
            50.0,
        );
        let u2 = spawn_unit(app.world_mut(), UnitArchetype::Archer, 80.0, 80.0, 70.0);
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .extend([u1, u2]);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("2 Units Selected"),
            "Should show count: {}",
            text.0
        );
        assert!(
            text.0.contains("Light Infantry"),
            "Should list archetype: {}",
            text.0
        );
        assert!(
            text.0.contains("Archer"),
            "Should list archetype: {}",
            text.0
        );
        assert!(text.0.contains("Avg HP"), "Should show avg HP: {}", text.0);
    }

    #[test]
    fn test_multi_selection_same_archetype_count() {
        let mut app = test_app();

        let u1 = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            100.0,
            100.0,
            50.0,
        );
        let u2 = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            80.0,
            100.0,
            60.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .extend([u1, u2]);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        assert!(
            text.0.contains("Light Infantry ×2"),
            "Should show count: {}",
            text.0
        );
    }

    #[test]
    fn test_selection_clears_when_empty() {
        let mut app = test_app();

        // First select a unit
        let unit = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            80.0,
            100.0,
            60.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);
        run_frames(&mut app, 4);

        // Then clear selection
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .clear();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        assert_eq!(text.0, "No Selection");
    }

    #[test]
    fn test_stale_entity_does_not_panic() {
        let mut app = test_app();

        let unit = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            80.0,
            100.0,
            60.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        // Despawn the entity
        app.world_mut().despawn(unit);

        // Render system should not panic when querying a stale entity
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();
        // Stale entity should fall back to entity index display
        assert!(
            !text.0.is_empty(),
            "Should show fallback text for stale entity"
        );
    }

    #[test]
    fn test_selection_panel_position_is_bottom_left() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(
            node.bottom,
            Val::Px(40.0),
            "Panel should be 40px from bottom"
        );
        assert_eq!(node.left, Val::Px(8.0), "Panel should be 8px from left");
    }

    #[test]
    fn test_single_unit_health_color_bar_length() {
        let mut app = test_app();

        let unit = spawn_unit(
            app.world_mut(),
            UnitArchetype::LightInfantry,
            50.0,
            100.0,
            50.0,
        );
        app.world_mut()
            .resource_mut::<SelectionState>()
            .selected
            .push(unit);

        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<SelectionPanelRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();

        // The bar should be 10 segments wide
        let hp_line = text.0.lines().find(|l| l.starts_with("HP")).unwrap();
        let bar_chars: Vec<char> = hp_line.chars().skip(4).take(BAR_WIDTH).collect();
        assert_eq!(
            bar_chars.len(),
            BAR_WIDTH,
            "HP bar should be {} chars wide, got: {:?}",
            BAR_WIDTH,
            bar_chars.iter().collect::<String>()
        );
    }
}
