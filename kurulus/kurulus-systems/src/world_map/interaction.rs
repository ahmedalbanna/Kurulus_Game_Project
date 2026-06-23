use bevy::input::ButtonInput;
use bevy::prelude::*;
use kurulus_core::prelude::*;
use kurulus_core::resources::world_map::{WorldMapHover, WorldMapSelection};

/// Centre of the world map in world coordinates.
/// The map is 48×36 tiles, each tile is 2.0 units, so centre = (48, 36).
const MAP_CENTRE: Vec2 = Vec2::new(
    (48.0 * WORLD_MAP_CELL_SIZE) * 0.5,
    (36.0 * WORLD_MAP_CELL_SIZE) * 0.5,
);

/// Camera scale when in world map mode to show the whole map.
const WORLD_MAP_ZOOM: f32 = 18.0;

// ── 1. State Toggle ────────────────────────────────────────────────────────

/// Toggles between `InGame` (city view) and `WorldMap` (strategic overworld)
/// when the player presses the **M** key.
///
/// This lets the player switch between building their city and viewing the
/// strategic campaign map at any time.  Game logic continues running in both
/// modes — only the camera and rendering change.
pub fn world_map_toggle_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Some(input) = keyboard else {
        return;
    };
    if input.just_pressed(KeyCode::KeyM) {
        let new_state = match state.get() {
            AppState::InGame => AppState::WorldMap,
            AppState::WorldMap => AppState::InGame,
            other => *other,
        };
        next_state.set(new_state);
        info!("🗺️  State switched to: {:?}", new_state);
    }
}

// ── 2. Camera Management ───────────────────────────────────────────────────

/// Adjusts the 2D camera position and orthographic scale based on the current
/// `AppState`:
///
/// * **InGame** — camera at (0, 0, 100) with default scale (1.0).
/// * **WorldMap** — camera centred on the world map with a large orthographic
///   scale so the entire 48×36 tile map is visible.
///
/// The pan offset from `WorldMapViewState` is applied when in world map mode,
/// allowing the player to drag the view.
pub fn world_map_camera_system(
    state: Res<State<AppState>>,
    view: Res<WorldMapViewState>,
    mut camera_query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok((mut transform, mut projection)) = camera_query.get_single_mut() else {
        return;
    };

    match state.get() {
        AppState::WorldMap => {
            // Position camera at map centre + pan offset
            transform.translation.x = MAP_CENTRE.x + view.pan_offset.x;
            transform.translation.y = MAP_CENTRE.y + view.pan_offset.y;
            transform.translation.z = 100.0;

            // Zoom out to show the full map
            if let Projection::Orthographic(ref mut ortho) = projection.as_mut() {
                ortho.scale = WORLD_MAP_ZOOM / view.zoom;
            }
        }
        AppState::InGame => {
            // Reset to default city view
            transform.translation = Vec3::new(0.0, 0.0, 100.0);

            if let Projection::Orthographic(ref mut ortho) = projection.as_mut() {
                ortho.scale = 1.0;
            }
        }
        _ => {
            // MainMenu / Paused — no change
        }
    }
}

// ── 3. Pan Interaction ─────────────────────────────────────────────────────

/// Click-and-drag panning for the world map view.
/// Only active when `AppState::WorldMap` and the left mouse button is held.
pub fn world_map_pan_system(
    state: Res<State<AppState>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut view: ResMut<WorldMapViewState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut last_cursor: Local<Option<Vec2>>,
) {
    if *state.get() != AppState::WorldMap {
        *last_cursor = None;
        return;
    }

    let Some(input) = mouse else {
        *last_cursor = None;
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        *last_cursor = None;
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    if input.pressed(MouseButton::Left) {
        if let Some(prev) = *last_cursor {
            // Convert delta from screen-pixel space to world space
            if let Ok(prev_world) = camera.viewport_to_world_2d(camera_transform, prev) {
                if let Ok(cur_world) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                    let delta = prev_world - cur_world;
                    view.pan_offset += delta;
                }
            }
        }
        *last_cursor = Some(cursor_pos);
    } else {
        *last_cursor = None;
    }
}

// ── 4. Zoom (Scroll Wheel) ─────────────────────────────────────────────────

