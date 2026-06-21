use bevy::prelude::*;
use kurulus_core::coord::WorldPosition;

pub struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_world_position_to_transform);
    }
}

/// Syncs WorldPosition to Transform for 2D sprites
fn sync_world_position_to_transform(
    mut sprites: Query<(&mut Transform, &WorldPosition), Changed<WorldPosition>>,
) {
    for (mut transform, pos) in sprites.iter_mut() {
        transform.translation.x = pos.world_2d.0.x;
        transform.translation.y = pos.world_2d.0.y;
    }
}

/// Component marker for sprites awaiting position sync
#[derive(Component)]
pub struct NeedsPositionSync;
