# Event Propagation — Priority, Chaining & Deferred Events

> **Description:** Defines how events flow through the listener chain, how they can be modified, cancelled, chained into new events, and deferred across frames.

---

## 1. Dispatch Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Immediate** | Event processed synchronously during emit. All listeners run before emit() returns. | Most gameplay events |
| **Queued** | Event added to frame-end queue. Processed in batch after all systems update. | UI updates, non-critical notifications |
| **Deferred** | Event scheduled to fire after a delay (game-time or real-time). | Delayed damage, trap triggers |
| **Conditional** | Event attempts to fire but is cancelled if conditions no longer hold. | Morale break — unit might rally before panic fires |

---

## 2. Listener Priority System

### 2.1 Priority Ranges

| Priority | Role | Example Listener |
|----------|------|-----------------|
| **1000-900** | Cancellers — can cancel before others process | Combat system: "Unit can't die, it's essential" |
| **899-700** | Transformers — modify event data | Difficulty scaler: "Reduce damage by 20%" |
| **699-500** | Gameplay reactors — respond to events | AI, Morale, Ability systems |
| **499-300** | UI updaters — refresh display | HUD, Minimap, Health bars |
| **299-100** | Audio/VFX — play sounds, effects | Audio Manager, VFX System |
| **99-0** | Loggers — record without side effects | Analytics, Debug log, Stats |

### 2.2 Priority Execution Example

```
Event: Combat_DamageDealt
  Data: { damage: 50, target: "unit_archer_01" }

  Listener "DifficultyScaler"  (priority 800):
    damage *= 0.8  (50 → 40 on "easy" difficulty)
    → modifies event.data.damage

  Listener "ArmorSystem"  (priority 700):
    damage -= target.armor  (40 → 35)
    → modifies event.data.damage

  Listener "MoraleSystem"  (priority 600):
    if damage > 20: reduce target's morale
    → calls event_bus.emit("Game_UnitMoraleChanged")

  Listener "HealthBar"  (priority 400):
    update health bar display with new value

  Listener "PlayHitSound"  (priority 200):
    play sfx_hit based on damage type

  Listener "Analytics"  (priority 50):
    log: "Damage dealt: 35, target: unit_archer_01"
```

---

## 3. Event Consumption & Cancellation

### 3.1 Consumed Events

When a listener marks an event as **consumed**, lower-priority listeners are skipped:

```
Event: Input_KeyDown (key: "Escape")

  Listener "MenuManager" (priority 900):
    if menu is open → consume event (prevent gameplay from hearing it)

  Listener "GameplayInput" (priority 500):
    NOT REACHED (event was consumed)
```

### 3.2 Cancelled Events

When a listener marks an event as **cancelled**, the action the event describes is reverted:

```
Event: Game_UnitKilled (unit: "hero_osman", killer: "enemy_01")

  Listener "EssentialUnitProtection" (priority 950):
    if unit has tag "essential":
      cancel event → unit survives with 1 HP
      emit "Combat_DamageDealt" with damage clamped

  Result: Hero Osman does NOT die. Instead, he survives at 1 HP.
```

### 3.3 Rules for Consumption & Cancellation

| Rule | Description |
|------|-------------|
| Only first N listeners can consume | Only listeners with priority >= 500 can mark consumed |
| Cancellation requires explicit handling | Systems must check `event.cancelled` before applying irreversible effects |
| Cancelling a "result" event | Cancelling `UnitKilled` prevents death, but the damage already happened |
| Consumed ≠ cancelled | A consumed event didn't reach low-priority listeners; a cancelled event's meaning was negated |

---

## 4. Event Chaining

Events can trigger new events as a side effect of processing:

```
Combat_DamageDealt(damage: 50, target: "warrior_01")
  │
  ├── MoraleSystem: damage > 30 → emit Game_UnitMoraleChanged(morale: -15)
  │                              │
  │                              └── CombatSystem: morale < 25 → emit Game_UnitMoraleBroken(unit: "warrior_01")
  │                                                              │
  │                                                              └── AISystem: order retreat → emit AI_StateChanged(state: "retreating")
  │
  ├── HealthBar: HP now 20% → emit UI_NotificationQueued(type: "low_health")
  │
  └── Analytics: log damage event
```

### 4.1 Chaining Safety Rules

| Rule | Rationale |
|------|-----------|
| **Max chain depth**: 10 | Prevent infinite loops |
| **Loop detection**: Event type + source already in call stack | Detect and break cycles |
| **Recursive events**: If an event listener emits the same event type, it's queued for next frame | Prevent stack overflow |
| **Chain logging**: Debug builds log the full event chain | Debugging complex cascades |

