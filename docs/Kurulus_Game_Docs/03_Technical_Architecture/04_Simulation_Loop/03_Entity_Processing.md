# Entity Processing — Lifecycle & Update Architecture

> **Description:** Defines how entities (units, buildings, projectiles) flow through their lifecycle — from creation to destruction — and how they are updated within the simulation pipeline.

---

## 1. Entity Lifecycle

```
                    ┌──────────┐
                    │  CREATE   │  Entity instance allocated, initialized
                    └─────┬────┘
                          │
                    ┌─────▼────┐
                    │  SPAWN   │  Entity added to world, takes first tick
                    └─────┬────┘
                          │
                    ┌─────▼────┐
              ┌─────│  ACTIVE  │──────┐
              │     └─────┬────┘      │
              │           │           │
         ┌────▼────┐     │    ┌───────▼──────┐
         │  PAUSED  │     │    │  GARRISONED  │
         │ (stasis) │     │    │ (in building)│
         └────┬────┘     │    └───────┬──────┘
              │           │           │
              └───────────┘           │
                          │           │
                    ┌─────▼───────────▼┐
                    │  DEACTIVATED     │  Removed from active update loop
                    │  (dead / dead)   │
                    └─────┬────────────┘
                          │
                    ┌─────▼────┐
                    │  DESTROY │  Memory freed / returned to pool
                    └──────────┘
```

---

## 2. Entity Update Strategy

### 2.1 Update Modes

| Mode | Description | Used By |
|------|-------------|---------|
| **Full Update** | Entity ticked every simulation tick | Active units, buildings |
| **Throttled Update** | Entity ticked every N ticks (N ≥ 2) | Idle units far from player, ambient entities |
| **Stasis** | Entity exists but not ticked | Garrisoned units, paused entities |
| **Event-Only** | No tick updates; only responds to events | Dead units (ragdoll/corpse), decorative buildings |

### 2.2 Throttling Strategy

| Entity Type | Update Rate | Condition |
|-------------|-------------|-----------|
| Selected units | Every tick | Player attention |
| Units in combat | Every tick | Combat engagement |
| Visible enemy units | Every tick | In player's view |
| Idle friendly units > 50m from camera | Every 3rd tick | Distance-based throttling |
| Idle enemy units > 100m from any unit | Every 5th tick | Distance-based throttling |
| Ambient units (deer, birds) | Every 10th tick | Non-critical |
| Corpse/remains | Every 60th tick | Only for decay timer |

### 2.3 Update Budget Scaling

```
Entity Count  |  Budget per Tick  |  Notes
─────────────────────────────────────────────────
< 50          |  < 0.5 ms         |  Early game
50 - 100      |  < 2.0 ms         |  Mid game
100 - 200     |  < 5.0 ms         |  Late game (with throttling)
200+          |  < 8.0 ms         |  Maximum capacity (heavy throttling)
```

---

## 3. Entity Component Update Model

Each entity can have multiple **components** that are updated during specific phases:

```
Entity "unit_swordsman_01"
│
├── IdentityComponent    (static refs, always active)
├── TransformComponent   (position, rotation, always active)
├── SurvivalComponent    (HP, morale, updated phase 6)
├── FSMComponent         (current state, updated phase 3 AI)
├── MovementComponent    (speed, path, updated phase 4)
├── CombatComponent      (attack stats, updated phase 6)
├── StealthComponent     (detection, updated phase 5)
├── StatusEffectComponent (active effects, updated phase 6)
├── CommandComponent     (order queue, updated phase 1)
└── FormationComponent   (formation data, updated phase 4)
```

### 3.1 Component Update Assignment

```
PHASE 1 (INPUT):    CommandComponent  — process orders
PHASE 3 (AI):       FSMComponent      — state transitions
PHASE 4 (MOVEMENT): MovementComponent — path following
                    TransformComponent — position update
                    FormationComponent — slot cohesion
PHASE 5 (STEALTH):  StealthComponent  — detection updates
PHASE 6 (COMBAT):   CombatComponent   — attack/defense
                    SurvivalComponent — HP/morale changes
                    StatusEffectComponent — effect ticks
PHASE 7 (ECONOMY):  (buildings only) ProductionComponent
PHASE 11 (POST):    SurvivalComponent — death check
```

---

## 4. Entity Registration & Lookup

### 4.1 Entity Registry

