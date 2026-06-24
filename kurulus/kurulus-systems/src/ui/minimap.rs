//! Minimap HUD — displays a bird's-eye view of the terrain around the
//! player, with unit positions overlaid as dots.
//!
//! ## Architecture
//!
//! 1. **`minimap_logic_system`** (FixedUpdate) — reads `TerrainGrid` and queries
//!    unit `WorldPosition`s, writes to `MinimapState`.
//!
//! 2. **`minimap_render_system`** (Update) — spawns / updates a single entity
//!    tagged with [`MinimapRoot`].
//!
//! In headless mode the `MinimapRoot` entity is inert (no `UiPlugin`).

use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Width of the minimap in characters (tiles).
const MAP_WIDTH: usize = 24;

/// Height of the minimap in characters (tiles).
const MAP_HEIGHT: usize = 16;

/// Font size for minimap text (smallest legible for a character grid).
const FONT_SIZE: f32 = 10.0;

/// Padding inside the panel.
const PANEL_PADDING: UiRect = UiRect::px(6.0, 6.0, 4.0, 4.0);

/// Background colour.
const BG_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.70);

/// Default text colour.
const TEXT_COLOR: Color = Color::srgb(0.80, 0.80, 0.85);

/// Terrain → character mapping for the minimap.
const TERRAIN_CHARS: [(TerrainType, char); 6] = [
    (TerrainType::OpenField, '.'),
    (TerrainType::Forest, 'f'),
    (TerrainType::Water, '~'),
    (TerrainType::Mountain, '^'),
    (TerrainType::Road, '='),
    (TerrainType::Building, '#'),
];

/// Default terrain character when no tile is discovered.
const UNKNOWN_CHAR: char = ' ';

// ─── Minimap State ─────────────────────────────────────────────────────────

/// Snapshot of the minimap grid for rendering.
/// Updated by the logic system; consumed by the render system.
#[derive(Resource, Clone, Debug)]
pub struct MinimapState {
    /// 2D grid row-major: `rows[y][x]` where (0,0) is top-left.
    pub rows: Vec<Vec<char>>,
    /// Width in tiles.
    pub width: usize,
    /// Height in tiles.
    pub height: usize,
    /// World-space centre of the minimap view.
    pub centre: IVec2,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            rows: vec![vec![UNKNOWN_CHAR; MAP_WIDTH]; MAP_HEIGHT],
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            centre: IVec2::ZERO,
        }
    }
}

// ─── Terrain → char helpers ─────────────────────────────────────────────────

fn terrain_char(terrain: TerrainType) -> char {
    TERRAIN_CHARS
        .iter()
        .find(|(t, _)| *t == terrain)
        .map(|(_, c)| *c)
        .unwrap_or('.')
}

/// Character for a friendly unit.
const FRIENDLY_CHAR: char = '@';

/// Character for an enemy unit.
const ENEMY_CHAR: char = 'o';

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Logic system: reads terrain + unit positions, populates `MinimapState`.
///
/// Samples the terrain grid around the game camera / player position and
/// overlays unit markers.  The terrain is rendered first, then units are
/// stamped on top so they are always visible.
pub fn minimap_logic_system(
    mut minimap: ResMut<MinimapState>,
    terrain: Res<TerrainGrid>,
    units: Query<(&WorldPosition, &FactionId)>,
) {
    let centre = minimap.centre;
    let half_w = (MAP_WIDTH / 2) as i32;
    let half_h = (MAP_HEIGHT / 2) as i32;

    let mut rows = vec![vec![UNKNOWN_CHAR; MAP_WIDTH]; MAP_HEIGHT];

    // ── Render terrain tiles ───────────────────────────────────────────
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let world_x = centre.x + x as i32 - half_w;
            let world_y = centre.y + y as i32 - half_h;
            let tile_terrain = terrain.get(IVec2::new(world_x, world_y));
            rows[y][x] = terrain_char(tile_terrain);
        }
    }

    // ── Overlay unit positions ─────────────────────────────────────────
    for (pos, faction) in units.iter() {
        let dx = pos.grid.0.x - centre.x;
        let dy = pos.grid.0.y - centre.y;
        let mx = dx + half_w;
        let my = dy + half_h;

        if mx >= 0 && mx < MAP_WIDTH as i32 && my >= 0 && my < MAP_HEIGHT as i32 {
            let ch = if *faction == FactionId::Kayi {
                FRIENDLY_CHAR
            } else {
                ENEMY_CHAR
            };
            rows[my as usize][mx as usize] = ch;
        }
    }

    minimap.rows = rows;
    minimap.width = MAP_WIDTH;
    minimap.height = MAP_HEIGHT;
}

