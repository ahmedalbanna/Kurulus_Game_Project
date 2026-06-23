use bevy::prelude::*;
use kurulus_core::coord::*;
use kurulus_core::components::unit::*;
use kurulus_core::components::stealth::Detection;
use kurulus_core::components::building::BuildingType;
use kurulus_core::components::city::{Zone, ZoneType};
use kurulus_core::resources::city::{
    OverlaySettings, OverlayMode, ZonePlacementState, BuildingPlacementState, overlay_colors,
};

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        // Note: building overlay colours are now rendered via real sprites
        // (see building_sprites::BuildingSpritePlugin). We keep only the
        // non-building gizmos here (grid, formations, detection, zone
        // boundaries, placement preview, and the no-op legend hook).
        app.add_systems(
            Update,
            (
                draw_grid_gizmos,
                draw_formation_gizmos,
                draw_detection_gizmos,
                draw_overlay_legend_gizmos,
                draw_zone_entity_gizmos,
                draw_placement_preview_gizmos,
                draw_building_placement_preview_gizmos,
            ),
        );
    }
}

// ─── Grid ────────────────────────────────────────────────────────────────────

/// Draws grid lines for visual reference.
fn draw_grid_gizmos(mut gizmos: Gizmos, camera_query: Query<&Transform, With<Camera2d>>) {
    if let Ok(cam) = camera_query.get_single() {
        let view_size = 50.0;
        let cam_x = cam.translation.x;
        let cam_y = cam.translation.y;

        for x in ((cam_x - view_size) as i32..(cam_x + view_size) as i32).step_by(1) {
            gizmos.line_2d(
                Vec2::new(x as f32, cam_y - view_size),
                Vec2::new(x as f32, cam_y + view_size),
                Color::srgba(0.3, 0.3, 0.3, 0.1),
            );
        }
        for y in ((cam_y - view_size) as i32..(cam_y + view_size) as i32).step_by(1) {
            gizmos.line_2d(
                Vec2::new(cam_x - view_size, y as f32),
                Vec2::new(cam_x + view_size, y as f32),
                Color::srgba(0.3, 0.3, 0.3, 0.1),
            );
        }
    }
}

// ─── Formations ──────────────────────────────────────────────────────────────

/// Draws formation leaders and follower positions.
fn draw_formation_gizmos(
    mut gizmos: Gizmos,
    leaders: Query<&WorldPosition, (With<Unit>, Without<FormationFollower>)>,
    followers: Query<(&WorldPosition, &Formation)>,
) {
    for leader_pos in leaders.iter() {
        gizmos.circle_2d(leader_pos.world_2d.0, 0.5, Color::srgb(0.0, 0.0, 1.0));
    }
    for (follower_pos, formation) in followers.iter() {
        if let Ok(leader_pos) = leaders.get(formation.leader) {
            gizmos.line_2d(
                follower_pos.world_2d.0,
                leader_pos.world_2d.0,
                Color::srgb(0.0, 1.0, 0.0),
            );
        }
    }
}

// ─── Stealth Detection ───────────────────────────────────────────────────────

