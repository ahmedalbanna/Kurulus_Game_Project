use bevy::prelude::*;
use kurulus_core::prelude::*;
use std::collections::{BinaryHeap, HashSet, HashMap};

/// Heuristic cost estimate (octile distance — allows 8-directional movement).
fn heuristic(a: IVec2, b: IVec2) -> f32 {
    let dx = (a.x - b.x).abs() as f32;
    let dy = (a.y - b.y).abs() as f32;
    // Octile distance: accounts for sqrt(2) diagonal cost
    let d = dx.max(dy);
    let s = dx.min(dy);
    (d - s) + s * 1.41421356
}

/// 8-directional neighbours of a grid cell.
fn neighbours(pos: IVec2) -> [(IVec2, f32); 8] {
    [
        (IVec2::new(pos.x + 1, pos.y), 1.0),
        (IVec2::new(pos.x - 1, pos.y), 1.0),
        (IVec2::new(pos.x, pos.y + 1), 1.0),
        (IVec2::new(pos.x, pos.y - 1), 1.0),
        (IVec2::new(pos.x + 1, pos.y + 1), 1.41421356),
        (IVec2::new(pos.x - 1, pos.y + 1), 1.41421356),
        (IVec2::new(pos.x + 1, pos.y - 1), 1.41421356),
        (IVec2::new(pos.x - 1, pos.y - 1), 1.41421356),
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
        other.f.partial_cmp(&self.f).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl std::cmp::PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Run A* search on the grid.
///
/// Returns an ordered list of grid positions from `start` to `goal`
/// (inclusive), or `None` if no path exists.
pub fn a_star(
    start: IVec2,
    goal: IVec2,
    obstacles: &HashSet<IVec2>,
    grid_bounds: Option<(IVec2, IVec2)>, // (min, max) inclusive
) -> Option<Vec<IVec2>> {
    if start == goal {
        return Some(vec![start]);
    }

    if obstacles.contains(&goal) {
        return None; // goal is blocked
    }

    let mut open = BinaryHeap::new();
    let mut g_scores: HashMap<IVec2, f32> = HashMap::new();
    let mut came_from: HashMap<IVec2, IVec2> = HashMap::new();

    let h = heuristic(start, goal);
    open.push(AStarNode { pos: start, g: 0.0, f: h });
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
                if neighbour.x < min.x || neighbour.y < min.y
                    || neighbour.x > max.x || neighbour.y > max.y
                {
                    continue;
                }
            }

            if obstacles.contains(&neighbour) {
                continue;
            }

            let tentative_g = current.g + edge_cost;

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

/// Collect all obstacle grid positions from the ECS world.
/// Buildings and their 2×2 footprint block movement.
pub fn collect_obstacles(
    buildings: Query<&WorldPosition, With<Building>>,
) -> HashSet<IVec2> {
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
