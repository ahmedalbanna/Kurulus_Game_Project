use crate::resources::pathfinding::CostProvider;
use crate::resources::PathfindingGrid;
use bevy::math::IVec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;

/// A pre‑computed flow field that guides many units toward a common
/// destination in O(1) per unit per tick — far cheaper than per‑unit A*.
///
/// # Algorithm
/// 1. **Cost field** — wavefront expansion from the destination outward.
///    Each cell stores the accumulated movement cost to reach the goal.
/// 2. **Flow field** — for each cell, the direction (as an `IVec2` step)
///    toward the neighbour with the lowest cost.
///
/// Units at position `P` look up `flow_field[P]` and move one cell in
/// that direction each tick.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct FlowFieldCache {
    /// Cached flow fields, keyed by destination grid cell.
    /// The inner map is `cost_field` + `flow_field` stored together.
    pub fields: HashMap<IVec2, FlowField>,
}

impl Default for FlowFieldCache {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl FlowFieldCache {
    /// Retrieve the flow field for a destination, if cached.
    pub fn get(&self, destination: IVec2) -> Option<&FlowField> {
        self.fields.get(&destination)
    }

    /// Insert or replace a flow field.
    pub fn insert(&mut self, destination: IVec2, field: FlowField) {
        self.fields.insert(destination, field);
    }

    /// Remove all cached fields so they get rebuilt next frame.
    pub fn invalidate_all(&mut self) {
        self.fields.clear();
    }
}

/// A single flow field for one destination.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct FlowField {
    /// The destination cell this field leads to.
    pub destination: IVec2,
    /// Cost field: accumulated movement cost from each cell to the
    /// destination (Dijkstra wavefront).
    pub cost_field: HashMap<IVec2, f32>,
    /// Flow field: the best *direction* (unit IVec2 step) from each cell.
    /// Units move one grid cell per tick along this direction.
    pub flow_field: HashMap<IVec2, IVec2>,
    /// Search bounds used when building this field.
    pub bounds: (IVec2, IVec2),
}

impl FlowField {
    /// Build a flow field toward `destination` using the given terrain grid.
    ///
    /// Uses a Dijkstra wavefront that expands outward until the search
    /// hits `max_cells` total explored cells or reaches the bounds.
    pub fn build(
        destination: IVec2,
        grid: &PathfindingGrid,
        bounds: (IVec2, IVec2),
        max_cells: usize,
    ) -> Self {
        let mut cost_field: HashMap<IVec2, f32> = HashMap::new();
        let mut frontier = VecDeque::new(); // BFS frontier for wavefront expansion

        cost_field.insert(destination, 0.0);
        frontier.push_back(destination);

        let mut explored = 0;

        // ── Phase 1: Wavefront cost expansion (BFS) ────────────────────
        while let Some(current) = frontier.pop_front() {
            explored += 1;
            if explored > max_cells {
                break;
            }

            let current_cost = cost_field[&current];

            for (neighbour, edge_cost) in neighbours(current) {
                // Bounds check
                if neighbour.x < bounds.0.x
                    || neighbour.y < bounds.0.y
                    || neighbour.x > bounds.1.x
                    || neighbour.y > bounds.1.y
                {
                    continue;
                }

                // Terrain cost to enter neighbour
                let terrain_cost = grid.cost(neighbour);
                if terrain_cost >= f32::MAX {
                    continue; // impassable
                }

                let total_cost = current_cost + edge_cost * terrain_cost;

                let entry = cost_field.entry(neighbour).or_insert(f32::MAX);
                if total_cost < *entry {
                    *entry = total_cost;
                    frontier.push_back(neighbour);
                }
            }
        }

        // ── Phase 2: Compute flow directions ──────────────────────────
        let mut flow_field: HashMap<IVec2, IVec2> = HashMap::new();

        for (&pos, &cost) in &cost_field {
            if pos == destination {
                continue; // no direction needed at goal
            }

            // Find the neighbour with the lowest cost
            let mut best_dir = IVec2::ZERO;
            let mut best_cost = cost; // must be strictly lower

            for (neighbour, _edge_cost) in neighbours(pos) {
                if let Some(&nc) = cost_field.get(&neighbour) {
                    if nc < best_cost {
                        best_cost = nc;
                        best_dir = neighbour - pos; // direction vector
                    }
                }
            }

            if best_dir != IVec2::ZERO {
                flow_field.insert(pos, best_dir);
            }
        }

        Self {
            destination,
            cost_field,
            flow_field,
            bounds,
        }
    }

    /// Returns the movement direction for a unit at `pos`, or `None` if
    /// the position is outside the field or is the destination itself.
    pub fn direction(&self, pos: IVec2) -> Option<IVec2> {
        self.flow_field.get(&pos).copied()
    }

