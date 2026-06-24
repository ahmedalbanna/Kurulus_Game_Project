use bevy::prelude::*;
use kurulus_core::prelude::*;
use std::collections::{BinaryHeap, HashMap, HashSet};

const SQRT_2: f32 = std::f32::consts::SQRT_2;

/// Heuristic cost estimate (octile distance — allows 8-directional movement).
fn heuristic(a: IVec2, b: IVec2) -> f32 {
    let dx = (a.x - b.x).abs() as f32;
    let dy = (a.y - b.y).abs() as f32;
    // Octile distance: accounts for sqrt(2) diagonal cost
    let d = dx.max(dy);
    let s = dx.min(dy);
    (d - s) + s * SQRT_2
}

/// 8-directional neighbours of a grid cell.
fn neighbours(pos: IVec2) -> [(IVec2, f32); 8] {
    [
        (IVec2::new(pos.x + 1, pos.y), 1.0),
        (IVec2::new(pos.x - 1, pos.y), 1.0),
        (IVec2::new(pos.x, pos.y + 1), 1.0),
        (IVec2::new(pos.x, pos.y - 1), 1.0),
        (IVec2::new(pos.x + 1, pos.y + 1), SQRT_2),
        (IVec2::new(pos.x - 1, pos.y + 1), SQRT_2),
        (IVec2::new(pos.x + 1, pos.y - 1), SQRT_2),
        (IVec2::new(pos.x - 1, pos.y - 1), SQRT_2),
    ]
}

/// Internal A* node.
#[derive(Clone, PartialEq)]
struct AStarNode {
    pos: IVec2,
    /// Cost from start to this node.
    g: f32,
    /// Estimated total cost (g + h).
    f: f32,
}

impl Eq for AStarNode {}

impl std::cmp::Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // BinaryHeap is a max-heap; we want smallest f first, so reverse.
        other
            .f
            .partial_cmp(&self.f)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl std::cmp::PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// CostProvider trait and its impls (HashSet<IVec2>, PathfindingGrid) are
// now defined in kurulus_core::resources::pathfinding so that the flow
// field module can also use them without circular dependencies.

/// Run A* search on the grid with a generic cost provider.
///
/// Returns an ordered list of grid positions from `start` to `goal`
/// (inclusive), or `None` if no path exists.
pub fn a_star<C: CostProvider>(
    start: IVec2,
    goal: IVec2,
    costs: &C,
    grid_bounds: Option<(IVec2, IVec2)>, // (min, max) inclusive
) -> Option<Vec<IVec2>> {
    if start == goal {
        return Some(vec![start]);
    }

    if costs.cost(goal) >= f32::MAX {
        return None; // goal is impassable
    }

    let mut open = BinaryHeap::new();
    let mut g_scores: HashMap<IVec2, f32> = HashMap::new();
    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();

    let h = heuristic(start, goal);
    open.push(AStarNode {
        pos: start,
        g: 0.0,
        f: h,
    });
    g_scores.insert(start, 0.0);

    // Safety cap: don't search more than 10,000 cells.
    let max_iterations = 10_000;
    let mut iterations = 0;

    while let Some(current) = open.pop() {
        iterations += 1;
        if iterations > max_iterations {
            return None;
        }

        if current.pos == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut p = goal;
            loop {
                path.push(p);
                if p == start {
                    break;
                }
                p = came_from[&p];
            }
            path.reverse();
            return Some(path);
        }

        // Skip if we already found a better path to this node
        if let Some(&best_g) = g_scores.get(&current.pos) {
            if current.g > best_g {
                continue;
            }
        }

        for (neighbour, edge_cost) in neighbours(current.pos) {
            // Bounds check
            if let Some((min, max)) = grid_bounds {
                if neighbour.x < min.x
                    || neighbour.y < min.y
                    || neighbour.x > max.x
                    || neighbour.y > max.y
                {
                    continue;
                }
            }

            // Terrain cost to enter the neighbour cell
            let terrain_cost = costs.cost(neighbour);
            if terrain_cost >= f32::MAX {
                continue; // impassable
            }

            // Effective cost = edge distance × terrain multiplier
            let tentative_g = current.g + edge_cost * terrain_cost;

            if tentative_g < g_scores.get(&neighbour).copied().unwrap_or(f32::MAX) {
                came_from.insert(neighbour, current.pos);
                g_scores.insert(neighbour, tentative_g);
                let f = tentative_g + heuristic(neighbour, goal);
                open.push(AStarNode {
                    pos: neighbour,
                    g: tentative_g,
                    f,
                });
            }
        }
    }

    None // no path found
}

