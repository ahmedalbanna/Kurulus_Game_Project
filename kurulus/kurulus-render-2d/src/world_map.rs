use bevy::math::IVec2;
use bevy::prelude::*;
use kurulus_core::prelude::*;

/// Size of each world map tile in world units.
/// The grid is 48×36 tiles, so the full map is 96×72 world units.
const TILE_SIZE: f32 = WORLD_MAP_CELL_SIZE;
const HALF_TILE: f32 = TILE_SIZE * 0.5;

pub struct WorldMapRenderPlugin;

impl Plugin for WorldMapRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_world_map_terrain_gizmos,
                draw_world_map_fog_gizmos,
                draw_world_map_region_borders_gizmos,
                draw_world_map_poi_markers_gizmos,
                draw_world_map_scout_markers_gizmos,
                draw_world_map_player_marker_gizmos,
            )
                .chain(),
        );
    }
}

// ── Terrain colours ─────────────────────────────────────────────────────────

/// Map each `WorldTerrain` variant to a distinct colour.
fn terrain_color(terrain: WorldTerrain) -> Color {
    match terrain {
        WorldTerrain::Grassland => Color::srgb(0.45, 0.70, 0.30),
        WorldTerrain::Forest => Color::srgb(0.25, 0.50, 0.15),
        WorldTerrain::Mountain => Color::srgb(0.55, 0.45, 0.30),
        WorldTerrain::Water => Color::srgb(0.20, 0.45, 0.70),
        WorldTerrain::Desert => Color::srgb(0.85, 0.80, 0.55),
        WorldTerrain::Snow => Color::srgb(0.95, 0.95, 0.98),
        WorldTerrain::Road => Color::srgb(0.60, 0.50, 0.35),
        WorldTerrain::Settlement => Color::srgb(0.85, 0.70, 0.45),
        WorldTerrain::Fortress => Color::srgb(0.70, 0.55, 0.40),
    }
}

/// Darker variants for the edge of each tile (subtle 3D effect).
fn terrain_color_dark(terrain: WorldTerrain) -> Color {
    match terrain {
        WorldTerrain::Grassland => Color::srgb(0.35, 0.55, 0.25),
        WorldTerrain::Forest => Color::srgb(0.18, 0.38, 0.10),
        WorldTerrain::Mountain => Color::srgb(0.42, 0.35, 0.22),
        WorldTerrain::Water => Color::srgb(0.14, 0.32, 0.55),
        WorldTerrain::Desert => Color::srgb(0.70, 0.65, 0.40),
        WorldTerrain::Snow => Color::srgb(0.82, 0.82, 0.88),
        WorldTerrain::Road => Color::srgb(0.48, 0.40, 0.28),
        WorldTerrain::Settlement => Color::srgb(0.70, 0.55, 0.32),
        WorldTerrain::Fortress => Color::srgb(0.58, 0.45, 0.30),
    }
}

/// Convert a world map tile position (IVec2) to a world-space Vec2 centre.
fn tile_to_world(pos: IVec2) -> Vec2 {
    Vec2::new(
        pos.x as f32 * TILE_SIZE + HALF_TILE,
        pos.y as f32 * TILE_SIZE + HALF_TILE,
    )
}

// ── 1. Terrain Gizmos ──────────────────────────────────────────────────────

/// Draws a coloured rectangle for every tile on the world map.
/// The colour is determined by `WorldTerrain` — green for grass, brown for
/// mountains, blue for water, etc.
fn draw_world_map_terrain_gizmos(mut gizmos: Gizmos, world_map: Res<WorldMap>) {
    for tile in world_map.tiles.iter() {
        let centre = tile_to_world(tile.position);
        let color = terrain_color(tile.terrain);
        let dark = terrain_color_dark(tile.terrain);

        // Main fill
        gizmos.rect_2d(centre, Vec2::splat(TILE_SIZE * 0.95), color);

        // Edge highlight (1px bottom-right edge)
        gizmos.rect_2d(
            centre + Vec2::new(TILE_SIZE * 0.02, -TILE_SIZE * 0.02),
            Vec2::splat(TILE_SIZE * 0.92),
            dark,
        );
    }
}

// ── 2. Fog of War ──────────────────────────────────────────────────────────

/// Fog overlay colours.
const FOG_UNEXPLORED: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
const FOG_EXPLORED: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);