---

## 5. Deferred Events

### 5.1 Scheduled Event Syntax

```json
{
  "schedule_id": "deferred_damage_01",
  "trigger_at": {
    "type": "delay",
    "game_seconds": 2.0
  },
  "event": {
    "type": "Combat_DamageDealt",
    "data": {
      "target_id": "captain_01",
      "damage": 15,
      "damage_type": "poison",
      "source_id": "ability_poison_arrow"
    }
  }
}
```

### 5.2 Scheduling Triggers

| Schedule By | Example |
|-------------|---------|
| **Delay (game seconds)** | "Fire event in 3.0 game seconds" |
| **Delay (real seconds)** | "Fire event in 1.5 real seconds" (for UI) |
| **At game time** | "Fire at mission time 0:00" (mission start) |
| **After X frames** | "Fire after 60 frames" |
| **On condition** | "Fire when condition becomes true (check every 0.5s)" |
| **Repeating interval** | "Fire every 5 game seconds, indefinitely" |
| **Repeating count** | "Fire every 2 seconds, 3 times total" |

### 5.3 Deferred Event Manager

```python
# Pseudocode
class DeferredEventManager:
    pending_events: PriorityQueue  # sorted by trigger time

    def update(delta_time: float):
        current_time = get_game_time()
        while pending_events.peek().trigger_time <= current_time:
            event = pending_events.pop()
            event_bus.emit(event.event_type, event.data)

    def schedule(event_type: str, data: dict, delay: float):
        deferred = DeferredEvent(
            event_type=event_type,
            data=data,
            trigger_time=get_game_time() + delay
        )
        pending_events.push(deferred)
```

---

## 6. Event Filtering & Throttling

### 6.1 Throttle Patterns

| Pattern | Description | Example |
|---------|-------------|---------|
| **Throttle** | Max N emissions per second | `Game_UnitMoved`: throttle to 10/sec per unit |
| **Debounce** | Wait for pause, then fire last | `UI_SelectionChanged`: debounce 100ms |
| **Rate Limit** | Block if > N in last M seconds | `Audio_PlaySFX`: max 5 per frame |
| **Coalesce** | Merge multiple into one | Multiple `ResourceChanged` with same type → one event |

### 6.2 Filter Rules

```
Event "Game_UnitMoved" emitted 60 times/second/unit
  │
  ├── ThrottleFilter: max 10/sec/unit
  │   ├── Allowed: event passes through
  │   └── Rejected: event silently dropped
  │
  ├── DistanceFilter: only emit if moved > 0.5m from last position
  │   ├── Allowed: significant movement
  │   └── Rejected: small movement → no event
  │
  └── Only spatial listeners receive filtered events
```

---

## 7. Cross-Frame Event Chains

Some event chains must span multiple frames or gameplay phases:

```
Frame 1:  Ambush_Prepared(ambush_id: "road_01", phase: "laying_trap")
              ↓
Frame 2:  (players units moving toward zone)
              ↓
Frame 3:  Ambush_Prepared(ambush_id: "road_01", phase: "waiting")
              ↓
Frame 4:  Game_UnitEnteredZone(unit: "player_swordsman", zone: "ambush_road_01")
              ↓
Frame 5:  [Trigger evaluation: conditions_met?]
              ↓
Frame 6:  Ambush_Triggered(ambush_id: "road_01")
              ↓
Frame 6:  Combat_DamageDealt(ambushers → victims)
              ↓
Frame 7:  UI_NotificationQueued("Ambush! Enemy units appear!")
```

---

## 8. Debugging Propagation

| Tool | Description |
|------|-------------|
| **Event Chain View** | Tree visualization showing parent → child events |
| **Priority Viewer** | For a given event type, show all listeners sorted by priority |
| **Cancellation Trace** | Highlight when/why an event was consumed or cancelled |
| **Deferred Queue View** | Show all pending deferred events with remaining time |
| **Performance Flame Chart** | Time spent in each listener during event dispatch |
| **Break on Event Chain** | "Break when EventChain depth > 5" |

---

## 9. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Event bus architecture |
| `02_Events_Catalog.md` | Complete event type reference |
| `03_Trigger_Conditions.md` | Rule-based triggers |
| `04_Simulation_Loop/02_Update_Pipeline.md` | Where deferred events are processed in the frame |
