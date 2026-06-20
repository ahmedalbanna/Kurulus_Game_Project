# Runtime State — Entity & System State Management

> **Description:** Defines how individual entities (units, buildings, heroes) and subsystems (economy, AI, objectives) manage their runtime state during gameplay. Covers entity FSM implementation, state synchronization, and debugging.

---

## 1. Entity Runtime State Design

Each entity in the game world has a **runtime state component** that tracks:

```
EntityRuntimeState
│
├── identity (static ref)
│   ├── instance_id: unique per session
│   ├── definition_id: links to static data
│   └── owner: player / ai_faction_1 / ai_faction_2
│
├── transform
│   ├── position (Vector3)
│   ├── rotation (float, yaw only for units)
│   └── scale (float, usually 1.0)
│
├── survival
│   ├── current_hp
│   ├── max_hp (from static data + modifiers)
│   ├── current_morale
│   ├── max_morale
│   ├── stamina / mana (heroes only)
│   └── status_effects[]
│
├── FSM state
│   ├── current_state: enum (IDLE, MOVE, ATTACK, ...)
│   ├── previous_state: enum
│   ├── state_timer: float (time in current state)
│   └── state_data: dictionary (state-specific payload)
│
├── command
│   ├── current_order: Order
│   ├── order_queue: Order[]
│   ├── target_entity: instance_id?
│   └── target_position: Vector3?
│
├── combat
│   ├── attack_timer: float (cooldown remaining)
│   ├── targets_in_range: instance_id[]
│   ├── current_target: instance_id?
│   └── threat_list: { instance_id: threat_score }
│
├── formation
│   ├── formation_id: string?
│   ├── formation_slot: int
│   ├── formation_offset: Vector2
│   └── formation_leader: instance_id?
│
├── stealth
│   ├── is_stealthed: bool
│   ├── stealth_zone_id: string?
│   ├── detection_level: float (0.0 = hidden, 1.0 = detected)
│   └── detected_by: instance_id[]
│
├── flags
│   ├── is_selected: bool
│   ├── is_alive: bool
│   ├── is_garrisoned: bool
│   ├── is_building: bool
│   ├── is_fleeing: bool
│   └── is_under_construction: bool (buildings)
│
└── metadata
    ├── created_at: float (game time)
    ├── created_by: string (building/ability that spawned this)
    └── tags: string[]
```

---

## 2. Entity FSM Implementation Pattern

### 2.1 State Transition Table (Infantry Unit Example)

| Current State | Input/Event | Guard Condition | Next State | Side Effects |
|---------------|-------------|-----------------|------------|-------------|
| IDLE | Move order | is_alive | MOVE | Set path, play move anim |
| IDLE | Attack order | is_alive, target_in_range | ATTACK | Set target, play attack anim |
| IDLE | Ability order | is_alive, ability_ready | ABILITY_USE | Activate ability |
| IDLE | Enter stealth zone | is_alive, zone_available | STEALTH | Fade out, play stealth SFX |
| MOVE | Destination reached | - | IDLE | Clear path, play idle anim |
| MOVE | Enemy in range | auto_attack_enabled | ATTACK | Stop moving, engage target |
| MOVE | Damage taken | hp < 50% | FLEE (if morale broken) | Play hit react, re-evaluate |
| MOVE | New order received | - | MOVE (repath) | Recalculate path |
| ATTACK | Target killed | - | IDLE | Clear target, look for next |
| ATTACK | Target out of range | - | MOVE | Chase target |
| ATTACK | Ability ready | - | ABILITY_USE | Interrupt attack for ability |
| STEALTH | Enemy detection | detection_level >= 1.0 | IDLE | Reveal, play detected SFX |
| STEALTH | Attack order | - | ATTACK (break stealth) | Reveal, ambush bonus |
| FLEE | Morale recovered | morale > break_threshold | IDLE | Stop fleeing |
| FLEE | Cornered / no escape | - | ATTACK (desperation) | Fight to the death |

