# Persistent State — Cross-Session Profile & Progression

> **Description:** Defines the data that survives across game sessions — player profile, campaign progress, persistent resources, unlocks, achievements, and settings.

---

## 1. Persistent Data Ownership

```
PERSISTENT STATE MANAGER
│
├── Profile Manager
│   ├── player_name
│   ├── created_at
│   └── total_play_time
│
├── Campaign Manager
│   ├── completed_missions[]
│   ├── current_mission
│   ├── highest_chapter
│   ├── mission_stars{}
│   └── mission_stats{}
│
├── Progression Manager
│   ├── sovereignty_points (current)
│   ├── total_sp_earned
│   ├── talent_allocations{}
│   ├── persistent_resources{}
│   └── unlocks{}
│
├── Statistics Manager
│   ├── total_kills
│   ├── total_deaths
│   ├── total_buildings_constructed
│   ├── ... (aggregate stats)
│   └── per-mission stats archive
│
├── Achievement Manager
│   └── unlocked_achievements[]
│
└── Settings Manager
    ├── audio_settings
    ├── graphics_settings
    ├── control_settings
    └── language
```

---

## 2. Profile State Operations

### 2.1 Create Profile

```
Trigger: Player starts New Game
Actions:
  1. Generate unique profile_id (UUID)
  2. Set player_name (default or entered)
  3. Timestamp created_at = now
  4. Initialize all progression to defaults
  5. Create first save file
  6. Emit: "System_ProfileCreated"
```

### 2.2 Load Profile

```
Trigger: Player selects "Continue" or loads a save
Actions:
  1. Load save file from disk
  2. Validate checksum and version
  3. Deserialize JSON/binary into profile state
  4. Verify cross-reference integrity (e.g., unlocks reference valid IDs)
  5. Populate all manager caches
  6. Emit: "System_GameLoaded" with profile summary
```

### 2.3 Save Profile

```
Trigger: Manual save, auto-save checkpoint, mission end, app quit
Actions:
  1. Snapshot current persistent state from all managers
  2. Serialize to target format
  3. Write to temp file → atomic rename
  4. Update save slot metadata
  5. Emit: "System_GameSaved"
```

### 2.4 Migration on Version Change

```
Trigger: Profile version < current game version
Actions:
  1. Detect version delta
  2. Apply migration scripts in order (v1→v2, v2→v3, ...)
     Each script: transform data, add new fields, fill defaults
  3. If migration fails → mark profile as "requires migration tool"
  4. If migration succeeds → update profile_version, write
  5. Log migration report
```

---

## 3. Campaign Progress Data

```json
{
  "completed_missions": [
    {
      "mission_id": "mission_01_kulucahisar",
      "completed_at": 1704067500,
      "difficulty": "medium",
      "stars": 3,
      "time_seconds": 723,
      "casualties": 5,
      "enemy_kills": 47,
      "ambushes_performed": 2,
      "buildings_constructed": 8
    }
  ],
  "current_mission": "mission_02_yarhisar",
  "highest_chapter": 1,
  "mission_stars_total": 3,
  "total_missions_available": 6
}
```

### 3.1 Star Rating Rules

| Stars | Condition |
|-------|-----------|
| ⭐⭐⭐ | Complete all objectives + casualties ≤ 5 + time ≤ 15 min |
| ⭐⭐  | Complete all objectives + either casualties ≤ 10 or time ≤ 20 min |
| ⭐   | Complete all objectives |
| 0   | Mission failed or skipped |

---

## 4. Persistent Resources & Progression

### 4.1 Sovereignty Points (SP)

| Property | Value |
|----------|-------|
| **Purpose** | Currency for talent tree purchases |
| **Earned by** | Mission completion (1-3 SP per mission), bonus objectives, achievements |
| **Total earnable** | 18 SP (3 per mission × 6 missions) |
| **Maximum unspent** | 18 (cannot exceed total earned) |
| **Reset** | Not resettable (permanent choices) — but all talents can eventually be unlocked |

