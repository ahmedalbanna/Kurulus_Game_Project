//! Player-facing building placement mode.
//!
//! Activated by pressing **U** — parallels the zone placement mode (B key).
//! While active, the player can hover the grid, cycle building types with
//! **Tab** / **Shift+Tab**, and left-click to send a [`BuildOrder`] event.
//!
//! The [`BuildOrder`] is consumed by `build_from_order_system` in the
//! `base_building` module, which validates zone compatibility and spawns
//! a `ConstructionSite` + `Building` pair.

use bevy::input::ButtonInput;
use bevy::math::IVec2;
use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Constants ───────────────────────────────────────────────────────────────

/// Dummy entity used for `BuildOrder.entity` when the player places a building
/// (no specific worker entity is available from UI placement).
const PLAYER_BUILD_ENTITY: Entity = Entity::from_raw(u32::MAX);

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Toggles building placement mode and selects the building type.
///
/// Keybinds:
///   • **U**            — toggle placement mode on/off
///   • **Escape**       — exit placement mode
///   • **Tab**          — next building type
///   • **Shift+Tab**    — previous building type
///
/// When activated, this system deactivates zone placement mode so the two
/// modes are mutually exclusive.
pub fn building_placement_toggle_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut placement: ResMut<BuildingPlacementState>,
    mut zone_placement: ResMut<ZonePlacementState>,
) {
    let Some(input) = keyboard else {
        return;
    };

    // ── Toggle mode ────────────────────────────────────────────────────
    if input.just_pressed(KeyCode::KeyU) {
        placement.active = !placement.active;
        if placement.active {
            // Deactivate zone placement when entering building mode
            zone_placement.active = false;
            zone_placement.hovered_grid = None;
        } else {
            placement.hovered_grid = None;
        }
        info!(
            "🏗️  Building placement mode: {}",
            if placement.active { "ON" } else { "OFF" }
        );
    }

    // ── Exit on Escape ─────────────────────────────────────────────────
    if input.just_pressed(KeyCode::Escape) && placement.active {
        placement.active = false;
        placement.hovered_grid = None;
        info!("🏗️  Building placement mode: OFF");
    }

    if !placement.active {
        return;
    }

    // ── Cycle building type (Tab / Shift+Tab) ──────────────────────────
    let n_types = BUILDING_TYPE_CYCLE.len();
    let current_idx = BUILDING_TYPE_CYCLE
        .iter()
        .position(|t| *t == placement.selected_type)
        .unwrap_or(0);

    if input.just_pressed(KeyCode::Tab) {
        let shift = input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight);
        let next_idx = if shift {
            // Shift+Tab → previous
            if current_idx == 0 {
                n_types - 1
            } else {
                current_idx - 1
            }
        } else {
            // Tab → next
            (current_idx + 1) % n_types
        };
        placement.selected_type = BUILDING_TYPE_CYCLE[next_idx];
        info!(
            "🏗️  Building type ({}/{}): {:?}",
            next_idx + 1,
            n_types,
            placement.selected_type,
        );
    }
}

/// Projects the mouse cursor onto the grid and updates `hovered_grid`.
///
/// Runs only when building placement mode is active. Gracefully handles
/// missing window, camera, or cursor (off-screen / unfocused).
pub fn building_placement_hover_system(
    mut placement: ResMut<BuildingPlacementState>,
    zone_placement: Res<ZonePlacementState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    // Building placement runs only in its own mode, and doesn't interfere
    // with zone placement (which has its own hover system).
    if !placement.active {
        placement.hovered_grid = None;
        return;
    }

    // Don't hover if zone placement is also active (shouldn't happen due
    // to mutual exclusion in the toggle system, but be defensive).
    if zone_placement.active {
        placement.hovered_grid = None;
        return;
    }

    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        placement.hovered_grid = None;
        return;
    };

    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        let grid_x = (world_pos.x / CELL_SIZE).floor() as i32;
        let grid_y = (world_pos.y / CELL_SIZE).floor() as i32;
        placement.hovered_grid = Some(IVec2::new(grid_x, grid_y));
    } else {
        placement.hovered_grid = None;
    }
}