### 2.2 State Handler Interface

```python
# Pseudocode — each entity's FSM state handler

class EntityStateHandler:
    def on_enter(self, entity, from_state):
        """Called when entering this state"""
        pass

    def on_update(self, entity, delta_time):
        """Called every frame while in this state"""
        pass

    def on_exit(self, entity, to_state):
        """Called when leaving this state"""
        pass

    def can_transition_to(self, entity, target_state) -> bool:
        """Guard check — can we transition to target_state?"""
        return True


# Concrete example: MOVE state
class MoveStateHandler(EntityStateHandler):
    def on_enter(self, entity, from_state):
        entity.animator.play("run")
        entity.pathfinder.set_destination(entity.command.target_position)

    def on_update(self, entity, delta_time):
        if entity.pathfinder.has_reached_destination():
            entity.fsm.transition_to(IDLE)
            return

        entity.pathfinder.update(delta_time)
        entity.transform.position = entity.pathfinder.current_position

        # Auto-attack check
        nearest_enemy = find_nearest_enemy(entity)
        if nearest_enemy and distance(entity, nearest_enemy) <= entity.attack_range:
            if entity.static_data.auto_attack:
                entity.command.current_target = nearest_enemy
                entity.fsm.transition_to(ATTACK)

    def on_exit(self, entity, to_state):
        entity.animator.stop("run")

    def can_transition_to(self, entity, target_state) -> bool:
        # Can always stop moving
        return True
```

---

## 3. Building Runtime State

| Property | Type | Description |
|----------|------|-------------|
| `current_hp` | float | Current HP (max from static data × tier) |
| `current_tier` | int | 1, 2, or 3 |
| `construction_progress` | float | 0.0-1.0 (during construction) |
| `upgrade_progress` | float | 0.0-1.0 (during upgrade) |
| `is_operational` | bool | True when construction complete and not upgrading |
| `production_queue` | Queue | Items being produced (unit/upgrade) |
| `production_timer` | float | Remaining time for current production |
| `garrisoned_units` | instance_id[] | Units inside (for garrison-capable buildings) |
| `resource_storage` | dict | Current stored resources (for resource buildings) |
| `work_assignment` | instance_id[] | Workers assigned (for economic buildings) |

### 3.1 Building FSM

```
RUBBLE
  │
  ▼
UNDER_CONSTRUCTION (construction_progress 0.0 → 1.0)
  │
  ▼
OPERATIONAL (can produce, garrison, etc.)
  │
  ├── UPGRADING (upgrade_progress 0.0 → 1.0 → back to OPERATIONAL at new tier)
  │
  ├── PRODUCING (actively producing unit/research)
  │
  └── DESTROYED (hp <= 0)
        │
        ▼
      RUBBLE (remains, can be cleared)
```

---

## 4. Subsystem Runtime State

### 4.1 Economy State

```json
{
  "resources": {
    "resource_gold": {"current": 340, "capacity": 500, "income_rate": 2.5},
    "resource_wood": {"current": 125, "capacity": 400, "income_rate": 1.8},
    "resource_food": {"current": 78, "capacity": 300, "income_rate": -0.5, "decay_active": true},
    "resource_iron": {"current": 42, "capacity": 200, "income_rate": 0.0}
  },
  "population": {"current": 18, "max": 30, "used_by_units": 15, "used_by_buildings": 3},
  "income_rates": {
    "per_worker": {"gold": 1.0, "wood": 1.0, "food": 1.5},
    "per_building": {"gold": 5.0, "wood": 3.0}
  },
  "pending_transactions": [
    {"type": "deduct", "resource": "gold", "amount": 50, "reason": "recruit_unit"}
  ]
}
```

### 4.2 Objective State

