# Region System — Map Zones, Territories & Control

> **Description:** Defines the regional map system for *Kuruluş* campaign — how the world is divided into zones, how region control works, and what gameplay effects regions have.

---

## 1. Region Architecture

The campaign map is divided into **regions**, each representing a distinct geographical and strategic area. Each mission takes place within one or more regions.

```
CAMPAIGN MAP
│
├── Chapter 1: "Bithynia"
│   ├── Region 1: Kulucahisar Valley (Mission 01)
│   │   ├── Zone A: Forest Approach (stealth zones)
│   │   ├── Zone B: Fortress Outer (combat zone)
│   │   └── Zone C: Keep Interior (boss area)
│   │
│   └── Region 2: Yarhisar Pass (Mission 02)
│       ├── Zone A: Mountain Trail (ambush corridor)
│       ├── Zone B: Village (friendly)
│       └── Zone C: Fortified Camp (enemy base)
│
├── Chapter 2: "Marmara"
│   ├── Region 3: Bilecik Plains (Mission 03)
│   └── Region 4: Marmaracık Coast (Mission 04)
│
└── Chapter 3: "Anatolia"
    ├── Region 5: Byzantine Fortress (Mission 05)
    └── Region 6: Yenişehir Valley (Mission 06)
```

---

## 2. Region Properties

| Property | Type | Example | Description |
|----------|------|---------|-------------|
| `id` | string | `region_kulucahisar` | Unique identifier |
| `display_name_key` | string | `region_kulucahisar_name` | Localized name |
| `chapter` | int | `1` | Which chapter contains this region |
| `mission_id` | string | `mission_01` | Mission played in this region |
| `terrain_biome` | enum | `forest_valley`, `mountain_pass`, `coastal_plains`, `fortress` | Visual theme |
| `size` | enum | `small`, `medium`, `large` | Relative map size |
| `faction_owner` | string | `byzantine` | Initial controlling faction |
| `strategic_value` | int (1-5) | `3` | Importance (determines AI priority) |
| `adjacent_regions` | string[] | `["region_yarhisar"]` | Neighbors for travel |
| `weather_pattern` | enum | `mild`, `rainy`, `snowy`, `dusty` | Default weather |
| `ambient_audio` | string | `ambient_forest_day` | Background sound |

---

## 3. Zone Types (Within Missions)

Each mission map is divided into **zones** that affect gameplay:

| Zone Type | Description | Gameplay Effects |
|-----------|-------------|------------------|
| **Spawn** | Player starting area | Safe from enemy patrols, no stealth needed |
| **Forest** | Dense woodland | Stealth available, movement slowed, vision reduced |
| **Grassland** | Open field | No stealth, full speed, full vision |
| **Mountain** | High elevation | +vision range, -movement speed, ranged damage bonus |
| **River/Shallow** | Water crossing | Movement slowed, units vulnerable |
| **Village** | Friendly settlement | Passive gold income, healing over time |
| **Fortification** | Enemy base walled area | Stealth reduced, defenders get defensive bonuses |
| **Ambush Corridor** | Narrow path | Both sides can ambush, limited escape routes |
| **Hilltop** | Elevated strategic point | Vision bonus, ranged advantage, capture point |
| **Hideout** | Stealth-friendly structure | Enter to vanish from enemy sight instantly |

### 3.1 Zone Effects Table

| Effect | Forest | Grassland | Mountain | Fortification |
|--------|--------|-----------|----------|---------------|
| Move Speed | -20% | 0% | -30% | -10% |
| Vision Range | -40% | 0% | +50% | -20% |
| Stealth Rating | +60% | -100% | +20% | -50% |
| Ranged Accuracy | -25% | 0% | +20% | -15% |
| Detection Range | -50% | 0% | +30% | +40% |

---

## 4. Region Control Effects

### 4.1 Ownership Effects

| Aspect | Owned Region | Enemy Region | Contested |
|--------|-------------|--------------|-----------|
| **Movement in campaign** | Free | Locked (must conquer) | Combat mission available |
| **Resource income** | Passive bonus | None | None |
| **Morale** | +5 bonus | -5 penalty | - |
| **Reinforcement speed** | Fast | None | Slow |

### 4.2 Region Conquest Order

```
CHAPTER 1                      CHAPTER 2                     CHAPTER 3
┌─────────────┐              ┌─────────────┐              ┌─────────────┐
│  Kulucahisar │──► Must     │   Bilecik   │──► Must      │  Byzantine   │
│  Valley      │    capture  │   Plains    │    capture   │  Fortress    │
│  (Mission 01)│    to open  │  (Mission 03)│   to open   │  (Mission 05)│
└─────────────┘              └─────────────┘              └─────────────┘
      │                            │                            │
      ▼                            ▼                            ▼
┌─────────────┐              ┌─────────────┐              ┌─────────────┐
│  Yarhisar   │              │ Marmaracık  │              │  Yenişehir  │
│  Pass       │              │  Coast      │              │  Valley     │
│  (Mission 02)│             │ (Mission 04)│             │  (Mission 06)│
└─────────────┘              └─────────────┘              └─────────────┘
```

---

## 5. Region Visual Themes

| Region | Terrain | Color Palette | Time of Day | Vegetation |
|--------|---------|---------------|-------------|------------|
| Kulucahisar Valley | Forest valley | Greens, browns, gold | Late afternoon | Dense oak, underbrush |
| Yarhisar Pass | Mountain pass | Grays, blues, white | Early morning | Sparse pine |
| Bilecik Plains | Open plains | Yellows, light green, amber | Midday | Tall grass, scattered trees |
| Marmaracık Coast | Coastal | Blues, sand, light green | Sunset | Palm, scrub |
| Byzantine Fortress | Fortress/stone | Grays, red accents, brown | Overcast | Minimal (urban) |
| Yenişehir Valley | Fertile valley | Rich greens, gold, blue sky | Morning | Mixed forest, fields |

---

## 6. Related Files

| File | Content |
|------|---------|
| `03_Fog_of_War.md` | Visibility and exploration system |
| `04_Faction_Relations.md` | How faction control affects regions |
| `05_Strategic_Events.md` | Dynamic events that change region state |
| `06_Campaign_Map_UI.md` | How regions display on campaign map |
