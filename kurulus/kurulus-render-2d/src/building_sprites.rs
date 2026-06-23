//! Building sprite rendering — replaces gizmo-based overlays with real sprites.
//!
//! ## Architecture
//!
//! Two systems in the [`Update`] schedule:
//!
//! 1. **`ensure_building_sprites`** (runs first) — adds a [`Sprite`] +
//!    [`Transform`] to any [`Building`] entity that lacks them.  Uses
//!    [`WorldPosition`] to set the initial transform.
//!
//! 2. **`update_building_sprite_colors`** (runs second) — sets each sprite's
//!    colour based on the active [`OverlayMode`], replicating the logic that
//!    was previously handled by gizmo drawing systems in `debug.rs`.
//!
//! When no overlay is active, buildings are drawn in a muted colour that hints
//! at their zone category, providing a clean default look.

use bevy::prelude::*;
use kurulus_core::components::building::Building;
use kurulus_core::components::city::{ServiceBuilding, Zone, ZoneType};
use kurulus_core::coord::*;
use kurulus_core::resources::city::{overlay_colors, CityState, OverlayMode, OverlaySettings};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Sprite size (world units). Matches the old gizmo rect: 0.8 × 0.8.
const SPRITE_SIZE: Vec2 = Vec2::splat(0.8);

// ─── Colour helpers ──────────────────────────────────────────────────────────

