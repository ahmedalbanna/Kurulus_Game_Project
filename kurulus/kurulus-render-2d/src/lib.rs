pub mod sprite;
pub mod camera;
pub mod debug;

use bevy::prelude::*;

pub struct Render2DPlugin;

impl Plugin for Render2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sprite::SpritePlugin,
            camera::Camera2DPlugin,
            debug::DebugRenderPlugin,
        ));
    }
}
