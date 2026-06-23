pub mod animation;
pub mod camera;
pub mod pbr;
pub mod terrain;

use bevy::prelude::*;

pub struct Render3DPlugin;

impl Plugin for Render3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            pbr::PBRPlugin,
            camera::Camera3DPlugin,
            terrain::TerrainPlugin,
            animation::AnimationPlugin,
        ));
    }
}