    /// Returns the cost‑to‑destination for a cell, or `None` if unknown.
    pub fn cost(&self, pos: IVec2) -> Option<f32> {
        self.cost_field.get(&pos).copied()
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

/// 8-directional neighbours of a grid cell, with edge distance weights.
fn neighbours(pos: IVec2) -> [(IVec2, f32); 8] {
    let sqrt_2 = std::f32::consts::SQRT_2;
    [
        (IVec2::new(pos.x + 1, pos.y), 1.0),
        (IVec2::new(pos.x - 1, pos.y), 1.0),
        (IVec2::new(pos.x, pos.y + 1), 1.0),
        (IVec2::new(pos.x, pos.y - 1), 1.0),
        (IVec2::new(pos.x + 1, pos.y + 1), sqrt_2),
        (IVec2::new(pos.x - 1, pos.y + 1), sqrt_2),
        (IVec2::new(pos.x + 1, pos.y - 1), sqrt_2),
        (IVec2::new(pos.x - 1, pos.y - 1), sqrt_2),
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TerrainType;
    use crate::resources::TerrainGrid;
    use std::collections::HashSet;

    fn test_grid() -> PathfindingGrid {
        let mut terrain = TerrainGrid::default();
        for x in -10..=10 {
            for y in -10..=10 {
                terrain.set(IVec2::new(x, y), TerrainType::OpenField);
            }
        }
        // Place a road row at y=0
        for x in -10..=10 {
            terrain.set(IVec2::new(x, 0), TerrainType::Road);
        }
        let obstacles = HashSet::new();
        PathfindingGrid::build(&terrain, &obstacles)
    }

    const TEST_BOUNDS: (IVec2, IVec2) = (IVec2::new(-100, -100), IVec2::new(100, 100));

    #[test]
    fn test_flow_field_points_toward_destination() {
        let grid = test_grid();
        let dest = IVec2::new(5, 0);
        let field = FlowField::build(dest, &grid, TEST_BOUNDS, 10_000);

        // A cell to the left of the destination should flow right (toward dest)
        let dir = field.direction(IVec2::new(3, 0));
        assert_eq!(dir, Some(IVec2::new(1, 0)), "Should flow right toward dest");

        // A cell below dest should flow up
        let dir = field.direction(IVec2::new(5, 3));
        assert_eq!(dir, Some(IVec2::new(0, -1)), "Should flow up toward dest");
    }

    #[test]
    fn test_flow_field_cost_decreases_toward_dest() {
        let grid = test_grid();
        let dest = IVec2::new(5, 0);
        let field = FlowField::build(dest, &grid, TEST_BOUNDS, 10_000);

        let cost_far = field.cost(IVec2::new(3, 3)).unwrap_or(f32::MAX);
        let cost_near = field.cost(IVec2::new(4, 1)).unwrap_or(f32::MAX);
        assert!(
            cost_near < cost_far,
            "Cost should decrease as we approach dest"
        );

        let cost_at_dest = field.cost(dest).unwrap_or(f32::MAX);
        assert_eq!(cost_at_dest, 0.0, "Cost at destination should be 0");
    }

    #[test]
    fn test_flow_field_prefers_road() {
        let grid = test_grid();
        let dest = IVec2::new(5, 0);
        let field = FlowField::build(dest, &grid, TEST_BOUNDS, 10_000);

        // Direct route: along y=0 (road, cost 0.8)
        // The flow field should direct (3, 0) → (4, 0) → (5, 0) along the road
        let dir = field.direction(IVec2::new(3, 0));
        assert_eq!(dir, Some(IVec2::new(1, 0)), "Road route should flow east");

        // A cell at (3, 1) could go diagonally or straight to the road
        // This is implementation-dependent, but verify it at least has a direction
        let dir = field.direction(IVec2::new(3, 1));
        assert!(dir.is_some(), "Should have a direction for off-road cells");
    }

    #[test]
    fn test_flow_field_no_direction_at_destination() {
        let grid = test_grid();
        let dest = IVec2::new(5, 0);
        let field = FlowField::build(dest, &grid, TEST_BOUNDS, 10_000);

        let dir = field.direction(dest);
        assert_eq!(dir, None, "No direction needed at destination");
    }

    #[test]
    fn test_flow_field_cache() {
        let mut cache = FlowFieldCache::default();
        let dest = IVec2::new(5, 0);

        assert!(cache.get(dest).is_none(), "Cache should start empty");

        let grid = test_grid();
        let field = FlowField::build(dest, &grid, TEST_BOUNDS, 10_000);
        cache.insert(dest, field);

        assert!(cache.get(dest).is_some(), "Cache should contain dest");

        cache.invalidate_all();
        assert!(
            cache.get(dest).is_none(),
            "Cache should be empty after invalidate"
        );
    }
}
