use bevy::prelude::*;
use kurulus_core::coord::*;
use kurulus_core::components::unit::*;
use kurulus_core::components::stealth::Detection;
use kurulus_core::components::building::Building;
use kurulus_core::components::city::{Zone, ZoneType, ServiceBuilding};
use kurulus_core::resources::city::{
    OverlaySettings, OverlayMode, CityState, ZonePlacementState, overlay_colors,
};

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                draw_grid_gizmos,
                draw_formation_gizmos,
                draw_detection_gizmos,
                draw_overlay_zone_gizmos,
                draw_overlay_service_gizmos,
                draw_overlay_happiness_gizmos,
                draw_overlay_legend_gizmos,
                draw_zone_entity_gizmos,
                draw_placement_preview_gizmos,
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

/// Helper: draw a filled semi-transparent rectangle at a world position.
fn draw_building_gizmo(gizmos: &mut Gizmos, pos: Vec2, color: Color) {
    let half = CELL_SIZE * 0.4;
    gizmos.rect_2d(pos, Vec2::splat(half * 2.0), color);
}

/// Helper: draw a circle around a building (for service coverage).
fn draw_building_circle(gizmos: &mut Gizmos, pos: Vec2, radius: f32, color: Color) {
    // circle_2d takes (position, radius, color)
    gizmos.circle_2d(pos, radius, color);
}

// ── 8a. Zone Overlay ─────────────────────────────────────────────────────────

/// Color-code buildings by their zone district.
/// - Residential → green
/// - Military    → red
/// - Economic    → yellow
/// - Religious   → blue
/// - Admin       → purple
/// - Unzoned     → grey
fn draw_overlay_zone_gizmos(
    mut gizmos: Gizmos,
    settings: Res<OverlaySettings>,
    buildings: Query<(&Building, &WorldPosition, Option<&Zone>)>,
) {
    if settings.mode != OverlayMode::Zones {
        return;
    }

    for (_building, pos, zone) in buildings.iter() {
        let color = match zone.map(|z| z.zone_type) {
            Some(ZoneType::Residential) => overlay_colors::ZONE_RESIDENTIAL,
            Some(ZoneType::Military) => overlay_colors::ZONE_MILITARY,
            Some(ZoneType::Economic) => overlay_colors::ZONE_ECONOMIC,
            Some(ZoneType::Religious) => overlay_colors::ZONE_RELIGIOUS,
            Some(ZoneType::Administrative) => overlay_colors::ZONE_ADMIN,
            None => overlay_colors::ZONE_UNASSIGNED,
        };

        // Make unzoned buildings more transparent so they don't dominate
        let alpha = if zone.is_some() { 0.5 } else { 0.2 };
        let mut c = color;
        c.set_alpha(alpha);
        draw_building_gizmo(&mut gizmos, pos.world_2d.0, c);
    }
}

// ── 8b. Service Coverage Overlay ─────────────────────────────────────────────

/// Show which buildings are within coverage radius of a selected service type.
/// Buildings within range → green, partial → yellow, out of range → red.
/// Also draws a faint circle showing the coverage radius of each service building.
fn draw_overlay_service_gizmos(
    mut gizmos: Gizmos,
    settings: Res<OverlaySettings>,
    all_buildings: Query<(&WorldPosition, Option<&ServiceBuilding>), With<Building>>,
) {
    if settings.mode != OverlayMode::ServiceCoverage {
        return;
    }

    let active = settings.active_service;

    // Collect all service-building positions that provide this service
    let service_positions: Vec<(Vec2, f32)> = all_buildings
        .iter()
        .filter_map(|(pos, svc)| {
            svc.and_then(|s| {
                if s.service_type == active {
                    Some((pos.world_2d.0, s.coverage_radius))
                } else {
                    None
                }
            })
        })
        .collect();

    // Draw coverage-radius circles around each service building
    for (svc_pos, radius) in &service_positions {
        draw_building_circle(&mut gizmos, *svc_pos, *radius, Color::srgba(0.2, 0.8, 0.2, 0.08));
    }

    // Colour each building based on whether it's within range of any service
    for (pos, _optional_svc) in all_buildings.iter() {
        let covered = service_positions
            .iter()
            .any(|(svc_pos, radius)| pos.world_2d.0.distance(*svc_pos) <= *radius);

        let color = if covered {
            overlay_colors::COVERAGE_FULL
        } else {
            overlay_colors::COVERAGE_NONE
        };
        let mut c = color;
        c.set_alpha(0.4);
        draw_building_gizmo(&mut gizmos, pos.world_2d.0, c);
    }
}

// ── 8c. Happiness Overlay ────────────────────────────────────────────────────

/// Color-code buildings by the city's current happiness level.
/// - Happiness ≥ 60 → green (happy)
/// - Happiness ≥ 40 → yellow (neutral)
/// - Happiness ≥ 20 → orange (unhappy)
/// - Happiness < 20 → red (miserable)
fn draw_overlay_happiness_gizmos(
    mut gizmos: Gizmos,
    settings: Res<OverlaySettings>,
    city: Res<CityState>,
    buildings: Query<&WorldPosition, With<Building>>,
) {
    if settings.mode != OverlayMode::Happiness {
        return;
    }

    let color = if city.happiness >= 60.0 {
        overlay_colors::HAPPY
    } else if city.happiness >= 40.0 {
        overlay_colors::NEUTRAL
    } else if city.happiness >= 20.0 {
        overlay_colors::UNHAPPY
    } else {
        overlay_colors::MISERABLE
    };

    let mut c = color;
    c.set_alpha(0.35);

    for pos in buildings.iter() {
        draw_building_gizmo(&mut gizmos, pos.world_2d.0, c);
    }
}

// ── 8d. Overlay Legend (placeholder — text requires bevy_ui, not available in gizmos) ──

/// Placeholder legend system.  In the current codebase, overlay text legends
/// will be added when proper `bevy_ui` HUD rendering is implemented.
/// For now the mode name is logged to the console when toggled.
fn draw_overlay_legend_gizmos(
    _settings: Res<OverlaySettings>,
) {
    // Gizmos do not support text rendering directly.
    // Legend will be implemented when the game gains a proper HUD layer.
}

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

/// Resolve a zone type to its standard colour.
fn zone_color(zone_type: ZoneType) -> Color {
    match zone_type {
        ZoneType::Residential => overlay_colors::ZONE_RESIDENTIAL,
        ZoneType::Military => overlay_colors::ZONE_MILITARY,
        ZoneType::Economic => overlay_colors::ZONE_ECONOMIC,
        ZoneType::Religious => overlay_colors::ZONE_RELIGIOUS,
        ZoneType::Administrative => overlay_colors::ZONE_ADMIN,
    }
}