/// Draws detection radius for units with Detection component.
fn draw_detection_gizmos(
    mut gizmos: Gizmos,
    detectors: Query<(&WorldPosition, &Detection)>,
) {
    for (pos, detection) in detectors.iter() {
        if detection.active {
            gizmos.circle_2d(
                pos.world_2d.0,
                detection.radius,
                Color::srgba(1.0, 0.0, 0.0, 0.2),
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// City View Overlays
// ═══════════════════════════════════════════════════════════════════════════════
//
// Each overlay system draws colored rectangles on top of buildings when
// the corresponding OverlayMode is active.  They share the same cell-size
// conventions as the rest of the game (CELL_SIZE = 1.0).

// Note: building overlay colours (Zones / ServiceCoverage / Happiness) are
// now rendered via real sprites in building_sprites::BuildingSpritePlugin.
// The helper functions draw_building_gizmo and draw_building_circle have been
// removed along with the three overlay systems.

// ── 8d. Overlay Legend (replaced by bevy_ui legend in UILogicPlugin) ──

/// Kept as a no-op to preserve the system registration in `DebugRenderPlugin`.
/// The actual text overlay is rendered via `bevy_ui` by
/// [`zone_legend_render_system`] in the `UILogicPlugin` — see
/// `kurulus-systems/src/ui/zone_legend.rs`.
fn draw_overlay_legend_gizmos() {}

// ── Zone Entity Rendering ──────────────────────────────────────────────────

/// Draws coloured rectangles for every placed zone entity.
///
/// Visible when:
///   • The **Zones** overlay mode is active (V key), **or**
///   • The zone **placement mode** is active (B key).
///
/// This gives the player continuous visual feedback about which districts
/// exist on the map, using the same colour scheme as the overlay system.
fn draw_zone_entity_gizmos(
    mut gizmos: Gizmos,
    overlay: Res<OverlaySettings>,
    placement: Res<ZonePlacementState>,
    zones: Query<(&Zone, &WorldPosition)>,
) {
    let visible = overlay.mode == OverlayMode::Zones || placement.active;
    if !visible {
        return;
    }

    for (zone, pos) in zones.iter() {
        let color = zone_color(zone.zone_type);
        let mut c = color;
        c.set_alpha(0.45);
        let half = CELL_SIZE * 0.45;
        gizmos.rect_2d(pos.world_2d.0, Vec2::splat(half * 2.0), c);
    }
}

/// Draws a semi-transparent preview rectangle at the hovered grid position
/// when zone placement mode is active.
///
/// The preview colour matches the selected zone type, helping the player
/// see exactly where a zone will be placed before they click.
fn draw_placement_preview_gizmos(
    mut gizmos: Gizmos,
    placement: Res<ZonePlacementState>,
) {
    if !placement.active {
        return;
    }
    let Some(grid) = placement.hovered_grid else {
        return;
    };

    let world_pos = Vec2::new(grid.x as f32 * CELL_SIZE, grid.y as f32 * CELL_SIZE);
    let color = zone_color(placement.selected_type);
    let mut c = color;
    c.set_alpha(0.60);
    let half = CELL_SIZE * 0.48;
    gizmos.rect_2d(world_pos, Vec2::splat(half * 2.0), c);
}

// ── Building Placement Preview ──────────────────────────────────────────────

/// Draws a semi-transparent preview rectangle at the hovered grid position
/// when building placement mode is active.
///
/// The preview colour matches the building's zone category (e.g. green for
/// Residential buildings, red for Military), helping the player see what
/// zone the building belongs to before they click.
fn draw_building_placement_preview_gizmos(
    mut gizmos: Gizmos,
    placement: Res<BuildingPlacementState>,
) {
    if !placement.active {
        return;
    }
    let Some(grid) = placement.hovered_grid else {
        return;
    };

    let world_pos = Vec2::new(grid.x as f32 * CELL_SIZE, grid.y as f32 * CELL_SIZE);
    // Try to map the building type to its zone-category colour; fall back to
    // a neutral blue for buildings that don't have a zone (infrastructure,
    // defensive).
    let color = zone_color_for_building(placement.selected_type);
    let mut c = color;
    c.set_alpha(0.55);
    let half = CELL_SIZE * 0.48;
    gizmos.rect_2d(world_pos, Vec2::splat(half * 2.0), c);
}

/// Pick a zone-category colour for a building type preview.
fn zone_color_for_building(building_type: BuildingType) -> Color {
    // Walk zone types until we find one that matches this building
    use kurulus_core::components::city::ZoneType;
    for zt in &[
        ZoneType::Residential,
        ZoneType::Military,
        ZoneType::Economic,
        ZoneType::Religious,
        ZoneType::Administrative,
    ] {
        if zt.matches_building(building_type) {
            return zone_color(*zt);
        }
    }
    // Fallback: neutral tan for infrastructure / defensive buildings
    Color::srgb(0.6, 0.5, 0.3)
}

/// Resolve a zone type to its standard colour.
fn zone_color(zone_type: ZoneType) -> Color {
    match zone_type {
        ZoneType::Residential => overlay_colors::ZONE_RESIDENTIAL,
        ZoneType::Military => overlay_colors::ZONE_MILITARY,
        ZoneType::Economic => overlay_colors::ZONE_ECONOMIC,
        ZoneType::Religious => overlay_colors::ZONE_RELIGIOUS,
        ZoneType::Administrative => overlay_colors::ZONE_ADMIN,
        ZoneType::Protection => overlay_colors::ZONE_PROTECTION,
    }
}
