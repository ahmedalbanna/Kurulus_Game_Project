# Kurulus Game — Memory & Progress

> Anchored summary of project state. Update after each significant work session.

---

## Current State (Week 18 — Complete)

**Test suite: 142 tests, all passing, 0 warnings, 0 errors.**

### What was built

| Feature | Files | Tests |
|---------|-------|-------|
| **Hero Selection/Switch** — Tab-key cycling through Kayi heroes (Osman, Orhan, Alaeddin, etc.) with wrap-around. Direct target (by HeroId), Next/Previous direction. Ignores enemy heroes. Fires `SelectionChanged` event. | `hero_switch.rs` (new), `kurulus-app/tests/hero_switch_tests.rs` (8 tests) | 8 |
| **Post-Mission Rewards** — `apply_mission_rewards_system` listens for `MissionEnded` events with `success: true`, sums gold from all completed objectives, adds to `ResourceStockpile.gold`, collects unique item unlocks into `CampaignState.unlocked_items`. | `narrative/mod.rs` (2 new systems + test helper), `kurulus-app/tests/campaign_tests.rs` (11 tests) | 11 |
| **Reflect Derives for BRP/MCP** — All Phase 3 components/resources now have `#[derive(Reflect)]` + `#[reflect(...)]` attributes and are registered via `register_type()` for runtime debugging. | All structs across `kurulus-core` and `kurulus-systems` | — |
| **World Map + Fog of War** — 48×36 tile map of Anatolia with 12 named regions, fog of war (Unexplored/Explored/Visible), region control with unit-count capture mechanics, terrain-weighted travel between regions, Bresenham LOS for mountains. | `world_map/mod.rs` (6 systems), `world.rs` (expanded), `resources/world_map.rs` (new), `events/world.rs` (7 events), `world_map_tests.rs` (27 tests) | 27 |
| **Points of Interest** — 14 historical locations across all 12 regions (Kayı Camp, Karacahisar Fort, Uludağ Shrine, Lake İznik, etc.). Auto-discovered when fog reveals their tile. Fires `PoiDiscovered` event. | `world_map/mod.rs` (define_pois, poi_discovery_system), `world.rs` (PointOfInterest, PoiId, PoiType enums) | 11 |
| **Random Encounters** — 10 encounter templates (Merchants, Bandits, Refugees, Scouts, Hermit, Wild Animals, Nomads, Omen, Traveler) with terrain-weighted triggers during travel. Each has 2-3 choices with gold/reputation outcomes. 8% base trigger chance per frame. Deterministic RNG via seeded shuffle. | `world_map/mod.rs` (build_encounter_table, encounter_system), `world.rs` (Encounter, EncounterType, EncounterChoice, EncounterOutcome), `resources/world_map.rs` (EncounterTemplate, roll_encounter) | 27 |

### Balance Changes (M3/M4/M6)

| Change | Old | New | Reason |
|--------|-----|-----|--------|
| M3 SurviveWave duration | 60s | 90s | Mongol raid felt too brief; 90s gives Mongols more time to be a threat |
| M4 RescueAlly SP reward | 2 SP | 3 SP | Hardest mission had lowest SP; reward parity with difficulty |
| M6 KillEnemyHero SP reward | 2 SP | 3 SP | Finale should feel more rewarding per objective |

### Bug Fixes

- **Cross-system objective auto-completion**: `track_m1_kill_objective_system` was auto-setting progress to 1.0 for `TutorialAttackEnemy` when no Byzantine units existed, before `apply_mission_rewards_system` could check `completed` status. Fixed tests to use `ObjectiveId::Custom(999)` to avoid tracker interference.
- **Removed debug eprintln! statements** from `apply_mission_rewards_system`.
- **Added `#[derive(Reflect)]` to `FactionId`** — required by new world map types using it in reflected structs.

### Architecture Decisions

- `apply_mission_rewards_system` runs in `FixedUpdate`, ordered **before** `campaign_save_system` so rewards are captured in the save snapshot.
- Gold summed per-mission and added as lump sum to `ResourceStockpile.gold` (f32).
- Item unlocks deduplicated via `Vec::contains()`.
- World Map stored as a **Resource** (not entity-based) for efficient tile lookups — `HashMap<IVec2, FogState>` cache for fog queries.
- Fog of War dims `Visible→Explored` each frame, then units re-reveal within vision range. This gives persistent "explored but dimmed" memory.
- Region control uses unit-count majority (>50% + ≥3) to prevent flicker from 1-2 stray units.
- Terrain-weighted travel cost: `distance × avg_terrain_cost / speed`. Mountains are 3× slower than grassland; water is impassable.
- Tactical grid → world map mapping at 8:1 ratio (`grid / 8`). A unit with `vision_range=16` reveals ~2 tiles radius on the world map.

### Test Highlights

- `test_fog_of_war_reveals_near_units` — Unit with vision_range=16 reveals world map tiles
- `test_fog_persists_as_explored` — After unit despawns, visible→explored transition works
- `test_region_captured_event_fired` — 5 Kayi units capture ErmeniBeli from Byzantines, `RegionCaptured` event fires with correct old/new owners
- `test_region_control_no_capture_with_few_units` — 1 unit doesn't trigger capture
- `test_travel_component_advances` — `WorldMapTravel` with 5s duration completes after 300 frames at 1/60
- `test_world_map_starting_fog` — Söğüt is Visible, distant Bursa is Unexplored
- `test_region_display_names` — Turkish unicode display names verified

---

## Future Work (Phase 4 — Season 2-3)

### Yenişehir City Builder
- City zoning districts, population system, road infrastructure
- Building upgrade chains, city services, health/happiness
- Village→Town→City progression, city view UI

### World Map (Remaining)
- Points of interest / random encounters during travel
- Territory control visualization
- Fog of war integration with scouting orders

### Beylik Politics System
- Relationship tracking, diplomatic actions, beylik AI personalities
- Council/court, events, intrigue, marriage alliances
- Tributary system, war/peace with war score

### Next-Gen Heroes Framework
- Skill trees, active/passive abilities, equipment, inventory
- Hero relationships, personal story arcs, mentorship
- Fatigue/injury, hero roster UI

---

## Completed Phases

- **Phase 0: Foundation** — ECS, plugins, core components ✓
- **Phase 1: Core Gameplay** — economy, combat, units, buildings ✓
- **Phase 2: Content** — 6 missions, narrative system, objectives, hero progression ✓
- **Phase 3: Integration** — mission flow, campaign persistence, hero switch, post-mission rewards, BRP/MCP debugging ✓
- **Phase 4 Started: World Map + Fog of War** — 12-region overworld map, fog system, region control, travel ✓
