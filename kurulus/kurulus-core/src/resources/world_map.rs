use bevy::prelude::*;
use bevy::math::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::components::{
    Encounter, EncounterChoice, EncounterDifficulty, EncounterOutcome, EncounterType,
    FactionId, FogState, PointOfInterest, PoiId, RegionDef, RegionId, WorldTerrain,
    WorldTile,
};

/// Global world map state — the strategic overworld layer.
/// Stores the tile grid, region definitions, player position, and fog of war.
#[derive(Resource, Reflect, Clone, Debug, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct WorldMap {
    /// Width of the world map in tiles
    pub width: u32,
    /// Height of the world map in tiles
    pub height: u32,
    /// 2D grid of world map tiles, row-major: tiles[y * width + x]
    pub tiles: Vec<WorldTile>,
    /// Region definitions with ownership, bounds, and properties
    pub regions: Vec<RegionDef>,
    /// Current player position on the world map (tile coords)
    pub player_position: IVec2,
    /// Current region the player is in (updated each frame)
    pub current_region: Option<RegionId>,
    /// Map of tile position -> faction's visible tiles for quick lookup
    pub explored_cache: HashMap<IVec2, FogState>,
    /// All points of interest on the world map
    pub points_of_interest: Vec<PointOfInterest>,
    /// Encounter table — weighted list of possible encounter types per terrain
    pub encounter_table: Vec<EncounterTemplate>,
    /// RNG state for deterministic encounters (frame count based)
    pub encounter_seed: u64,
    /// Track which POI IDs have been discovered (for event dedup)
    pub discovered_pois: Vec<PoiId>,
}

impl Default for WorldMap {
    fn default() -> Self {
        Self {
            width: 48,
            height: 36,
            tiles: Vec::new(),
            regions: Vec::new(),
            player_position: IVec2::ZERO,
            current_region: None,
            explored_cache: HashMap::new(),
            points_of_interest: Vec::new(),
            encounter_table: Vec::new(),
            encounter_seed: 0,
            discovered_pois: Vec::new(),
        }
    }
}

impl WorldMap {
    /// Initialise the world map with flat terrain and unexplored fog.
    pub fn init(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let count = (width * height) as usize;
        self.tiles = Vec::with_capacity(count);
        for y in 0..height {
            for x in 0..width {
                self.tiles.push(WorldTile {
                    position: IVec2::new(x as i32, y as i32),
                    terrain: WorldTerrain::Grassland,
                    fog: FogState::Unexplored,
                    region: None,
                    elevation: 0.0,
                });
            }
        }
    }