/// Sends a [`BuildOrder`] event on left click when in building placement mode.
///
/// * **Left click** — sends a `BuildOrder` at the hovered grid position.
/// * **Right click** — exits placement mode (convenience).
pub fn building_placement_execute_system(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut placement: ResMut<BuildingPlacementState>,
    mut build_order_writer: EventWriter<BuildOrder>,
) {
    let Some(input) = mouse else {
        return;
    };
    if !placement.active {
        return;
    }
    let Some(grid_pos) = placement.hovered_grid else {
        return;
    };

    // ── Left click: place ──────────────────────────────────────────────
    if input.just_pressed(MouseButton::Left) {
        build_order_writer.send(BuildOrder {
            entity: PLAYER_BUILD_ENTITY,
            building_type: placement.selected_type,
            position: GridPos(grid_pos),
        });
        info!(
            "🏗️  Build order: {:?} at ({}, {})",
            placement.selected_type, grid_pos.x, grid_pos.y,
        );
    }

    // ── Right click: exit placement mode ───────────────────────────────
    if input.just_pressed(MouseButton::Right) {
        placement.active = false;
        placement.hovered_grid = None;
        info!("🏗️  Building placement mode: OFF");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use kurulus_core::components::building::BuildingType;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<BuildingPlacementState>()
            .init_resource::<ZonePlacementState>()
            .add_event::<BuildOrder>()
            .add_systems(
                Update,
                (
                    building_placement_toggle_system,
                    building_placement_hover_system,
                    building_placement_execute_system,
                ),
            );
        app
    }

    /// Simulate a key press by inserting a fresh `ButtonInput<KeyCode>`.
    fn press_key(app: &mut App, key: KeyCode) {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(key);
        app.insert_resource(input);
    }

    /// Simulate a mouse-button press by inserting a fresh `ButtonInput<MouseButton>`.
    fn press_mouse(app: &mut App, button: MouseButton) {
        let mut input = ButtonInput::<MouseButton>::default();
        input.press(button);
        app.insert_resource(input);
    }

    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ─── Toggle system ─────────────────────────────────────────────────

    #[test]
    fn test_toggle_activates_mode() {
        let mut app = test_app();
        assert!(!app.world().resource::<BuildingPlacementState>().active);

        // Press U
        press_key(&mut app, KeyCode::KeyU);
        run_frames(&mut app, 1);

        assert!(app.world().resource::<BuildingPlacementState>().active);
    }

    #[test]
    fn test_toggle_deactivates_zone_mode() {
        let mut app = test_app();

        // First activate zone mode
        app.world_mut().resource_mut::<ZonePlacementState>().active = true;

        // Then activate building mode
        press_key(&mut app, KeyCode::KeyU);
        run_frames(&mut app, 1);

        assert!(app.world().resource::<BuildingPlacementState>().active);
        assert!(
            !app.world().resource::<ZonePlacementState>().active,
            "Zone placement should be deactivated"
        );
    }

    #[test]
    fn test_escape_exits_mode() {
        let mut app = test_app();

        // Activate building mode
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .active = true;
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .hovered_grid = Some(IVec2::new(5, 5));

        // Press Escape
        press_key(&mut app, KeyCode::Escape);
        run_frames(&mut app, 1);

        assert!(!app.world().resource::<BuildingPlacementState>().active);
        assert!(app
            .world()
            .resource::<BuildingPlacementState>()
            .hovered_grid
            .is_none());
    }

    #[test]
    fn test_tab_cycles_building_type() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .active = true;

        let initial = app
            .world()
            .resource::<BuildingPlacementState>()
            .selected_type;

        // Press Tab to cycle
        press_key(&mut app, KeyCode::Tab);
        run_frames(&mut app, 1);

        let next = app
            .world()
            .resource::<BuildingPlacementState>()
            .selected_type;
        assert_ne!(initial, next, "Tab should change building type");
    }

    // ─── Execute system ────────────────────────────────────────────────

    #[test]
    fn test_left_click_sends_build_order() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .active = true;
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .hovered_grid = Some(IVec2::new(3, 4));
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .selected_type = BuildingType::Barracks;

        // Left click — run only 1 frame since MinimalPlugins doesn't clear
        // just_pressed, so the event would be re-sent every frame.
        press_mouse(&mut app, MouseButton::Left);
        run_frames(&mut app, 1);

        // Check that a BuildOrder event was sent
        let reader = app
            .world_mut()
            .get_resource_mut::<Events<BuildOrder>>()
            .unwrap();
        let mut cursor = reader.get_cursor();
        let orders: Vec<BuildOrder> = cursor.read(&reader).cloned().collect();
        assert_eq!(orders.len(), 1, "Should have sent one BuildOrder");
        assert_eq!(orders[0].building_type, BuildingType::Barracks);
        assert_eq!(orders[0].position, GridPos(IVec2::new(3, 4)));
    }

    #[test]
    fn test_right_click_exits_mode() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .active = true;
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .hovered_grid = Some(IVec2::new(3, 4));

        // Right click
        press_mouse(&mut app, MouseButton::Right);
        run_frames(&mut app, 1);

        assert!(!app.world().resource::<BuildingPlacementState>().active);
        assert!(app
            .world()
            .resource::<BuildingPlacementState>()
            .hovered_grid
            .is_none());
    }

    #[test]
    fn test_no_order_when_not_active() {
        let mut app = test_app();
        // Not activating building mode
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .hovered_grid = Some(IVec2::new(3, 4));

        press_mouse(&mut app, MouseButton::Left);
        run_frames(&mut app, 1);

        let reader = app
            .world_mut()
            .get_resource_mut::<Events<BuildOrder>>()
            .unwrap();
        let mut cursor = reader.get_cursor();
        let orders: Vec<BuildOrder> = cursor.read(&reader).cloned().collect();
        assert!(orders.is_empty(), "No order should be sent when not active");
    }

    #[test]
    fn test_no_order_without_hover() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<BuildingPlacementState>()
            .active = true;
        // No hovered_grid set

        press_mouse(&mut app, MouseButton::Left);
        run_frames(&mut app, 1);

        let reader = app
            .world_mut()
            .get_resource_mut::<Events<BuildOrder>>()
            .unwrap();
        let mut cursor = reader.get_cursor();
        let orders: Vec<BuildOrder> = cursor.read(&reader).cloned().collect();
        assert!(orders.is_empty(), "No order without hovered grid");
    }

    // ─── BUILDING_TYPE_CYCLE tests ─────────────────────────────────────

    #[test]
    fn test_building_cycle_contains_all_types() {
        use std::collections::HashSet;
        let all_types: HashSet<&BuildingType> = BUILDING_TYPE_CYCLE.iter().collect();
        // Check a few key types are present
        assert!(all_types.contains(&&BuildingType::Oba));
        assert!(all_types.contains(&&BuildingType::Barracks));
        assert!(all_types.contains(&&BuildingType::Market));
        assert!(all_types.contains(&&BuildingType::Mosque));
        assert!(all_types.contains(&&BuildingType::CouncilChamber));
        assert!(all_types.contains(&&BuildingType::Road));
    }

    #[test]
    fn test_cycle_no_duplicates() {
        use std::collections::HashSet;
        let unique: HashSet<&BuildingType> = BUILDING_TYPE_CYCLE.iter().collect();
        assert_eq!(
            unique.len(),
            BUILDING_TYPE_CYCLE.len(),
            "Cycle should not contain duplicates"
        );
    }
}
