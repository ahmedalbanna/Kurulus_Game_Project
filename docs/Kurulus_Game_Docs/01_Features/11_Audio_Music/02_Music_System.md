# Music System — Dynamic Score & Audio Architecture

> **Description:** Defines the dynamic music system for *Kuruluş* — how the musical score adapts to gameplay intensity, locations, and narrative moments.

---

## 1. Music Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Turkish/Arabic inspired** | Traditional instruments (ney, oud, kemençe, darbuka) blended with orchestral |
| **Dynamic layering** | Music intensity shifts based on gameplay state |
| **Melodic themes** | Each hero, faction, and region has a unique melodic motif |
| **Seamless transitions** | No abrupt cuts — always crossfade or layer transition |
| **Non-intrusive** | Music supports, never overwhelms gameplay |

---

## 2. Music State Machine

```
┌──────────────────────────────────────────────────┐
│                  MUSIC MANAGER                     │
│                                                    │
│  STATES:                                           │
│  ┌────────────┐  ┌──────────┐  ┌────────────┐    │
│  │ EXPLORATION │  │  COMBAT  │  │  NARRATIVE  │   │
│  │ (calm)     │  │ (intense)│  │ (story)    │    │
│  └────────────┘  └──────────┘  └────────────┘    │
│                                                    │
│  SUB-STATES:                                       │
│  │  peaceful    │  │ skirmish  │  │ dialogue    │   │
│  │  building    │  │ battle    │  │ cinematic   │   │
│  │  exploring   │  │ siege     │  │ victory    │   │
│  │  stealth     │  │ boss      │  │ defeat      │   │
│  └────────────┘  └──────────┘  └────────────┘    │
└──────────────────────────────────────────────────┘
```

---

## 3. Music Layers

| Layer | Description | Instruments | Volume |
|-------|-------------|-------------|--------|
| **Bed** | Ambient pad, drone | Strings (sustained), ney drone | -12dB |
| **Rhythm** | Percussion, pulse | Darbuka, bendir, frame drum | -8dB |
| **Melody** | Main melodic line | Ney, oud, kemençe, violin | -6dB |
| **Countermelody** | Secondary melody | Kanun, cello, clarinet | -10dB |
| **Accents** | Stings, hits, emphasis | Percussion hits, brass stabs | -6dB |

### 3.1 Layer Activation by State

| State | Bed | Rhythm | Melody | Countermelody | Accents |
|-------|-----|--------|--------|---------------|---------|
| Peaceful | ✅ | ❌ | ❌ | ❌ | ❌ |
| Building | ✅ | ✅ (light) | ❌ | ❌ | ❌ |
| Exploring | ✅ | ✅ (light) | ✅ (occasional) | ❌ | ❌ |
| Stealth | ✅ (low) | ❌ | ❌ | ❌ | ❌ |
| Skirmish | ✅ | ✅ | ✅ | ✅ | ✅ (light) |
| Battle | ✅ (full) | ✅ (full) | ✅ (full) | ✅ (full) | ✅ (frequent) |
| Siege | ✅ (full) | ✅ (full) | ✅ (full) | ✅ | ✅ (heavy) |
| Boss | ✅ (full) | ✅ (full) | ✅ (intense) | ✅ | ✅ (all) |
| Dialogue | ✅ (low) | ❌ | ❌ | ❌ | ❌ |
| Cinematic | ✅ | ✅ | ✅ | ✅ | As needed |
| Victory | ✅ | ✅ | ✅ (triumphant) | ✅ | ✅ |
| Defeat | ✅ (low) | ❌ | ✅ (somber) | ❌ | ❌ |

---

## 4. Musical Themes

### 4.1 Hero Themes

| Hero | Instrument | Motif | Emotion |
|------|-----------|-------|---------|
| **Osman Bey** | Ney + full orchestra | Ascending, heroic | Determination, leadership |
| **Turgut Alp** | Kemençe (loud) | Energetic, driving | Courage, ferocity |
| **Buran** | Ney (high register) | Quick, agile | Cunning, alertness |

