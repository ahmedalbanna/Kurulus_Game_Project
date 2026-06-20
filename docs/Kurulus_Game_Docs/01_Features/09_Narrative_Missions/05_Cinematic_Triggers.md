# Cinematic Triggers — Scripted Sequence System

> **Description:** Defines the cinematic trigger system — scripted sequences that control cameras, UI, and player input to deliver narrative moments during missions.

---

## 1. Cinematic Types

| Type | Description | Duration | Player Input Locked? |
|------|-------------|----------|---------------------|
| **Establishing Shot** | Camera sweep over mission area at start | 3-5 sec | ✅ Yes |
| **Story Moment** | Character conversation with camera focus | 10-30 sec | ✅ Yes |
| **Action Sequence** | Scripted combat/pursuit camera | 5-15 sec | ⚠️ Partial (movement only) |
| **Reaction Shot** | Brief camera focus on event (building collapses, hero ability) | 2-4 sec | ❌ No |
| **Mission Intro** | Full opening cinematic | 20-60 sec | ✅ Yes |
| **Mission Outro** | Closing cinematic / epilogue | 15-40 sec | ✅ Yes |

---

## 2. Trigger Definition

```json
{
  "cinematic_id": "cine_m01_castle_reveal",
  "type": "establishing_shot",
  "trigger": {
    "type": "event",
    "params": {
      "event_type": "Game_UnitEnteredZone",
      "zone_id": "zone_approach"
    }
  },
  "sequence": [
    {
      "time": 0.0,
      "action": "camera_move",
      "params": {
        "from": {"x": 10.0, "y": 15.0, "z": 5.0},
        "to": {"x": 30.0, "y": 8.0, "z": 20.0},
        "duration": 3.0,
        "easing": "ease_in_out"
      }
    },
    {
      "time": 1.0,
      "action": "show_subtitle",
      "params": {
        "text_key": "cine_castle_reveal",
        "duration": 4.0
      }
    },
    {
      "time": 3.0,
      "action": "focus_on_entity",
      "params": {
        "entity_id": "building_castle_gate",
        "zoom_level": 0.7,
        "duration": 2.0
      }
    },
    {
      "time": 5.0,
      "action": "return_camera_to_player",
      "params": {
        "transition_duration": 1.0
      }
    }
  ],
  "on_end": {
    "type": "enable_player_input",
    "params": {}
  },
  "can_skip": true,
  "priority": 80
}
```

---

## 3. Sequence Actions

| Action | Parameters | Description |
|--------|------------|-------------|
| `camera_move` | `from`, `to`, `duration`, `easing` | Move camera along path |
| `camera_focus` | `entity_id`, `zoom_level`, `duration` | Focus camera on entity |
| `camera_shake` | `intensity`, `duration` | Screen shake effect |
| `show_subtitle` | `text_key`, `duration` | Display subtitle text |
| `play_audio` | `audio_id`, `volume` | Play SFX or music |
| `pause_simulation` | `duration` | Pause game time |
| `resume_simulation` | `{}` | Resume game time |
| `lock_input` | `{}` | Disable player input |
| `unlock_input` | `{}` | Enable player input |
| `set_time_scale` | `scale`, `transition_duration` | Slow-motion effect |
| `show_ui` | `element_id` | Show specific UI element |
| `hide_ui` | `element_id` | Hide specific UI element |
| `spawn_effect` | `effect_id`, `position`, `duration` | Spawn visual effect |
| `return_camera_to_player` | `transition_duration` | Return control to player |
| `trigger_event` | `event_type`, `event_data` | Emit a game event |

---

## 4. Camera Control During Cinematics

### 4.1 Camera Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **Fixed** | Locked to specific position/angle | Establishing shots |
| **Follow** | Follow an entity's position | Character conversations |
| **Path** | Move along a spline curve | Sweep shots, fly-throughs |
| **Orbit** | Orbit around a point | Reveal moments |
| **Dolly** | Zoom in/out on a subject | Dramatic reveals |

### 4.2 Camera Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| position | Vector3 | - | Camera position |
| look_at | Vector3 | - | Point camera looks at |
| fov | float | 45° | Field of view |
| distance | float | - | Distance from subject (orbit) |
| rotation | float | - | Camera rotation (fixed mode) |
| easing | enum | `linear` | Movement curve (`ease_in`, `ease_out`, `ease_in_out`) |

---

## 5. Skip System

```
CINEMATIC PLAYING
      │
      │ Player presses ESC or Space
      │
      ▼
CONFIRMATION:
  "Skip cutscene?"
  [YES] [NO]
      │
      ├── YES
      │     ├── Immediately end all sequence actions
      │     ├── Return camera to player
      │     ├── Unlock input
      │     ├── Resume simulation
      │     └── Execute on_skip callbacks
      │
      └── NO → Continue playing
```

### 5.1 Skip Rules

| Cinematic Type | Can Skip? | Skip Destroys Event? |
|----------------|-----------|---------------------|
| Establishing | ✅ Yes | No (already revealed) |
| Story Moment | ✅ Yes | No |
| Action Sequence | ⚠️ Only after first viewing | Yes (permanent) |
| Mission Intro | ✅ Yes | No |
| Mission Outro | ✅ Yes | No |

---

## 6. Mission Cinematics Summary

| Mission | Intro Cinematic | Mid Cinematics | Outro Cinematic |
|---------|----------------|----------------|-----------------|
| 01 | Castle reveal, Osman's speech | Gate breach, Commander death | Liberation scene |
| 02 | Yarhisar pass panorama | Bridge scene, Seljuk meeting | United front |
| 03 | Bilecik plains establishing | Mercenary negotiation, betrayal | New allies |
| 04 | Coast vista, Mongol scout | Storm scene, Mongol charge | Warning of greater threat |
| 05 | Fortress establishing, siege prep | Wall breach, captain's treason | Fortress falls |
| 06 | Yenişehir valley, army assembled | Alliance arrival, final battle | Victory, founding of state |

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_Mission_Structure.md` | Where cinematics fit in mission flow |
| `04_Dialogue_System.md` | Dialogue within cinematic sequences |
| `06_Branching_Narrative.md` | How choices affect cinematic content |
