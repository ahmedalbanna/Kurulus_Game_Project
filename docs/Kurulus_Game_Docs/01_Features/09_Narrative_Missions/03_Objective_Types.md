# Objective Types — Complete Objective Reference

> **Description:** Catalog of every objective type used in *Kuruluş* missions. Each entry defines how the objective works, how progress is measured, and completion/failure conditions.

---

## 1. Objective Architecture

```json
{
  "objective_id": "obj_capture_keep",
  "type": "capture_point",
  "display_name_key": "obj_capture_keep_name",
  "description_key": "obj_capture_keep_desc",
  "category": "primary",       // primary | secondary | bonus | hidden
  "state": "inactive",          // inactive | active | completed | failed
  "progress": {
    "type": "percentage",
    "current": 0.0,
    "max": 100.0
  },
  "optional": false,
  "order": 1,
  "prerequisites": [],
  "next_objectives": ["obj_secure_courtyard"],
  "failure_condition": {
    "type": "hero_death",
    "params": {"hero_id": "hero_osman"}
  }
}
```

---

## 2. Objective Type Catalog

### `capture_point`

| Aspect | Detail |
|--------|--------|
| **Description** | Capture and hold a specific location |
| **Progress** | Percentage controlled (0-100%) |
| **Completion** | Control reaches 100% |
| **Failure** | Capture progress resets if enemies enter zone |
| **Parameters** | `zone_id`, `capture_time_seconds`, `required_units` |
| **Example** | "Capture the Byzantine keep" |

### `eliminate_all`

| Aspect | Detail |
|--------|--------|
| **Description** | Kill all enemy units of a specific type/faction |
| **Progress** | Remaining count |
| **Completion** | Zero enemies remaining |
| **Parameters** | `target_tag`, `faction_id`, `count` |
| **Example** | "Eliminate all Byzantine defenders" |

### `eliminate_target`

| Aspect | Detail |
|--------|--------|
| **Description** | Kill a specific named target (hero, commander) |
| **Progress** | Target alive/dead |
| **Completion** | Target HP ≤ 0 |
| **Failure** | Target escapes (leaves map) |
| **Parameters** | `target_id`, `must_be_killed_by_hero` |
| **Example** | "Defeat Byzantine Commander Nikephoros" |

### `survive`

| Aspect | Detail |
|--------|--------|
| **Description** | Survive for a duration or until allies arrive |
| **Progress** | Time elapsed / total time |
| **Completion** | Timer expires |
| **Failure** | All heroes die / critical building destroyed |
| **Parameters** | `duration_seconds`, `defeat_if_heroes_die` |
| **Example** | "Hold the pass for 5 minutes" |

### `escort`

| Aspect | Detail |
|--------|--------|
| **Description** | Escort a friendly unit to a destination |
| **Progress** | Distance to destination |
| **Completion** | Escorted unit reaches destination |
| **Failure** | Escorted unit dies |
| **Parameters** | `escortee_id`, `destination_zone_id` |
| **Example** | "Escort the supply caravan to the village" |

### `build`

| Aspect | Detail |
|--------|--------|
| **Description** | Construct a specific building |
| **Progress** | Construction percentage |
| **Completion** | Building operational |
| **Parameters** | `building_id`, `count` (optional) |
| **Example** | "Build a barracks and train 5 swordsmen" |

### `gather_resource`

| Aspect | Detail |
|--------|--------|
| **Description** | Gather a specific amount of a resource |
| **Progress** | Amount gathered / target |
| **Completion** | Target reached |
| **Parameters** | `resource_type`, `amount` |
| **Example** | "Gather 500 gold" |

### `trigger_ambush`

| Aspect | Detail |
|--------|--------|
| **Description** | Successfully trigger an ambush |
| **Progress** | Ambushes triggered / target |
| **Completion** | Target reached |
| **Parameters** | `count`, `ambush_type` |
| **Example** | "Ambush 3 Byzantine patrols" |

### `stealth_objective`

| Aspect | Detail |
|--------|--------|
| **Description** | Complete an objective while stealthed |
| **Progress** | Binary (completed while stealthed?) |
| **Completion** | Object reached without detection |
| **Failure** | Detection resets or fails objective |
| **Parameters** | `stealth_required`, `detection_fails` |
| **Example** | "Reach the gate without being detected" |

### `interact`

| Aspect | Detail |
|--------|--------|
| **Description** | Interact with a world object (lever, door, item) |
| **Progress** | Binary |
| **Completion** | Interaction performed |
| **Parameters** | `interactable_id` |
| **Example** | "Open the hidden passage" |

### `protect`

| Aspect | Detail |
|--------|--------|
| **Description** | Keep a unit or building alive |
| **Progress** | Target HP |
| **Completion** | Objective timer expires OR related objective completes |
| **Failure** | Target HP ≤ 0 |
| **Parameters** | `target_id`, `min_hp_percent` |
| **Example** | "Protect the villager while he repairs the bridge" |

### `explore`

| Aspect | Detail |
|--------|--------|
| **Description** | Reveal a specific area on the map |
| **Progress** | Percentage revealed |
| **Completion** | 100% revealed |
| **Parameters** | `zone_id`, `reveal_percentage` |
| **Example** | "Explore the forest to find the hidden path" |

---

## 3. Objective Category Rules

| Category | Color | Required for Victory? | Counts Toward Stars? |
|----------|-------|-----------------------|---------------------|
| **Primary** | Gold | ✅ Yes | ✅ Yes (base) |
| **Secondary** | Silver | ❌ No | ✅ Yes (bonus star) |
| **Bonus** | Blue | ❌ No | ✅ Yes (bonus star) |
| **Hidden** | Hidden | ❌ No | ✅ Yes (discovery) |

---

## 4. Objective State Transitions

```
INACTIVE (not yet available)
    │
    │ prerequisite met
    ▼
ACTIVE (player can work on it)
    │
    ├── Completion condition met
    │       │
    │       ▼
    │   COMPLETED
    │       │
    │       ├── Award rewards
    │       ├── Emit: Mission_ObjectiveCompleted
    │       └── Activate next objectives
    │
    └── Failure condition met
            │
            ▼
        FAILED
            │
            ├── Emit: Mission_ObjectiveFailed
            └── (optional) Trigger defeat if primary
```

---

## 5. Objective Notification

```
Primary Objective:
  ☐ [GOLD] Capture the Byzantine Keep (0%)

Secondary/Bonus:
  ☐ [SILVER] Eliminate the commander
  ☐ [BLUE] Complete with < 5 casualties

On completion:
  ✅ Eliminate the commander — COMPLETE!
  Rewards: +50 Gold, +1 SP

On failure:
  ❌ Protect the village — FAILED!
  "The village has been destroyed."
```

---

## 6. Related Files

| File | Content |
|------|---------|
| `02_Mission_Structure.md` | How objectives fit into mission phases |
| `04_Dialogue_System.md` | Dialogue triggered by objective state |
| `06_Branching_Narrative.md` | How objectives change based on choices |