### 4.2 Faction Themes

| Faction | Style | Instruments | Emotion |
|---------|-------|-------------|---------|
| **Kayı Tribe (Player)** | Turkish folk + orchestral | Ney, kemençe, darbuka | Heroic, warm |
| **Byzantine** | Orthodox chant + brass | Choir, brass, organ | Oppressive, grand |
| **Mongol** | Asian steppe + percussion | Morin khuur, throat singing, frame drums | Wild, threatening |
| **Seljuk** | Persian classical + strings | Santur, setar, strings | Noble, refined |

### 4.3 Region Themes

| Region | Mood | Reference |
|--------|------|-----------|
| Kulucahisar Valley | Hopeful, rising | Opening journey |
| Yarhisar Pass | Tense, cautious | Mountain crossing |
| Bilecik Plains | Expansive, free | Open plains |
| Marmaracık Coast | Melancholic, beautiful | Seaside sunset |
| Byzantine Fortress | Oppressive, dark | Enemy stronghold |
| Yenişehir Valley | Triumphant, full | Final battle |

---

## 5. Dynamic Transition System

### 5.1 Transition Types

| Type | Duration | Method | Use |
|------|----------|--------|-----|
| **Crossfade** | 2-4 sec | Volume fade | Gradual intensity changes |
| **Stinger** | 0.5-1 sec | Immediate hit | Combat start, ambush |
| **Layering** | 1-2 sec | Add/remove layer | Incremental intensity |
| **Abrupt** | 0 sec | Sudden cut | Death, defeat |
| **Phrase Sync** | 4-8 sec | Wait for phrase end | Musical transitions |

### 5.2 Transition Triggers

| Game Event | Transition | New State |
|------------|------------|-----------|
| Combat starts | Stinger + layer in 2s → | Combat (skirmish) |
| Combat ends | Crossfade 3s → | Previous state |
| Ambush triggered | Stinger + abrupt → | Combat (ambush variant) |
| Hero uses ultimate | Accent hit → | Combat (intensified) |
| Stealth entered | Crossfade 4s → layer out → | Stealth |
| Dialogue starts | Crossfade 2s → lower volume | Dialogue (bed only) |
| Building | Gradually add rhythm → | Building |
| Victory | Crossfade 3s → | Victory theme |
| Defeat | Abrupt cut → | Defeat theme |

---

## 6. Music Asset List

| Track | Duration | Type | State |
|-------|----------|------|-------|
| `mus_menu_theme` | 2:30 | Loop | Menu |
| `mus_campaign_map` | 3:00 | Loop | Campaign |
| `mus_exploration_01` | 2:45 | Loop | Exploration |
| `mus_exploration_stealth` | 2:15 | Loop | Stealth |
| `mus_combat_skirmish` | 1:30 | Loop | Skirmish |
| `mus_combat_battle` | 2:00 | Loop | Battle |
| `mus_combat_siege` | 2:30 | Loop | Siege |
| `mus_combat_boss` | 1:45 | Loop | Boss |
| `mus_dialogue_bed` | 1:00 | Loop | Dialogue |
| `mus_victory_01` | 0:30 | One-shot | Victory |
| `mus_defeat_01` | 0:15 | One-shot | Defeat |
| `stinger_combat_start` | 0:08 | One-shot | Combat start |
| `stinger_ambush` | 0:05 | One-shot | Ambush |
| `stinger_hero_ultimate` | 0:03 | One-shot | Ultimate ability |
| `stinger_victory` | 0:05 | One-shot | Mission complete |

---

## 7. Related Files

| File | Content |
|------|---------|
| `03_SFX_Catalog.md` | Sound effects reference |
| `04_Voiceover_Guide.md` | Voiceover recording guide |
| `05_Dynamic_Audio.md` | Dynamic audio mixing system |
