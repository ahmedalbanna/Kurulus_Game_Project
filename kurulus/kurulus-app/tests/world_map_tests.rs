use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

// ─── Helpers ───────────────────────────────────────────────────────────────

fn make_world_map_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<WorldMap>();
    app
}

fn update_n(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
    }
}

// ── World Map Initialisation Tests ─────────────────────────────────────────

#[test]
fn test_world_map_initialises_tiles() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    assert!(!world_map.tiles.is_empty(), "Tiles should be generated");
    assert_eq!(
        world_map.tiles.len() as u32,
        world_map.width * world_map.height
    );
    assert!(world_map.regions.len() == 12, "Should have 12 regions");
}

#[test]
fn test_world_map_has_regions() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    let region_ids: Vec<RegionId> = world_map.regions.iter().map(|r| r.region_id).collect();
    assert!(region_ids.contains(&RegionId::Sogut), "Sögüt should exist");
    assert!(region_ids.contains(&RegionId::Bursa), "Bursa should exist");
    assert!(region_ids.contains(&RegionId::Iznik), "İznik should exist");
}

#[test]
fn test_world_map_starting_fog() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    // Sögüt area should be revealed at start
    let sogut_center = world_map.region_center(RegionId::Sogut).unwrap();
    assert_eq!(
        world_map.fog_at(sogut_center),
        FogState::Visible,
        "Starting region should be visible"
    );
    // Far away region should be unexplored
    let bursa_center = world_map.region_center(RegionId::Bursa).unwrap();
    assert_eq!(
        world_map.fog_at(bursa_center),
        FogState::Unexplored,
        "Distant region should be unexplored initially"
    );
}

#[test]
fn test_player_starts_in_sogut() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    assert_eq!(
        world_map.current_region,
        Some(RegionId::Sogut),
        "Player starts in Söğüt"
    );
}

// ── Fog of War Tests ───────────────────────────────────────────────────────

#[test]
fn test_fog_of_war_reveals_near_units() {
    let mut app = make_world_map_app();
    app.update();

    // Spawn a unit with vision on the tactical grid
    app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(40, 40), CELL_SIZE),
        Stats {
            move_speed: 3.0,
            attack_range: 1.5,
            attack_damage: 12.0,
            attack_speed: 1.0,
            vision_range: 16.0,
            morale: 50.0,
            morale_max: 100.0,
        },
        Health {
            current: 80.0,
            max: 80.0,
            armor: 3.0,
            armor_type: ArmorType::Light,
        },
    ));

    update_n(&mut app, 5);

    let world_map = app.world().resource::<WorldMap>();
    let visible_count = world_map.tiles_with_fog(FogState::Visible).len();
    assert!(
        visible_count > 0,
        "Unit vision should reveal tiles (visible_count={})",
        visible_count
    );
}

#[test]
fn test_fog_persists_as_explored() {
    let mut app = make_world_map_app();
    app.update();

    // Spawn a unit, run frames to reveal area, then despawn
    let unit_entity = app
        .world_mut()
        .spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(40, 40), CELL_SIZE),
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 1.0,
                vision_range: 16.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
        ))
        .id();

    update_n(&mut app, 5);

    // Despawn the unit
    app.world_mut().entity_mut(unit_entity).despawn();
    update_n(&mut app, 5);

    let world_map = app.world().resource::<WorldMap>();
    // Previously visible tiles should now be Explored (dimmed)
    let explored_count = world_map.tiles_with_fog(FogState::Explored).len();
    assert!(
        explored_count > 0,
        "Previously visible tiles should become Explored"
    );
}

// ── Region Control Tests ───────────────────────────────────────────────────

#[test]
fn test_region_control_no_capture_with_few_units() {
    let mut app = make_world_map_app();
    app.update();

    // Place 1 Kayi unit in Byzantine region — not enough to capture
    app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(120, 60), CELL_SIZE),
        Stats::default(),
        Health::default(),
    ));

    update_n(&mut app, 5);

    let world_map = app.world().resource::<WorldMap>();
    let ermeni = world_map.get_region(RegionId::ErmeniBeli).unwrap();
    assert_eq!(
        ermeni.owner,
        FactionId::Byzantine,
        "1 unit shouldn't capture"
    );
}

