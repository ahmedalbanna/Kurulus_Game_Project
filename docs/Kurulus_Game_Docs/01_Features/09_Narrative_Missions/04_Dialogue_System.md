# Dialogue System — In-Mission Conversations & Barks

> **Description:** Defines the dialogue system for *Kuruluş* — how in-mission conversations work, dialogue tree structure, conditional dialogue, and character barks (short contextual lines).

---

## 1. Dialogue Types

| Type | Description | Duration | Interrupts Gameplay? |
|------|-------------|----------|---------------------|
| **Mission Briefing** | Pre-mission narrative setup | 20-40 sec | Before mission starts |
| **In-Mission Dialogue** | Character conversations during gameplay | 5-20 sec | No (continues in background) |
| **Triggered Dialogue** | Story events triggered by player actions | 5-15 sec | Brief pause or continues |
| **Combat Barks** | Short character lines during combat | 1-3 sec | No |
| **Notification Barks** | System messages with audio | 2-4 sec | No |
| **Cinematic Dialogue** | Scripted cutscene conversations | 10-60 sec | Yes (player input disabled) |

---

## 2. Dialogue Tree Structure

```json
{
  "dialogue_id": "dlg_m01_intro",
  "type": "in_mission",
  "trigger": {
    "type": "objective_state",
    "params": {"objective_id": "obj_reach_gate", "state": "completed"}
  },
  "lines": [
    {
      "id": "line_01",
      "speaker": "hero_osman",
      "text_key": "dlg_m01_intro_01",
      "duration": 3.0,
      "emotion": "determined",
      "animation": "point_forward",
      "audio_clip": "vo_osman_m01_intro_01",
      "next_line": "line_02",
      "conditions": null
    },
    {
      "id": "line_02",
      "speaker": "hero_turgut",
      "text_key": "dlg_m01_intro_02",
      "duration": 4.0,
      "emotion": "cautious",
      "animation": "cross_arms",
      "audio_clip": "vo_turgut_m01_intro_02",
      "next_line": "line_03",
      "conditions": null
    },
    {
      "id": "line_03",
      "speaker": "hero_osman",
      "text_key": "dlg_m01_intro_03",
      "duration": 3.5,
      "emotion": "inspiring",
      "animation": "raise_sword",
      "audio_clip": "vo_osman_m01_intro_03",
      "next_line": null,  // End of dialogue
      "conditions": null
    }
  ],
  "on_complete": {
    "type": "set_objective",
    "params": {"objective_id": "obj_reach_gate", "state": "active"}
  },
  "priority": 100,
  "interrupts": false
}
```

---

## 3. Line Properties

| Property | Type | Example | Description |
|----------|------|---------|-------------|
| `speaker` | string | `hero_osman` | Character ID speaking |
| `text_key` | string | `dlg_m01_intro_01` | Localized text key |
| `duration` | float | `3.0` | Time to display on screen |
| `emotion` | string | `determined` | Character portrait expression |
| `animation` | string | `point_forward` | Character animation to play |
| `audio_clip` | string | `vo_osman_m01_intro_01` | Voiceover audio reference |
| `next_line` | string? | `line_02` | Next line in sequence |
| `conditions` | Condition[] | See below | Conditional next line |

---

## 4. Conditional Dialogue

Lines can have conditions that determine if they play:

```json
{
  "lines": [
    {
      "id": "choice_a",
      "text_key": "dlg_choice_attack",
      "conditions": [
        {"type": "has_unit", "params": {"unit_tag": "siege", "count": 1}}
      ]
    },
    {
      "id": "choice_b",
      "text_key": "dlg_choice_stealth",
      "conditions": [
        {"type": "has_stealthed_unit", "params": {"count": 3}}
      ]
    },
    {
      "id": "choice_c",
      "text_key": "dlg_choice_default",
      "conditions": null  // Fallback always available
    }
  ]
}
```

### 4.1 Conditional Line Triggers

| Condition Type | Params | Description |
|---------------|--------|-------------|
| `has_unit` | `unit_tag`, `count` | Player has at least N units with tag |
| `has_stealthed_unit` | `count` | Player has stealthed units |
| `resource_amount` | `resource_type`, `amount`, `comparison` | Resource check |
| `objective_state` | `objective_id`, `state` | Objective completed/failed |
| `relation_with` | `faction_id`, `level` | Faction relation check |
| `talent_purchased` | `talent_id` | Player has specific talent |
| `previous_choice` | `choice_id` | Player made a specific earlier choice |
| `random_chance` | `probability` | Random dialogue variation |

---

## 5. Dialogue UI Display

### 5.1 In-Mission Dialogue Overlay

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                   │
│              [ Portrait ]  Osman Bey                              │
│              ┌──────────┐  "By the grace of God, we will          │
│              │          │   take this castle. The people          │
│              │   🧔    │   of this land have suffered            │
│              │          │   under Byzantine rule too long."       │
│              └──────────┘                                         │
│                                        [Click to continue →]     │
└──────────────────────────────────────────────────────────────────┘
```

### 5.2 Combat Barks

```
[Floating text above unit, auto-dismisses]

  Osman: "Allahu Akbar!"     (when using ultimate)
  Turgut: "For the tribe!"   (when charging)
  Buran: "I see them!"       (when detecting enemy)
  Swordsman: "Attack!"       (when engaging)
  Archer: "Loose!"           (when firing)
```

### 5.3 Dialogue Queue Priority

| Priority | Type | Can Interrupt? | Queued If Busy? |
|----------|------|---------------|-----------------|
| 100 | Cinematic | ✅ Yes | No (replaces) |
| 80 | Story dialogue | ✅ Yes | No (replaces) |
| 60 | Combat bark | ❌ No | Yes (queued) |
| 40 | Notification bark | ❌ No | Yes (if same type) |
| 20 | Ambient chatter | ❌ No | Dropped if queue full |

---

## 6. Voiceover Integration

| Aspect | Detail |
|--------|--------|
| **VO Format** | 44.1kHz, 16-bit, mono, OGG Vorbis |
| **Naming Convention** | `vo_{character}_{mission}_{scene}_{line_id}.ogg` |
| **Language** | Arabic primary (ar), English fallback |
| **Lip Sync** | Generated from audio (RSS + viseme data) |
| **Priority** | Story VO → Combat Bark → Ambient |
| **Ducking** | Music ducks -6dB during VO, SFX -3dB |

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_Mission_Structure.md` | When dialogue triggers in missions |
| `03_Objective_Types.md` | Dialogue triggered by objective state |
| `05_Cinematic_Triggers.md` | Cinematic dialogue (full cutscene) |
| `06_Branching_Narrative.md` | Choice-based dialogue |