/// Draws a semi-transparent overlay on top of terrain tiles based on fog state.
///
/// * **Unexplored** — very dark, nearly opaque (0.85 alpha)
/// * **Explored** — dimmed, semi-transparent (0.35 alpha)
/// * **Visible** — no overlay (terrain is fully visible)
fn draw_world_map_fog_gizmos(mut gizmos: Gizmos, world_map: Res<WorldMap>) {
    for tile in world_map.tiles.iter() {
        let overlay = match tile.fog {
            FogState::Visible => continue, // no overlay
            FogState::Explored => FOG_EXPLORED,
            FogState::Unexplored => FOG_UNEXPLORED,
        };
        let centre = tile_to_world(tile.position);
        gizmos.rect_2d(centre, Vec2::splat(TILE_SIZE * 0.95), overlay);
    }
}

// ── 3. Region Borders ──────────────────────────────────────────────────────

/// Region border colour per faction (matching the existing convention).
fn owner_color(owner: FactionId) -> Color {
    match owner {
        FactionId::Kayi => Color::srgb(0.85, 0.20, 0.10), // Kayı red
        FactionId::Byzantine => Color::srgb(0.55, 0.35, 0.85), // Byzantine purple
        FactionId::Mongol => Color::srgb(0.85, 0.60, 0.10), // Mongol gold
        FactionId::Neutral => Color::srgb(0.60, 0.60, 0.60), // Grey
        FactionId::Dodurga => Color::srgb(0.10, 0.70, 0.40), // Dodurga green
        FactionId::Candar => Color::srgb(0.85, 0.50, 0.10), // Candar orange
        FactionId::Germiyan => Color::srgb(0.30, 0.30, 0.80), // Germiyan blue
        FactionId::Karaman => Color::srgb(0.70, 0.20, 0.60), // Karaman purple
    }
}

/// Draws an outline around each region's bounding box, tinted by the owner
/// faction's colour.  This gives a quick visual of territory control.
fn draw_world_map_region_borders_gizmos(mut gizmos: Gizmos, world_map: Res<WorldMap>) {
    for region in world_map.regions.iter() {
        if region.tiles.is_empty() {
            continue;
        }

        // Compute bounding box of the region's tiles
        let min_x = region.tiles.iter().map(|p| p.x).min().unwrap_or(0);
        let max_x = region.tiles.iter().map(|p| p.x).max().unwrap_or(0);
        let min_y = region.tiles.iter().map(|p| p.y).min().unwrap_or(0);
        let max_y = region.tiles.iter().map(|p| p.y).max().unwrap_or(0);

        // Convert to world coordinates (corners of the bbox)
        let world_min = Vec2::new(min_x as f32 * TILE_SIZE, min_y as f32 * TILE_SIZE);
        let world_max = Vec2::new(
            (max_x + 1) as f32 * TILE_SIZE,
            (max_y + 1) as f32 * TILE_SIZE,
        );
        let size = world_max - world_min;
        let centre = world_min + size * 0.5;

        let color = owner_color(region.owner);
        gizmos.rect_2d(centre, size, color);
    }
}

// ── 4. Points of Interest ──────────────────────────────────────────────────

/// Colour for discovered vs undiscovered POIs.
const POI_DISCOVERED: Color = Color::srgb(1.0, 0.85, 0.20); // gold
const POI_UNDISCOVERED: Color = Color::srgb(0.50, 0.50, 0.50); // grey

/// Draws a small circle marker at every POI position.
/// Discovered POIs are gold, undiscovered are grey.
/// The circle is slightly larger for discovered POIs.
fn draw_world_map_poi_markers_gizmos(mut gizmos: Gizmos, world_map: Res<WorldMap>) {
    for poi in world_map.points_of_interest.iter() {
        let centre = tile_to_world(poi.position);
        let (color, radius) = if poi.discovered {
            (POI_DISCOVERED, HALF_TILE * 0.45)
        } else {
            (POI_UNDISCOVERED, HALF_TILE * 0.30)
        };
        gizmos.circle_2d(centre, radius, color);
        // Inner dot for discovered POIs
        if poi.discovered {
            gizmos.circle_2d(centre, radius * 0.4, Color::srgb(1.0, 1.0, 0.8));
        }
    }
}

// ── 5. Scout Party Markers ─────────────────────────────────────────────────