#[test]
fn test_region_captured_event_fired() {
    let mut app = make_world_map_app();
    app.update();

    // Place 5 Kayi units in ErmeniBeli (region bounds: x=10..16, y=4..10)
    // World pos at (104, 40) maps to world tile (13, 5), well within ErmeniBeli
    for i in 0..5 {
        app.world_mut().spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(104 + i * 2, 40), CELL_SIZE),
            Stats::default(),
            Health::default(),
        ));
    }

    // Run updates one at a time and check for events on each frame
    let mut captured = false;
    let mut region_captured = None;
    for _ in 0..20 {
        app.update();
        let events: Vec<RegionCaptured> = {
            let events_res = app.world().resource::<Events<RegionCaptured>>();
            let mut reader = events_res.get_cursor();
            reader.read(events_res).cloned().collect()
        };
        if !events.is_empty() {
            captured = true;
            region_captured = Some(events[0].clone());
        }
    }

    let world_map = app.world().resource::<WorldMap>();
    let ermeni = world_map.get_region(RegionId::ErmeniBeli).unwrap();
    assert_eq!(
        ermeni.owner, FactionId::Kayi,
        "Kayi should capture ErmeniBeli"
    );

    assert!(captured, "RegionCaptured event should be fired");
    let ev = region_captured.unwrap();
    assert_eq!(ev.region, RegionId::ErmeniBeli);
    assert_eq!(ev.old_owner, FactionId::Byzantine);
    assert_eq!(ev.new_owner, FactionId::Kayi);
}

// ── World Map Travel Tests ─────────────────────────────────────────────────

#[test]
fn test_travel_between_regions() {
    let mut app = make_world_map_app();
    app.update();

    let world_map = app.world().resource::<WorldMap>();
    let sogut_center = world_map.region_center(RegionId::Sogut).unwrap();
    let domanic_center = world_map.region_center(RegionId::Domanic).unwrap();
    let duration = world_map.travel_duration(sogut_center, domanic_center, 3.0);
    assert!(duration > 0.0, "Travel duration should be positive");
    assert!(duration < 100.0, "Travel should not take absurdly long");
}

#[test]
fn test_travel_component_advances() {
    let mut app = make_world_map_app();
    app.update();

    let entity = app
        .world_mut()
        .spawn((
            WorldMapTravel {
                from_region: RegionId::Sogut,
                to_region: RegionId::Domanic,
                progress: 0.0,
                duration: 5.0,
                faction: FactionId::Kayi,
            },
        ))
        .id();

    update_n(&mut app, 300); // 5 seconds at 1/60

    let travel = app.world().get::<WorldMapTravel>(entity);
    assert!(
        travel.is_none() || travel.unwrap().progress >= travel.unwrap().duration,
        "Travel should complete after duration"
    );
}

// ── WorldMap Resource API Tests ────────────────────────────────────────────

#[test]
fn test_tile_in_radius() {
    let mut world_map = WorldMap::default();
    world_map.init(20, 20);

    let tiles = world_map.tiles_in_radius(IVec2::new(10, 10), 3.0);
    assert!(
        tiles.len() >= 20,
        "Should have ~28 tiles in radius 3 (got {})",
        tiles.len()
    );
    assert!(
        tiles.len() <= 40,
        "Should not exceed 40 tiles in radius 3 (got {})",
        tiles.len()
    );
    assert!(tiles.contains(&IVec2::new(10, 10)));
}

#[test]
fn test_region_center() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);

    let region = RegionDef {
        region_id: RegionId::Sogut,
        name: "Test".into(),
        tiles: vec![IVec2::new(5, 10), IVec2::new(6, 10), IVec2::new(5, 11), IVec2::new(6, 11)],
        owner: FactionId::Kayi,
        resources: RegionResources::default(),
        strategic_value: 1,
        garrison_strength: 0.0,
    };
    for &pos in &region.tiles {
        if let Some(tile) = world_map.get_tile_mut(pos) {
            tile.region = Some(region.region_id);
        }
    }
    world_map.regions.push(region);

    let center = world_map.region_center(RegionId::Sogut).unwrap();
    assert_eq!(center, IVec2::new(5, 10));
}

#[test]
fn test_region_display_names() {
    assert_eq!(RegionId::Sogut.display_name(), "Söğüt");
    assert_eq!(RegionId::Iznik.display_name(), "İznik");
    assert_eq!(RegionId::Bursa.display_name(), "Bursa");
    assert_eq!(RegionId::Inegol.display_name(), "İnegöl");
}

