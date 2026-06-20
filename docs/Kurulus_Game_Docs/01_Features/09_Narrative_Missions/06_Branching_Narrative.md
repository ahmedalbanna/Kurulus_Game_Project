# Branching Narrative — Choice Points & Consequences

> **Description:** Defines the branching narrative system — how player choices during missions affect story outcomes, rewards, and mission variants.

---

## 1. Branching Architecture

```
MISSION START
      │
      ├── [Choice Point 1] Approach
      │     ├── Stealth approach → Variant A
      │     └── Direct assault → Variant B
      │
      ├── [Choice Point 2] Ally or Alone
      │     ├── Recruit mercenaries → Variant C
      │     └── Go without → Variant D (harder)
      │
      ├── [Choice Point 3] Target Priority
      │     ├── Kill commander first → Reward: Leader bonus
      │     └── Secure keep first → Reward: Resource bonus
      │
      └── [End] Consequence resolution
            ├── High reputation path → Special dialogue in Mission 04
            └── Low reputation path → Different enemy composition
```

---

## 2. Choice Point Definition

```json
{
  "choice_id": "choice_m01_approach",
  "display_name_key": "choice_approach_name",
  "description_key": "choice_approach_desc",
  "trigger": {
    "type": "objective_complete",
    "params": {"objective_id": "obj_reach_gate"}
  },
  "options": [
    {
      "id": "approach_stealth",
      "display_name_key": "choice_approach_stealth",
      "description_key": "choice_approach_stealth_desc",
      "icon": "ico_stealth",
      "requirements": [
        {"type": "has_talent", "params": {"talent_id": "talent_stealth_bonus"}}
      ],
      "consequences": {
        "immediate": {
          "type": "set_objectives",
          "params": {"objective_id": "obj_stealth_infiltrate", "state": "active"}
        },
        "story_flags": {
          "approached_stealth": true,
          "casualty_expectation": "low"
        },
        "rewards": {
          "bonus_gold": 100,
          "bonus_sp": 1,
          "unlock_dialogue": "dlg_stealth_success"
        }
      }
    },
    {
      "id": "approach_assault",
      "display_name_key": "choice_approach_assault",
      "description_key": "choice_approach_assault_desc",
      "icon": "ico_swords",
      "requirements": [
        {"type": "has_units", "params": {"count": 10, "tag": "melee"}}
      ],
      "consequences": {
        "immediate": {
          "type": "spawn_enemies",
          "params": {"unit_id": "unit_spearman", "count": 5, "position": {"x": 30, "z": 20}}
        },
        "story_flags": {
          "approached_stealth": false,
          "casualty_expectation": "high"
        },
        "rewards": {
          "xp_bonus": 200,
          "unlock_dialogue": "dlg_assault_bravery"
        }
      }
    }
  ],
  "time_limit": 30.0,
  "if_timeout": "approach_assault",  // Default if player doesn't choose
  "priority": 90
}
```

---

## 3. Story Flags System

Player choices are stored as **story flags** — a set of key-value pairs that persist across missions.

```json
{
  "story_flags": {
    "approached_stealth_m01": true,
    "saved_village_m02": true,
    "befriended_mercenaries": false,
    "killed_commander_first_m03": true,
    "allied_seljuks_m04": true,
    "mongol_betrayal_triggered": false,
    "reputation_mercenaries": "hostile",
    "reputation_seljuks": "allied"
  }
}
```

### 3.1 Flag Effects on Later Content

| Flag | Affects Mission | Effect |
|------|----------------|--------|
| `approached_stealth_m01` | Mission 02 | Extra stealth zones available |
| `saved_village_m02` | Mission 04 | Village provides reinforcements |
| `befriended_mercenaries` | Mission 03, 05 | Mercenaries available for hire |
| `killed_commander_first_m03` | Mission 05 | Enemy leader is weaker (demoralized) |
| `allied_seljuks_m04` | Mission 06 | Seljuk army joins final battle |
| `reputation_mercenaries` | All | Affects mercenary prices and availability |

---

## 4. Mission Variants

Based on story flags, missions can have **variants** — modified versions of the same mission:

### 4.1 Variant Example: Mission 02 — Yarhisar Pass

| Variant | Trigger | Changes |
|---------|---------|---------|
| **Default** | Always available | Standard enemy layout, 3 patrol routes |
| **Stealth Bonus** | Flag: `approached_stealth_m01 = true` | Extra forest zone with hidden path, 2 fewer patrols |
| **Hard Mode** | Flag: `saved_village_m02 = false` | Village destroyed, -1 allied unit, enemies reinforced |

### 4.2 Variant Data

```json
{
  "variant_id": "m02_variant_stealth",
  "mission_id": "mission_02",
  "trigger_flags": {
    "approached_stealth_m01": true
  },
  "modifications": {
    "map_overrides": {
      "add_zone": "zone_hidden_path",
      "remove_zone": null
    },
    "enemy_changes": {
      "remove_patrols": ["patrol_03", "patrol_04"],
      "add_guards": null
    },
    "objective_changes": {
      "add_objective": "obj_use_hidden_path",
      "remove_objective": null
    },
    "dialogue_changes": {
      "replace_dialogue": [
        {"old_id": "dlg_m02_intro", "new_id": "dlg_m02_stealth_intro"}
      ]
    },
    "reward_changes": {
      "bonus_gold": 50
    }
  }
}
```

---

## 5. Choice UI Display

```
┌──────────────────────────────────────────────────────┐
│  CHOICE: How do you approach the castle?              │
│                                                        │
│  ┌────────────────────────────┐  ┌──────────────────┐ │
│  │ 🤫 STEALTH APPROACH        │  │ ⚔️ DIRECT ASSAULT│ │
│  │                             │  │                   │ │
│  │ Infiltrate through the      │  │ March openly      │ │
│  │ forest. Fewer casualties    │  │ toward the gate.  │ │
│  │ but requires stealthed      │  │ Faster but more    │ │
│  │ units.                      │  │ dangerous.         │ │
│  │                             │  │                    │ │
│  │ Requirement: 1+ stealth     │  │ No requirements    │ │
│  │             unit available  │  │                    │ │
│  │                             │  │                    │ │
│  │ Reward: +100 bonus gold     │  │ Reward: +200 XP    │ │
│  └────────────────────────────┘  └──────────────────┘ │
│                                                        │
│  ⏱ Auto-chooses in: 30 seconds...                      │
└──────────────────────────────────────────────────────┘
```

---

## 6. Consequence Tracking

```json
{
  "consequence_log": [
    {
      "mission_id": "mission_01",
      "choice_id": "choice_m01_approach",
      "selected": "approach_stealth",
      "tick": 4520,
      "game_time": 300.5,
      "consequences_applied": {
        "objectives_changed": true,
        "story_flags_set": 2,
        "rewards_awarded": true
      }
    }
  ]
}
```

---

## 7. Replay Value

| Feature | How It Works |
|---------|-------------|
| **Replay Mission** | Player can replay any completed mission from campaign map |
| **Choice Revisit** | On replay, all choices reset — player can try different path |
| **Variant Discovery** | Missed variants shown as "???" in mission details until discovered |
| **Reward Farming** | Replaying grants reduced rewards (50% of original) |
| **Achievement Tracking** | Some achievements require specific choice paths |

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Mission_Structure.md` | How variants affect mission flow |
| `03_Objective_Types.md` | Objectives that change based on choices |
| `04_Dialogue_System.md` | Dialogue branches |
| `05_Cinematic_Triggers.md` | Cinematics that change based on choices |
