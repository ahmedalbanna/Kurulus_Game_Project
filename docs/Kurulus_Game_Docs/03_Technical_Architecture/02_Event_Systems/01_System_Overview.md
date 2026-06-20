# Event Systems — System Overview

> **Description:** Defines the event-driven architecture that enables decoupled communication between *Kuruluş* game systems. Events are the backbone of gameplay reactivity — from a unit dying to an ambush being triggered to a building completing construction.

---

## 1. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Decoupled systems** | Combat, UI, AI, Audio never reference each other directly — they communicate through events |
| 2 | **Predictable event flow** | Every event has a clear lifecycle: emit → process → (optional) cancel → conclude |
| 3 | **Debug transparency** | Full event log available in debug builds: "what happened, when, to whom" |
| 4 | **Performance** | Event dispatch is O(n) where n = number of listeners. Priority system prevents listener storms |
| 5 | **Extensibility** | New systems can subscribe to existing events without modifying the event source |

---

## 2. Event Bus Architecture

```
                  ┌─────────────────────────────────────┐
                  │           EVENT BUS                  │
                  │   (Global Message Router)             │
                  │                                      │
                  │  ┌──────────────────────────────┐    │
                  │  │      Event Queue              │    │
                  │  │  ┌───┐ ┌───┐ ┌───┐ ┌───┐    │    │
                  │  │  │ E │ │ E │ │ E │ │...│    │    │
                  │  │  └───┘ └───┘ └───┘ └───┘    │    │
                  │  └──────────────────────────────┘    │
                  │                                      │
                  │  ┌──────────────────────────────┐    │
                  │  │    Listener Registry           │    │
                  │  │  • EventType → [Listener*]    │    │
                  │  │  • Priority sorting            │    │
                  │  │  • One-shot / persistent       │    │
                  │  └──────────────────────────────┘    │
                  │                                      │
                  │  ┌──────────────────────────────┐    │
                  │  │    Event Data Pool             │    │
                  │  │  (Object pooling for events)   │    │
                  │  └──────────────────────────────┘    │
                  └─────────────────────────────────────┘
                              │              ▲
                  ┌───────────┘              │
                  ▼                          │
          ┌───────────────┐       ┌──────────────────┐
          │  Event Sources │       │ Event Listeners   │
          │  (Emitters)    │       │ (Subscribers)     │
          │                │       │                   │
          │ • Game systems  │       │ • UI updates      │
          │ • AI decisions  │       │ • Audio playback  │
          │ • Player input  │       │ • Analytics       │
          │ • Timer/coroutine│      │ • Achievement     │
          │ • Network       │       │ • Save triggers   │
          │ • Script/triggers│      │ • Debug logging   │
          └────────────────┘       └──────────────────┘
```

---

## 3. Event Lifecycle

```
  ┌──────────┐
  │  CREATE   │  Event object allocated from pool, fields populated
  └─────┬────┘
        ▼
  ┌──────────┐
  │   EMIT    │  Event pushed to the Event Bus queue
  └─────┬────┘
        ▼
  ┌──────────┐
  │  PRE-     │  Global pre-processors run (logging, metrics, filters)
  │ PROCESS   │
  └─────┬────┘
        ▼
  ┌──────────┐
  │ DISPATCH  │  Event routed to all registered listeners (in priority order)
  │           │
  │           ├── Listener 1 (priority 100) → processes → may mark consumed
  │           ├── Listener 2 (priority 50)  → processes
  │           ├── Listener 3 (priority 0)   → processes
  │           └── ...
  └─────┬────┘
        ▼
  ┌──────────┐
  │  POST-    │  Global post-processors run (cleanup, event chaining)
  │ PROCESS   │
  └─────┬────┘
        ▼
  ┌──────────┐
  │  RECYCLE  │  Event returned to object pool (or garbage collected)
  └──────────┘
```

---

## 4. Event Categories

