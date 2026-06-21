use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Logical grid position (shared 2D/3D) — used for pathfinding, LOS, building placement
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos(pub IVec2);

/// Continuous world position (2D render)
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WorldPos2D(pub Vec2);

/// Continuous world position (3D render) — includes height
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WorldPos3D(pub Vec3);

/// Unified position — all systems read/write this; render systems project to 2D/3D
#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WorldPosition {
    pub grid: GridPos,
    pub world_2d: WorldPos2D,
    pub world_3d: WorldPos3D,
}

impl WorldPosition {
    pub fn new(grid: IVec2, cell_size: f32) -> Self {
        let world_2d = Vec2::new(grid.x as f32 * cell_size, grid.y as f32 * cell_size);
        let world_3d = Vec3::new(world_2d.x, 0.0, world_2d.y);
        Self {
            grid: GridPos(grid),
            world_2d: WorldPos2D(world_2d),
            world_3d: WorldPos3D(world_3d),
        }
    }

    /// Update continuous position from grid (snapping)
    pub fn snap_to_grid(&mut self, cell_size: f32) {
        self.world_2d .0 = Vec2::new(
            self.grid.0.x as f32 * cell_size,
            self.grid.0.y as f32 * cell_size,
        );
        self.world_3d .0 = Vec3::new(self.world_2d .0.x, self.world_3d .0.y, self.world_2d .0.y);
    }

    /// Update grid from continuous position (for free-form movement)
    pub fn quantize(&mut self, cell_size: f32) {
        self.grid.0 = IVec2::new(
            (self.world_2d .0.x / cell_size).round() as i32,
            (self.world_2d .0.y / cell_size).round() as i32,
        );
    }
}

impl Default for WorldPosition {
    fn default() -> Self {
        Self {
            grid: GridPos(IVec2::ZERO),
            world_2d: WorldPos2D(Vec2::ZERO),
            world_3d: WorldPos3D(Vec3::ZERO),
        }
    }
}

/// Cell size constant (matches tilemap/navmesh)
pub const CELL_SIZE: f32 = 1.0; // 1 world unit = 1 meter

/// Height levels for 2D LOS blocking (maps to 3D elevation)
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeightLevel {
    Ground = 0,
    Wall = 1,
    Tower = 2,
    Mountain = 3,
}
