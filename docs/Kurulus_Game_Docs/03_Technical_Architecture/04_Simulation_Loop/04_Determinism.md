# Determinism — Lockstep Simulation, Replay & Debug

> **Description:** Defines how *Kuruluş* achieves deterministic simulation — where identical inputs produce identical outputs regardless of platform, frame rate, or execution order. Essential for replay functionality, debugging, and future multiplayer support.

---

## 1. Determinism Requirements

| Requirement | Importance | Reasoning |
|-------------|-----------|-----------|
| **Deterministic Random** | Critical | RNG must produce same sequence given same seed |
| **Fixed Timestep** | Critical | Variable timestep breaks determinism |
| **Ordered Updates** | Critical | Systems must update in the same order every tick |
| **No Floating-Point Variance** | High | Different CPUs/GPUs produce different FP results |
| **Deterministic Collision** | High | Collision resolution must be order-independent |
| **Deterministic Pathfinding** | High | A* with tie-breaking must be consistent |
| **No External Input Mid-Tick** | Critical | All input captured before tick begins |

---

## 2. Random Number Generator (RNG)

### 2.1 Design

```python
class DeterministicRNG:
    def __init__(self, seed: int):
        self.state = seed
        self.call_count = 0

    def next_int(self, min: int, max: int) -> int:
        self.state = (self.state * 1103515245 + 12345) & 0x7FFFFFFF
        self.call_count += 1
        return min + (self.state % (max - min + 1))

    def next_float(self) -> float:
        self.state = (self.state * 1103515245 + 12345) & 0x7FFFFFFF
        self.call_count += 1
        return self.state / 2147483648.0  # Divide by 2^31

    def save_state(self) -> tuple:
        return (self.state, self.call_count)

    def restore_state(self, state: tuple):
        self.state, self.call_count = state
```

### 2.2 RNG Ownership

| RNG Instance | Owned By | Used For |
|-------------|----------|----------|
| **Global Mission RNG** | Mission Controller | Spawn placement, loot drops, weather |
| **Per-Faction RNG** | Each AI faction | AI decision variance |
| **Per-Entity RNG** | Each entity | Hit/miss rolls, damage variance |
| **Visual RNG** | VFX/Audio systems | Non-deterministic visual variation |

### 2.3 RNG Seed Source

```
Global seed = profile_id consistent_hash + mission_id hash
  │
  ├── Faction A seed = global_seed + faction_A_index
  ├── Faction B seed = global_seed + faction_B_index
  └── Per-entity seed = faction_seed + entity_spawn_tick + entity_type_hash
```

---

## 3. Floating-Point Consistency

### 3.1 Strategies

| Strategy | Description | Applied To |
|----------|-------------|------------|
| **Integer arithmetic** | Use fixed-point integers where possible | Grid positions, HP, damage |
| **Deterministic math library** | Custom math that produces identical results across platforms | Angle calculations, distance checks |
| **Snap to grid** | Quantize positions to a grid cell center after each move | Movement resolution |
| **Order-independent ops** | Sort entities before processing to avoid FP order variance | Area damage, aura effects |

### 3.2 Fixed-Point Representation

```python
# Use integer-based position for deterministic simulation
# 1 unit = 1000 fixed-point millimeters
class FixedPoint:
    def __init__(self, value: int):
        self.value = value  # integer, scale = 1000

    @staticmethod
    def from_float(f: float):
        return FixedPoint(int(f * 1000))

    def to_float(self) -> float:
        return self.value / 1000.0

    def __add__(self, other): return FixedPoint(self.value + other.value)
    def __sub__(self, other): return FixedPoint(self.value - other.value)
    def __mul__(self, scalar: int): return FixedPoint(self.value * scalar)
```

---

## 4. Input Recording & Replay

### 4.1 Input Capture

Every tick, the game records all player inputs that affect simulation:

```json
{
  "tick": 452,
  "inputs": [
    {
      "type": "command",
      "command_type": "move",
      "target_ids": ["unit_001", "unit_002"],
      "target_position": {"x": 12500, "z": 8300}
    },
    {
      "type": "ability",
      "ability_id": "hero_osman_sword_storm",
      "source_id": "hero_osman",
      "target_position": {"x": 13000, "z": 8000}
    }
  ],
  "rng_snapshot": {
    "global_seed": 12345,
    "call_count": 342
  }
}
```

