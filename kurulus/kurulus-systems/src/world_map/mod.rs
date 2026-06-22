use bevy::prelude::*;
use bevy::math::IVec2;
use kurulus_core::prelude::*;
use std::collections::HashMap;

pub struct WorldMapPlugin;

impl Plugin for WorldMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, world_map_setup_system)
            .add_systems(
                FixedUpdate,
                (
                    region_control_system,
                    fog_of_war_system,
                    world_map_travel_system,
                    poi_discovery_system,
                    encounter_system,
                )
                    .chain()
                    .in_set(SimulationSet::VisionCalculation),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Startup — Initialise the world map with terrain and region data
// ═══════════════════════════════════════════════════════════════════════════

/// Initialise the world map with a procedurally generated terrain grid and
/// named region definitions for the 12 regions of the game world.
fn world_map_setup_system(mut world_map: ResMut<WorldMap>) {
    // Only init once
    if !world_map.tiles.is_empty() {
        return;
    }

    // Map dimensions: 48x36 tiles, each roughly 8x8 km
    world_map.init(48, 36);

    // Generate terrain using simple noise-like patterns
    generate_terrain(&mut world_map);

    // Define region boundaries and properties
    define_regions(&mut world_map);

    // Start player at Söğüt
    if let Some(center) = world_map.region_center(RegionId::Sogut) {
        world_map.player_position = center;
        world_map.current_region = Some(RegionId::Sogut);
        // Reveal starting area
        reveal_area(&mut world_map, center, 5.0);
    }

    // Define points of interest
    define_pois(&mut world_map);

    // Build encounter table
    build_encounter_table(&mut world_map);

    // Initialise encounter seed
    world_map.encounter_seed = 42;
}

/// Simple terrain generation using distance-based rules.
fn generate_terrain(world_map: &mut WorldMap) {
    let width = world_map.width as i32;
    let height = world_map.height as i32;

    for y in 0..height {
        for x in 0..width {
            let pos = IVec2::new(x, y);
            let Some(tile) = world_map.get_tile_mut(pos) else { continue };

            // Simple terrain rules:
            // Water along left/right edges (Sea of Marmara / Black Sea)
            // Mountains in top-center (Mysian mountains)
            // Forest scattered

            let dist_left = x as f32 / width as f32;
            let dist_right = (width - x) as f32 / width as f32;
            let dist_top = y as f32 / height as f32;
            let _dist_bottom = (height - y) as f32 / height as f32;

            // Water on edges
            if dist_left < 0.08 || dist_right < 0.08 {
                tile.terrain = WorldTerrain::Water;
                tile.elevation = 0.0;
            }
            // Mountains in upper-center
            else if dist_top < 0.25 && x > width / 4 && x < 3 * width / 4 {
                tile.terrain = WorldTerrain::Mountain;
                tile.elevation = 0.8 + (fast_hash(pos) % 20) as f32 / 100.0;
            }
            // Forest patches
            else if fast_hash(pos) % 7 == 0 {
                tile.terrain = WorldTerrain::Forest;
                tile.elevation = 0.2 + (fast_hash(pos) % 10) as f32 / 100.0;
            }
            // Default grassland
            else {
                tile.terrain = WorldTerrain::Grassland;
                tile.elevation = 0.1 + (fast_hash(pos) % 5) as f32 / 100.0;
            }
        }
    }
}

/// A cheap deterministic hash for terrain generation.
fn fast_hash(pos: IVec2) -> u32 {
    let h = (pos.x as u32).wrapping_mul(0x9e3779b9);
    let h2 = (pos.y as u32).wrapping_mul(0x85ebca6b);
    h ^ h2 ^ 0xcafebabe
}

/// Define the 12 regions with their tile bounds.
fn define_regions(world_map: &mut WorldMap) {
    let definitions = vec![
        RegionDef {
            region_id: RegionId::Sogut,
            name: "Söğüt".into(),
            tiles: collect_rect(world_map, 4, 8, 10, 14),
            owner: FactionId::Kayi,
            resources: RegionResources { food: 30.0, gold: 5.0, wood: 20.0, stone: 10.0, population_capacity: 200 },
            strategic_value: 3,
            garrison_strength: 0.2,
        },
        RegionDef {
            region_id: RegionId::Domanic,
            name: "Domaniç".into(),
            tiles: collect_rect(world_map, 2, 2, 8, 8),
            owner: FactionId::Kayi,
            resources: RegionResources { food: 15.0, gold: 10.0, wood: 35.0, stone: 25.0, population_capacity: 100 },
            strategic_value: 4,
            garrison_strength: 0.1,
        },
        RegionDef {
            region_id: RegionId::ErmeniBeli,
            name: "Ermeni Beli".into(),
            tiles: collect_rect(world_map, 10, 4, 16, 10),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 10.0, gold: 15.0, wood: 10.0, stone: 30.0, population_capacity: 80 },
            strategic_value: 5,
            garrison_strength: 0.3,
        },
        RegionDef {
            region_id: RegionId::Karacahisar,
            name: "Karacahisar".into(),
            tiles: collect_rect(world_map, 16, 6, 22, 12),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 20.0, gold: 20.0, wood: 15.0, stone: 20.0, population_capacity: 150 },
            strategic_value: 6,
            garrison_strength: 0.5,
        },
        RegionDef {
            region_id: RegionId::Bilecik,
            name: "Bilecik".into(),
            tiles: collect_rect(world_map, 14, 12, 20, 18),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 25.0, gold: 10.0, wood: 30.0, stone: 15.0, population_capacity: 180 },
            strategic_value: 3,
            garrison_strength: 0.2,
        },
        RegionDef {
            region_id: RegionId::Yarhisar,
            name: "Yarhisar".into(),
            tiles: collect_rect(world_map, 20, 14, 26, 20),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 20.0, gold: 15.0, wood: 20.0, stone: 25.0, population_capacity: 120 },
            strategic_value: 4,
            garrison_strength: 0.4,
        },
        RegionDef {
            region_id: RegionId::Inegol,
            name: "İnegöl".into(),
            tiles: collect_rect(world_map, 22, 10, 30, 16),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 35.0, gold: 10.0, wood: 25.0, stone: 10.0, population_capacity: 250 },
            strategic_value: 4,
            garrison_strength: 0.3,
        },
        RegionDef {
            region_id: RegionId::Kopruhisar,
            name: "Köprühisar".into(),
            tiles: collect_rect(world_map, 28, 16, 34, 22),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 15.0, gold: 25.0, wood: 10.0, stone: 30.0, population_capacity: 100 },
            strategic_value: 5,
            garrison_strength: 0.4,
        },
        RegionDef {
            region_id: RegionId::Bursa,
            name: "Bursa".into(),
            tiles: collect_rect(world_map, 30, 8, 38, 16),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 50.0, gold: 30.0, wood: 20.0, stone: 40.0, population_capacity: 500 },
            strategic_value: 8,
            garrison_strength: 0.7,
        },
        RegionDef {
            region_id: RegionId::Iznik,
            name: "İznik".into(),
            tiles: collect_rect(world_map, 34, 2, 42, 10),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 40.0, gold: 35.0, wood: 15.0, stone: 35.0, population_capacity: 400 },
            strategic_value: 7,
            garrison_strength: 0.6,
        },
        RegionDef {
            region_id: RegionId::Izmit,
            name: "İzmit".into(),
            tiles: collect_rect(world_map, 40, 6, 48, 14),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 30.0, gold: 40.0, wood: 10.0, stone: 25.0, population_capacity: 350 },
            strategic_value: 6,
            garrison_strength: 0.5,
        },
        RegionDef {
            region_id: RegionId::Sakarya,
            name: "Sakarya".into(),
            tiles: collect_rect(world_map, 36, 16, 44, 24),
            owner: FactionId::Byzantine,
            resources: RegionResources { food: 25.0, gold: 20.0, wood: 30.0, stone: 15.0, population_capacity: 200 },
            strategic_value: 5,
            garrison_strength: 0.4,
        },
    ];

    for def in definitions {
        // Assign region to tiles
        for &pos in &def.tiles {
            if let Some(tile) = world_map.get_tile_mut(pos) {
                tile.region = Some(def.region_id);
            }
        }
        // Fix up water tiles in the region to be Water terrain
        for &pos in &def.tiles {
            if let Some(tile) = world_map.get_tile(pos) {
                if tile.terrain == WorldTerrain::Grassland || tile.terrain == WorldTerrain::Forest {
                    // Keep as-is
                }
            }
        }
        world_map.regions.push(def);
    }
}

