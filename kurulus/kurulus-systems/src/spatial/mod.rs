use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpatialGrid>()
            .add_systems(
                FixedUpdate,
                // Phase 1: Auto-tag new entities that have WorldPosition
                // but aren't yet tracked.  Uses Commands (deferred), so tags
                // are visible only after apply_deferred (set boundary).
                auto_tag_unit_entities.in_set(SimulationSet::InputProcessing),
            )
            .add_systems(
                FixedUpdate,
                // Phase 2: Rebuild spatial grid — runs in CommandQueueing
                // so that auto_tag_unit_entities Commands have been flushed.
                spatial_grid_update_system.in_set(SimulationSet::CommandQueueing),
            );
    }
}

/// Automatically insert `TrackedBySpatial` on any entity that has
/// `WorldPosition` but is not yet tracked.  This ensures all units and
/// combat-relevant entities participate in spatial queries without
/// requiring manual tagging at every spawn site.
///
/// Uses deferred Commands — tags become visible after apply_deferred
/// (at the InputProcessing → CommandQueueing boundary).
///
/// The spatial grid is a pure optimisation — including extra entities
/// (buildings, etc.) is harmless because consuming systems filter by
/// faction, component, and distance as needed.
fn auto_tag_unit_entities(
    mut commands: Commands,
    units: Query<Entity, (With<WorldPosition>, Without<TrackedBySpatial>)>,
) {
    for entity in units.iter() {
        commands.entity(entity).insert(TrackedBySpatial);
    }
}

/// Rebuild the `SpatialGrid` every frame from all `TrackedBySpatial` entities.
///
/// This runs in `InputProcessing` (the first system set), so all subsequent
/// systems (AI, flanking, stealth, etc.) see up-to-date spatial data.
fn spatial_grid_update_system(
    mut grid: ResMut<SpatialGrid>,
    tracked: Query<(&WorldPosition, Entity), With<TrackedBySpatial>>,
) {
    grid.clear();
    for (pos, entity) in tracked.iter() {
        grid.insert(entity, pos.grid.0);
    }
}