### 4.2 Replay System

```python
class ReplaySystem:
    def __init__(self):
        self.input_log: list[dict] = []
        self.is_recording = False
        self.is_playing = False
        self.current_replay_index = 0

    def start_recording(self, seed: int):
        self.input_log = []
        self.is_recording = True
        log_header = {
            "game_version": "1.0.0",
            "mission_id": current_mission.id,
            "seed": seed,
            "timestamp": time.time(),
            "ticks": 0
        }

    def record_tick(self, tick: int, inputs: list):
        if self.is_recording:
            self.input_log.append({"tick": tick, "inputs": inputs})

    def start_playback(self, replay_data: dict):
        self.is_playing = True
        self.current_replay_index = 0
        # Reset RNG to recorded seed
        rng.restore_state(replay_data["seed"], 0)
        # Load mission
        load_mission(replay_data["mission_id"])

    def get_input_for_tick(self, tick: int) -> list:
        if self.is_playing and self.current_replay_index < len(self.input_log):
            entry = self.input_log[self.current_replay_index]
            if entry["tick"] == tick:
                self.current_replay_index += 1
                return entry["inputs"]
        return []

    def save_replay(self, filename: str):
        replay_file = {
            "header": self.log_header,
            "inputs": self.input_log
        }
        save_json(filename, replay_file)

    def load_replay(self, filename: str) -> dict:
        return load_json(filename)
```

---

## 5. Determinism Violations Checklist

| Violation | Impact | Detection |
|-----------|--------|-----------|
| Using `Time.time` / `DateTime.Now` | RNG or timer depends on real time | Automated test compares two runs |
| Using system random (non-seeded) | Different random results | Code review |
| Iterating over unsorted hash | Order varies per run | Use sorted data structures |
| Float math across platforms | Slight value differences | Fixed-point conversion |
| Physics engine non-determinism | Different collision outcomes | Deterministic physics config |
| Event listener registration order | Different dispatch order | Sort listeners by priority + ID |
| Thread race condition | Non-deterministic state | Mutex-protected state, deterministic threading model |
| External input mid-tick | State depends on input timing | Buffer all input, process at tick start |

---

## 6. Determinism Test Framework

### 6.1 Test Methodology

```python
def test_determinism():
    # Run 1
    seed = 12345
    state_1 = run_simulation(seed, input_sequence)

    # Run 2 (same seed, same input)
    state_2 = run_simulation(seed, input_sequence)

    # Compare
    assert state_1 == state_2, "Simulation not deterministic!"

    # Run 3 (different seed)
    state_3 = run_simulation(54321, input_sequence)
    assert state_1 != state_3, "Seed should change output!"

def run_simulation(seed, inputs) -> bytes:
    rng = DeterministicRNG(seed)
    world = create_test_world()

    for tick_inputs in inputs:
        process_inputs(world, tick_inputs)
        run_simulation_tick(world)

    return serialize_world_state(world)
```

### 6.2 Acceptance Criteria

| Criterion | Threshold |
|-----------|-----------|
| Same seed + same input → same state | 100% identical bytes |
| Different seed → different state | ≥ 99% probability |
| Same seed across platforms | Same result on Windows, macOS, Linux |
| Same seed across frame rates | Same result at 30 FPS and 60 FPS |
| Replay from recorded input | ±0 ticks drift after 10,000 ticks |

---

## 7. Replay Debug Tools

| Tool | Description |
|------|-------------|
| **Tick-by-tick playback** | Step through replay one tick at a time |
| **Speed control** | 0.5×, 1×, 2×, 10× replay speed |
| **State comparison** | Compare entity states between two replay runs at same tick |
| **Divergence detector** | Auto-stop when replay diverges from recorded state |
| **Input visualizer** | Show recorded inputs overlaid on timeline |
| **Replay export** | Export replay as video (for bug reports) |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Game loop architecture |
| `02_Update_Pipeline.md` | Fixed update order essential for determinism |
| `03_Entity_Processing.md` | Entity update within deterministic framework |