// ─── Legacy API (backward compatible) ──────────────────────────────────────

/// Forward-compatible wrapper: A* with `PathfindingGrid` cost provider.
#[allow(dead_code)]
pub fn a_star_with_terrain(
    start: IVec2,
    goal: IVec2,
    grid: &PathfindingGrid,
    bounds: Option<(IVec2, IVec2)>,
) -> Option<Vec<IVec2>> {
    a_star(start, goal, grid, bounds)
}

// ─── Path recalculation ─────────────────────────────────────────────────────

/// Returns `true` if the next waypoint (or any intermediate cell) is blocked
/// by an obstacle or impassable terrain, meaning the unit should recalculate.
pub fn is_path_blocked(path: &[GridPos], current_index: usize, grid: &PathfindingGrid) -> bool {
    for i in current_index..path.len() {
        if !grid.is_passable(path[i].0) {
            return true;
        }
    }
    false
}

/// Returns `true` if the path ahead (starting at `current_index + 1`)
/// is blocked within `lookahead` waypoints.
#[allow(dead_code)]
pub fn is_ahead_blocked(
    path: &[GridPos],
    current_index: usize,
    grid: &PathfindingGrid,
    lookahead: usize,
) -> bool {
    let start = current_index + 1;
    let end = (start + lookahead).min(path.len());
    for i in start..end {
        if !grid.is_passable(path[i].0) {
            return true;
        }
    }
    false
}

/// Destroyed buildings no longer block movement.  Returns updated obstacle
/// set that excludes any positions that are now free.
#[allow(dead_code)]
pub fn update_obstacles_for_destruction(
    current_obstacles: &HashSet<IVec2>,
    destroyed_positions: &[IVec2],
) -> HashSet<IVec2> {
    let mut updated = current_obstacles.clone();
    for pos in destroyed_positions {
        updated.remove(pos);
    }
    updated
}

// ─── Obstacle map (existing API preserved) ─────────────────────────────────

/// Collect all obstacle grid positions from the ECS world.
/// Buildings and their 2×2 footprint block movement.
pub fn collect_obstacles(buildings: Query<&WorldPosition, With<Building>>) -> HashSet<IVec2> {
    let mut obs = HashSet::new();

    for pos in buildings.iter() {
        obs.insert(pos.grid.0);
        // Buildings occupy a 2×2 footprint by default
        obs.insert(IVec2::new(pos.grid.0.x + 1, pos.grid.0.y));
        obs.insert(IVec2::new(pos.grid.0.x, pos.grid.0.y + 1));
        obs.insert(IVec2::new(pos.grid.0.x + 1, pos.grid.0.y + 1));
    }

    obs
}

/// Obstacle map resource that is rebuilt each frame before pathfinding runs.
#[derive(Resource, Clone, Default)]
pub struct ObstacleMap {
    pub obstacles: HashSet<IVec2>,
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use kurulus_core::components::TerrainType;

