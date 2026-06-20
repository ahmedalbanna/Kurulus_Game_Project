# Trigger Conditions — Rule-Based Event Triggers

> **Description:** Defines the declarative trigger system that translates game state changes into event emissions. Triggers are the "when X happens, do Y" backbone of missions, tutorials, ambushes, and scripted sequences.

---

## 1. Trigger Architecture

```
┌─────────────────────────────────────────────┐
│              TRIGGER ENGINE                   │
│                                               │
│  ┌────────────┐  ┌────────────────────────┐  │
│  │  Condition  │  │      Action            │  │
│  │  Evaluator  │  │      Executor          │  │
│  │             │  │                        │  │
│  │ • Evaluate  │  │ • Emit game event      │  │
│  │   conditions│  │ • Run script/sequence  │  │
│  │ • Track     │  │ • Show dialog          │  │
│  │   state     │  │ • Spawn units          │  │
│  │ • Check     │  │ • Modify world         │  │
│  │   interval  │  │ • Play audio           │  │
│  └──────┬──────┘  └───────────┬────────────┘  │
│         │                     │                 │
│         └─────────┬───────────┘                 │
│                   │                             │
│         ┌─────────▼───────────┐                 │
│         │  Trigger Registry    │                 │
│         │  • All registered    │                 │
│         │    triggers          │                 │
│         │  • Eval frequency    │                 │
│         │  • Priority order    │                 │
│         │  • Active/inactive   │                 │
│         │  • One-shot/repeat   │                 │
│         └─────────────────────┘                 │
└─────────────────────────────────────────────────┘
```

---

## 2. Trigger Definition Schema

```json
{
  "trigger_id": "trg_ambush_road_01",
  "enabled": true,
  "one_shot": true,
  "priority": 50,
  "cooldown_seconds": 0,

  "evaluate_on": [
    "Game_UnitEnteredZone",
    "Game_UnitMoved"
  ],

  "conditions": {
    "all_of": [
      {
        "type": "unit_in_zone",
        "params": {
          "zone_id": "ambush_zone_road_01",
          "unit_tag": "player"
        }
      },
      {
        "type": "game_time_greater_than",
        "params": { "seconds": 30.0 }
      },
      {
        "type": "ambush_not_triggered",
        "params": { "ambush_id": "ambush_road_01" }
      }
    ]
  },

  "actions": [
    {
      "type": "trigger_ambush",
      "params": {
        "ambush_id": "ambush_road_01",
        "trigger_position_from_event": true
      }
    },
    {
      "type": "play_audio",
      "params": { "sfx_id": "sfx_ambush_horn" }
    },
    {
      "type": "spawn_units",
      "params": {
        "unit_ids": ["unit_bandit_archer", "unit_bandit_archer"],
        "position": {"x": 25.0, "y": 0.0, "z": 30.0},
        "count": 2
      }
    },
    {
      "type": "show_dialog",
      "params": {
        "character_id": "hero_osman",
        "line_key": "dialog_ambush_01",
        "duration": 4.0
      }
    },
    {
      "type": "set_objective",
      "params": {
        "objective_id": "obj_defeat_ambush",
        "state": "active"
      }
    }
  ]
}
```

---

## 3. Condition Types Reference

### 3.1 Spatial Conditions

| Condition Type | Parameters | Description |
|---------------|------------|-------------|
| `unit_in_zone` | `zone_id`, `unit_tag?` | Any unit with matching tag is in zone |
| `unit_near_position` | `position`, `radius`, `unit_tag?`, `count?` | N units near a world position |
| `unit_entered_zone` | `zone_id`, `unit_tag?` | Fires exactly when a unit crosses zone boundary |
| `grid_cell_occupied` | `cell_x`, `cell_y`, `cell_z`, `unit_tag?` | Check if a specific grid cell is occupied |
| `distance_between` | `entity_a_id`, `entity_b_id`, `operator`, `value` | Compare distance between two entities |
| `building_at_location` | `position`, `radius`, `building_id?` | Check if building exists at location |

### 3.2 Game State Conditions

| Condition Type | Parameters | Description |
|---------------|------------|-------------|
| `game_time_greater_than` | `seconds` | Elapsed mission time > threshold |
| `game_time_between` | `min_seconds`, `max_seconds` | Time is within a window |
| `resource_amount` | `resource_type`, `operator`, `value` | Player resource comparison |
| `population_count` | `operator`, `value` | Population check |
| `objective_state` | `objective_id`, `state` | Check an objective's current state |
| `difficulty_level` | `difficulty` | `easy`, `medium`, `hard` |

