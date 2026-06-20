# Simulation Loop — System Overview

> **Description:** Defines the core game loop architecture for *Kuruluş* — how time advances, in what order systems update, and how fixed/variable timesteps are managed. Technology-agnostic, applicable to any game engine.

---

## 1. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Deterministic simulation** | Same inputs → same outputs, regardless of frame rate. Essential for replay and debugging |
| 2 | **Decoupled update phases** | Systems update in a defined order — no dependency cycles |
| 3 | **Fixed timestep for gameplay** | Physics, AI, and combat run at a fixed rate. Rendering interpolates between ticks |
| 4 | **Graceful performance degradation** | If frame rate drops, gameplay simulation stays stable; only visuals degrade |
| 5 | **Pause/resume transparency** | Simulation state is fully pausable and resumable at any point |

---

## 2. Game Loop Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     GAME LOOP (per frame)                     │
│                                                               │
│   ┌───────────────────┐                                       │
│   │  INPUT POLLING     │  ← Keyboard, mouse, gamepad           │
│   └─────────┬─────────┘                                       │
│             │                                                  │
│             ▼                                                  │
│   ┌───────────────────┐    ┌────────────────────────────┐     │
│   │  SIMULATION        │───►│  Fixed Timestep Loop       │     │
│   │  (if unpaused)     │    │  (e.g., 60 ticks/sec)      │     │
│   │                    │    │                             │     │
│   │  ┌─────────────┐   │    │  Tick 1: Process input      │     │
│   │  │ Accumulator  │   │    │  Tick 2: Update AI          │     │
│   │  │ (dt leftover)│   │    │  Tick 3: Update physics     │     │
│   │  └─────────────┘   │    │  Tick 4: Update combat       │     │
│   └─────────┬─────────┘    │  Tick 5: Update economy       │     │
│             │              │  ...                           │     │
│             │              └──────────────┬─────────────────┘     │
│             │                             │                       │
│             ▼                             ▼                       │
│   ┌──────────────────────────────────────────────┐               │
│   │  RENDERING (variable rate)                     │               │
│   │  • Interpolate entity positions                │               │
│   │  • Camera movement (smoothed)                  │               │
│   │  • UI update (event-driven + polling)          │               │
│   │  • Audio update (event-driven)                 │               │
│   └──────────────────────────────────────────────┘               │
└──────────────────────────────────────────────────────────────────┘
```

---

## 3. Fixed vs Variable Timestep

| Aspect | Fixed Timestep | Variable Timestep |
|--------|---------------|-------------------|
| **Rate** | Constant (e.g., 60 Hz) | Matches frame rate |
| **Simulation** | All gameplay physics/AI | Used for |
| **Determinism** | ✅ Deterministic | ❌ Non-deterministic |
| **Pause** | Stopped entirely | Stopped entirely |
| **Consumes** | Input events | Input events |
| **Used by** | Gameplay simulation, AI, Combat, Economy, Movement, Morale | Rendering, UI animation, Camera movement, Audio |
| **Interpolation** | Not needed for simulation; rendering interpolates between ticks | Everything runs at display rate |

### 3.1 Fixed Timestep Configuration

```json
{
  "simulation": {
    "tick_rate": 60,
    "tick_duration_seconds": 0.0166667,
    "max_accumulator_seconds": 0.1,
    "max_frames_to_catch_up": 5
  },
  "interpolation": {
    "position": true,
    "rotation": true,
    "hp_bar": true,
    "animation_blend": true
  }
}
```

---

## 4. Simulation Tick Pipeline

Each simulation tick processes systems in a strict order:

```
TICK START (Δt = 16.67ms)
  │
  ├── 1. Process Buffered Input
  │     • Dequeue player commands
  │     • Dispatch to affected entities
  │
  ├── 2. Update Trigger Conditions
  │     • Evaluate conditions for active triggers
  │     • Queue triggered events
  │
  ├── 3. Update AI
  │     • Faction AI planning
  │     • Entity AI state machines
  │     • Threat evaluation
  │
  ├── 4. Update Movement & Pathfinding
  │     • Resolve paths for entities with move orders
  │     • Update positions
  │     • Collision resolution
  │
  ├── 5. Update Stealth / Detection
  │     • Stealth zone checks
  │     • Detection calculations
  │     • Ambush readiness checks
  │
  ├── 6. Update Combat
  │     • Melee engagement
  │     • Ranged attacks
  │     • Siege damage
  │     • Morale updates
  │     • Status effect ticks
  │
  ├── 7. Update Economy
  │     • Resource income
  │     • Food consumption
  │     • Production queue advancement
  │     • Building construction/upgrade progress
  │
  ├── 8. Update Objectives
  │     • Check objective completion conditions
  │     • Update progress values
  │
  ├── 9. Process Deferred Events
  │     • Fire any deferred events whose time has come
  │
  ├── 10. Process Event Queue
  │      • Dispatch all queued events from this tick
  │      • Process immediate events (chained from above)
  │
  └── 11. End-of-Tick Cleanup
        • Remove dead entities
        • Clear transient flags
        • Update entity dirty flags
