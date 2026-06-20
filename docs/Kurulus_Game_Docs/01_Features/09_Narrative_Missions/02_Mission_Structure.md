# Mission Structure — Phases, States & Victory Conditions

> **Description:** Defines the standardized structure for all campaign missions — the states a mission passes through, victory/defeat conditions, and phase transitions.

---

## 1. Mission State Machine

```
MISSION_CREATED (data loaded)
      │
      ▼
MISSION_INTRO (briefing, cinematic)
      │
      ▼
MISSION_ACTIVE (gameplay phase)
      │
      ├── PHASE_1 (opening — scout, set up)
      ├── PHASE_2 (mid-game — expand, engage)
      ├── PHASE_3 (climax — final assault, boss)
      └── BONUS_PHASE (optional, triggered by choice)
      │
      ├──► MISSION_VICTORY (objectives complete)
      │       │
      │       ├── VICTORY_CINEMATIC (outro sequence)
      │       ├── RESULTS_SCREEN (stats, rewards)
      │       └── RETURN_TO_CAMPAIGN
      │
      └──► MISSION_DEFEAT (conditions met)
              │
              ├── DEFEAT_CINEMATIC (outro)
              ├── RESULTS_SCREEN (stats, tips)
              └── RETRY / RETURN_TO_CAMPAIGN
```

---

## 2. Mission Phase Definition

### 2.1 Phase Properties

| Property | Type | Example |
|----------|------|---------|
| `phase_id` | string | `phase_01_scout` |
| `display_name_key` | string | `phase_01_name` |
| `order` | int | `1` |
| `objectives` | string[] | `["obj_01_reach_gate"]` |
| `starting_units` | string[] | `["hero_osman", "unit_swordsman_01"]` |
| `triggers_on_start` | string[] | `["trg_intro_dialogue"]` |
| `next_phase_condition` | string | `all_objectives_complete` |
| `time_limit` | float? | `null` or `300.0` |

### 2.2 Phase Progression

```
PHASE 1: Reconnaissance
  Objectives:
    ☐ Reach the castle gate
    ☐ Observe patrol patterns
    ☐ (Bonus) Avoid detection

  Condition for phase 2: Gate reached
      │
      ▼
PHASE 2: Infiltration
  Objectives:
    ☐ Breach the outer wall (siege or stealth)
    ☐ Secure the courtyard
    ☐ (Bonus) Use ambush at least once

  Condition for phase 3: Courtyard secured
      │
      ▼
PHASE 3: Assault
  Objectives:
    ☐ Defeat the Byzantine commander
    ☐ Capture the inner keep
    ☐ (Bonus) Commander defeated by hero
```

---

## 3. Victory Conditions

### 3.1 Primary Victory (Always Wins)

| Condition | Type | Example |
|-----------|------|---------|
| All primary objectives complete | `all_primary_done` | Standard mission win |
| Capture designated point | `capture_flag` | "Capture the fortress" |
| Eliminate specific target | `eliminate_target` | "Kill the Byzantine commander" |
| Survive for duration | `survive_timer` | "Hold out for 10 minutes" |

### 3.2 Secondary Victory (Alternate Win)

| Condition | Unlock | Example |
|-----------|--------|---------|
| All primaries + bonus objectives | 3★ rating | Perfect mission |
| All primaries + no hero deaths | Special reward | Hero survives bonus |
| All primaries under time limit | Speed bonus | Extra SP |
| All primaries + low casualties | Tactical bonus | Extra resources |

---

## 4. Defeat Conditions

| Condition | Type | Example |
|-----------|------|---------|
| All heroes dead | `heroes_all_dead` | Standard defeat |
| Critical building destroyed | `building_destroyed` | "Keep destroyed" |
| Timer expires | `timer_expired` | "Failed to capture in time" |
| Player base overrun | `base_overrun` | "Enemy forces overwhelm you" |
| Objective failed (failure condition) | `objective_failed` | "Village destroyed" |

---

## 5. Mission Results

### 5.1 Star Rating Calculation

```
Stars = Base (1) + Casualty Bonus (1) + Time Bonus (1) + Objective Bonus (1)

  Base:     Always 1 star for completing primary objectives
  Casualty: 1 star if friendly casualties ≤ 5 (or threshold per mission)
  Speed:    1 star if completed under par time (varies per mission)
  Bonus:    1 star if all bonus objectives completed
```

### 5.2 Results Screen Data

```
┌─────────────────────────────────────────────────────┐
│  MISSION COMPLETE                                    │
│  Mission: Yarhisar Pass                              │
│  Rating: ★★★                                        │
│                                                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐              │
│  │ Time    │  │ Kills   │  │ Losses  │              │
│  │ 12:34   │  │ 47      │  │ 8       │              │
│  └─────────┘  └─────────┘  └─────────┘              │
│                                                       │
│  Rewards:                                            │
│  ✦ SP: +3 (2 base + 1 bonus)                        │
│  💰 Gold: +134 (20% of 670 earned)                   │
│  🌲 Wood: +42                                        │
│  🆕 Unlocked: Archery Range, Archer unit             │
│                                                       │
│  ╔══════════════╗  ╔══════════════╗                  │
│  ║ NEXT MISSION ║  ║ CAMPAIGN MAP ║                  │
│  ╚══════════════╝  ╚══════════════╝                  │
└─────────────────────────────────────────────────────┘
```

---

## 6. Related Files

| File | Content |
|------|---------|
| `03_Objective_Types.md` | All objective type definitions |
| `04_Dialogue_System.md` | Dialogue during mission phases |
| `06_Branching_Narrative.md` | How choices affect mission flow |