    #[test]
    fn test_a_star_simple() {
        let obstacles = HashSet::new();
        let path = a_star(IVec2::new(0, 0), IVec2::new(2, 0), &obstacles, None);
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&IVec2::new(0, 0)));
        assert_eq!(path.last(), Some(&IVec2::new(2, 0)));
    }

    #[test]
    fn test_a_star_blocked_goal() {
        let mut obstacles = HashSet::new();
        obstacles.insert(IVec2::new(2, 0));
        let path = a_star(IVec2::new(0, 0), IVec2::new(2, 0), &obstacles, None);
        assert!(path.is_none(), "Should return None when goal is blocked");
    }

    #[test]
    fn test_a_star_no_path() {
        let mut obstacles = HashSet::new();
        // Block the entire column between start and goal
        obstacles.insert(IVec2::new(1, 0));
        obstacles.insert(IVec2::new(1, 1));
        obstacles.insert(IVec2::new(1, -1));
        obstacles.insert(IVec2::new(1, -2));
        obstacles.insert(IVec2::new(1, 2));
        let path = a_star(
            IVec2::new(0, 0),
            IVec2::new(2, 0),
            &obstacles,
            Some((IVec2::splat(-100), IVec2::splat(100))),
        );
        assert!(
            path.is_some(),
            "Should find an alternate route around the wall"
        );
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&IVec2::new(0, 0)));
        assert_eq!(path.last(), Some(&IVec2::new(2, 0)));
    }

    #[test]
    fn test_a_star_with_terrain_prefers_road() {
        let mut terrain = TerrainGrid::default();
        // Two routes from (0,0) to (2,0):
        // Route A: direct via (1,0) — road (0.8 cost)
        // Route B: via (0,1)->(1,1)->(2,1) — forest (1.5 cost)
        terrain.set(IVec2::new(0, 0), TerrainType::OpenField);
        terrain.set(IVec2::new(1, 0), TerrainType::Road);
        terrain.set(IVec2::new(2, 0), TerrainType::OpenField);
        terrain.set(IVec2::new(0, 1), TerrainType::Forest);
        terrain.set(IVec2::new(1, 1), TerrainType::Forest);
        terrain.set(IVec2::new(2, 1), TerrainType::Forest);

        let grid = PathfindingGrid::build(&terrain, &HashSet::new());
        let path = a_star(IVec2::new(0, 0), IVec2::new(2, 0), &grid, None);
        assert!(path.is_some());
        let path = path.unwrap();

        // The optimal path should go through (1,0) — the road cell
        assert!(
            path.contains(&IVec2::new(1, 0)),
            "A* with terrain should prefer the road cell (1,0): {:?}",
            path
        );
    }

    #[test]
    fn test_is_path_blocked_detects_obstacle() {
        let mut grid = PathfindingGrid::default();
        grid.blocked.insert(IVec2::new(1, 0));

        let path = vec![
            GridPos(IVec2::new(0, 0)),
            GridPos(IVec2::new(1, 0)),
            GridPos(IVec2::new(2, 0)),
        ];
        assert!(is_path_blocked(&path, 0, &grid));
        assert!(is_path_blocked(&path, 1, &grid));
        // After passing the blocked cell, no longer blocked
        assert!(!is_path_blocked(&path, 2, &grid));
    }

    #[test]
    fn test_is_path_blocked_no_obstacle() {
        let grid = PathfindingGrid::default();
        let path = vec![
            GridPos(IVec2::new(0, 0)),
            GridPos(IVec2::new(1, 0)),
            GridPos(IVec2::new(2, 0)),
        ];
        assert!(!is_path_blocked(&path, 0, &grid));
    }

    #[test]
    fn test_is_ahead_blocked() {
        let mut grid = PathfindingGrid::default();
        grid.blocked.insert(IVec2::new(2, 0));

        let path = vec![
            GridPos(IVec2::new(0, 0)),
            GridPos(IVec2::new(1, 0)),
            GridPos(IVec2::new(2, 0)),
            GridPos(IVec2::new(3, 0)),
        ];

        // Lookahead 1 from index 0 → checks (1,0) → not blocked
        assert!(!is_ahead_blocked(&path, 0, &grid, 1));
        // Lookahead 2 from index 0 → checks (1,0), (2,0) → blocked!
        assert!(is_ahead_blocked(&path, 0, &grid, 2));
        // Lookahead 1 from index 1 → checks (2,0) → blocked!
        assert!(is_ahead_blocked(&path, 1, &grid, 1));
        // Past the blockage
        assert!(!is_ahead_blocked(&path, 2, &grid, 1));
    }
}
