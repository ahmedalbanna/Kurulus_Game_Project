use bevy::prelude::*;

pub struct Camera2DPlugin;

impl Plugin for Camera2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_2d_camera);
    }
}

fn setup_2d_camera(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, 0.0, 100.0)));
}
