# AGENTS.md — Kurulus Game Project

## Quick start

```bash
# Run (2D default)
cd kurulus && cargo run -p kurulus-app

# 3D rendering
cargo run -p kurulus-app --features render-3d --no-default-features

# Headless test harness (no rendering)
cargo run -p kurulus-app --features headless-test --no-default-features

# With BRP remote debugging for MCP tools
cargo run -p kurulus-app --features remote

# Run all tests
cargo test

# Check only (fast)
cargo check -p kurulus-app
```

All commands from `kurulus/` subdirectory.

## Feature flags (kurulus-app)

| Feature | Effect |
|---------|--------|
| `default` = `render-2d` | 2D sprite rendering |
| `render-3d` | 3D PBR rendering |
| `headless-test` | No rendering, logic-only test harness |
| `remote` | Enables BRP (`RemotePlugin` + `RemoteHttpPlugin`) for MCP |

Only one render feature at a time (`--no-default-features` when switching to render-3d or headless-test).

## Workspace (7 crates)

```
kurulus/
├── kurulus-app         ← Entrypoint (main.rs → plugins.rs::register_plugins)
├── kurulus-core        ← Components, Events, Resources, States, SystemSets, GridPos
├── kurulus-config      ← RON asset loader for config files (units, buildings, heroes, missions)
├── kurulus-systems     ← All 22 game logic plugins, plus `register_logic_plugins()`
├── kurulus-render-2d   ← 2D sprite/camera/debug → Render2DPlugin
├── kurulus-render-3d   ← 3D PBR/terrain/camera → Render3DPlugin
└── kurulus-editor      ← WIP mission editor (stub)
```

**Dep order**: core → config → systems → app → (render-2d | render-3d optional)

Core is the dependency hub — all crates depend on `kurulus-core`.

## Architecture essentials

- **ECS with Bevy 0.15**, edition 2021, resolver = "2"
- **Three states**: `AppState` (MainMenu/InGame/Paused), `GamePhase` (Planning/Execution/Harvest), `MissionState` (Loading/Active/Complete/Failed)
- **9 ordered system sets** (`SimulationSet`): InputProcessing → ... → ComponentCleanup. Systems register via `.in_set(SimulationSet::Foo)`.
- **22 logic plugins** registered in specific order in `plugins.rs` (CoreLoop → WorldMap → AI → Economy → UnitCommand → Combat → Stealth → Hero → BaseBuilding → Progression → Narrative → Diplomacy → BeylikPolitics → NextGenHeroes → RomanceLoyalty → SuccessionLegacy → SiegeWarfare → ImperialAdmin → NavalWarfare → CoRule → AudioLogic → UILogic)
- **Event-driven**: ~40 event types in `kurulus-core/src/events/`. Systems read events, mutate components, emit events.
- **Grid position system**: `GridPos(IVec2)` for logical grid, `WorldPosition` unifies grid + 2D + 3D coords. `CELL_SIZE = 1.0`.
- **Config via RON**: `kurulus-config/assets/` contains `.ron` files loaded at runtime by `RonAssetLoader`.

## Testing patterns

Tests live in `kurulus/kurulus-app/tests/`. Standard integration test pattern:

```rust
let mut app = App::new();
app.add_plugins(MinimalPlugins)
   .add_plugins(register_logic_plugins)  // from kurulus-systems (registers all states, resources, events, logic plugins)
   .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(1.0 / 60.0)))
   .add_event::<SomeEvent>()             // if not already registered
   .init_resource::<SomeResource>();     // if not already registered

// Spawn entities in app.world_mut(), run app.update() in loop, query results
```

Key: `register_logic_plugins` handles states + resources + events + all 22 logic plugins. Individual tests may need extra `.add_event` / `.init_resource` calls. Use `ManualDuration` to control time deterministically (no real-time dependency).

96+ tests total, all passing. `cargo check` must emit 0 warnings, 0 errors.

## BRP / MCP integration

- MCP servers configured in `opencode.json` at project root.
- Server binaries installed at `/teamspace/studios/this_studio/.cargo/bin/`.
- BRP activated by `--features remote` (adds `RemotePlugin` + `RemoteHttpPlugin` behind `#[cfg(feature = "remote")]` in `plugins.rs`).
- Types registered for BRP reflection in `register_logic_plugins()` via `.register_type::<T>()`.
- Port defaults: BRP = 15702.

Setup commands:
```bash
./setup.sh --check        # verify everything
./scripts/setup-mcp.sh    # install/verify MCP tools only
```

## Reference docs

| Document | Location |
|----------|----------|
| Game Design Document (Arabic/English) | `docs/GDD.md` (1361 lines, covers all 6 seasons, 22 systems) |
| Core Design Bible | `docs/Kurulus_Game_Docs/00_Core_Design/` |
| Feature Specs (all 22 systems) | `docs/Kurulus_Game_Docs/01_Features/` |
| Campaign & Mission Design | `docs/Kurulus_Game_Docs/02_Campaign/` |
| Technical Architecture | `docs/Kurulus_Game_Docs/03_Technical_Architecture/` |
| Reference Materials | `docs/Kurulus_Game_Docs/04_Reference/` |
| Production Plan & Roadmap | `docs/Kurulus_Game_Docs/05_Production/` |
| Sprint progress | `anchor/summary.md` |

Consult `docs/` for design intent before implementing feature changes — code may lag behind docs.

## Other notes

- `Cargo.lock` is gitignored (workspace-level).
- `input.txt` at root contains the current development task description.
- No linter/formatter config detected (relies on `cargo check` for correctness).
- All components use `Serialize`/`Deserialize` (via serde) or `Reflect`.
- Resource config types (`PhaseTransitionConfig`, `CombatActivityTracker`, `OrdersIssuedDuringPlanning`) drive the Planning→Execution→Harvest game loop.