/// Desaturated zone hint for the "no overlay" state.
fn neutral_tint(zone_type: ZoneType) -> Color {
    match zone_type {
        ZoneType::Residential => Color::srgb(0.4, 0.7, 0.4),
        ZoneType::Military => Color::srgb(0.7, 0.4, 0.4),
        ZoneType::Economic => Color::srgb(0.7, 0.7, 0.4),
        ZoneType::Religious => Color::srgb(0.4, 0.5, 0.7),
        ZoneType::Administrative => Color::srgb(0.6, 0.4, 0.7),
        ZoneType::Protection => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Full-saturation zone colour (matches `overlay_colors` constants).
fn saturated_tint(zone_type: ZoneType) -> Color {
    match zone_type {
        ZoneType::Residential => overlay_colors::ZONE_RESIDENTIAL,
        ZoneType::Military => overlay_colors::ZONE_MILITARY,
        ZoneType::Economic => overlay_colors::ZONE_ECONOMIC,
        ZoneType::Religious => overlay_colors::ZONE_RELIGIOUS,
        ZoneType::Administrative => overlay_colors::ZONE_ADMIN,
        ZoneType::Protection => overlay_colors::ZONE_PROTECTION,
    }
}

/// Resolve the sprite colour for a single building given the current context.
fn sprite_color_for_building(
    zone: Option<&Zone>,
    overlay: &OverlaySettings,
    city_state: Option<&CityState>,
    covered: bool,
) -> Color {
    match overlay.mode {
        OverlayMode::None => zone
            .map(|z| neutral_tint(z.zone_type))
            .unwrap_or(Color::srgb(0.5, 0.5, 0.5)),
        OverlayMode::Zones => zone
            .map(|z| saturated_tint(z.zone_type))
            .unwrap_or(overlay_colors::ZONE_UNASSIGNED),
        OverlayMode::ServiceCoverage => {
            if covered {
                overlay_colors::COVERAGE_FULL
            } else {
                overlay_colors::COVERAGE_NONE
            }
        }
        OverlayMode::Happiness => match city_state {
            Some(cs) if cs.happiness >= 60.0 => overlay_colors::HAPPY,
            Some(cs) if cs.happiness >= 40.0 => overlay_colors::NEUTRAL,
            Some(cs) if cs.happiness >= 20.0 => overlay_colors::UNHAPPY,
            _ => overlay_colors::MISERABLE,
        },
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Add a [`Sprite`] + [`Transform`] to every [`Building`] that doesn't have one.
///
/// Runs every frame but only touches entities that are missing sprites (newly
/// spawned buildings or entities that had their sprite removed).
pub fn ensure_building_sprites(
    mut commands: Commands,
    buildings: Query<(Entity, &WorldPosition), (With<Building>, Without<Sprite>)>,
) {
    for (entity, pos) in buildings.iter() {
        commands.entity(entity).insert((
            Sprite::from_color(Color::srgb(0.5, 0.5, 0.5), SPRITE_SIZE),
            Transform::from_translation(Vec3::new(pos.world_2d.0.x, pos.world_2d.0.y, 0.0)),
        ));
    }
}

/// Update every building sprite's colour to match the current overlay mode.
pub fn update_building_sprite_colors(
    overlay: Res<OverlaySettings>,
    city_state: Option<Res<CityState>>,
    // Read-only query for service-position detection (no &mut Sprite).
    position_query: Query<(&WorldPosition, Option<&ServiceBuilding>), With<Building>>,
    // Mutable query for the actual sprite update.
    mut sprite_query: Query<
        (&Building, &WorldPosition, &mut Sprite, Option<&Zone>),
        With<Building>,
    >,
) {
    // ── Pre-compute service coverage ────────────────────────────────────
    let service_positions: Vec<(Vec2, f32)> = if overlay.mode == OverlayMode::ServiceCoverage {
        let active = overlay.active_service;
        position_query
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
            .collect()
    } else {
        Vec::new()
    };

    // ── Update each building sprite ─────────────────────────────────────
    for (_building, pos, mut sprite, zone) in sprite_query.iter_mut() {
        let covered = service_positions
            .iter()
            .any(|(svc_pos, radius)| pos.world_2d.0.distance(*svc_pos) <= *radius);

        sprite.color = sprite_color_for_building(zone, &overlay, city_state.as_deref(), covered);
    }
}

// ─── Plugin ──────────────────────────────────────────────────────────────────

/// Plugin that adds building sprite systems to the [`Update`] schedule.
pub struct BuildingSpritePlugin;

impl Plugin for BuildingSpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                ensure_building_sprites,
                update_building_sprite_colors.after(ensure_building_sprites),
            ),
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use kurulus_core::components::building::Building;
    use kurulus_core::components::city::ServiceBuilding;
    use kurulus_core::components::city::ServiceType;
    use kurulus_core::components::city::Zone;
    use kurulus_core::components::city::ZoneType;
    use kurulus_core::components::unit::FactionId;
    use kurulus_core::coord::WorldPosition;
    use kurulus_core::resources::city::CityState;
    use kurulus_core::resources::city::OverlayMode;
    use kurulus_core::resources::city::OverlaySettings;
    use std::time::Duration;

    // ─── Test helpers ───────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<OverlaySettings>()
            .init_resource::<CityState>()
            .add_systems(
                Update,
                (ensure_building_sprites, update_building_sprite_colors),
            );
        app
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    /// Spawn a test building at the given grid position.
    fn spawn_building(app: &mut App, x: i32, y: i32, zone_type: Option<ZoneType>) {
        let mut entity = app.world_mut().spawn((
            Building {
                building_type: kurulus_core::components::building::BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "test".into(),
            },
            WorldPosition::new(IVec2::new(x, y), 1.0),
        ));
        if let Some(zt) = zone_type {
            entity.insert(Zone {
                zone_type: zt,
                level: 1,
            });
        }
    }

    // ─── Neutral tint tests ─────────────────────────────────────────────

    #[test]
    fn test_neutral_tints_return_srgb() {
        for zt in &[
            ZoneType::Residential,
            ZoneType::Military,
            ZoneType::Economic,
            ZoneType::Religious,
            ZoneType::Administrative,
        ] {
            let c = neutral_tint(*zt);
            let srgb = c.to_srgba();
            assert!(srgb.red >= 0.0 && srgb.green >= 0.0 && srgb.blue >= 0.0);
        }
    }

    // ─── ensure_building_sprites tests ──────────────────────────────────

    #[test]
    fn test_sprites_added_to_buildings() {
        let mut app = test_app();
        spawn_building(&mut app, 5, 5, None);
        run_frames(&mut app, 2);

        // Building entity should now have Sprite + Transform
        let mut q = app.world_mut().query::<(&Building, &Sprite, &Transform)>();
        let results: Vec<_> = q.iter(app.world()).collect();
        assert_eq!(results.len(), 1, "Building should have Sprite + Transform");
        let (_, sprite, xform) = results[0];
        assert_eq!(sprite.custom_size, Some(SPRITE_SIZE));
        assert!((xform.translation.x - 5.0).abs() < 0.001);
        assert!((xform.translation.y - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_sprites_not_duplicated_on_subsequent_frames() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, None);
        run_frames(&mut app, 5);

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<Building>, With<Sprite>)>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "Should have exactly one sprite entity");
    }

    // ─── No overlay = neutral tint ──────────────────────────────────────

    #[test]
    fn test_no_overlay_uses_neutral_colors() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, Some(ZoneType::Residential));
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = neutral_tint(ZoneType::Residential);
        let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
        assert!(diff < 0.01, "Expected neutral tint, got {:?}", sprite.color);
    }

    #[test]
    fn test_no_overlay_no_zone_uses_grey() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, None);
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let grey = Color::srgb(0.5, 0.5, 0.5);
        let diff = (sprite.color.to_srgba().red - grey.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - grey.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - grey.to_srgba().blue).abs();
        assert!(diff < 0.01, "Expected grey, got {:?}", sprite.color);
    }

    // ─── Zone overlay uses saturated colors ─────────────────────────────

    #[test]
    fn test_zone_overlay_uses_saturated_colors() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, Some(ZoneType::Residential));
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = saturated_tint(ZoneType::Residential);
        let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
        assert!(
            diff < 0.01,
            "Expected saturated tint, got {:?}",
            sprite.color
        );
    }

    #[test]
    fn test_zone_overlay_no_zone_uses_unassigned() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, None);
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = overlay_colors::ZONE_UNASSIGNED;
        let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
        assert!(
            diff < 0.01,
            "Expected unassigned grey, got {:?}",
            sprite.color
        );
    }

    // ─── Happiness overlay ──────────────────────────────────────────────

    #[test]
    fn test_happiness_overlay_colors() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, None);

        // Set high happiness
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Happiness;
        app.world_mut().resource_mut::<CityState>().happiness = 80.0;
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = overlay_colors::HAPPY;
        let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
        assert!(diff < 0.01, "Expected happy, got {:?}", sprite.color);
    }

    #[test]
    fn test_happiness_overlay_low_shows_miserable() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, None);

        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Happiness;
        app.world_mut().resource_mut::<CityState>().happiness = 5.0;
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        let expected = overlay_colors::MISERABLE;
        let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
            + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
            + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
        assert!(diff < 0.01, "Expected miserable, got {:?}", sprite.color);
    }

    // ─── Service coverage overlay ───────────────────────────────────────

    #[test]
    fn test_service_coverage_near_service_is_green() {
        let mut app = test_app();

        // A service building at (0, 0) with radius 5.0
        app.world_mut().spawn((
            Building {
                building_type: kurulus_core::components::building::BuildingType::Well,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 1,
                name: "well".into(),
            },
            WorldPosition::new(IVec2::new(0, 0), 1.0),
            ServiceBuilding {
                service_type: ServiceType::Water,
                coverage_radius: 5.0,
                effectiveness: 1.0,
            },
        ));

        // A consumer building at (3, 0) — within range
        spawn_building(&mut app, 3, 0, None);

        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::ServiceCoverage;
        app.world_mut()
            .resource_mut::<OverlaySettings>()
            .active_service = ServiceType::Water;
        run_frames(&mut app, 3);

        // Find the consumer building's sprite
        let mut q = app
            .world_mut()
            .query::<(&Building, &Sprite, &WorldPosition)>();
        for (b, sprite, _pos) in q.iter(app.world()) {
            if b.building_type != kurulus_core::components::building::BuildingType::Well {
                let expected = overlay_colors::COVERAGE_FULL;
                let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
                    + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
                    + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
                assert!(
                    diff < 0.01,
                    "Expected green (covered), got {:?}",
                    sprite.color
                );
            }
        }
    }

    #[test]
    fn test_service_coverage_out_of_range_is_red() {
        let mut app = test_app();

        // A service building at (0, 0) with radius 2.0
        app.world_mut().spawn((
            Building {
                building_type: kurulus_core::components::building::BuildingType::Well,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 1,
                name: "well".into(),
            },
            WorldPosition::new(IVec2::new(0, 0), 1.0),
            ServiceBuilding {
                service_type: ServiceType::Water,
                coverage_radius: 2.0,
                effectiveness: 1.0,
            },
        ));

        // A consumer building at (10, 0) — far out of range
        spawn_building(&mut app, 10, 0, None);

        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::ServiceCoverage;
        app.world_mut()
            .resource_mut::<OverlaySettings>()
            .active_service = ServiceType::Water;
        run_frames(&mut app, 3);

        let mut q = app
            .world_mut()
            .query::<(&Building, &Sprite, &WorldPosition)>();
        for (b, sprite, _pos) in q.iter(app.world()) {
            if b.building_type != kurulus_core::components::building::BuildingType::Well {
                let expected = overlay_colors::COVERAGE_NONE;
                let diff = (sprite.color.to_srgba().red - expected.to_srgba().red).abs()
                    + (sprite.color.to_srgba().green - expected.to_srgba().green).abs()
                    + (sprite.color.to_srgba().blue - expected.to_srgba().blue).abs();
                assert!(
                    diff < 0.01,
                    "Expected red (uncovered), got {:?}",
                    sprite.color
                );
            }
        }
    }

    // ─── Switched overlay modes update colours ──────────────────────────

    #[test]
    fn test_switching_mode_updates_colors() {
        let mut app = test_app();
        spawn_building(&mut app, 0, 0, Some(ZoneType::Military));
        run_frames(&mut app, 3);

        // Initially neutral (mode = None)
        let color_before = {
            let (_, sprite, _) = app
                .world_mut()
                .query::<(&Building, &Sprite, &Transform)>()
                .iter(app.world())
                .next()
                .unwrap();
            sprite.color
        };
        assert_eq!(color_before, neutral_tint(ZoneType::Military));

        // Switch to Zones
        app.world_mut().resource_mut::<OverlaySettings>().mode = OverlayMode::Zones;
        run_frames(&mut app, 3);

        let (_, sprite, _) = app
            .world_mut()
            .query::<(&Building, &Sprite, &Transform)>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(sprite.color, saturated_tint(ZoneType::Military));
    }
}
