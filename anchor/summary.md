## Goal
- Complete Phase 4 Season 2-3: Yenişehir City Builder + World Map + Fog of War UI

## Constraints & Preferences
- Bevy 0.15 ECS architecture with 7-crate workspace
- Event-driven phase transitions (Planning → Execution → Harvest)
- A* pathfinding on GridPos for unit movement
- Single-player isometric RTS with guerrilla warfare focus
- Must integrate with existing BRP/MCP debugging tools

## Progress
### Done
- **Phase 4 — City Builder Expansion** (Weeks 19-23)
  - City Event Log HUD: single-entity `Node + Text`, 14 tests
  - Tax collection system: economic-power scaling formula, 7 tests
  - Zone-building interaction: `BuildRejected` event (ZoneMismatch/CellOccupied), zone validation, 8 tests
  - Overlay Legend HUD: mode-aware legend text (Zones / ServiceCoverage / Happiness), 9 tests
  - Building Visuals: `BuildingSpritePlugin` replaces gizmo overlays with colored `Sprite` entities; mode-responsive colors, 12 tests
  - Building Placement UI: U-key mode, Tab/Shift+Tab cycles 20 types, preview gizmo, left-click sends `BuildOrder`, 10 tests
  - 235 tests total after Phase 4 city builder

- **Phase 4 — World Map + Fog of War** (Weeks 24-25)
  - `AppState::WorldMap` variant added between InGame and Paused
  - `WorldMapViewState` resource: pan_offset + zoom (0.5-4.0)
  - World map gizmo rendering (terrain, fog overlay, region borders, POI markers, player diamond marker)
  - Toggle system (`M` key) — InGame ↔ WorldMap
  - Camera management — repos to map centre (48,36) with scale 18.0 in WorldMap; resets in InGame
  - Click-drag pan + scroll wheel zoom
  - State gating for city builder input systems — zone/building placement disabled in WorldMap mode
  - 10 new tests (4 render + 6 interaction)

- **Phase 4 — World Map UI** (Week 25)
  - `WorldMapHover` resource: mouse → tile → region/POI resolution
  - `WorldMapSelection` resource: region selection on click (toggle)
  - Region/POI tooltip (top-left): name, owner, strategic value, resources, garrison, pop cap
  - Map legend overlay (bottom-left): terrain colours, fog states, POI icons, controls reference
  - Selection info panel (right side): full region detail with resource bars, POI list
  - 6 new UI tests (tooltip text, legend completeness, resource bar, selection panel)
  - 251 tests total

- **Phase 4 — Scouting & Exploration** (Week 26)
  - `ScoutParty` component: position, destination, status, discoveries, encounter state
  - `ScoutStatus` enum: Idle / Travelling / Returning / Encountered
  - `ScoutPartyCounter` resource for unique scout IDs
  - `ScoutOrdered`, `RecallScout`, `ScoutEncounterResolved` events
  - **7 scouting systems** in `world_map/scouting.rs`:
    - `scout_spawn_system` — creates scout at player position from `ScoutOrdered`
    - `scout_travel_system` — interpolates position, handles arrival/return
    - `scout_fog_reveal_system` — reveals fog in radius 2 around scout position
    - `scout_poi_discovery_system` — discovers POIs within 2 tiles
    - `scout_encounter_system` — rolls for encounters (higher in unexplored tiles)
    - `scout_encounter_resolution_system` — applies outcome gold/reputation, resumes scout
    - `scout_recall_system` / `scout_recall_complete_system` — return and disband
  - Scout marker rendering in `WorldMapRenderPlugin` (green dots, orange on encounter)
  - Scout active count in world map legend
  - 16 new tests (14 scouting + 1 render + 1 legend)
  - **267 tests total**, 0 failures

### In Progress
- (none)

### Blocked
- (none)

## Key Decisions
- Scout parties are entities with `ScoutParty + OnWorldMap` components (supports multiple simultaneous scouts)
- Scout spawn/travel/fog/POI/encounter systems run in `FixedUpdate`; event-handling systems in `Update`
- Fog reveal on scout movement uses radius-2 circle + extra path sampling to avoid narrow unexplored bands
- POI discovery is radius-based (2 tiles) and de-dup by `WorldMap::discover_poi` + scout's own discoveries list
- Encounter roll base 8%, +6% bonus in Unexplored tiles, +2% in Explored; 10s cooldown per scout
- Encounter resolution applies simplified generic outcomes (+15/+5/-10 gold) until full encounter UI is built
- Scout recall with `travel_duration=0` (same position) instantly completes and despawns
- Scout recall at player position immediately despawns (no pointless 0-distance travel)
- `ScoutPartyCounter` resource tracks next ID; scouts despawn naturally on return completion

## Next Steps
- Beylik Politics System: faction relationships, diplomatic actions, beylik AI
- World Map interaction depth: mission launch from region selection, travel animation
- Next-Gen Heroes Framework: skill trees, equipment, hero relationships
- Scouting UI polish: send scouts via region click, encounter popup panel

## Critical Context
- **267 tests pass** across all 7 crates (0 failures), up from 251
- `cargo check --all-features`: 0 warnings, 0 errors
- Phase 3 foundations intact: combat, economy, formation, mission, stealth, AI, tech tree
- All existing tests continue to pass
- Phase 4 added 171+ new tests across city builder and world map features
- World map data model pre-existing — rendering, UI, and scouting were the gaps
- Scouting uses `encounter_system` style rolls but with scout-specific cooldowns and higher chance in unexplored

## Relevant Files
- `kurulus-core/src/states/app.rs`: `AppState` with MainMenu / InGame / WorldMap / Paused
- `kurulus-core/src/resources/world_map.rs`: WorldMap, WorldMapViewState, WorldMapHover, WorldMapSelection, WORLD_MAP_CELL_SIZE
- `kurulus-core/src/resources/city.rs`: CityEventLogRoot, ZoneLegendRoot, WorldMapTooltipRoot, WorldMapSelectionRoot, WorldMapLegendRoot
- `kurulus-core/src/components/world.rs`: WorldTile, WorldTerrain, FogState, RegionDef, RegionResources, PointOfInterest, PoiId, RegionId, ScoutParty, ScoutStatus
- `kurulus-core/src/events/world.rs`: ScoutOrdered, RecallScout, ScoutEncounterResolved
- `kurulus-render-2d/src/world_map.rs`: WorldMapRenderPlugin — terrain/fog/region/POI/scout/player gizmo rendering
- `kurulus-systems/src/world_map/scouting.rs`: All 7 scouting systems + ScoutPartyCounter resource + 14 tests
- `kurulus-systems/src/world_map/interaction.rs`: toggle, camera, pan, zoom, hover, click systems + 6 tests
- `kurulus-systems/src/world_map/mod.rs`: WorldMapPlugin registration (now includes scouting module + FixedUpdate/Update systems)
- `kurulus-systems/src/ui/world_map.rs`: tooltip, legend (with scout count), selection panel rendering + 7 tests
- `kurulus-systems/src/lib.rs`: register_logic_plugins registers ScoutParty/ScoutStatus for BRP + ScoutOrdered/RecallScout/ScoutEncounterResolved events