| Category | Examples | Dispatch Mode |
|----------|----------|---------------|
| **Input Events** | `KeyDown`, `MouseClick`, `SelectionBox`, `CommandIssue` | Immediate |
| **Gameplay Events** | `UnitSpawned`, `UnitKilled`, `BuildingCompleted`, `ResourceChanged`, `MoraleBroken` | Immediate |
| **Combat Events** | `DamageDealt`, `HitLanded`, `CritTriggered`, `ArmorPierced`, `StatusEffectApplied` | Immediate |
| **Ambush/Stealth Events** | `StealthEntered`, `StealthDetected`, `AmbushTriggered`, `AmbushEvaded` | Immediate |
| **AI Events** | `AIStateChanged`, `AITargetAcquired`, `AIThreatEvaluated`, `AIPlanGenerated` | Immediate |
| **UI Events** | `SelectionChanged`, `PanelOpened`, `TooltipRequested`, `NotificationDismissed` | Immediate |
| **Audio Events** | `PlaySFX`, `PlayVoiceover`, `StopMusic`, `SetMusicIntensity` | Immediate (fire-and-forget) |
| **System Events** | `GameLoaded`, `GameSaved`, `SceneTransition`, `DataReloaded`, `ApplicationQuit` | Immediate |
| **Timed Events** | `DelayedAction`, `RepeatingTimer`, `TickerEvent` | Queued / Scheduled |
| **Debug/Cheat Events** | `ConsoleCommand`, `ToggleDebugOverlay`, `SpawnUnit` | Immediate |

---

## 5. Event Data Structure (Logical)

```csharp
// Pseudocode — technology-agnostic event structure

class GameEvent {
    string        type;          // "UnitKilled", "BuildingCompleted", etc.
    EventCategory category;      // Gameplay, Combat, UI, Audio, System...
    int           priority;      // Higher = processed first (0-1000)
    float         timestamp;     // Game time when emitted

    // Event-specific payload
    Dictionary<string, object> data;

    // Lifecycle flags
    bool          consumed;      // If true, skip remaining lower-priority listeners
    bool          cancelled;     // If true, the action described by the event is cancelled

    // Pooling
    bool          isPooled;
    void          Reset();       // Clear all fields before returning to pool
}
```

---

## 6. Listener Registration

```python
# Pseudocode registration patterns

# Pattern 1: Class-based listener
class AchievementListener:
    def on_init(self, event_bus):
        event_bus.subscribe("UnitKilled", self.on_unit_killed, priority=50)
        event_bus.subscribe("BuildingCompleted", self.on_building_completed, priority=50)

    def on_unit_killed(self, event):
        # Check if first kill → unlock "First Blood" achievement
        ...
    def on_building_completed(self, event):
        ...

# Pattern 2: Lambda / inline (for simple cases)
event_bus.subscribe("ResourceChanged", lambda e: update_ui_resource_display(e.data))


# Pattern 3: One-shot listener (fires once, auto-unsubscribes)
event_bus.subscribe_one_shot("HeroLevelUp", lambda e: show_level_up_fx(e.data))
```

---

## 7. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Event creation latency | < 1 µs (from pool) | Object pooling required |
| Dispatch to 10 listeners | < 5 µs | |
| Dispatch to 100 listeners | < 50 µs | Use priority to avoid storms |
| Event pool pressure | < 1% GC overhead | Pre-allocate pool for hot events |
| Memory per pooled event | ~64 bytes | Minimize payload size |
| Maximum events per frame | 500 | Beyond this triggers warning in debug |

---

## 8. Debugging & Observability

| Tool | Description |
|------|-------------|
| **Event Log** | Circular buffer of last 1000 events. Time + type + source + payload summary |
| **Event Profiler** | Per-type histogram: emit count, avg dispatch time, total time |
| **Event Graph** | Visual graph showing event sources, subscribers, and chains |
| **Breakpoint on Event** | "Break when UnitKilled is emitted with data.unit_id = 'hero_osman'" |
| **Live Filter** | Show only events matching type pattern (e.g. `*Killed`, `Stealth*`) |
| **Stress Test** | Script that rapidly emits 10,000 events and measures system responsiveness |

---

## 9. Related Files

| File | Description |
|------|-------------|
| `02_Events_Catalog.md` | Complete catalog of all event types |
| `03_Trigger_Conditions.md` | Rule-based trigger system for events |
| `04_Event_Propagation.md` | Priority, chaining, cancellation, deferred events |
| `04_Simulation_Loop/02_Update_Pipeline.md` | Where events fit in the frame update |
