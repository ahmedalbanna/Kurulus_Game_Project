use bevy::prelude::*;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: Implement terrain generation with heightmap and splatmap
    }
}

/// Component marking a terrain chunk
#[derive(Component)]
pub struct TerrainChunk {
    pub size: u32,
    pub resolution: u32,
}
