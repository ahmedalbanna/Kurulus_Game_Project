use crate::coord::GridPos;
use bevy::prelude::*;
use std::collections::HashMap;

/// A grid-based spatial hash for efficient neighbour queries.
///
/// Rather than O(n²) nested loops, systems can call `entities_in_range()` to
/// retrieve only entities in nearby cells, dramatically reducing iteration
/// overhead for large unit counts.
///
/// The grid is **fully rebuilt** every frame by the `SpatialPlugin` update
/// system, ensuring no stale data lingers.
#[derive(Resource, Debug, Reflect)]
pub struct SpatialGrid {
    /// Size of each cell in grid units (e.g. 10 means 10×10-tile buckets).
    pub cell_size: i32,
    /// Internal map: cell key (IVec2) → entities in that cell.
    cells: HashMap<IVec2, Vec<Entity>>,
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self {
            cell_size: 10,
            cells: HashMap::new(),
        }
    }
}

impl SpatialGrid {
    /// Create a new grid with the given cell size (in grid units).
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size: cell_size.max(1),
            cells: HashMap::new(),
        }
    }

    /// Compute the cell key for a grid position.
    fn cell_key(&self, grid_pos: IVec2) -> IVec2 {
        IVec2::new(
            grid_pos.x.div_euclid(self.cell_size),
            grid_pos.y.div_euclid(self.cell_size),
        )
    }

    /// Insert an entity at the given grid position.
    pub fn insert(&mut self, entity: Entity, grid_pos: IVec2) {
        let key = self.cell_key(grid_pos);
        self.cells.entry(key).or_default().push(entity);
    }

    /// Remove all entities from the grid (full rebuild).
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    /// Return all entities that could be within `range` of `center`.
    ///
    /// This returns entities from all cells that intersect the range circle's
    /// bounding square. Callers should still filter by exact distance.
    pub fn entities_in_range(&self, center: GridPos, range: f32) -> Vec<Entity> {
        let center_key = self.cell_key(center.0);
        let cell_radius = (range / self.cell_size as f32).ceil() as i32;

        let mut result = Vec::new();
        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let key = IVec2::new(center_key.x + dx, center_key.y + dy);
                if let Some(entries) = self.cells.get(&key) {
                    result.extend(entries);
                }
            }
        }
        result
    }

    /// Return all entities in the same cell as `center`.
    pub fn entities_in_same_cell(&self, center: GridPos) -> Vec<Entity> {
        let key = self.cell_key(center.0);
        self.cells.get(&key).cloned().unwrap_or_default()
    }

    /// Number of cells currently occupied.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Number of entities currently tracked.
    pub fn entity_count(&self) -> usize {
        self.cells.values().map(|v| v.len()).sum()
    }
}
