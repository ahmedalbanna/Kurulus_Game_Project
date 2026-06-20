# Dynamic Data — Runtime State & Save System

> **Description:** Defines how dynamic (runtime-mutable) data is structured, stored, serialized, and deserialized throughout a *Kuruluş* play session. Covers session state, persistent profile, and save/load mechanics.

---

## 1. Dynamic Data Categories

```
                    ┌────────────────────────────┐
                    │     DYNAMIC DATA LAYER      │
                    │                             │
                    │  ┌─────────────────────┐    │
                    │  │   Session State       │    │
                    │  │  (volatile, lost on   │    │
                    │  │   app quit)           │    │
                    │  │                       │    │
                    │  │ • Current mission     │    │
                    │  │ • Entity instances    │    │
                    │  │ • Runtime HP/stamina  │    │
                    │  │ • AI decision state   │    │
                    │  │ • Active effects      │    │
                    │  │ • Timer state         │    │
                    │  └─────────────────────┘    │
                    │                             │
                    │  ┌─────────────────────┐    │
                    │  │   Persistent State    │    │
                    │  │  (saved to disk,      │    │
                    │  │   survives app quit)  │    │
                    │  │                       │    │
                    │  │ • Player profile      │    │
                    │  │ • Completed missions  │    │
                    │  │ • Unlocked content    │    │
                    │  │ • Total resources     │    │
                    │  │ • Talent/spent SP     │    │
                    │  │ • Settings/options    │    │
                    │  │ • Replay data         │    │
                    │  └─────────────────────┘    │
                    │                             │
                    │  ┌─────────────────────┐    │
                    │  │   Config State       │    │
                    │  │  (persistent, low-   │    │
                    │  │   frequency writes)  │    │
                    │  │                       │    │
                    │  │ • Volume levels       │    │
                    │  │ • Graphics quality    │    │
                    │  │ • Key bindings        │    │
                    │  │ • Language            │    │
                    │  └─────────────────────┘    │
                    └────────────────────────────┘
```

---

## 2. Session State Architecture

### 2.1 Entity Instance Data

Each entity (unit, building, hero) at runtime has:

```json
{
  "instance_id": "entity_0001",
  "definition_id": "unit_swordsman_basic",
  "owner": "player",
  "position": {"x": 12.5, "y": 0.0, "z": 8.3},
  "rotation": 45.0,
  "current_hp": 87,
  "max_hp": 100,
  "current_morale": 65,
  "max_morale": 70,
  "status_effects": [
    {"effect_id": "buff_inspire", "remaining_sec": 3.2}
  ],
  "active_ability_cooldowns": {
    "ability_charge": 0.0,
    "ability_shield_wall": 4.7
  },
  "command_state": {
    "current_order": "move",
    "target_position": {"x": 20.0, "y": 0.0, "z": 15.0},
    "order_queue": []
  },
  "formation_data": {
    "formation_id": "line",
    "slot_index": 3,
    "offset": {"x": -1.5, "z": 0.0}
  },
  "flags": {
    "is_selected": false,
    "is_stealthed": true,
    "is_garrisoned": false,
    "is_building": false
  }
}
```

### 2.2 World State

```json
{
  "time_elapsed_sec": 452.3,
  "day_night_cycle": "day",
  "weather": "clear",
  "resources": {
    "resource_gold": 340,
    "resource_wood": 125,
    "resource_food": 78,
    "resource_iron": 42,
    "resource_leather": 10,
    "resource_silk": 0,
    "resource_gems": 5
  },
  "population": {
    "current": 18,
    "max": 30
  },
  "mission_objectives": {
    "obj_01_capture_flag": "completed",
    "obj_02_defend_position": "active",
    "obj_03_eliminate_commander": "inactive",
    "obj_04_survive": "inactive"
  },
  "discovered_zones": ["zone_a1", "zone_a2", "zone_b1"],
  "fog_of_war": {"revealed_tiles": [/* ... */]},
  "active_events": [
    {"event_id": "ambush_timer", "remaining_sec": 12.0}
  ]
}
```

---

## 3. Save System Design

### 3.1 Save File Structure

```
save_file.bin
├── HEADER (64 bytes)
│   ├── magic_number: 0x4B555200  ("KUR\0")
│   ├── version_major: 1
│   ├── version_minor: 0
│   ├── timestamp: 1704067200
│   ├── play_time_sec: 4523
│   └── checksum: CRC32
├── PROFILE DATA (variable)
│   ├── player_name
│   ├── total_play_time
│   ├── completed_missions[]
│   ├── total_resources (persistent)
│   ├── talent_allocations[]
│   └── achievements[]
├── MISSION SNAPSHOT (variable)
│   ├── mission_id
│   ├── session_state (JSON)
│   │   ├── world_state
│   │   ├── entity_instances[]
│   │   ├── ai_state
│   │   └── event_queue[]
│   └── screenshot_thumbnail (optional, PNG)
├── REPLAY DATA (optional)
│   ├── rng_seed
│   ├── input_sequence[]
│   └── checksum
└── FOOTER (16 bytes)
    ├── data_size
    └── footer_checksum
```

