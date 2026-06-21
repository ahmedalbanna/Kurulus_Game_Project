use bevy::prelude::*;
use kurulus_core::coord::WorldPosition;

pub struct PBRPlugin;

impl Plugin for PBRPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_world_position_to_3d);
    }
}

/// Syncs WorldPosition to Transform for 3D PBR entities
fn sync_world_position_to_3d(
    mut entities: Query<(&mut Transform, &WorldPosition), Changed<WorldPosition>>,
) {
    for (mut transform, pos) in entities.iter_mut() {
        transform.translation = pos.world_3d.0;
    }
}
