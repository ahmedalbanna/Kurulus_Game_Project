use bevy::prelude::*;

pub struct Camera3DPlugin;

impl Plugin for Camera3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_3d_camera);
    }
}

/// Sets up an orthographic 3D camera for isometric rendering.
/// This is used in Phase 2 when transitioning from 2D prototype to full 3D.
fn setup_3d_camera(mut commands: Commands) {
    use bevy::render::camera::ScalingMode;

    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        Msaa::Off,
        Transform::from_xyz(0.0, 30.0, 30.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Orthographic(OrthographicProjection {
            near: 0.0,
            far: 1000.0,
            viewport_origin: Vec2::new(0.5, 0.5),
            scaling_mode: ScalingMode::WindowSize,
            scale: 20.0,
            area: Rect::new(-1.0, -1.0, 1.0, 1.0),
        }),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

/// Returns a true isometric transform (45° yaw, 35.264° pitch).
/// Use this in Phase 2 for the final isometric camera angle.
pub fn isometric_transform() -> Transform {
    Transform::from_rotation(
        Quat::from_rotation_y(45.0_f32.to_radians())
            * Quat::from_rotation_x(35.264_f32.to_radians()),
    )
}