```python
class EntityRegistry:
    def __init__(self):
        self.by_id: dict[str, Entity] = {}
        self.by_owner: dict[str, list[Entity]] = {"player": [], "ai_1": [], "ai_2": []}
        self.by_type: dict[str, list[Entity]] = {"unit": [], "building": [], "projectile": []}
        self.by_tag: dict[str, list[Entity]] = {}
        self.spatial_hash: SpatialHash = SpatialHash(cell_size=10.0)

    def register(self, entity):
        self.by_id[entity.instance_id] = entity
        self.by_owner[entity.owner].append(entity)
        self.by_type[entity.entity_type].append(entity)
        for tag in entity.tags:
            self.by_tag.setdefault(tag, []).append(entity)
        self.spatial_hash.insert(entity)

    def unregister(self, entity):
        self.by_id.pop(entity.instance_id, None)
        safe_remove(self.by_owner[entity.owner], entity)
        safe_remove(self.by_type[entity.entity_type], entity)
        for tag in entity.tags:
            safe_remove(self.by_tag.get(tag, []), entity)
        self.spatial_hash.remove(entity)

    def find_in_radius(self, position: Vector3, radius: float, filter_tag: str = None) -> list[Entity]:
        candidates = self.spatial_hash.query(position, radius)
        if filter_tag:
            return [e for e in candidates if filter_tag in e.tags]
        return candidates
```

### 4.2 Spatial Hash Grid

```
Grid cell size: 10m × 10m
Grid range: covers entire mission map (e.g., 200m × 200m → 20 × 20 = 400 cells)

Spatial hash query:
  position + radius → get overlapping cells → collect entities → filter by distance

Performance:
  Query radius 5m  →  < 0.01 ms (typically 0-5 candidates)
  Query radius 20m →  < 0.05 ms (typically 5-30 candidates)
```

---

## 5. Spawning Entities

```python
def spawn_unit(definition_id: str, owner: str, position: Vector3) -> str:
    # 1. Look up static definition
    defn = data_manager.get_unit(definition_id)

    # 2. Generate instance ID
    instance_id = generate_id("unit")

    # 3. Allocate entity object (from pool if available)
    entity = entity_pool.allocate()

    # 4. Initialize from definition
    entity.instance_id = instance_id
    entity.definition_id = definition_id
    entity.owner = owner
    entity.transform.position = position
    entity.survival.hp = defn.base_stats.hp
    entity.survival.morale = defn.base_stats.morale_base
    # ... initialize all other components from defn

    # 5. Register in world
    entity_registry.register(entity)

    # 6. Set initial FSM state
    entity.fsm.transition_to(IDLE)

    # 7. Emit spawn event
    event_bus.emit("Game_UnitSpawned", {
        "unit_id": definition_id,
        "instance_id": instance_id,
        "position": position,
        "owner": owner
    })

    return instance_id
```

---

## 6. Destroying Entities

```python
def kill_entity(instance_id: str, killer_id: str = None, damage_type: str = None):
    entity = entity_registry.by_id.get(instance_id)
    if not entity or not entity.flags.is_alive:
        return

    # 1. Mark dead
    entity.flags.is_alive = False

    # 2. Transition to dead state
    entity.fsm.transition_to(DEAD)

    # 3. Emit death event
    event_bus.emit("Game_UnitKilled", {
        "unit_id": entity.definition_id,
        "instance_id": instance_id,
        "killer_id": killer_id,
        "position": entity.transform.position,
        "damage_type": damage_type
    })

    # 4. Schedule removal (deferred to POST_TICK)
    death_registry.schedule_removal(instance_id, delay_seconds=2.0)  # Corpse linger time

def remove_entity(instance_id: str):
    entity = entity_registry.by_id.get(instance_id)
    if not entity:
        return

    # 1. Unregister from all indices
    entity_registry.unregister(entity)

    # 2. Emit removal event
    event_bus.emit("System_EntityRemoved", {"instance_id": instance_id})

    # 3. Return to pool
    entity.reset()
    entity_pool.free(entity)
```

---

## 7. Entity Pooling

```python
class EntityPool:
    def __init__(self, initial_size=300):
        self.pool = [Entity() for _ in range(initial_size)]
        self.active_count = 0

    def allocate(self) -> Entity:
        if self.pool:
            entity = self.pool.pop()
            self.active_count += 1
            return entity
        else:
            self.active_count += 1
            return Entity()  # Fallback: allocate new (expand pool)

    def free(self, entity: Entity):
        entity.reset()
        self.pool.append(entity)
        self.active_count -= 1

    @property
    def utilization(self) -> float:
        total = len(self.pool) + self.active_count
        return self.active_count / total if total > 0 else 0.0
```

---

## 8. Performance: Entity Update Loop

```python
# Pseudocode — simplified per-tick entity update

def update_entities(phase: int):
    for entity in entity_registry.by_id.values():
        if not entity.is_active_for_phase(phase):
            continue  # Throttled: skip this tick

        if not entity.flags.is_alive:
            continue  # Dead entities don't update

        # Update only the components relevant to this phase
        entity.update_phase(phase, fixed_dt)
```

---

## 9. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Game loop architecture |
| `02_Update_Pipeline.md` | Phase ordering where entities update |
| `04_Determinism.md` | Ensuring entity updates are deterministic |