/// Colour for friendly scout markers.
const SCOUT_COLOR: Color = Color::srgb(0.20, 0.85, 0.40); // bright green
const SCOUT_ENCOUNTER_COLOR: Color = Color::srgb(1.0, 0.60, 0.0); // orange when encountered

/// Draws a small circle marker at every scout party's current position.
/// Friendly scouts are green; scouts in an encounter state are orange.
fn draw_world_map_scout_markers_gizmos(mut gizmos: Gizmos, scouts: Query<&ScoutParty>) {
    for scout in scouts.iter() {
        let centre = tile_to_world(scout.position);
        let color = match scout.status {
            ScoutStatus::Encountered => SCOUT_ENCOUNTER_COLOR,
            _ => SCOUT_COLOR,
        };
        // Outer ring
        gizmos.circle_2d(centre, HALF_TILE * 0.55, color);
        // Inner fill (semi-transparent version of the same color)
        let fill = match color {
            Color::Srgba(c) => Color::Srgba(bevy::color::Srgba { alpha: 0.4, ..c }),
            _ => color,
        };
        gizmos.circle_2d(centre, HALF_TILE * 0.30, fill);
    }
}

// ── 6. Player Position Marker ──────────────────────────────────────────────

/// Draws a bright diamond/rhombus shape at the player's current position on
/// the world map, making the player's hero party easy to spot.
fn draw_world_map_player_marker_gizmos(mut gizmos: Gizmos, world_map: Res<WorldMap>) {
    let centre = tile_to_world(world_map.player_position);

    // Diamond shape via 4-point polygon
    let half = HALF_TILE * 0.7;
    let points = vec![
        centre + Vec2::new(0.0, half),  // top
        centre + Vec2::new(half, 0.0),  // right
        centre + Vec2::new(0.0, -half), // bottom
        centre + Vec2::new(-half, 0.0), // left
    ];
    gizmos.linestrip_2d(points, Color::srgb(1.0, 1.0, 0.4));

    // Fill
    let fill_points = vec![
        centre + Vec2::new(0.0, half * 0.95),
        centre + Vec2::new(half * 0.95, 0.0),
        centre + Vec2::new(0.0, -half * 0.95),
        centre + Vec2::new(-half * 0.95, 0.0),
    ];
    for i in 0..4 {
        let a = fill_points[i];
        let b = fill_points[(i + 1) % 4];
        let c = centre;
        gizmos.line_2d(a, b, Color::srgb(1.0, 1.0, 0.4));
        gizmos.line_2d(a, c, Color::srgba(1.0, 1.0, 0.4, 0.3));
        gizmos.line_2d(b, c, Color::srgba(1.0, 1.0, 0.4, 0.3));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_color_returns_valid() {
        // All terrain types should map to a valid colour (no panics)
        for terrain in &[
            WorldTerrain::Grassland,
            WorldTerrain::Forest,
            WorldTerrain::Mountain,
            WorldTerrain::Water,
            WorldTerrain::Desert,
            WorldTerrain::Snow,
            WorldTerrain::Road,
            WorldTerrain::Settlement,
            WorldTerrain::Fortress,
        ] {
            let _ = terrain_color(*terrain);
            let _ = terrain_color_dark(*terrain);
        }
    }

    #[test]
    fn test_owner_color_returns_valid() {
        for faction in &[
            FactionId::Kayi,
            FactionId::Dodurga,
            FactionId::Candar,
            FactionId::Germiyan,
            FactionId::Karaman,
            FactionId::Byzantine,
            FactionId::Mongol,
            FactionId::Neutral,
        ] {
            let _ = owner_color(*faction);
        }
    }

    #[test]
    fn test_tile_to_world_origin() {
        let world = tile_to_world(IVec2::new(0, 0));
        assert!((world.x - HALF_TILE).abs() < 0.001);
        assert!((world.y - HALF_TILE).abs() < 0.001);
    }

    #[test]
    fn test_tile_to_world_non_zero() {
        let world = tile_to_world(IVec2::new(10, 5));
        assert!((world.x - (10.0 * TILE_SIZE + HALF_TILE)).abs() < 0.001);
        assert!((world.y - (5.0 * TILE_SIZE + HALF_TILE)).abs() < 0.001);
    }

    #[test]
    fn test_scout_marker_colors() {
        // Scout marker colours should not panic
        let _ = SCOUT_COLOR;
        let _ = SCOUT_ENCOUNTER_COLOR;
    }
}
