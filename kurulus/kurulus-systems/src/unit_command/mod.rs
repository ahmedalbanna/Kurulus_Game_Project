use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct UnitCommandPlugin;

impl Plugin for UnitCommandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                formation_movement_system,
                free_form_movement_system,
                grid_quantization_system,
                pathfinding_system,
            )
                .in_set(SimulationSet::FormationMovement),
        );
    }
}

/// Moves units in formation relative to their leader
fn formation_movement_system(
    time: Res<bevy::time::Time>,
    _move_events: EventReader<MoveOrder>,
    leaders: Query<(Entity, &WorldPosition), (With<Unit>, Without<FormationFollower>)>,
    mut followers: Query<(Entity, &mut WorldPosition, &Formation, &Stats), With<FormationFollower>>,
) {
    for (_follower_e, mut follower_pos, formation, stats) in followers.iter_mut() {
        if let Ok((_leader_e, leader_pos)) = leaders.get(formation.leader) {
            let target = leader_pos.world_2d.0 + formation.offset;
            let current = follower_pos.world_2d.0;
            let direction = (target - current).normalize_or_zero();
            let distance = stats.move_speed * time.delta_secs();
            follower_pos.world_2d.0 += direction * distance;
        }
    }
}

/// Moves units freely (not in formation) toward their target
fn free_form_movement_system(
    _time: Res<bevy::time::Time>,
    _units: Query<(&mut WorldPosition, &Stats), (With<Unit>, Without<FormationFollower>)>,
) {
    // TODO: Implement free-form movement
}

/// Quantizes continuous positions back to grid after movement
fn grid_quantization_system(_time: Res<GameTime>, mut positions: Query<&mut WorldPosition>) {
    for mut pos in positions.iter_mut() {
        pos.quantize(CELL_SIZE);
    }
}

/// Calculates paths for movement orders (A* or similar)
fn pathfinding_system(_move_events: EventReader<MoveOrder>, _positions: Query<&WorldPosition>) {
    // TODO: Implement A* pathfinding
}