/// Render system: spawns / updates the minimap UI entity.
pub fn minimap_render_system(
    mut commands: Commands,
    minimap: Res<MinimapState>,
    root_query: Query<Entity, With<MinimapRoot>>,
) {
    let entity = if let Ok(entity) = root_query.get_single() {
        entity
    } else {
        let _ = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(8.0),
                right: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Stretch,
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
            MinimapRoot,
            Name::new("Minimap"),
        ));
        return;
    };

    // Build a monospace grid string
    let body = minimap
        .rows
        .iter()
        .map(|row| row.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");

    commands.entity(entity).insert(Text::new(&body));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::collections::HashMap;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<MinimapState>()
            .init_resource::<TerrainGrid>()
            .add_systems(FixedUpdate, minimap_logic_system)
            .add_systems(Update, minimap_render_system);
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    fn set_tile(world: &mut World, x: i32, y: i32, terrain: TerrainType) {
        world
            .resource_mut::<TerrainGrid>()
            .set(IVec2::new(x, y), terrain);
    }

    fn spawn_unit(world: &mut World, x: i32, y: i32, faction: FactionId) -> Entity {
        world
            .spawn((
                WorldPosition::new(IVec2::new(x, y), 1.0),
                faction,
                Name::new("Unit"),
            ))
            .id()
    }

    // ── Helper tests ───────────────────────────────────────────────────

    #[test]
    fn test_terrain_char_all() {
        let mut seen = HashMap::new();
        for (t, c) in &TERRAIN_CHARS {
            assert_eq!(terrain_char(*t), *c, "Terrain {:?} maps to {}", t, c);
            seen.insert(*c, true);
        }
        assert_eq!(seen.len(), TERRAIN_CHARS.len(), "All terrain chars unique");
    }

    #[test]
    fn test_default_state_size() {
        let state = MinimapState::default();
        assert_eq!(state.width, MAP_WIDTH);
        assert_eq!(state.height, MAP_HEIGHT);
        assert_eq!(state.rows.len(), MAP_HEIGHT);
        assert_eq!(state.rows[0].len(), MAP_WIDTH);
    }

    // ── Logic system tests ─────────────────────────────────────────────

    #[test]
    fn test_logic_renders_terrain() {
        let mut app = test_app();

        // Set some terrain tiles
        set_tile(app.world_mut(), 0, 0, TerrainType::Forest);
        set_tile(app.world_mut(), 1, 0, TerrainType::Water);

        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);

        let state = app.world().resource::<MinimapState>();
        // Center of the map should contain Forest 'f' at (0,0) offset
        let cx = MAP_WIDTH / 2;
        let cy = MAP_HEIGHT / 2;
        assert_eq!(state.rows[cy][cx], 'f', "Centre tile should be forest");
        assert_eq!(state.rows[cy][cx + 1], '~', "Next tile should be water");
    }

    #[test]
    fn test_logic_shows_unknown_for_unset_tiles() {
        let mut app = test_app();
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::new(100, 100);

        run_frames(&mut app, 10);

        let state = app.world().resource::<MinimapState>();
        // Far-away coordinate should default to OpenField (not unknown)
        // since `terrain.get()` returns OpenField for missing tiles
        let cx = MAP_WIDTH / 2;
        let cy = MAP_HEIGHT / 2;
        assert_eq!(
            state.rows[cy][cx], '.',
            "Unset tile should show open field '.'"
        );
    }

    #[test]
    fn test_logic_renders_friendly_unit() {
        let mut app = test_app();

        spawn_unit(app.world_mut(), 0, 0, FactionId::Kayi);
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);

        let state = app.world().resource::<MinimapState>();
        let cx = MAP_WIDTH / 2;
        let cy = MAP_HEIGHT / 2;
        assert_eq!(
            state.rows[cy][cx], '@',
            "Friendly unit at (0,0) should show '@'"
        );
    }

    #[test]
    fn test_logic_renders_enemy_unit() {
        let mut app = test_app();

        spawn_unit(app.world_mut(), 2, 1, FactionId::Byzantine);
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);

        let state = app.world().resource::<MinimapState>();
        let cx = MAP_WIDTH / 2;
        let cy = MAP_HEIGHT / 2;
        assert_eq!(
            state.rows[cy + 1][cx + 2],
            'o',
            "Enemy unit at (2,1) should show 'o'"
        );
    }

    #[test]
    fn test_logic_out_of_bounds_unit_ignored() {
        let mut app = test_app();

        // Unit very far away — should not cause a panic
        spawn_unit(app.world_mut(), 1000, 1000, FactionId::Kayi);
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);
        // Should not panic
    }

    #[test]
    fn test_logic_unit_overrides_terrain() {
        let mut app = test_app();

        set_tile(app.world_mut(), 0, 0, TerrainType::Forest);
        spawn_unit(app.world_mut(), 0, 0, FactionId::Kayi);
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);

        let state = app.world().resource::<MinimapState>();
        let cx = MAP_WIDTH / 2;
        let cy = MAP_HEIGHT / 2;
        assert_eq!(state.rows[cy][cx], '@', "Unit should override terrain char");
    }

    // ── Render system tests ────────────────────────────────────────────

    #[test]
    fn test_render_spawns_entity() {
        let mut app = test_app();
        run_frames(&mut app, 2);

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<MinimapRoot>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should spawn MinimapRoot entity");
    }

    #[test]
    fn test_render_produces_grid_text() {
        let mut app = test_app();

        set_tile(app.world_mut(), 0, 0, TerrainType::Forest);
        set_tile(app.world_mut(), 1, 0, TerrainType::Water);
        spawn_unit(app.world_mut(), 0, 0, FactionId::Kayi);
        app.world_mut().resource_mut::<MinimapState>().centre = IVec2::ZERO;

        run_frames(&mut app, 10);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<MinimapRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let text = app.world().get::<Text>(roots[0]).unwrap();

        // Should have MAP_HEIGHT lines
        let lines: Vec<&str> = text.0.lines().collect();
        assert_eq!(lines.len(), MAP_HEIGHT, "Should have {} rows", MAP_HEIGHT);
        // Each line should be MAP_WIDTH characters
        for line in &lines {
            assert_eq!(
                line.len(),
                MAP_WIDTH,
                "Each row should be {} chars",
                MAP_WIDTH
            );
        }

        // Center should show '@' (friendly unit overrides forest)
        let cx = MAP_WIDTH / 2;
        assert_eq!(lines[MAP_HEIGHT / 2].chars().nth(cx).unwrap(), '@');
    }

    #[test]
    fn test_render_position() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<MinimapRoot>>()
            .iter(app.world())
            .collect();
        assert_eq!(roots.len(), 1);
        let node = app.world().get::<Node>(roots[0]).unwrap();
        assert_eq!(
            node.bottom,
            Val::Px(8.0),
            "Minimap should be 8px from bottom"
        );
        assert_eq!(node.right, Val::Px(8.0), "Minimap should be 8px from right");
    }

    #[test]
    fn test_render_uses_monospace_format() {
        let mut app = test_app();
        run_frames(&mut app, 4);

        let roots: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<MinimapRoot>>()
            .iter(app.world())
            .collect();
        let text = app.world().get::<Text>(roots[0]).unwrap();
        let lines: Vec<&str> = text.0.lines().collect();
        assert_eq!(lines.len(), MAP_HEIGHT);
        // All lines should have equal length
        let first_len = lines[0].len();
        for line in &lines {
            assert_eq!(line.len(), first_len, "All rows must have equal length");
        }
    }
}