```

---

## 5. Frame Pipeline (Variable Rate)

```
FRAME START
  │
  ├── 1. Poll Input Devices
  │     • Store raw input state
  │     • Generate input events
  │
  ├── 2. Simulation Catch-Up
  │     • While accumulator >= fixed_dt:
  │         - Run one simulation tick
  │         - accumulator -= fixed_dt
  │     • Clamp accumulator to max_accumulator_seconds (prevent spiral of death)
  │
  ├── 3. Interpolation
  │     • Interpolate entity transforms (tick N → N+1)
  │     • Smooth camera movement
  │     • Smooth UI animations
  │
  ├── 4. Rendering
  │     • Culling
  │     • Draw calls
  │     • Post-processing
  │     • UI overlay
  │
  ├── 5. Audio
  │     • Process audio events
  │     • Update 3D audio positions (interpolated)
  │
  └── 6. End of Frame
        • Swap buffers
        • Present to screen
        • delta_time = time since last frame
        • accumulator += delta_time
```

---

## 6. Performance Budget (per frame)

| Stage | Budget (60 FPS) | Budget (30 FPS) | Notes |
|-------|----------------|-----------------|-------|
| Input polling | 0.1 ms | 0.1 ms | Negligible |
| Simulation (per tick) | 8 ms (total for all systems) | 16 ms | Scales with entity count |
| Interpolation | 0.5 ms | 0.5 ms | |
| Rendering | 8 ms | 16 ms | Heavy GPU work |
| Audio | 0.5 ms | 0.5 ms | Mostly async |
| Frame overhead | 0.5 ms | 0.5 ms | Buffer swap, etc. |
| **Total** | **~16.67 ms** | **~33.33 ms** | Target < budget |

---

## 7. Spiral of Death Prevention

If the game frame rate drops significantly, the fixed timestep accumulator can grow unbounded, causing the "spiral of death":

```python
# Prevention logic
MAX_ACCUMULATOR = 0.1  # 100ms max catch-up

def tick(delta_time):
    accumulator += min(delta_time, MAX_ACCUMULATOR)

    frames_this_update = 0
    while accumulator >= FIXED_DT and frames_this_update < MAX_FRAMES:
        run_simulation_tick()
        accumulator -= FIXED_DT
        frames_this_update += 1

    # If we hit MAX_FRAMES, we're falling behind — skip remaining time
    if frames_this_update >= MAX_FRAMES:
        accumulator = 0  # Discard remaining time to catch up
        log_warning("Performance: dropping simulation ticks")
```

| Threshold | Behavior |
|-----------|----------|
| accumulator < FIXED_DT | No simulation tick this frame (smooth) |
| accumulator = 1-5 × FIXED_DT | Run that many ticks (catching up) |
| accumulator > MAX_ACCUMULATOR | Clamp to max, run up to MAX_FRAMES ticks |
| Frame rate < 10 FPS | Reduce visual quality (dynamic resolution scaling) |
| Frame rate < 5 FPS | Show "slow connection / low performance" warning |

---

## 8. Pause & Time Scale

| Feature | Implementation |
|---------|---------------|
| **Pause** | accumulator stops accumulating. Fixed timestep loop does not run. |
| **Slow motion** | Time scale multiplier (0.0 - 1.0). delta_time × time_scale fed to accumulator. |
| **Fast forward** | Time scale > 1.0 (up to 4×). More ticks per real second. |
| **Step frame** | On key press, run exactly one simulation tick. |
| **Rewind** | Not supported (requires full state recording — future enhancement). |

---

## 9. Related Files

| File | Description |
|------|-------------|
| `02_Update_Pipeline.md` | Detailed subsystem update order and responsibilities |
| `03_Entity_Processing.md` | Entity lifecycle — spawn, update, destroy |
| `04_Determinism.md` | Deterministic simulation requirements |
| `02_Event_Systems/04_Event_Propagation.md` | Deferred event processing integration |