#[test]
fn test_world_map_bounds() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);
    assert!(world_map.is_valid_pos(IVec2::new(0, 0)));
    assert!(world_map.is_valid_pos(IVec2::new(47, 35)));
    assert!(!world_map.is_valid_pos(IVec2::new(-1, 0)));
    assert!(!world_map.is_valid_pos(IVec2::new(0, 36)));
    assert!(!world_map.is_valid_pos(IVec2::new(48, 0)));
}

#[test]
fn test_dim_visible_tiles() {
    let mut world_map = WorldMap::default();
    world_map.init(10, 10);
    world_map.reveal_tile(IVec2::new(3, 3));
    world_map.reveal_tile(IVec2::new(4, 4));
    assert_eq!(world_map.tiles_with_fog(FogState::Visible).len(), 2);
    world_map.dim_visible_tiles();
    assert_eq!(world_map.tiles_with_fog(FogState::Visible).len(), 0);
    assert_eq!(world_map.tiles_with_fog(FogState::Explored).len(), 2);
}

#[test]
fn test_travel_duration_scales_with_distance() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);
    let short = world_map.travel_duration(IVec2::new(5, 5), IVec2::new(6, 5), 3.0);
    let long = world_map.travel_duration(IVec2::new(5, 5), IVec2::new(40, 30), 3.0);
    assert!(long > short, "Longer distance should take more time");
}

// ── Points of Interest Tests ──────────────────────────────────────────────

#[test]
fn test_poi_definitions_exist() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    assert_eq!(world_map.points_of_interest.len(), 14, "Should have 14 POIs");
    // Key POIs should exist
    let poi_ids: Vec<PoiId> = world_map.points_of_interest.iter().map(|p| p.poi_id).collect();
    assert!(poi_ids.contains(&PoiId::KayiCamp));
    assert!(poi_ids.contains(&PoiId::KaracahisarFort));
    assert!(poi_ids.contains(&PoiId::IznikLake));
}

#[test]
fn test_poi_discovered_on_fog_reveal() {
    let mut app = make_world_map_app();
    app.update();

    // Before reveal: undiscovered
    let world_map = app.world().resource::<WorldMap>();
    let poi = world_map.get_poi(PoiId::KayiCamp).unwrap();
    assert!(!poi.discovered, "KayiCamp starts undiscovered");
    // Söğüt area is revealed at start — KayiCamp should become discovered
    // KayiCamp is at (7, 11) and Söğüt center is near (7, 11)
    // Starting reveal radius is 5.0 from Söğüt center
    let _ = world_map;

    // Run several frames for discovery system to process
    update_n(&mut app, 10);

    let world_map = app.world().resource::<WorldMap>();
    let poi = world_map.get_poi(PoiId::KayiCamp).unwrap();
    assert!(poi.discovered, "KayiCamp should be discovered after start reveal");
}

#[test]
fn test_poi_discovery_event_fired() {
    let mut app = make_world_map_app();
    app.update();

    // Check for PoiDiscovered events
    let mut discovered_pois = Vec::new();
    for _ in 0..10 {
        app.update();
        let events: Vec<PoiDiscovered> = {
            let events_res = app.world().resource::<Events<PoiDiscovered>>();
            let mut reader = events_res.get_cursor();
            reader.read(events_res).cloned().collect()
        };
        for ev in events {
            discovered_pois.push(ev.poi_id);
        }
    }

    // At least the nearby POIs should be discovered
    assert!(
        discovered_pois.contains(&PoiId::KayiCamp),
        "KayiCamp should fire discovery event"
    );
}

#[test]
fn test_distant_poi_starts_undiscovered() {
    let mut app = make_world_map_app();
    app.update();
    update_n(&mut app, 10);

    let world_map = app.world().resource::<WorldMap>();
    // Bursa POIs should still be undiscovered (far from start)
    let poi = world_map.get_poi(PoiId::UludagShrine);
    if let Some(poi) = poi {
        // Only check if the POI exists and is far enough
        assert!(!poi.discovered, "UludagShrine should remain undiscovered at start");
    }
}

#[test]
fn test_all_pois_have_valid_positions() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();

    for poi in &world_map.points_of_interest {
        assert!(
            world_map.is_valid_pos(poi.position),
            "POI {:?} at {:?} is out of bounds",
            poi.poi_id,
            poi.position
        );
        // Should have a region assignment
        assert!(
            poi.region.is_some(),
            "POI {:?} has no region",
            poi.poi_id
        );
    }
}