### 3.2 Serialization Format Decision

| Format | Pros | Cons | Verdict |
|--------|------|------|---------|
| **JSON** | Human-readable, debug-friendly, diffable in version control | Verbose, slower, larger | Use for dev builds |
| **Binary (BSON/Protocol Buffers)** | Compact, fast, deterministic | Opaque, harder to debug | Use for release builds |
| **Hybrid** | JSON for profile, binary for mission snapshot | Two code paths | ✅ **Recommended** |

### 3.3 Save Triggers

| Event | Type | Notes |
|-------|------|-------|
| Mission complete (victory) | Auto-save | Overwrites checkpoint |
| Mission start | Auto-save | Creates checkpoint |
| Player opens menu | Auto-save | Quick-save slot |
| Player manually saves | Manual | 3 save slots |
| Critical checkpoint reached | Auto-save | E.g. capturing a keep |
| Application quit | Auto-save | Graceful shutdown |

### 3.4 Save Slot Management

```
Save Slots:
├── Slot 1: "Quick Save"        (overwritten by auto-save)
├── Slot 2: "Manual Save 1"     (player-named)
├── Slot 3: "Manual Save 2"     (player-named)
├── Slot 4: "Manual Save 3"     (player-named)
└── Slot 5: "Checkpoint"        (auto-saved at mission start)

Each slot shows:
  • Screenshot thumbnail (if available)
  • Mission name
  • Play time
  • Date/time
  • Save version
```

---

## 4. Persistent Profile Data

### 4.1 Profile Schema

```json
{
  "profile_version": 1,
  "player_name": "Kurulus",
  "created_at": 1704067200,
  "total_play_time_sec": 14400,
  "game_completed": false,

  "campaign_progress": {
    "completed_missions": ["mission_01", "mission_02"],
    "current_mission": "mission_03",
    "highest_chapter": 1,
    "mission_stars": {
      "mission_01": 3,
      "mission_02": 2
    }
  },

  "persistent_resources": {
    "sovereignty_points": 8,
    "total_sp_earned": 14,
    "legacy_gold": 500,
    "legacy_wood": 200
  },

  "talent_allocations": {
    "blacksmith": [1, 0, 0, 0, 0],
    "cavalry": [1, 1, 0, 0, 0],
    "faith": [0, 0, 0, 0, 0]
  },

  "unlocks": {
    "units": ["unit_swordsman_basic", "unit_archer_basic"],
    "buildings": ["building_barracks", "building_lumber_mill"],
    "hero_abilities": ["hero_osman_sword_storm"],
    "cosmetics": []
  },

  "statistics": {
    "total_kills": 147,
    "total_deaths": 23,
    "total_buildings_constructed": 15,
    "total_gold_earned": 3200,
    "ambushes_performed": 8,
    "units_recruited": 45,
    "heroes_deployed": 3
  },

  "achievements": [
    {"id": "ach_first_blood", "unlocked_at": 1704067500},
    {"id": "ach_architect", "unlocked_at": 1704070000}
  ],

  "settings": {
    "language": "ar",
    "master_volume": 0.8,
    "sfx_volume": 1.0,
    "music_volume": 0.7,
    "ui_scale": 1.0,
    "camera_speed": 0.5,
    "invert_y": false,
    "graphics_preset": "medium",
    "resolution_width": 1920,
    "resolution_height": 1080,
    "fullscreen": true,
    "v_sync": true,
    "shadow_quality": "high"
  }
}
```

---

## 5. Save Integrity & Anti-Corruption

| Measure | Implementation |
|---------|---------------|
| **Magic Number** | First 4 bytes verify it's a valid save file |
| **Version Check** | Major version must match app major version |
| **Checksum (CRC32)** | Header + data checksummed; if mismatch, reject load |
| **Atomic Write** | Write to temp file → rename on completion. If crash during write, old file is intact |
| **Backup Copy** | Keep last-known-good save as `.bak` |
| **Size Limit** | Reject saves > 10 MB (indicates corruption or malicious tampering) |
| **Graceful Fallback** | If save fails to load, offer: retry / load backup / start new game |

---

## 6. Performance Considerations

| Concern | Strategy |
|---------|----------|
| **Save file size** | Target < 1 MB for full mission snapshot |
| **Save latency** | Always save on background thread; show "Saving..." indicator |
| **Load latency** | Deserialize on background thread; show progress bar |
| **Memory: entity instances** | Pool entity objects to avoid GC churn |
| **Memory: session state** | Free session state on mission exit; keep only profile in memory |
| **Replay data** | Circular buffer of last N minutes; export full log on manual save |

---

## 7. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Overall data architecture |
| `02_Static_Data.md` | Read-only definitions |
| `04_Data_Pipeline.md` | Import and validation |
| `03_State_Management/03_Persistent_State.md` | State machine for save/load transitions |
