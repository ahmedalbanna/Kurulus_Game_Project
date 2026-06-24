use crate::resources::TerrainGrid;
use bevy::math::IVec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Movement cost for impassable cells.
const IMPASSABLE_COST: f32 = f32::MAX;

/// Trait for types that can provide movement costs for A* pathfinding.
///
/// This allows A* to work with both the simple `HashSet<IVec2>` obstacle set
/// (cost = 1.0 for passable, INF for blocked) and the richer `PathfindingGrid`
/// that accounts for terrain costs.
pub trait CostProvider {
    /// Returns the movement cost to enter cell `pos`.
    /// Should return `f32::MAX` for impassable cells.
    fn cost(&self, pos: IVec2) -> f32;
}

impl CostProvider for HashSet<IVec2> {
    fn cost(&self, pos: IVec2) -> f32 {
        if self.contains(&pos) {
            f32::MAX
        } else {
            1.0
        }
    }
}

impl CostProvider for PathfindingGrid {
    fn cost(&self, pos: IVec2) -> f32 {
        if self.blocked.contains(&pos) {
            return f32::MAX;
        }
        // Return the per-cell terrain cost recorded by `build()`,
        // defaulting to 1.0 for cells not in the terrain grid.
        self.costs.get(&pos).copied().unwrap_or(1.0)
    }
}

/// Combined pathfinding grid that merges building obstacles with terrain
/// movement costs. Rebuilt each frame before A* is run.
///
/// This provides a unified cost function for the enhanced A* algorithm so
/// that units naturally prefer roads (cost 0.8) over forests (cost 1.5) and
/// avoid impassable terrain entirely.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct PathfindingGrid {
    /// Cells that are blocked by buildings, static obstacles, or are
    /// inherently impassable (deep water, mountains, etc.).
    pub blocked: HashSet<IVec2>,
    /// Per-cell movement cost multiplier, keyed by grid position.
    /// Cells in `blocked` will still have a cost here (for BRP introspection),
    /// but A* treats them as impassable regardless.
    pub costs: HashMap<IVec2, f32>,
}

impl Default for PathfindingGrid {
    fn default() -> Self {
        Self {
            blocked: HashSet::new(),
            costs: HashMap::new(),
        }
    }
}

impl PathfindingGrid {
    /// Returns `true` if a grid cell is passable (not blocked).
    pub fn is_passable(&self, pos: IVec2) -> bool {
        !self.blocked.contains(&pos)
    }

    /// Returns the movement cost for a given grid cell.
    ///
    /// Returns `IMPASSABLE_COST` for blocked cells so A* will never
    /// route through them, and returns the terrain cost (from
    /// `TerrainGrid`) for passable cells.
    pub fn movement_cost(&self, pos: IVec2, terrain: &TerrainGrid) -> f32 {
        if self.blocked.contains(&pos) {
            return IMPASSABLE_COST;
        }
        terrain.movement_cost(pos)
    }

    /// Batch build the grid from a terrain grid and a set of obstacle cells
    /// (buildings, etc.).  Also marks inherently impassable terrain types
    /// (water, mountain) as blocked so A* respects terrain passability.
    pub fn build(terrain: &TerrainGrid, obstacles: &HashSet<IVec2>) -> Self {
        let mut blocked = HashSet::new();
        let mut costs = HashMap::new();

        // Start from building obstacles
        for &obs in obstacles {
            blocked.insert(obs);
        }

        // Mark impassable terrain types (deep water, mountains) as blocked
        // even without explicit building obstacles, and record all costs.
        for (&pos, ttype) in &terrain.tiles {
            let cost = ttype.movement_cost();
            costs.insert(pos, cost);
            if cost >= 2.5 {
                // Water (3.0) and Mountain (2.5) are blocked for ground units.
                // Buildings (2.0) are blocked only if in the obstacle set.
                blocked.insert(pos);
            }
        }

        Self { blocked, costs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TerrainType;

    fn test_terrain() -> TerrainGrid {
        let mut t = TerrainGrid::default();
        t.set(IVec2::new(0, 0), TerrainType::OpenField);
        t.set(IVec2::new(1, 0), TerrainType::Road);
        t.set(IVec2::new(2, 0), TerrainType::Forest);
        t.set(IVec2::new(0, 1), TerrainType::Water);
        t.set(IVec2::new(1, 1), TerrainType::Mountain);
        t.set(IVec2::new(2, 1), TerrainType::Building);
        t
    }

    #[test]
    fn test_build_grid_from_terrain_and_obstacles() {
        let terrain = test_terrain();
        let mut obstacles = HashSet::new();
        // Building adds obstacle at (2, 1) plus its 2×2 footprint
        obstacles.insert(IVec2::new(2, 1));
        obstacles.insert(IVec2::new(3, 1));
        obstacles.insert(IVec2::new(2, 2));
        obstacles.insert(IVec2::new(3, 2));

        let grid = PathfindingGrid::build(&terrain, &obstacles);

        // Open field is passable
        assert!(
            grid.is_passable(IVec2::new(0, 0)),
            "OpenField should be passable"
        );
        // Road is passable
        assert!(
            grid.is_passable(IVec2::new(1, 0)),
            "Road should be passable"
        );
        // Forest is passable (cost 1.5 < 2.5 threshold)
        assert!(
            grid.is_passable(IVec2::new(2, 0)),
            "Forest should be passable"
        );
        // Water is impassable (cost 3.0 >= 2.5)
        assert!(
            !grid.is_passable(IVec2::new(0, 1)),
            "Water should be impassable"
        );
        // Mountain is impassable (cost 2.5 >= 2.5)
        assert!(
            !grid.is_passable(IVec2::new(1, 1)),
            "Mountain should be impassable"
        );
        // Building (cost 2.0) is passable unless in obstacle set
        assert!(
            !grid.is_passable(IVec2::new(2, 1)),
            "Building in obstacle set should be blocked"
        );
    }

    #[test]
    fn test_movement_cost_reflects_terrain() {
        let terrain = test_terrain();
        let obstacles = HashSet::new();
        let grid = PathfindingGrid::build(&terrain, &obstacles);

        assert_eq!(grid.movement_cost(IVec2::new(0, 0), &terrain), 1.0);
        assert_eq!(grid.movement_cost(IVec2::new(1, 0), &terrain), 0.8);
        assert_eq!(grid.movement_cost(IVec2::new(2, 0), &terrain), 1.5);
        // Blocked cell returns IMPASSABLE_COST
        assert_eq!(
            grid.movement_cost(IVec2::new(0, 1), &terrain),
            IMPASSABLE_COST
        );
    }
}