#[test]
fn test_undiscovered_pois_in_radius() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);

    // Add a test POI
    world_map.points_of_interest.push(PointOfInterest {
        poi_id: PoiId::KayiCamp,
        name: "Test".into(),
        position: IVec2::new(5, 5),
        poi_type: PoiType::Camp,
        discovered: false,
        description: "Test".into(),
        region: Some(RegionId::Sogut),
    });

    let nearby = world_map.undiscovered_pois_in_radius(IVec2::new(5, 5), 2.0);
    assert_eq!(nearby.len(), 1, "Should find undiscovered POI in radius");

    let far = world_map.undiscovered_pois_in_radius(IVec2::new(40, 30), 2.0);
    assert_eq!(far.len(), 0, "Should not find POI far away");
}

// ── Encounter Tests ───────────────────────────────────────────────────────

#[test]
fn test_encounter_table_populated() {
    let mut app = make_world_map_app();
    app.update();
    let world_map = app.world().resource::<WorldMap>();
    assert!(!world_map.encounter_table.is_empty(), "Encounter table should have entries");
    // Should have entries for grassland, forest, mountain, etc.
    let grass_encounters: usize = world_map.encounter_table
        .iter()
        .filter(|t| t.terrains.contains(&WorldTerrain::Grassland))
        .count();
    assert!(grass_encounters > 0, "Should have grassland encounters");
}

#[test]
fn test_roll_encounter_can_return_none() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);
    build_encounter_table_test(&mut world_map);

    // With low luck (0.0) encounter should trigger sometimes
    // With high luck (0.5) should never trigger (> 0.08 threshold)
    for _ in 0..20 {
        world_map.encounter_seed = world_map.encounter_seed.wrapping_add(1);
        let luck = (world_map.encounter_seed % 100) as f32 / 100.0;
        if luck > 0.08 {
            let result = world_map.roll_encounter(WorldTerrain::Grassland, luck);
            assert!(result.is_none(), "High luck should not trigger encounter");
        }
    }
}

#[test]
fn test_encounter_has_valid_choices() {
    let mut world_map = WorldMap::default();
    world_map.init(48, 36);
    build_encounter_table_test(&mut world_map);

    world_map.encounter_seed = 100;
    let luck = 0.01; // Trigger encounter
    let encounter = world_map.roll_encounter(WorldTerrain::Grassland, luck);
    assert!(encounter.is_some(), "Low luck should trigger encounter");

    let encounter = encounter.unwrap();
    assert!(!encounter.choices.is_empty(), "Encounter should have choices");
    assert!(!encounter.description.is_empty(), "Encounter should have description");
    assert!(encounter.choices.len() <= 3, "Should have at most 3 choices");
}

// ── POI Display Name Tests ────────────────────────────────────────────────

#[test]
fn test_poi_display_names() {
    assert_eq!(PoiId::KayiCamp.display_name(), "Kayı Camp");
    assert_eq!(PoiId::KaracahisarFort.display_name(), "Karacahisar Fort");
    assert_eq!(PoiId::IznikLake.display_name(), "Lake İznik");
}

#[test]
fn test_poi_type_display_names() {
    assert_eq!(PoiType::Camp.display_name(), "Camp");
    assert_eq!(PoiType::Fort.display_name(), "Fort");
    assert_eq!(PoiType::SacredSite.display_name(), "Sacred Site");
    assert_eq!(PoiType::MountainPass.display_name(), "Mountain Pass");
}

/// Build a minimal encounter table for testing without full app setup.
fn build_encounter_table_test(world_map: &mut WorldMap) {
    world_map.encounter_table = vec![
        EncounterTemplate {
            encounter_type: EncounterType::Merchants,
            description: "Merchants approach.".into(),
            difficulty: EncounterDifficulty::Easy,
            terrains: vec![WorldTerrain::Grassland],
            choices: vec![
                ("Trade".into(), EncounterOutcome {
                    gold_delta: 20.0, reputation_delta: 0.0,
                    result_text: "Done.".into(),
                }),
                ("Ignore".into(), EncounterOutcome {
                    gold_delta: 0.0, reputation_delta: 0.0,
                    result_text: "Ok.".into(),
                }),
            ],
        },
    ];
}