### 4.2 Persistent Resources Pool

Resources earned in missions that carry over:

| Resource | Earned From | Cap | Used For |
|----------|-------------|-----|----------|
| **Legacy Gold** | Mission bonus rewards, achievement bonuses | 10,000 | Starting gold in next missions, cosmetic unlocks |
| **Legacy Wood** | Mission bonus rewards | 5,000 | Persistent upgrade investments |

### 4.3 Persistent Unlocks

```json
{
  "units": {
    "unit_swordsman_basic": {"unlocked": true, "unlock_source": "default"},
    "unit_archer_basic": {"unlocked": true, "unlock_source": "mission_01"},
    "unit_cavalry_scout": {"unlocked": false, "unlock_source": "mission_03", "progress": 0}
  },
  "buildings": {
    "building_barracks": {"unlocked": true, "unlock_source": "default"},
    "building_archery_range": {"unlocked": false, "unlock_source": "mission_02"}
  },
  "abilities": {
    "hero_osman_sword_storm": {"unlocked": false, "unlock_source": "talent_blacksmith_tier2"}
  }
}
```

---

## 5. Settings Data

### 5.1 Audio Settings

| Setting | Type | Default | Range |
|---------|------|---------|-------|
| master_volume | float | 0.8 | 0.0 - 1.0 |
| music_volume | float | 0.7 | 0.0 - 1.0 |
| sfx_volume | float | 1.0 | 0.0 - 1.0 |
| voiceover_volume | float | 1.0 | 0.0 - 1.0 |
| ambient_volume | float | 0.8 | 0.0 - 1.0 |
| ui_volume | float | 0.6 | 0.0 - 1.0 |

### 5.2 Graphics Settings

| Setting | Type | Default | Options |
|---------|------|---------|---------|
| graphics_preset | enum | "medium" | "low", "medium", "high", "ultra", "custom" |
| resolution_width | int | 1920 | 1280, 1366, 1600, 1920, 2560 |
| resolution_height | int | 1080 | 720, 768, 900, 1080, 1440 |
| fullscreen | bool | true | - |
| v_sync | bool | true | - |
| shadow_quality | enum | "high" | "off", "low", "medium", "high" |
| texture_quality | enum | "high" | "low", "medium", "high" |
| anti_aliasing | enum | "fxaa" | "off", "fxaa", "msaa_2x", "msaa_4x" |
| post_processing | bool | true | - |
| view_distance | enum | "medium" | "low", "medium", "high", "ultra" |
| particle_quality | enum | "medium" | "low", "medium", "high" |

### 5.3 Control Settings

| Setting | Type | Default | Notes |
|---------|------|---------|-------|
| camera_speed | float | 0.5 | 0.0 - 1.0 |
| camera_edge_scroll | bool | true | - |
| camera_edge_scroll_size | int | 10 | pixels |
| invert_y | bool | false | - |
| mouse_sensitivity | float | 0.5 | 0.0 - 1.0 |
| key_bindings | map | default | Remappable action → key pairs |
| scroll_wheel_zoom_invert | bool | false | - |

---

## 6. Persistence I/O Boundaries

| Operation | Allowed From | Blocked From | Notes |
|-----------|-------------|--------------|-------|
| Read profile | Any state | - | Always safe |
| Write profile | Pause menu, mission complete, app quit | During gameplay (auto-save is queued) | Never write mid-frame |
| Read settings | Boot (load defaults), any state | - | Always safe |
| Write settings | Options menu, any state | - | Safe anytime |
| Delete save | Save/load screen | Gameplay | Confirmation required |

---

## 7. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | State management architecture |
| `02_Game_States.md` | Application/scene states |
| `04_Runtime_State.md` | Entity state machines |
| `01_Data_Architecture/03_Dynamic_Data.md` | Save file format, serialization |
