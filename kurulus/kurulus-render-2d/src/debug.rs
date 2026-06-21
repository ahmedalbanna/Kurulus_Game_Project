use bevy::prelude::*;
use kurulus_core::coord::*;
use kurulus_core::components::unit::*;
use kurulus_core::components::stealth::Detection;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            draw_grid_gizmos,
            draw_formation_gizmos,
            draw_detection_gizmos,
        ));
    }
}

/// Draws grid lines for visual reference
fn draw_grid_gizmos(
    mut gizmos: Gizmos,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
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

/// Draws formation leaders and follower positions
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

/// Draws detection radius for units with Detection component
fn draw_detection_gizmos(
    mut gizmos: Gizmos,
    detectors: Query<(&WorldPosition, &Detection)>,
) {
    for (pos, detection) in detectors.iter() {
        if detection.active {
            gizmos.circle_2d(pos.world_2d.0, detection.radius, Color::srgba(1.0, 0.0, 0.0, 0.2));
        }
    }
}