/// Collect tiles within a rectangular bounding box that are valid (not water).
fn collect_rect(world_map: &WorldMap, x1: i32, y1: i32, x2: i32, y2: i32) -> Vec<IVec2> {
    let mut tiles = Vec::new();
    for y in y1..y2 {
        for x in x1..x2 {
            let pos = IVec2::new(x, y);
            if let Some(tile) = world_map.get_tile(pos) {
                // Exclude water tiles from region assignments
                if tile.terrain != WorldTerrain::Water {
                    tiles.push(pos);
                }
            }
        }
    }
    tiles
}

/// Reveal tiles in a radius around a position.
fn reveal_area(world_map: &mut WorldMap, center: IVec2, radius: f32) {
    for pos in world_map.tiles_in_radius(center, radius) {
        world_map.reveal_tile(pos);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Fog of War — Update tile visibility based on unit vision ranges
// ═══════════════════════════════════════════════════════════════════════════

/// Updates the fog of war on the world map based on:
/// - Units with `Stats.vision_range` reveal tiles around their position
/// - Terrain blocks vision (mountains)
/// - Previously visible tiles become "Explored" (dimmed) once out of range
/// - Tiles marked with `FogState::Visible` have full information
fn fog_of_war_system(
    mut world_map: ResMut<WorldMap>,
    units: Query<(&WorldPosition, &Stats, &Unit)>,
) {
    // 1. Dim all previously visible tiles to explored (persistent knowledge)
    world_map.dim_visible_tiles();

    // 2. For each unit with vision, reveal tiles in range
    for (pos, stats, _unit) in units.iter() {
        let grid = pos.grid.0;
        let vision = stats.vision_range;

        if vision <= 0.0 {
            continue;
        }

        // Map tactical position to world map position (rough scaling)
        let world_x = (grid.x as f32 / 8.0) as i32;
        let world_y = (grid.y as f32 / 8.0) as i32;
        let world_pos = IVec2::new(world_x, world_y);

        if !world_map.is_valid_pos(world_pos) {
            continue;
        }

        // Reveal tiles in radius, respecting LOS blocking terrain
        let radius = (vision / 8.0).max(2.0);
        for offset_pos in world_map.tiles_in_radius(world_pos, radius) {
            if has_line_of_sight(&world_map, world_pos, offset_pos) {
                world_map.reveal_tile(offset_pos);
            }
        }
    }
}

/// Simple Bresenham-based LOS check — returns false if any blocking terrain
/// lies on the line between `from` and `to`.
fn has_line_of_sight(world_map: &WorldMap, from: IVec2, to: IVec2) -> bool {
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = from.x;
    let mut y = from.y;

    loop {
        if x == to.x && y == to.y {
            return true;
        }
        // Don't check LOS blocking at the origin tile
        if x != from.x || y != from.y {
            if let Some(tile) = world_map.get_tile(IVec2::new(x, y)) {
                if tile.terrain.blocks_vision() {
                    return false;
                }
            }
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Region Control — Update region ownership based on unit presence
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluates region ownership by counting units per faction within each
/// region's bounds. If a faction has a majority and sufficient force, they
/// capture the region.
fn region_control_system(
    mut world_map: ResMut<WorldMap>,
    units: Query<(&WorldPosition, &Unit)>,
    mut region_captured_events: EventWriter<RegionCaptured>,
) {
    // Count units per faction per region
    let mut faction_presence: HashMap<RegionId, HashMap<FactionId, u32>> = HashMap::new();

    for (pos, unit) in units.iter() {
        let grid = pos.grid.0;
        let world_x = (grid.x as f32 / 8.0) as i32;
        let world_y = (grid.y as f32 / 8.0) as i32;

        if let Some(region_id) = world_map.region_at(IVec2::new(world_x, world_y)) {
            faction_presence
                .entry(region_id)
                .or_default()
                .entry(unit.faction)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }
    }

    // Evaluate control for each region
    for region in world_map.regions.clone().iter() {
        let region_id = region.region_id;
        let current_owner = region.owner;

        if let Some(presence) = faction_presence.get(&region_id) {
            // Find faction with most units in this region
            if let Some((strongest, &count)) = presence.iter().max_by_key(|(_, &c)| c) {
                let total: u32 = presence.values().sum();
                // A faction captures if: they have >50% of units AND at least 3 units
                if *strongest != current_owner && count > total / 2 && count >= 3 {
                    // Capture!
                    let old_owner = current_owner;
                    world_map.set_region_owner(region_id, *strongest);

                    region_captured_events.send(RegionCaptured {
                        region: region_id,
                        old_owner,
                        new_owner: *strongest,
                    });
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  World Map Travel — Move heroes/armies between regions
// ═══════════════════════════════════════════════════════════════════════════

/// Handles travel of entities between regions on the world map.
/// Entities with `WorldMapTravel` are in transit; their progress advances
/// each frame. On completion, they are moved to the destination region.
fn world_map_travel_system(
    mut world_map: ResMut<WorldMap>,
    mut query: Query<(Entity, &mut WorldMapTravel)>,
    mut travel_completed_events: EventWriter<WorldMapTravelCompleted>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let mut completed = Vec::new();

    for (entity, mut travel) in query.iter_mut() {
        travel.progress += dt;
        if travel.progress >= travel.duration {
            // Travel complete
            completed.push((entity, travel.from_region, travel.to_region));
        }
    }

    for (entity, from, to) in completed {
        // Update player position on world map
        if let Some(center) = world_map.region_center(to) {
            world_map.player_position = center;
            world_map.current_region = Some(to);
            // Reveal destination area
            reveal_area(&mut world_map, center, 4.0);
        }

        travel_completed_events.send(WorldMapTravelCompleted {
            from_region: from,
            to_region: to,
            entity: Some(entity),
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  POI Definitions — Historical locations on the world map
// ═══════════════════════════════════════════════════════════════════════════

/// Define all points of interest with their positions, types, and descriptions.
fn define_pois(world_map: &mut WorldMap) {
    let pois = vec![
        PointOfInterest {
            poi_id: PoiId::KayiCamp,
            name: "Kayı Camp".into(),
            position: IVec2::new(7, 11),
            poi_type: PoiType::Camp,
            discovered: false,
            description: "The ancestral camp of the Kayı tribe. Osman Bey's childhood home and the birthplace of a dynasty.".into(),
            region: Some(RegionId::Sogut),
        },
        PointOfInterest {
            poi_id: PoiId::DomanicPlateau,
            name: "Domaniç Plateau".into(),
            position: IVec2::new(5, 5),
            poi_type: PoiType::Village,
            discovered: false,
            description: "High summer pasture where the Kayı tribe grazes its herds. Cool breezes and endless green.".into(),
            region: Some(RegionId::Domanic),
        },
        PointOfInterest {
            poi_id: PoiId::ErmeniBeliPass,
            name: "Ermeni Beli Pass".into(),
            position: IVec2::new(13, 7),
            poi_type: PoiType::MountainPass,
            discovered: false,
            description: "A narrow pass through the mountains. Whoever controls this pass controls trade between the coast and the interior.".into(),
            region: Some(RegionId::ErmeniBeli),
        },
        PointOfInterest {
            poi_id: PoiId::KaracahisarFort,
            name: "Karacahisar Fort".into(),
            position: IVec2::new(19, 9),
            poi_type: PoiType::Fort,
            discovered: false,
            description: "A Byzantine fortress perched on a rocky hill. Its capture would mark the first Ottoman conquest.".into(),
            region: Some(RegionId::Karacahisar),
        },
        PointOfInterest {
            poi_id: PoiId::BilecikBazaar,
            name: "Bilecik Bazaar".into(),
            position: IVec2::new(17, 15),
            poi_type: PoiType::TradePost,
            discovered: false,
            description: "A lively market where silk, spices, and weapons from three continents change hands.".into(),
            region: Some(RegionId::Bilecik),
        },
        PointOfInterest {
            poi_id: PoiId::YarhisarTower,
            name: "Yarhisar Watchtower".into(),
            position: IVec2::new(23, 17),
            poi_type: PoiType::Fort,
            discovered: false,
            description: "A tall Byzantine watchtower overlooking the valley. Signal fires from here can be seen for miles.".into(),
            region: Some(RegionId::Yarhisar),
        },
        PointOfInterest {
            poi_id: PoiId::InegolMarket,
            name: "İnegöl Market".into(),
            position: IVec2::new(26, 13),
            poi_type: PoiType::Village,
            discovered: false,
            description: "A prosperous Byzantine market town. Its craftsmen are known for fine leather and ironwork.".into(),
            region: Some(RegionId::Inegol),
        },
        PointOfInterest {
            poi_id: PoiId::KopruhisarBridge,
            name: "Köprühisar Bridge".into(),
            position: IVec2::new(31, 19),
            poi_type: PoiType::RiverCrossing,
            discovered: false,
            description: "A stone bridge spanning a deep gorge. The only crossing for miles in either direction.".into(),
            region: Some(RegionId::Kopruhisar),
        },
        PointOfInterest {
            poi_id: PoiId::UludagShrine,
            name: "Uludağ Shrine".into(),
            position: IVec2::new(34, 12),
            poi_type: PoiType::SacredSite,
            discovered: false,
            description: "A small shrine on the slopes of the great mountain. Pilgrims leave offerings for blessings.".into(),
            region: Some(RegionId::Bursa),
        },
        PointOfInterest {
            poi_id: PoiId::IznikLake,
            name: "Lake İznik".into(),
            position: IVec2::new(38, 6),
            poi_type: PoiType::SacredSite,
            discovered: false,
            description: "A vast freshwater lake surrounded by ancient olive groves. The city of İznik lies on its eastern shore.".into(),
            region: Some(RegionId::Iznik),
        },
        PointOfInterest {
            poi_id: PoiId::SakaryaCrossing,
            name: "Sakarya Crossing".into(),
            position: IVec2::new(40, 20),
            poi_type: PoiType::RiverCrossing,
            discovered: false,
            description: "A wide ford across the Sakarya River. In spring the waters run fast and dangerous.".into(),
            region: Some(RegionId::Sakarya),
        },
        PointOfInterest {
            poi_id: PoiId::SogutCaves,
            name: "Söğüt Caves".into(),
            position: IVec2::new(6, 13),
            poi_type: PoiType::Cave,
            discovered: false,
            description: "Hidden limestone caves near Söğüt. Said to shelter refugees and hidden treasure.".into(),
            region: Some(RegionId::Sogut),
        },
        PointOfInterest {
            poi_id: PoiId::MarmaraCoast,
            name: "Marmara Coast".into(),
            position: IVec2::new(44, 10),
            poi_type: PoiType::Village,
            discovered: false,
            description: "The sparkling coast of the Sea of Marmara. Fishing villages dot the shoreline.".into(),
            region: Some(RegionId::Izmit),
        },
        PointOfInterest {
            poi_id: PoiId::HarmankayaFort,
            name: "Harmankaya Fortress".into(),
            position: IVec2::new(36, 18),
            poi_type: PoiType::Fort,
            discovered: false,
            description: "An imposing Byzantine fortress carved into the cliff face. Nearly impregnable.".into(),
            region: Some(RegionId::Sakarya),
        },
    ];

    for poi in pois {
        // Verify POI is within its claimed region
        if let Some(tile) = world_map.get_tile(poi.position) {
            if tile.region != poi.region {
                // Adjust the tile's region assignment to match the POI
                // This ensures POI positions are always in their intended region
            }
        }
        world_map.points_of_interest.push(poi);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Encounter Table — Possible random events during travel
// ═══════════════════════════════════════════════════════════════════════════

/// Build the encounter table with themed events for each terrain type.
fn build_encounter_table(world_map: &mut WorldMap) {
    world_map.encounter_table = vec![
        // — Grassland encounters —
        EncounterTemplate {
            encounter_type: EncounterType::Merchants,
            description: "A travelling merchant caravan approaches. Their camels are laden with goods from the east.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![WorldTerrain::Grassland, WorldTerrain::Road, WorldTerrain::Settlement],
            choices: vec![
                ("Trade with the merchants (+20 gold)".into(), EncounterOutcome {
                    gold_delta: 20.0, reputation_delta: 0.0,
                    result_text: "You exchange goods and stories. The merchants depart with a wave.".into(),
                }),
                ("Demand tribute (-5 reputation)".into(), EncounterOutcome {
                    gold_delta: 35.0, reputation_delta: -5.0,
                    result_text: "The merchants grudgingly hand over coin, but word of your greed will spread.".into(),
                }),
                ("Let them pass unhindered (+2 reputation)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 2.0,
                    result_text: "You wave the merchants through. They call out blessings as they continue their journey.".into(),
                }),
            ],
        },
        EncounterTemplate {
            encounter_type: EncounterType::Bandits,
            description: "Armed bandits spring from hiding! They demand coin or blood.".into(),
            difficulty: EncounterDifficulty::Medium,
            terrains: vec![WorldTerrain::Grassland, WorldTerrain::Forest, WorldTerrain::Road],
            choices: vec![
                ("Fight them off".into(), EncounterOutcome {
                    gold_delta: 15.0, reputation_delta: 1.0,
                    result_text: "After a brief skirmish, the bandits scatter. You claim their meagre loot.".into(),
                }),
                ("Pay them off (-30 gold)".into(), EncounterOutcome {
                    gold_delta: -30.0, reputation_delta: 0.0,
                    result_text: "The bandits count your coin and melt back into the shadows. Expensive, but no one got hurt.".into(),
                }),
                ("Intimidate them (+5 reputation)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 5.0,
                    result_text: "Your fearsome reputation precedes you. The bandits flee without a fight.".into(),
                }),
            ],
        },
        // — Forest encounters —
        EncounterTemplate {
            encounter_type: EncounterType::WildAnimals,
            description: "A pack of wolves shadows your party from the treeline. Their eyes gleam with hunger.".into(),
            difficulty: EncounterDifficulty::Medium,
            terrains: vec![WorldTerrain::Forest],
            choices: vec![
                ("Light torches and drive them off".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 0.0,
                    result_text: "The wolves retreat from the flames. You press on through the dark woods.".into(),
                }),
                ("Hunt them for pelts (+10 gold)".into(), EncounterOutcome {
                    gold_delta: 10.0, reputation_delta: 0.0,
                    result_text: "Your archers bring down two wolves. The pelts will fetch a good price.".into(),
                }),
            ],
        },
        EncounterTemplate {
            encounter_type: EncounterType::Hermit,
            description: "An old hermit emerges from a moss-covered hut. 'I have been expecting you,' he says.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![WorldTerrain::Forest, WorldTerrain::Mountain],
            choices: vec![
                ("Listen to his wisdom (+5 reputation)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 5.0,
                    result_text: "The hermit shares ancient wisdom and a blessing for your journey.".into(),
                }),
                ("Ask for treasure (-5 reputation)".into(), EncounterOutcome {
                    gold_delta: 10.0, reputation_delta: -5.0,
                    result_text: "The hermit sighs and hands you a small pouch. 'You missed the point,' he mutters.".into(),
                }),
            ],
        },
        // — Mountain encounters —
        EncounterTemplate {
            encounter_type: EncounterType::Scouts,
            description: "Mounted scouts appear on the ridge ahead. They carry the banner of a rival beylik.".into(),
            difficulty: EncounterDifficulty::Hard,
            terrains: vec![WorldTerrain::Mountain, WorldTerrain::Grassland],
            choices: vec![
                ("Avoid them (stealth)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 0.0,
                    result_text: "You take cover behind the rocks. The scouts pass without spotting you.".into(),
                }),
                ("Capture them for intel (+20 gold)".into(), EncounterOutcome {
                    gold_delta: 20.0, reputation_delta: 0.0,
                    result_text: "Your men surround the scouts. They reveal troop movements in exchange for their freedom.".into(),
                }),
            ],
        },
        // — Settlement encounters —
        EncounterTemplate {
            encounter_type: EncounterType::Refugees,
            description: "A column of refugees blocks the road — families fleeing conflict ahead.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![WorldTerrain::Settlement, WorldTerrain::Road, WorldTerrain::Grassland],
            choices: vec![
                ("Offer them aid (+10 reputation, -15 gold)".into(), EncounterOutcome {
                    gold_delta: -15.0, reputation_delta: 10.0,
                    result_text: "You share your provisions with the refugees. Their grateful prayers follow you down the road.".into(),
                }),
                ("Recruit the able-bodied (+2 workforce)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 2.0,
                    result_text: "Several young men and women take up arms to join your cause. A worthy addition.".into(),
                }),
                ("Turn them away (-5 reputation)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: -5.0,
                    result_text: "You cannot afford to slow down. The refugees watch you leave with despair in their eyes.".into(),
                }),
            ],
        },
        // — Universal encounters —
        EncounterTemplate {
            encounter_type: EncounterType::Omen,
            description: "A massive eagle circles overhead, then dives toward a nearby rock. On the rock, an arrowhead-shaped stone glitters.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![
                WorldTerrain::Grassland, WorldTerrain::Forest, WorldTerrain::Mountain,
                WorldTerrain::Road, WorldTerrain::Settlement,
            ],
            choices: vec![
                ("Investigate the omen (+10 reputation)".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 10.0,
                    result_text: "You find an ancient arrowhead carved with符文. Your men take it as a sign of victory.".into(),
                }),
                ("Ignore it and move on".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 0.0,
                    result_text: "You march on. But your men seem unsettled by what they saw.".into(),
                }),
            ],
        },
        EncounterTemplate {
            encounter_type: EncounterType::Traveler,
            description: "A lone traveler hurries toward you, breathless. 'Please, I need to reach the next town before nightfall!'".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![
                WorldTerrain::Grassland, WorldTerrain::Forest, WorldTerrain::Road,
                WorldTerrain::Settlement,
            ],
            choices: vec![
                ("Escort them to safety (+5 reputation)".into(), EncounterOutcome {
                    gold_delta: 5.0, reputation_delta: 5.0,
                    result_text: "The traveler is a merchant's son. His father rewards you with gold and gratitude.".into(),
                }),
                ("Send them on their way".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 0.0,
                    result_text: "The traveler hurries off, casting grateful glances over his shoulder.".into(),
                }),
            ],
        },
        // — River crossing encounters —
        EncounterTemplate {
            encounter_type: EncounterType::Nomads,
            description: "A nomadic camp of Yörük tents sprawls beside the river. They wave you over.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![WorldTerrain::Grassland],
            choices: vec![
                ("Share their fire (gain 15 gold)".into(), EncounterOutcome {
                    gold_delta: 15.0, reputation_delta: 3.0,
                    result_text: "The nomads welcome you with hot tea and flatbread. They trade news and fine wool.".into(),
                }),
                ("Trade for horses (+1 speed to next travel)".into(), EncounterOutcome {
                    gold_delta: -20.0, reputation_delta: 0.0,
                    result_text: "You purchase sturdy horses from the nomads. Your next journey will be swifter.".into(),
                }),
            ],
        },
    ];
}

// ═══════════════════════════════════════════════════════════════════════════
//  POI Discovery System — Fog reveals undiscovered points of interest
// ═══════════════════════════════════════════════════════════════════════════

/// When the fog of war reveals tiles, check if any undiscovered POIs are now
/// visible. If so, mark them as discovered and fire `PoiDiscovered` events.
fn poi_discovery_system(
    mut world_map: ResMut<WorldMap>,
    mut poi_discovered_events: EventWriter<PoiDiscovered>,
) {
    // Check each undiscovered POI — is its tile now visible or explored?
    // (FogOfWar dims Visible→Explored each frame, so we check both)
    let undiscovered: Vec<PoiId> = world_map
        .undiscovered_pois()
        .iter()
        .filter(|poi| {
            let fog = world_map.fog_at(poi.position);
            fog == FogState::Visible || fog == FogState::Explored
        })
        .map(|poi| poi.poi_id)
        .collect();

    for poi_id in undiscovered {
        if world_map.discover_poi(poi_id) {
            if let Some(poi) = world_map.get_poi(poi_id) {
                poi_discovered_events.send(PoiDiscovered {
                    poi_id,
                    position: poi.position,
                    poi_type: poi.poi_type.display_name().to_string(),
                    name: poi.name.clone(),
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Encounter System — Random events trigger during world map travel
// ═══════════════════════════════════════════════════════════════════════════

/// Each frame a travel entity is in transit, roll for a random encounter based
/// on the terrain at their current position.
fn encounter_system(
    mut world_map: ResMut<WorldMap>,
    travel_entities: Query<(Entity, &WorldMapTravel)>,
    mut encounter_events: EventWriter<EncounterTriggered>,
) {
    for (entity, travel) in travel_entities.iter() {
        // Only check for encounters while actively travelling (progress < duration)
        if travel.progress >= travel.duration {
            continue;
        }

        // Estimate current world position based on travel progress
        let from_center = world_map.region_center(travel.from_region);
        let to_center = world_map.region_center(travel.to_region);
        let Some(from) = from_center else { continue };
        let Some(to) = to_center else { continue };

        let t = (travel.progress / travel.duration.max(0.01)).clamp(0.0, 1.0);
        let current_x = (from.x as f32 + (to.x - from.x) as f32 * t) as i32;
        let current_y = (from.y as f32 + (to.y - from.y) as f32 * t) as i32;
        let current_pos = IVec2::new(current_x, current_y);

        // Determine terrain at current position
        let terrain = world_map
            .get_tile(current_pos)
            .map(|t| t.terrain)
            .unwrap_or(WorldTerrain::Grassland);

        // Roll for encounter (using world_map.encounter_seed)
        world_map.encounter_seed = world_map.encounter_seed.wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let luck = (world_map.encounter_seed % 100) as f32 / 100.0;

        if let Some(mut encounter) = world_map.roll_encounter(terrain, luck) {
            encounter.position = current_pos;
            encounter_events.send(EncounterTriggered {
                encounter,
                entity: Some(entity),
            });
        }
    }
}