    /// Get a tile reference at the given position.
    pub fn get_tile(&self, pos: IVec2) -> Option<&WorldTile> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i32 || pos.y >= self.height as i32 {
            return None;
        }
        let idx = (pos.y as u32 * self.width + pos.x as u32) as usize;
        self.tiles.get(idx)
    }

    /// Get a mutable tile reference at the given position.
    pub fn get_tile_mut(&mut self, pos: IVec2) -> Option<&mut WorldTile> {
        if pos.x < 0 || pos.y < 0 || pos.x >= self.width as i32 || pos.y >= self.height as i32 {
            return None;
        }
        let idx = (pos.y as u32 * self.width + pos.x as u32) as usize;
        self.tiles.get_mut(idx)
    }

    /// Reveal a tile (set to Visible) and mark it explored.
    pub fn reveal_tile(&mut self, pos: IVec2) {
        if let Some(tile) = self.get_tile_mut(pos) {
            tile.fog = FogState::Visible;
            self.explored_cache.insert(pos, FogState::Visible);
        }
    }

    /// Explore a tile (set to Explored, not currently visible).
    pub fn explore_tile(&mut self, pos: IVec2) {
        if let Some(tile) = self.get_tile_mut(pos) {
            if tile.fog == FogState::Unexplored {
                tile.fog = FogState::Explored;
                self.explored_cache.insert(pos, FogState::Explored);
            }
        }
    }

    /// Reset all Visible tiles back to Explored (end of visibility update).
    pub fn dim_visible_tiles(&mut self) {
        for tile in self.tiles.iter_mut() {
            if tile.fog == FogState::Visible {
                tile.fog = FogState::Explored;
            }
        }
    }

    /// Get the fog state for a tile position.
    pub fn fog_at(&self, pos: IVec2) -> FogState {
        self.get_tile(pos)
            .map(|t| t.fog)
            .unwrap_or(FogState::Unexplored)
    }

    /// Get all tiles whose fog matches the given state.
    pub fn tiles_with_fog(&self, state: FogState) -> Vec<&WorldTile> {
        self.tiles.iter().filter(|t| t.fog == state).collect()
    }

    /// Determine which region a tile position belongs to.
    pub fn region_at(&self, pos: IVec2) -> Option<RegionId> {
        self.get_tile(pos).and_then(|t| t.region)
    }

    /// Get a region definition by ID.
    pub fn get_region(&self, region_id: RegionId) -> Option<&RegionDef> {
        self.regions.iter().find(|r| r.region_id == region_id)
    }

    /// Get a mutable region definition by ID.
    pub fn get_region_mut(&mut self, region_id: RegionId) -> Option<&mut RegionDef> {
        self.regions.iter_mut().find(|r| r.region_id == region_id)
    }

    /// Set the owner of a region.
    pub fn set_region_owner(&mut self, region_id: RegionId, owner: FactionId) {
        if let Some(region) = self.get_region_mut(region_id) {
            region.owner = owner;
            // Tiles track region implicitly; no per-tile owner change needed
        }
    }

    /// Find all tiles within a given radius of a position (Bresenham circle).
    pub fn tiles_in_radius(&self, center: IVec2, radius: f32) -> Vec<IVec2> {
        let r = radius.ceil() as i32;
        let mut result = Vec::new();
        for dy in -r..=r {
            for dx in -r..=r {
                let pos = IVec2::new(center.x + dx, center.y + dy);
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius && self.is_valid_pos(pos) {
                    result.push(pos);
                }
            }
        }
        result
    }

    /// Check if a position is within the world map bounds.
    pub fn is_valid_pos(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && pos.x < self.width as i32 && pos.y < self.height as i32
    }

    /// Calculate travel duration between two points based on terrain and distance.
    pub fn travel_duration(&self, from: IVec2, to: IVec2, speed: f32) -> f32 {
        let dx = (to.x - from.x) as f32;
        let dy = (to.y - from.y) as f32;
        let distance = (dx * dx + dy * dy).sqrt();
        // Average terrain cost along the line
        let terrain_cost = self.average_terrain_cost(from, to);
        distance * terrain_cost / speed.max(0.1)
    }

    /// Average terrain cost along a straight line between two points.
    fn average_terrain_cost(&self, from: IVec2, to: IVec2) -> f32 {
        let steps = 8.max((from.x - to.x).abs() + (from.y - to.y).abs());
        let mut total_cost = 0.0_f32;
        let mut count = 0;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = from.x as f32 + (to.x - from.x) as f32 * t;
            let y = from.y as f32 + (to.y - from.y) as f32 * t;
            let pos = IVec2::new(x.round() as i32, y.round() as i32);
            if let Some(tile) = self.get_tile(pos) {
                total_cost += tile.terrain.travel_cost();
                count += 1;
            }
        }
        if count == 0 { 1.0 } else { total_cost / count as f32 }
    }

    // ── Points of Interest ────────────────────────────────────────────

    /// Get all POIs that are currently undiscovered.
    pub fn undiscovered_pois(&self) -> Vec<&PointOfInterest> {
        self.points_of_interest.iter().filter(|poi| !poi.discovered).collect()
    }

    /// Get all POIs that have been discovered.
    pub fn discovered_pois_list(&self) -> Vec<&PointOfInterest> {
        self.points_of_interest.iter().filter(|poi| poi.discovered).collect()
    }

    /// Get a POI by ID.
    pub fn get_poi(&self, poi_id: PoiId) -> Option<&PointOfInterest> {
        self.points_of_interest.iter().find(|poi| poi.poi_id == poi_id)
    }

    /// Get a mutable POI by ID.
    pub fn get_poi_mut(&mut self, poi_id: PoiId) -> Option<&mut PointOfInterest> {
        self.points_of_interest.iter_mut().find(|poi| poi.poi_id == poi_id)
    }

    /// Mark a POI as discovered.
    pub fn discover_poi(&mut self, poi_id: PoiId) -> bool {
        if self.discovered_pois.contains(&poi_id) {
            return false;
        }
        if let Some(poi) = self.get_poi_mut(poi_id) {
            poi.discovered = true;
            self.discovered_pois.push(poi_id);
            true
        } else {
            false
        }
    }

    /// Find a POI at a given tile position.
    pub fn poi_at(&self, pos: IVec2) -> Option<&PointOfInterest> {
        self.points_of_interest.iter().find(|poi| poi.position == pos)
    }

    /// Find all POIs within a given radius of a position.
    pub fn pois_in_radius(&self, center: IVec2, radius: f32) -> Vec<&PointOfInterest> {
        self.points_of_interest.iter().filter(|poi| {
            let dx = (poi.position.x - center.x) as f32;
            let dy = (poi.position.y - center.y) as f32;
            (dx * dx + dy * dy).sqrt() <= radius
        }).collect()
    }

    /// Find all undiscovered POIs within a radius.
    pub fn undiscovered_pois_in_radius(&self, center: IVec2, radius: f32) -> Vec<&PointOfInterest> {
        self.pois_in_radius(center, radius)
            .into_iter()
            .filter(|poi| !poi.discovered)
            .collect()
    }

    // ── Encounters ─────────────────────────────────────────────────────

    /// Roll for a random encounter given current terrain and a luck roll (0.0–1.0).
    /// Returns `None` if no encounter occurs.
    pub fn roll_encounter(&mut self, terrain: WorldTerrain, luck: f32) -> Option<Encounter> {
        // Encounters are rare — base 8% chance per check
        if luck > 0.08 {
            return None;
        }
        // Pick a random encounter template matching this terrain
        let candidates: Vec<&EncounterTemplate> = self.encounter_table
            .iter()
            .filter(|t| t.terrains.contains(&terrain))
            .collect();
        if candidates.is_empty() {
            return None;
        }
        let idx = (self.encounter_seed as usize) % candidates.len();
        self.encounter_seed = self.encounter_seed.wrapping_add(1);
        let tmpl = candidates[idx];
        Some(tmpl.generate(self.encounter_seed))
    }

    /// Get the tile position of a region's center (approximate centroid).
    pub fn region_center(&self, region_id: RegionId) -> Option<IVec2> {
        let region = self.get_region(region_id)?;
        if region.tiles.is_empty() {
            return None;
        }
        let avg_x = region.tiles.iter().map(|p| p.x).sum::<i32>() / region.tiles.len() as i32;
        let avg_y = region.tiles.iter().map(|p| p.y).sum::<i32>() / region.tiles.len() as i32;
        Some(IVec2::new(avg_x, avg_y))
    }
}