```json
{
  "objectives": [
    {
      "id": "obj_01_capture_keep",
      "state": "active",
      "progress": 0.6,
      "progress_text": "60% controlled",
      "started_at": 45.0,
      "completed_at": null,
      "children": ["obj_01a_breach_gate", "obj_01b_eliminate_garrison"]
    },
    {
      "id": "obj_01a_breach_gate",
      "state": "completed",
      "completed_at": 120.5
    },
    {
      "id": "obj_01b_eliminate_garrison",
      "state": "active",
      "remaining_enemies": 3
    }
  ],
  "primary_objective": "obj_01_capture_keep",
  "all_completed": false,
  "defeat_triggered": false
}
```

### 4.3 AI State (Per Faction)

```json
{
  "faction_id": "byzantine_ai",
  "global_state": "expand",
  "resources": { /* similar to economy state */ },
  "owned_units": ["instance_042", "instance_043", ...],
  "owned_buildings": ["instance_100", ...],
  "current_plan": {
    "type": "expand_to_zone",
    "target_zone": "zone_b3",
    "priority": 0.8,
    "steps": [
      {"action": "build_barracks", "position": {"x": 30, "z": 25}},
      {"action": "train_units", "count": 5, "unit_type": "unit_spearman"},
      {"action": "move_to_zone", "zone_id": "zone_b3"}
    ]
  },
  "threat_evaluation": {
    "player": {"threat_level": 0.6, "distance": 40.0, "last_contact_time": 12.0},
    "mongol_ai": {"threat_level": 0.2, "distance": 80.0, "last_contact_time": 0.0}
  }
}
```

---

## 5. State Synchronization

### 5.1 Entity State Dirty Tracking

For performance, entity state changes use a dirty flag system:

```python
class EntityRuntimeState:
    def __init__(self):
        self._dirty_flags = set()

    def mark_dirty(self, field_path: str):
        self._dirty_flags.add(field_path)

    def get_and_clear_dirty(self) -> set:
        flags = self._dirty_flags.copy()
        self._dirty_flags.clear()
        return flags

    @property
    def hp(self):
        return self._hp

    @hp.setter
    def hp(self, value):
        if self._hp != value:
            self._hp = value
            self.mark_dirty("hp")
```

### 5.2 Synchronization Targets

| Consumer | Polling Method | Frequency | Trigger |
|----------|---------------|-----------|---------|
| UI (health bars) | Dirty flag pull | Per frame | When position or HP changes |
| UI (selection panel) | Event-driven | On selection change | SelectionChanged event |
| Minimap | Dirty flag pull | 4x per second | When position changes |
| AI sensor | Spatial query | 2x per second | Independent |
| Save system | Full snapshot | On save trigger | Save event |
| Audio emitter | Event-driven | On event | Relevant events |

---

## 6. Performance Budget

| Metric | Target | Notes |
|--------|--------|-------|
| Max tracked entities | 200 units + 50 buildings | Soft cap per mission |
| Entity state memory | ~256 bytes per entity | Shared static data excluded |
| State update cost | < 0.5 µs per entity per frame | Simple FSM update |
| FSM transition cost | < 2 µs per transition | Lookup + side effects |
| State query (by type) | < 1 µs | Pre-built index |
| Serialization time | < 10 ms for full snapshot | On background thread |

---

## 7. Debugging Runtime State

| Tool | Description |
|------|-------------|
| **Entity Inspector** | Select entity → view all runtime fields in real-time |
| **State Machine Viewer** | Visual FSM for selected entity: current state highlighted, transition log |
| **State History** | Rolling buffer of last 50 state transitions per entity |
| **Bulk State Editor** | Dev tool to modify entity fields (e.g., set HP to 999) |
| **State Comparison** | Compare two entities' state side by side |
| **State Breakpoint** | "Break when entity X enters state FLEE" |
| **State Statistics** | Heat map: most common state per entity type, average time in each state |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | State machine architecture overview |
| `02_Game_States.md` | Application/scene level states |
| `03_Persistent_State.md` | Cross-session state |
| `04_Simulation_Loop/03_Entity_Processing.md` | How entities are updated each frame |
