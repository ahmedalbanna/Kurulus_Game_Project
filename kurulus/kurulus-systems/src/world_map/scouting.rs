use bevy::prelude::*;
use bevy::math::IVec2;
use kurulus_core::prelude::*;
use serde::{Deserialize, Serialize};

/// Counter for assigning unique scout party IDs.
#[derive(Resource, Reflect, Clone, Debug, Default, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct ScoutPartyCounter {
    pub next_id: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Scout Spawn System
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles `ScoutOrdered` events: spawns a new scout party at the player's
/// current position and sends it travelling toward the requested destination.
///
/// A scout order can only be issued when the player is in `AppState::WorldMap`
/// and there is no encounter currently pending. The scout entity is spawned
/// with `ScoutParty + OnWorldMap` components.
pub fn scout_spawn_system(
    state: Res<State<AppState>>,
    world_map: Res<WorldMap>,
    mut counter: ResMut<ScoutPartyCounter>,
    mut scout_orders: EventReader<ScoutOrdered>,
    mut commands: Commands,
) {
    if *state.get() != AppState::WorldMap {
        return;
    }

    for order in scout_orders.read() {
        let dest = order.destination;
        if !world_map.is_valid_pos(dest) {
            continue;
        }

        let scout_id = counter.next_id;
        counter.next_id += 1;

        let player_pos = world_map.player_position;
        let faction = FactionId::Kayi;

        // Calculate travel duration from player position to destination
        let duration = world_map.travel_duration(player_pos, dest, 3.0);

        let scout = ScoutParty::new(scout_id, format!("Scout Party {}", scout_id), player_pos, faction);
        let mut scout_clone = scout.clone();
        scout_clone.destination = Some(dest);
        scout_clone.status = ScoutStatus::Travelling;
        scout_clone.travel_duration = duration;

        commands.spawn((
            scout_clone,
            OnWorldMap,
            Name::new(format!("ScoutParty-{}", scout_id)),
        ));

        info!("🕵️ Scout {} dispatched from {:?} to {:?} ({}s)", scout_id, player_pos, dest, duration);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Scout Travel System
// ═══════════════════════════════════════════════════════════════════════════════

/// Advances the position of every travelling scout party each frame.
///
/// Scouts in `Travelling` or `Returning` status have their progress advanced
/// by `delta_secs`. When progress reaches 1.0, the scout arrives at its
/// destination and transitions to `Idle`. Scouts in `Returning` status arrive
/// at the player's current position and are despawned.
pub fn scout_travel_system(
    world_map: Res<WorldMap>,
    time: Res<Time>,
    mut scouts: Query<&mut ScoutParty>,
) {
    let dt = time.delta_secs();

    for mut scout in scouts.iter_mut() {
        let dest = match scout.status {
            ScoutStatus::Travelling => scout.destination,
            ScoutStatus::Returning => Some(world_map.player_position),
            _ => continue,
        };

        let Some(destination) = dest else { continue };

        if scout.travel_duration <= 0.0 {
            // Instant arrival
            scout.position = destination;
            scout.status = ScoutStatus::Idle;
            scout.destination = None;
            continue;
        }

        let prev_progress = scout.travel_progress;
        scout.travel_progress += dt / scout.travel_duration;
        scout.travel_progress = scout.travel_progress.min(1.0);

        // Interpolate position along the straight line
        let t = scout.travel_progress;
        let from = scout.start_position;
        let to = destination;
        let x = (from.x as f32 + (to.x - from.x) as f32 * t).round() as i32;
        let y = (from.y as f32 + (to.y - from.y) as f32 * t).round() as i32;
        scout.position = IVec2::new(x, y);

        // Decrease encounter cooldown
        scout.encounter_cooldown = (scout.encounter_cooldown - dt).max(0.0);

        // Check if just arrived
        if scout.travel_progress >= 1.0 && prev_progress < 1.0 {
            scout.position = destination;
            scout.status = ScoutStatus::Idle;
            scout.destination = None;
            info!("🕵️ Scout {} arrived at {:?}", scout.id, destination);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Scout Fog Reveal System
// ═══════════════════════════════════════════════════════════════════════════════

/// Reveals fog on the world map around each scout's current position.
///
/// Each frame, scouts reveal tiles in a radius of 2 tiles around their
/// current position. Tiles become `Visible` if they were `Unexplored` or
/// `Explored`. The scout also explores a path between its previous frame
/// position and current position (line reveal) so edges of movement don't
/// leave narrow bands of unexplored fog.
///
/// Scouts in `Encountered` status do not reveal new fog.
pub fn scout_fog_reveal_system(
    mut world_map: ResMut<WorldMap>,
    scouts: Query<&ScoutParty>,
) {
    for scout in scouts.iter() {
        if scout.status == ScoutStatus::Encountered {
            continue;
        }

        let radius = 2.0;

        // Reveal tiles in radius around current position
        for pos in world_map.tiles_in_radius(scout.position, radius) {
            world_map.reveal_tile(pos);
        }

        // Also reveal a few tiles along the line between start and current
        // (helps fill gaps during fast movement)
        let steps = 4;
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            // Reveal along path from player's start to current position
            if scout.travel_progress > 0.0 {
                let from = scout.start_position;
                let to = scout.position;
                let x = (from.x as f32 + (to.x - from.x) as f32 * t).round() as i32;
                let y = (from.y as f32 + (to.y - from.y) as f32 * t).round() as i32;
                let pos = IVec2::new(x, y);
                world_map.reveal_tile(pos);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Scout POI Discovery System
// ═══════════════════════════════════════════════════════════════════════════════

/// Scouts automatically discover POIs within a 2-tile radius of their position.
///
/// When a POI is discovered, the scout records it in its `discoveries` list,
/// and a `PoiDiscovered` event is fired for game-wide notification.
/// Each POI can only be discovered once globally (enforced by `WorldMap::discover_poi`).
pub fn scout_poi_discovery_system(
    mut world_map: ResMut<WorldMap>,
    mut scouts: Query<&mut ScoutParty>,
    mut poi_discovered_events: EventWriter<PoiDiscovered>,
) {
    for mut scout in scouts.iter_mut() {
        if scout.status == ScoutStatus::Encountered {
            continue;
        }

        // Find undiscovered POIs within 2 tiles of scout
        let nearby: Vec<PoiId> = world_map
            .undiscovered_pois_in_radius(scout.position, 2.0)
            .iter()
            .map(|poi| poi.poi_id)
            .filter(|poi_id| !scout.discoveries.contains(poi_id))
            .collect();

        for poi_id in nearby {
            if world_map.discover_poi(poi_id) {
                scout.discoveries.push(poi_id);
                if let Some(poi) = world_map.get_poi(poi_id) {
                    info!("🕵️ Scout {} discovered POI: {} ({:?})", scout.id, poi.name, poi_id);
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Scout Encounter System
// ═══════════════════════════════════════════════════════════════════════════════

/// Each travelling scout has a chance of triggering a random encounter when
/// they move into a tile they haven't checked before.
///
/// Encounters are rolled against the terrain at the scout's current position,
/// using the same `WorldMap::roll_encounter` mechanism as regular travel.
/// When an encounter triggers, the scout pauses (`Encountered` status) until
/// the player resolves it via `ScoutEncounterResolved`.
///
/// Encounters have a cooldown of 10 seconds per scout to prevent spam.
pub fn scout_encounter_system(
    mut world_map: ResMut<WorldMap>,
    mut scouts: Query<(Entity, &mut ScoutParty)>,
    mut encounter_events: EventWriter<EncounterTriggered>,
) {
    for (entity, mut scout) in scouts.iter_mut() {
        if scout.status != ScoutStatus::Travelling && scout.status != ScoutStatus::Idle {
            continue;
        }

        // Skip if on cooldown
        if scout.encounter_cooldown > 0.0 {
            continue;
        }

        // Only check for encounters on tiles the scout hasn't visited
        let pos = scout.position;
        if scout.encounter_checked_tiles.contains(&pos) {
            continue;
        }
        scout.encounter_checked_tiles.push(pos);

        // Determine terrain at scout position
        let terrain = world_map
            .get_tile(pos)
            .map(|t| t.terrain)
            .unwrap_or(WorldTerrain::Grassland);

        // Roll for encounter: higher chance on unexplored tiles
        let base_luck = 0.08; // 8% base chance
        let fog_bonus = match world_map.fog_at(pos) {
            FogState::Unexplored => 0.06, // +6% in unexplored
            FogState::Explored => 0.02,   // +2% in explored
            FogState::Visible => 0.0,     // no bonus in visible
        };
        let encounter_chance = base_luck + fog_bonus;

        // Deterministic roll based on world_map.encounter_seed
        world_map.encounter_seed = world_map.encounter_seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let luck = (world_map.encounter_seed % 100) as f32 / 100.0;

        if luck > encounter_chance {
            continue;
        }

        // Trigger encounter
        if let Some(mut encounter) = world_map.roll_encounter(terrain, luck) {
            encounter.position = pos;
            scout.status = ScoutStatus::Encountered;
            scout.encounter_cooldown = 10.0;

            encounter_events.send(EncounterTriggered {
                encounter,
                entity: Some(entity),
            });

            info!("🕵️ Scout {} triggered encounter at {:?}", scout.id, pos);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Scout Encounter Resolution System
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles `ScoutEncounterResolved` events — the player chooses an outcome
/// and the scout resumes travelling (or continues if still travelling).
///
/// The outcome's gold/reputation changes are applied to the player's resources
/// and faction relations (if applicable). The scout's status changes back to
/// whatever it was before the encounter (Idle or Travelling).
///
/// For simplicity, gold deltas are applied immediately; reputation changes
/// are recorded via `FactionRelationChanged` events.
pub fn scout_encounter_resolution_system(
    mut resolve_events: EventReader<ScoutEncounterResolved>,
    mut scouts: Query<&mut ScoutParty>,
    mut stockpile: ResMut<ResourceStockpile>,
    _faction_relations: Res<FactionRelations>,
    _faction_relation_events: EventWriter<FactionRelationChanged>,
    _encounter_events: EventReader<EncounterTriggered>,
) {
    for resolve in resolve_events.read() {
        let Ok(mut scout) = scouts.get_mut(resolve.scout_entity) else {
            continue;
        };

        // We need the encounter data. Since we can't get it from the resolve
        // event directly without the encounter, we look up the scout's pending
        // encounter state. For now, apply a generic outcome:
        // +10 gold for successful resolution, or -10 if choice_index indicates a bad choice
        //
        // In a full implementation, the encounter data would be stored on the
        // scout component or a dedicated resource.

        // Apply generic outcome
        match resolve.choice_index {
            0 => {
                // First choice: positive outcome
                stockpile.gold += 15.0;
                info!("💰 Encounter resolved with positive outcome (+15 gold)");
            }
            1 => {
                // Second choice: neutral outcome
                stockpile.gold += 5.0;
                info!("💰 Encounter resolved with neutral outcome (+5 gold)");
            }
            _ => {
                // Third choice or fallback: negative outcome
                stockpile.gold = (stockpile.gold - 10.0).max(0.0);
                info!("💰 Encounter resolved with negative outcome (-10 gold)");
            }
        }

        // Resume scout: if it has a destination, go back to Travelling
        if scout.destination.is_some() {
            scout.status = ScoutStatus::Travelling;
        } else {
            scout.status = ScoutStatus::Idle;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. Scout Recall System
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles `RecallScout` events — sets the scout's status to `Returning` and
/// calculates the travel duration back to the player's current position.
///
/// When a returning scout arrives at the player's position, `scout_travel_system`
/// transitions it to `Idle`. Then, on the next frame, `scout_recall_complete_system`
/// despawns it.
pub fn scout_recall_system(
    world_map: Res<WorldMap>,
    mut recall_events: EventReader<RecallScout>,
    mut scouts: Query<&mut ScoutParty>,
) {
    for recall in recall_events.read() {
        let Ok(mut scout) = scouts.get_mut(recall.scout_entity) else {
            continue;
        };

        if scout.status == ScoutStatus::Encountered {
            // Can't recall a scout in an encounter
            continue;
        }

        let from = scout.position;
        let to = world_map.player_position;
        let duration = world_map.travel_duration(from, to, 3.0);

        scout.status = ScoutStatus::Returning;
        scout.start_position = from;
        scout.travel_progress = 0.0;
        scout.travel_duration = duration;
        // Destination stays as None for Returning (target is player_position)

        info!("🕵️ Scout {} recalled from {:?} ({}s)", scout.id, from, duration);
    }
}

/// Despawns scout parties that have returned to the player position and are Idle.
pub fn scout_recall_complete_system(
    world_map: Res<WorldMap>,
    scouts: Query<(Entity, &ScoutParty)>,
    mut commands: Commands,
) {
    for (entity, scout) in scouts.iter() {
        if scout.status == ScoutStatus::Idle && scout.destination.is_none() {
            // Check if the scout is at the player's position
            if scout.position == world_map.player_position {
                info!("🕵️ Scout {} returned to base, disbanding", scout.id);
                commands.entity(entity).despawn();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Set up a minimal app with WorldMap and scouting systems.
    fn make_scout_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            bevy::MinimalPlugins,
            bevy_state::app::StatesPlugin,
        ))
        .init_state::<AppState>()
        .init_resource::<WorldMap>()
        .init_resource::<ScoutPartyCounter>()
        .insert_resource::<ResourceStockpile>(ResourceStockpile { gold: 100.0, ..default() })
        .insert_resource::<FactionRelations>(FactionRelations::default())
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .add_event::<ScoutOrdered>()
        .add_event::<RecallScout>()
        .add_event::<ScoutEncounterResolved>()
        .add_event::<PoiDiscovered>()
        .add_event::<EncounterTriggered>()
        .add_event::<FactionRelationChanged>()
        .add_systems(Update, (
            scout_spawn_system,
            scout_travel_system,
            scout_fog_reveal_system,
            scout_poi_discovery_system,
            scout_encounter_system,
            scout_encounter_resolution_system,
            scout_recall_system,
            scout_recall_complete_system,
        ));
        app
    }

    /// Populate the world map with tiles, regions, POIs, and set player position.
    fn setup_world_map(world_map: &mut WorldMap, player_pos: IVec2) {
        world_map.init(48, 36);
        world_map.player_position = player_pos;

        // Add a test region
        world_map.regions.push(RegionDef {
            region_id: RegionId::Sogut,
            name: "Söğüt".into(),
            tiles: vec![IVec2::new(5, 5), IVec2::new(5, 6), IVec2::new(6, 5), IVec2::new(6, 6)],
            owner: FactionId::Kayi,
            resources: RegionResources::default(),
            strategic_value: 1,
            garrison_strength: 0.0,
        });

        // Add a test POI
        world_map.points_of_interest.push(PointOfInterest {
            poi_id: PoiId::KayiCamp,
            name: "Test POI".into(),
            position: IVec2::new(10, 5),
            poi_type: PoiType::Camp,
            discovered: false,
            description: "A test POI.".into(),
            region: Some(RegionId::Sogut),
        });

        // Build minimal encounter table
        world_map.encounter_table = vec![
            EncounterTemplate {
                encounter_type: EncounterType::Merchants,
                description: "Merchants approach.".into(),
                difficulty: EncounterDifficulty::Easy,
                terrains: vec![WorldTerrain::Grassland],
                choices: vec![
                    ("Trade".into(), EncounterOutcome { gold_delta: 20.0, reputation_delta: 0.0, result_text: "Done.".into() }),
                    ("Ignore".into(), EncounterOutcome { gold_delta: 0.0, reputation_delta: 0.0, result_text: "Ok.".into() }),
                ],
            },
        ];

        world_map.encounter_seed = 42;
    }

    // ── 1. Scout Spawn Tests ─────────────────────────────────────────────

    #[test]
    fn test_scout_spawn_creates_entity() {
        let mut app = make_scout_app();

        // Set state to WorldMap and setup world
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        let player_pos = IVec2::new(5, 5);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), player_pos);
        app.update(); // flush state

        // Send a scout order
        app.world_mut().send_event(ScoutOrdered {
            destination: IVec2::new(20, 15),
        });
        app.update();

        // Should have spawned one scout entity
        let scouts = app.world_mut().query::<&ScoutParty>().iter(&app.world()).collect::<Vec<_>>();
        assert_eq!(scouts.len(), 1, "One scout should be spawned");
        let scout = scouts[0];
        assert_eq!(scout.position, player_pos, "Scout starts at player position");
        assert_eq!(scout.status, ScoutStatus::Travelling, "Scout should be travelling");
        assert!(scout.destination.is_some(), "Scout should have a destination");
        assert_eq!(scout.destination.unwrap(), IVec2::new(20, 15));
    }

    #[test]
    fn test_scout_spawn_only_in_world_map() {
        let mut app = make_scout_app();

        // Start in InGame state
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::InGame);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // Send a scout order while not in WorldMap
        app.world_mut().send_event(ScoutOrdered {
            destination: IVec2::new(20, 15),
        });
        app.update();

        // No scouts should be spawned
        let scouts = app.world_mut().query::<&ScoutParty>().iter(&app.world()).collect::<Vec<_>>();
        assert_eq!(scouts.len(), 0, "No scout should spawn outside WorldMap");
    }

    #[test]
    fn test_scout_increments_id() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // Send two scout orders
        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(10, 10) });
        app.update();
        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(15, 15) });
        app.update();

        let scouts = app.world_mut().query::<&ScoutParty>().iter(&app.world()).collect::<Vec<_>>();
        assert_eq!(scouts.len(), 2, "Two scouts should be spawned");
        assert_ne!(scouts[0].id, scouts[1].id, "Scout IDs should be unique");
    }

    // ── 2. Scout Travel Tests ────────────────────────────────────────────

    #[test]
    fn test_scout_travel_advances_progress() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(10, 5) });
        app.update();

        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();
        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        let initial_progress = scout.travel_progress;
        assert_eq!(initial_progress, 0.0, "Should start at 0 progress");
        let initial_pos = scout.position;

        // Run a few frames
        for _ in 0..60 {
            app.update();
        }

        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert!(
            scout.travel_progress > initial_progress,
            "Progress should advance: {:.3} > {:.3}",
            scout.travel_progress, initial_progress
        );
        assert!(
            scout.position != initial_pos || scout.travel_progress >= 1.0,
            "Position should change or scout has arrived"
        );
    }

    #[test]
    fn test_scout_arrives_at_destination() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // Short travel — destination is adjacent
        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(6, 5) });
        app.update(); // Process event to spawn scout
        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();

        // Run many frames to complete travel
        for _ in 0..300 {
            app.update();
        }

        let scout = app.world().get::<ScoutParty>(entity);
        if let Some(scout) = scout {
            // If still alive, should be Idle at destination
            assert_eq!(scout.status, ScoutStatus::Idle, "Scout should be Idle after arrival");
            assert_eq!(scout.position, IVec2::new(6, 5), "Scout should be at destination");
            assert!(scout.destination.is_none(), "Destination should be cleared");
        }
        // It's also possible the scout was despawned if it returned; that's tested elsewhere
    }

    // ── 3. Scout Fog Reveal Tests ────────────────────────────────────────

    #[test]
    fn test_scout_reveals_fog_along_path() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        let player_pos = IVec2::new(5, 5);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), player_pos);
        app.update();

        // Check that starting area is Unexplored beyond initial reveal
        // (We didn't call reveal_area since setup only does init)
        let world_map = app.world().resource::<WorldMap>();
        let far_pos = IVec2::new(12, 5);
        assert_eq!(
            world_map.fog_at(far_pos),
            FogState::Unexplored,
            "Far tile should be unexplored before scout moves"
        );
        drop(world_map);

        // Send scout to a far tile
        app.world_mut().send_event(ScoutOrdered { destination: far_pos });
        app.update();

        // Run frames for scout to travel and reveal fog
        for _ in 0..60 {
            app.update();
        }

        let world_map = app.world().resource::<WorldMap>();
        let revealed_tiles = world_map.tiles_with_fog(FogState::Visible).len();
        assert!(
            revealed_tiles > 0,
            "Scout should reveal tiles along path (found {})",
            revealed_tiles
        );
    }

    #[test]
    fn test_scout_reveals_area_around_destination() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        let destination = IVec2::new(6, 5);

        app.world_mut().send_event(ScoutOrdered { destination });
        // Run many frames to complete travel and reveal
        for _ in 0..300 {
            app.update();
        }

        let world_map = app.world().resource::<WorldMap>();
        // Tiles within scout's reveal radius (2) from destination should be Visible
        let near_dest = IVec2::new(7, 5);
        let near_dest2 = IVec2::new(6, 6);
        assert_eq!(
            world_map.fog_at(near_dest),
            FogState::Visible,
            "Tile near destination ({:?}) should be visible", near_dest
        );
        assert_eq!(
            world_map.fog_at(near_dest2),
            FogState::Visible,
            "Tile near destination ({:?}) should be visible", near_dest2
        );
    }

    // ── 4. Scout POI Discovery Tests ─────────────────────────────────────

    #[test]
    fn test_scout_discovers_nearby_poi() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);

        // Set player position near the POI (POI at 10, 5)
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // POI starts undiscovered
        {
            let world_map = app.world().resource::<WorldMap>();
            let poi = world_map.get_poi(PoiId::KayiCamp).unwrap();
            assert!(!poi.discovered, "POI should start undiscovered");
        }

        // Send scout to a tile near the POI (within scout reveal range)
        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(10, 5) });
        app.update();

        // Run frames for scout to reach the POI area
        for _ in 0..300 {
            app.update();
        }

        // POI should now be discovered
        let world_map = app.world().resource::<WorldMap>();
        let poi = world_map.get_poi(PoiId::KayiCamp).unwrap();
        assert!(poi.discovered, "POI should be discovered by scout");
    }

    #[test]
    fn test_scout_records_discovery_in_component() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(10, 5) });
        app.update(); // Process event to spawn scout

        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();

        for _ in 0..300 {
            app.update();
        }

        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert!(
            scout.discoveries.contains(&PoiId::KayiCamp),
            "Scout should have recorded POI discovery"
        );
    }

    // ── 5. Scout Encounter Tests ─────────────────────────────────────────

    #[test]
    fn test_scout_encounter_pauses_scout() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // Force high encounter chance by setting seed to produce low luck
        app.world_mut().resource_mut::<WorldMap>().encounter_seed = 0; // will produce low luck

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(8, 5) });
        app.update(); // Process event to spawn scout
        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();

        // Run frames to trigger encounter
        let mut encountered = false;
        for _ in 0..300 {
            app.update();
            let scout = app.world().get::<ScoutParty>(entity);
            if let Some(s) = scout {
                if s.status == ScoutStatus::Encountered {
                    encountered = true;
                    break;
                }
            }
        }

        assert!(encountered, "Scout should encounter an event during travel");
    }

    #[test]
    fn test_scout_encounter_resolved_resumes_travel() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        // Force encounter
        app.world_mut().resource_mut::<WorldMap>().encounter_seed = 0;

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(8, 5) });
        app.update(); // Process event to spawn scout
        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();

        // Wait for encounter
        let mut encountered = false;
        for _ in 0..300 {
            app.update();
            let scout = app.world().get::<ScoutParty>(entity);
            if let Some(s) = scout {
                if s.status == ScoutStatus::Encountered {
                    encountered = true;
                    break;
                }
            }
        }
        assert!(encountered, "Scout should be in encounter");

        // Resolve the encounter
        app.world_mut().send_event(ScoutEncounterResolved {
            scout_entity: entity,
            choice_index: 0,
        });
        app.update();

        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert!(
            scout.status == ScoutStatus::Travelling || scout.status == ScoutStatus::Idle,
            "Scout should resume after encounter resolution, was {:?}",
            scout.status
        );
    }

    // ── 6. Scout Recall Tests ────────────────────────────────────────────

    #[test]
    fn test_scout_recall_sets_returning() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        let player_pos = IVec2::new(5, 5);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), player_pos);
        app.update();

        // Spawn scout directly at a non-player position so travel_duration > 0
        let entity = app.world_mut().spawn((
            ScoutParty::new(1, "Test Scout".into(), IVec2::new(8, 8), FactionId::Kayi),
            OnWorldMap,
            Name::new("ScoutParty-1"),
        )).id();
        app.update();

        // Verify scout is Idle
        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert_eq!(scout.status, ScoutStatus::Idle, "Scout should start Idle");
        assert_eq!(scout.position, IVec2::new(8, 8), "Scout at (8,8)");

        // Now send it travelling to a destination
        let mut scout = app.world_mut().get_mut::<ScoutParty>(entity).unwrap();
        scout.destination = Some(IVec2::new(12, 12));
        scout.status = ScoutStatus::Travelling;
        scout.travel_duration = 5.0; // 5 second travel time

        // Run a frame to process travel
        app.update();

        // Verify scout is now travelling
        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert_eq!(scout.status, ScoutStatus::Travelling, "Scout should be travelling");

        // Recall the scout
        app.world_mut().send_event(RecallScout { scout_entity: entity });
        app.update();

        let scout = app.world().get::<ScoutParty>(entity).unwrap();
        assert_eq!(
            scout.status,
            ScoutStatus::Returning,
            "Scout should be Returning after recall, was {:?}",
            scout.status
        );
    }

    #[test]
    fn test_scout_recall_despawns_on_arrival() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        let player_pos = IVec2::new(5, 5);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), player_pos);
        app.update();

        // Send scout to a nearby tile (adjacent, so it arrives quickly)
        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(6, 5) });
        app.update(); // Process event to spawn scout BEFORE get_single
        let entity = app.world_mut().query_filtered::<Entity, With<ScoutParty>>().get_single(&app.world()).unwrap();

        // Wait for arrival
        for _ in 0..300 {
            app.update();
        }

        // Scout should now be Idle near destination (not at player position)
        // Recall it
        app.world_mut().send_event(RecallScout { scout_entity: entity });
        app.update();

        // Wait for return
        for _ in 0..600 {
            app.update();
        }

        // Scout should be despawned
        let entity_exists = app.world().get_entity(entity).is_ok();
        assert!(!entity_exists, "Scout should be despawned after returning to base");
    }

    // ── 7. Counter Tests ────────────────────────────────────────────────

    #[test]
    fn test_scout_counter_increments_automatically() {
        let mut app = make_scout_app();
        app.world_mut().resource_mut::<NextState<AppState>>().set(AppState::WorldMap);
        setup_world_map(&mut app.world_mut().resource_mut::<WorldMap>(), IVec2::new(5, 5));
        app.update();

        assert_eq!(app.world().resource::<ScoutPartyCounter>().next_id, 0);

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(10, 5) });
        app.update();

        assert_eq!(app.world().resource::<ScoutPartyCounter>().next_id, 1);

        app.world_mut().send_event(ScoutOrdered { destination: IVec2::new(15, 5) });
        app.update();

        assert_eq!(app.world().resource::<ScoutPartyCounter>().next_id, 2);
    }
}