/// Template for generating random encounters on the world map.
#[derive(Reflect, Clone, Debug, Serialize, Deserialize)]
pub struct EncounterTemplate {
    pub encounter_type: EncounterType,
    pub description: String,
    pub difficulty: EncounterDifficulty,
    /// Which terrain types this encounter can occur on
    pub terrains: Vec<WorldTerrain>,
    /// Possible choices and their outcomes
    pub choices: Vec<(String, EncounterOutcome)>,
}

impl EncounterTemplate {
    /// Generate a concrete `Encounter` from this template.
    pub fn generate(&self, seed: u64) -> Encounter {
        let num_choices = self.choices.len().min(3);
        let mut shuffled: Vec<usize> = (0..self.choices.len()).collect();
        // Simple deterministic shuffle
        for i in (1..shuffled.len()).rev() {
            let j = (seed.wrapping_add(i as u64) as usize) % (i + 1);
            shuffled.swap(i, j);
        }
        let chosen: Vec<EncounterChoice> = shuffled[..num_choices]
            .iter()
            .map(|&i| {
                let (text, outcome) = &self.choices[i];
                EncounterChoice {
                    text: text.clone(),
                    outcome: outcome.clone(),
                }
            })
            .collect();
        Encounter {
            encounter_type: self.encounter_type,
            description: self.description.clone(),
            difficulty: self.difficulty,
            choices: chosen,
            faction: None,
            position: IVec2::ZERO,
        }
    }
}
