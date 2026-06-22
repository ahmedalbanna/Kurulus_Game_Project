## Goal
- Implement Week 17: Full Ambush & Terrain (terrain stealth modifiers, night bonus, noise detection, ambush placement, fog of war)

## Constraints & Preferences
- Bevy 0.15 ECS architecture with 7-crate workspace
- Event-driven phase transitions (Planning → Execution → Harvest)
- A* pathfinding on GridPos for unit movement
- Single-player isometric RTS with guerrilla warfare focus
- Must integrate with existing BRP/MCP debugging tools

## Progress
### Done
- Phase 3 Week 13 Complete: Missions M4-M6 + campaign persistence
- Phase 3 Week 14 Complete: AI System — Behavior Tree Engine — 20 tests
- Phase 3 Week 15 Complete: AI Advanced — Factions & Difficulty
- Phase 3 Week 16 Complete: Tech Tree — 48 nodes, SP economy, hero research, 22 unit tests
- Phase 3 Week 17 Complete: Full Ambush & Terrain
  - TerrainType, TerrainTile, TerrainGrid (TerrainGrid resource with stealth_modifier/blocks_vision/movement_cost per TerrainType)
  - Stealth modifiers: Forest (+0.30), OpenField (-0.10), Mountain (+0.15), Building (+0.20), River/Water (-0.05)
  - Night stealth bonus: +0.20 (applied in stealth_detection_system and ambush_auto_hide_system)
  - NoiseLevel component: noise scales linearly with move_speed
  - UnitVisibility component: footprint per archetype (Scout 0.2 → HeavyCavalry 0.8)
  - AmbushMarker component + PlaceAmbushOrder event for zone placement
  - ambush_placement_system: costs 30 gold, spawns AmbushZone with 1.5× damage, 60s duration
  - ambush_auto_hide_system: auto-hides units inside ambush zones
  - Enhanced stealth_detection_system: terrain + night + noise − size factors
  - Enhanced ambush_trigger_system: reads actual MeleeWeapon damage instead of hardcoded 10.0
  - Enhanced vision_calculation_system: terrain blocking (mountain/building blocks vision)
- Fixed `GameTime::advance()` bug: `total_seconds` initialized to match `hour=8, minute=0` so the clock doesn't reset to midnight on first tick
- Fixed stealth modifier triple-application: `terrain_stealth_system` and `night_stealth_system` are now no-ops (effective modifiers computed in detection/ambush systems)
- Fixed `ambush_auto_hide_system` storing unit entity instead of zone entity in `ambush_zone`
- 96 tests total, all passing; 0 compiler warnings, 0 errors

### In Progress
- (none)

### Blocked
- (none)

## Key Decisions
- Phase transitions driven by game state instead of fixed timers
- A* pathfinding on GridPos with 8-directional octile heuristic
- Damage pipeline: MeleeResolution → DamageDealt → DamageApplication → Health update → Dead
- Hero abilities use ActiveBuffs for temporary stat modifications
- Sequential objectives enforced by tracking systems
- CampaignState tracks completed/unlocked missions; ResourcePersistence carries stockpile
- Tuple system limit (16) handled by splitting into multiple add_systems calls
- AI difficulty configs as Resources for runtime flexibility
- Faction strategy transitions via triggers (army threshold, game time)
- Difficulty multipliers applied to decision intervals, gather rates, attack frequency, target selection quality
- Terrain stealth modifiers: Forest +0.3, OpenField -0.1, Mountain +0.15, Building +0.2
- Night stealth bonus: +0.20 during night hours (GameTime::is_night())
- Noise from movement: linear scaling (move_speed / 10.0), multiplied by 0.5 for stealth penalty
- Unit visibility footprint: Scout 0.2, LightInfantry/Archer/Worker 0.3, LightCavalry 0.5, HeavyInfantry 0.7, HeavyCavalry 0.8, SiegeCrew 0.9; multiplied by 0.3 for stealth penalty
- Ambush zones consume 30 gold, auto-hide units inside, +50% weapon damage
- Effective stealth = clamp(base + terrain_mod + night_mod − noise_factor − size_factor, 0, 1) computed fresh each detection/ambush tick
- Detection formula: (dist / radius) × (1.0 − effective_stealth) < 0.5 → revealed
- `terrain_stealth_system` and `night_stealth_system` kept as no-op hooks for future visual effects

## Next Steps
- Write comprehensive tests for terrain stealth, night bonus, noise detection, unit size, ambush damage, auto-hide, fog of war
- Implement basic fog of war rendering integration (visibility grid + terrain blocking)

## Critical Context
- Phase 2: All integration tests passing (combat, economy, formation, mission, stealth)
- Phase 3 Week 11: All 8 hero abilities implemented (Orhan + Alaeddin)
- Phase 3 Week 12: M1/M2/M3 complete, 18 tests
- Phase 3 Week 13: M4-M6 complete, campaign persistence
- Phase 3 Week 14: AI Behavior Tree Engine, 20 tests, 72 total
- Phase 3 Week 15: AI Factions & Difficulty, threat eval, faction bonuses, strategy transitions
- Phase 3 Week 16: Tech Tree — 48 nodes, SP economy, hero-assisted research, 94 tests total
- Phase 3 Week 17: Full Ambush & Terrain — 96 total tests
- `cargo check`: 0 warnings, 0 errors across all 7 crates
- MissionId enum extended to Mission06
- Win/loss conditions per mission (hero death, Oba destroyed, time limit, all units dead)

## Relevant Files
- kurulus/kurulus-core/src/components/stealth.rs: Stealth, Detection, AmbushZone, NoiseLevel, UnitVisibility, AmbushMarker
- kurulus/kurulus-core/src/components/terrain.rs: TerrainType, TerrainTile (Forest, OpenField, Mountain, Building, River, Water)
- kurulus/kurulus-core/src/resources/terrain.rs: TerrainGrid resource (HashMap<IVec2, TerrainType> with stealth_modifier/blocks_vision/movement_cost)
- kurulus/kurulus-core/src/events/command.rs: PlaceAmbushOrder event
- kurulus/kurulus-systems/src/stealth/mod.rs: Full stealth pipeline (6 systems in FixedUpdate)
- kurulus/kurulus-core/src/resources/time.rs: GameTime (fixed total_seconds init)
- kurulus/kurulus-app/tests/stealth_tests.rs: 2 stealth tests (in-range detection, out-of-range hidden)