### 3.3 Entity Conditions

| Condition Type | Parameters | Description |
|---------------|------------|-------------|
| `unit_alive` | `unit_id`/`instance_id` | Specific unit is alive |
| `unit_health_below` | `instance_id`, `percentage` | HP threshold check |
| `unit_count_with_tag` | `tag`, `owner`, `operator`, `value` | Count of units matching criteria |
| `building_exists` | `building_id`, `owner?` | Building has been constructed |
| `building_tier` | `instance_id`, `tier` | Building is at specific tier |
| `hero_has_ability` | `hero_id`, `ability_id` | Ability unlocked |

### 3.4 Meta Conditions

| Condition Type | Parameters | Description |
|---------------|------------|-------------|
| `all_of` | `conditions[]` | AND — all sub-conditions must pass |
| `any_of` | `conditions[]` | OR — at least one sub-condition passes |
| `none_of` | `conditions[]` | NOT — none of the sub-conditions pass |
| `trigger_fired` | `trigger_id` | Another trigger has already fired |
| `trigger_not_fired` | `trigger_id` | Another trigger has not yet fired |
| `random_chance` | `probability` (0.0-1.0) | Random roll — useful for variety |

---

## 4. Trigger Evaluation Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Polled** | Evaluated periodically (every N frames or every M seconds) | `unit_in_zone` — must check repeatedly |
| **Event-driven** | Evaluated only when a specific event fires | `unit_entered_zone` — fires exactly on event |
| **Hybrid** | Event-driven initial check, then polled for secondary conditions | Ambush trigger: on zone entry + time check |

### 4.1 Polling Frequency Guidelines

| Evaluation Frequency | Use Case | Performance Cost |
|---------------------|----------|-----------------|
| Every frame | Critical combat conditions | High — minimize |
| Every 0.5 sec | Unit position / zone checks | Medium |
| Every 1 sec | Resource / population checks | Low |
| Every 5 sec | Long-duration mission triggers | Very low |
| Once (one-shot) | Tutorial / introduction triggers | Negligible |

---

## 5. Trigger Priority & Conflict Resolution

| Priority Range | Purpose |
|---------------|---------|
| **100-90** | System-level triggers (save, auto-pause) |
| **89-70** | Mission-critical triggers (victory, defeat) |
| **69-50** | Combat/ambush triggers |
| **49-30** | Tutorial / guidance triggers |
| **29-10** | Atmospheric / cosmetic triggers |
| **9-0** | Debug / development triggers |

If two triggers fire in the same frame, the higher-priority trigger's actions execute first. Same-priority triggers execute in registration order (undefined — avoid relying on it).

---

## 6. Trigger Lifecycle

```
  INACTIVE (disabled)
      │
      ▼
   REGISTERED (loaded, waiting for evaluation)
      │
      ▼
   ACTIVE (evaluating conditions)
      │
      ├── Conditions met? ──► FIRING (executing actions)
      │                              │
      │                    ┌─────────┴────────┐
      │                    │                  │
      │               ONE_SHOT           REPEATABLE
      │                    │                  │
      │               DISPOSED          COOLDOWN
      │                                      │
      │                                   ACTIVE (re-enter)
      │
      └── Conditions not met → stay ACTIVE
```

---

## 7. Performance Budget

| Metric | Budget |
|--------|--------|
| Maximum active triggers | 100 per mission |
| Polled trigger evaluations per frame | < 20 |
| Max nested condition depth | 5 levels |
| Trigger registration cost | < 0.1 ms |
| Average evaluation cost (event-driven) | < 0.01 ms |
| Average evaluation cost (polled) | < 0.05 ms |

---

## 8. Debugging Triggers

| Tool | Description |
|------|-------------|
| **Trigger Inspector** | List all triggers with: ID, state, last evaluation result, next eval time |
| **Trigger History** | Log of every trigger firing with conditions snapshotted |
| **Manual Fire** | Console command `fire_trigger <trigger_id>` for testing |
| **Condition Override** | Force a condition to true/false via console |
| **Trigger Graph** | Visual graph of trigger → conditions → actions → events → triggers |

---

## 9. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Event bus architecture context |
| `02_Events_Catalog.md` | Full list of events that triggers can respond to |
| `04_Event_Propagation.md` | How triggered events propagate through the system |
