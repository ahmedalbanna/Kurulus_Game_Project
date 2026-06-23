pub mod sprite;
pub mod camera;
pub mod debug;
pub mod building_sprites;
pub mod world_map;

use bevy::prelude::*;

pub struct Render2DPlugin;

impl Plugin for Render2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            sprite::SpritePlugin,
            camera::Camera2DPlugin,
            debug::DebugRenderPlugin,
            building_sprites::BuildingSpritePlugin,
            world_map::WorldMapRenderPlugin,
        ));
    }
}