/// Zoom in/out on the world map using the mouse scroll wheel.
/// Zoom is clamped to [0.5, 4.0] to prevent excessive zooming.
///
/// Uses `Option<Res<Events<MouseWheel>>>` so it gracefully does nothing when
/// the input plugin is not available (e.g. headless/test mode with
/// `MinimalPlugins`).
pub fn world_map_zoom_system(
    state: Res<State<AppState>>,
    scroll_events: Option<Res<Events<bevy::input::mouse::MouseWheel>>>,
    mut view: ResMut<WorldMapViewState>,
) {
    if *state.get() != AppState::WorldMap {
        return;
    }

    let Some(events) = scroll_events else {
        return;
    };

    let mut reader = events.get_cursor();
    for ev in reader.read(&*events) {
        // Scroll up = zoom in (increase zoom factor)
        // Scroll down = zoom out (decrease zoom factor)
        view.zoom = (view.zoom + ev.y * 0.25).clamp(0.5, 4.0);
    }
}

// ── 5. Hover Detection ──────────────────────────────────────────────────────

/// Converts the mouse cursor position to a world-map tile coordinate and
/// resolves which region (if any) and POI (if any) the cursor is hovering over.
///
/// Results are stored in `WorldMapHover` for downstream UI systems.
pub fn world_map_hover_system(
    state: Res<State<AppState>>,
    world_map: Res<WorldMap>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut hover: ResMut<WorldMapHover>,
) {
    // Reset hover state
    hover.hovered_tile = None;
    hover.hovered_region = None;
    hover.hovered_poi = None;

    if *state.get() != AppState::WorldMap {
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Convert world position to tile coordinates
    let tile_x = (world_pos.x / WORLD_MAP_CELL_SIZE).floor() as i32;
    let tile_y = (world_pos.y / WORLD_MAP_CELL_SIZE).floor() as i32;

    if tile_x < 0
        || tile_y < 0
        || tile_x >= world_map.width as i32
        || tile_y >= world_map.height as i32
    {
        return;
    }

    let tile_pos = IVec2::new(tile_x, tile_y);
    hover.hovered_tile = Some(tile_pos);

    // ── Resolve region ────────────────────────────────────────────────
    for region in world_map.regions.iter() {
        if region.tiles.contains(&tile_pos) {
            hover.hovered_region = Some(region.region_id);
            break;
        }
    }

    // ── Resolve POI (exact tile match) ────────────────────────────────
    for poi in world_map.points_of_interest.iter() {
        if poi.position == tile_pos {
            hover.hovered_poi = Some(poi.poi_id);
            break;
        }
    }
}

// ── 6. Region Selection ────────────────────────────────────────────────────

/// Left-click on a region while in world map mode selects it (stored in
/// `WorldMapSelection`).  Clicking the same region again deselects it.
/// Right-click or Escape also deselects.
pub fn world_map_click_system(
    state: Res<State<AppState>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    hover: Res<WorldMapHover>,
    mut selection: ResMut<WorldMapSelection>,
) {
    if *state.get() != AppState::WorldMap {
        selection.selected_region = None;
        return;
    }

    // Right-click or Escape deselects
    let Some(input) = mouse else { return };
    let Some(kb) = keyboard else { return };

    if input.just_pressed(MouseButton::Right) || kb.just_pressed(KeyCode::Escape) {
        selection.selected_region = None;
        return;
    }

    // Left-click on a region selects it (toggle if already selected)
    if input.just_pressed(MouseButton::Left) {
        if let Some(region) = hover.hovered_region {
            if selection.selected_region == Some(region) {
                selection.selected_region = None; // deselect on re-click
            } else {
                selection.selected_region = Some(region);
            }
        } else {
            // Clicking empty space deselects
            selection.selected_region = None;
        }
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

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins((bevy::MinimalPlugins, bevy_state::app::StatesPlugin))
            .init_state::<AppState>()
            .init_resource::<WorldMap>()
            .init_resource::<WorldMapViewState>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(Update, (world_map_toggle_system, world_map_camera_system));
        app
    }

    /// Start in InGame, then toggle to WorldMap.
    #[test]
    fn test_toggle_from_ingame_to_worldmap() {
        let mut app = make_app();
        // Set initial state to InGame
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();

        // Initially InGame
        assert_eq!(*app.world().resource::<State<AppState>>(), AppState::InGame);

        // Set next state to WorldMap via NextState (simulates M key press)
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();

        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::WorldMap
        );
    }

    /// Toggle back from WorldMap to InGame.
    #[test]
    fn test_toggle_back_to_ingame() {
        let mut app = make_app();

        // Set initial state to WorldMap
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();
        assert_eq!(
            *app.world().resource::<State<AppState>>(),
            AppState::WorldMap
        );

        // Toggle back to InGame
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();

        assert_eq!(*app.world().resource::<State<AppState>>(), AppState::InGame);
    }

    /// Camera position should change to map centre on WorldMap state.
    #[test]
    fn test_camera_moves_to_map_centre_in_world_map() {
        let mut app = make_app();

        // Spawn a camera entity
        app.world_mut().spawn((
            Camera2d,
            Camera {
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                ..OrthographicProjection::default_2d()
            }),
        ));

        app.update();

        // Switch to WorldMap
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();

        let transform = app
            .world_mut()
            .query::<&Transform>()
            .get_single(&app.world());
        assert!(transform.is_ok());
        let transform = transform.unwrap();
        assert!(
            (transform.translation.x - MAP_CENTRE.x).abs() < 0.1,
            "Camera x should move to map centre ({}), got {}",
            MAP_CENTRE.x,
            transform.translation.x
        );
        assert!(
            (transform.translation.y - MAP_CENTRE.y).abs() < 0.1,
            "Camera y should move to map centre ({}), got {}",
            MAP_CENTRE.y,
            transform.translation.y
        );

        // Orthographic scale should increase
        let projection = app
            .world_mut()
            .query::<&Projection>()
            .get_single(&app.world());
        if let Some(Projection::Orthographic(ortho)) = projection.ok() {
            assert!(
                (ortho.scale - WORLD_MAP_ZOOM).abs() < 0.1,
                "Ortho scale should be {} in world map, got {}",
                WORLD_MAP_ZOOM,
                ortho.scale
            );
        }
    }

    /// Pan offset accumulates correctly.
    #[test]
    fn test_pan_offset_applied_to_camera() {
        let mut app = make_app();

        app.world_mut().spawn((
            Camera2d,
            Camera {
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                ..OrthographicProjection::default_2d()
            }),
        ));

        // Set pan offset
        app.world_mut()
            .resource_mut::<WorldMapViewState>()
            .pan_offset = Vec2::new(10.0, 5.0);

        // Switch to WorldMap
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();

        let transform = app
            .world_mut()
            .query::<&Transform>()
            .get_single(&app.world());
        assert!(transform.is_ok());
        let transform = transform.unwrap();
        assert!(
            (transform.translation.x - (MAP_CENTRE.x + 10.0)).abs() < 0.1,
            "Camera x should include pan offset"
        );
        assert!(
            (transform.translation.y - (MAP_CENTRE.y + 5.0)).abs() < 0.1,
            "Camera y should include pan offset"
        );
    }

    /// Zoom changes are persisted in the resource and affect camera scale.
    #[test]
    fn test_zoom_affects_camera() {
        let mut app = make_app();

        app.world_mut().spawn((
            Camera2d,
            Camera {
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                ..OrthographicProjection::default_2d()
            }),
        ));

        // Set zoom to 2.0
        app.world_mut().resource_mut::<WorldMapViewState>().zoom = 2.0;

        // Switch to WorldMap
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();

        let projection = app
            .world_mut()
            .query::<&Projection>()
            .get_single(&app.world());
        if let Some(Projection::Orthographic(ortho)) = projection.ok() {
            let expected = WORLD_MAP_ZOOM / 2.0;
            assert!(
                (ortho.scale - expected).abs() < 0.1,
                "Ortho scale should be {:.1} with zoom 2.0, got {:.1}",
                expected,
                ortho.scale
            );
        }
    }

    /// Camera resets when returning to InGame.
    #[test]
    fn test_camera_resets_on_return_to_ingame() {
        let mut app = make_app();

        app.world_mut().spawn((
            Camera2d,
            Camera {
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.0, 100.0),
            Projection::Orthographic(OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                ..OrthographicProjection::default_2d()
            }),
        ));

        // Set pan offset
        app.world_mut()
            .resource_mut::<WorldMapViewState>()
            .pan_offset = Vec2::new(10.0, 5.0);
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::WorldMap);
        app.update();

        // Return to InGame
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::InGame);
        app.update();

        let transform = app
            .world_mut()
            .query::<&Transform>()
            .get_single(&app.world());
        assert!(transform.is_ok());
        let transform = transform.unwrap();
        assert!(
            (transform.translation.x).abs() < 0.1,
            "Camera x should reset to 0 on InGame"
        );
        assert!(
            (transform.translation.y).abs() < 0.1,
            "Camera y should reset to 0 on InGame"
        );
    }
}
