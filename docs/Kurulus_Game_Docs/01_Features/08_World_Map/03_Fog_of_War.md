# Fog of War — Visibility & Exploration System

> **Description:** Defines the Fog of War (FoW) system for *Kuruluş* — how vision works, what players can see, and how stealth interacts with visibility.

---

## 1. Fog of War Layers

```
┌─────────────────────────────────────────────────────────┐
│                    FOG OF WAR SYSTEM                      │
│                                                           │
│  ┌────────────────────┐  ┌────────────────────┐          │
│  │   PERMANENTLY       │  │   CURRENTLY         │          │
│  │   REVEALED          │  │   VISIBLE           │          │
│  │   (explored map)    │  │   (unit vision)     │          │
│  │                     │  │                     │          │
│  │ • Terrain shape     │  │ • Enemy units       │          │
│  │ • Static features   │  │ • Enemy movement    │          │
│  │ • Zone boundaries   │  │ • Combat events     │          │
│  │ • Building ruins    │  │ • Building activity  │          │
│  └────────────────────┘  └────────────────────┘          │
│                                                           │
│  ┌────────────────────┐  ┌────────────────────┐          │
│  │   FOGGED           │  │   SHROUDED          │          │
│  │   (revealed but    │  │   (never visited)   │          │
│  │   no current view) │  │                     │          │
│  │                     │  │ • No information     │          │
│  │ • Static terrain    │  │ • Solid black        │          │
│  │ • Last-known enemy  │  │ • Minimap hidden     │          │
│  │   positions         │  │                     │          │
│  └────────────────────┘  └────────────────────┘          │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Vision Rules

### 2.1 Vision Sources

| Source | Range | Shape | Notes |
|--------|-------|-------|-------|
| **Units (standard)** | 15m | Circle | Blocked by terrain |
| **Units (mountain bonus)** | 22.5m | Circle | +50% on elevation |
| **Watch Tower T1** | 30m | Circle | Wide area |
| **Watch Tower T2** | 40m | Circle | Very wide |
| **Watch Tower T3** | 55m | Circle | Nearly entire zone |
| **Hero Osman** | 18m | Circle | Slightly above average |
| **Hero Buran** | 22m | Circle | Scout bonus |
| **Hero Turgut** | 14m | Circle | Front-line fighter |
| **Hills/High ground** | +10m | Circle | Stacking bonus |

### 2.2 Vision Blocking

| Obstacle | Blocks Vision? | Notes |
|----------|---------------|-------|
| Buildings | ✅ Yes | Units behind buildings are hidden |
| Walls | ✅ Yes | Cannot see through walls |
| Forest (dense) | ⚠️ Partial | Units inside hidden, edge visible |
| Forest (sparse) | ❌ No | Partial visual cover |
| Hills | ❌ No | Actually extends vision |
| Rivers | ❌ No | No vision obstruction |
| Mountains | ✅ Yes | Line-of-sight blocker |

### 2.3 Reveal Duration

When a unit moves out of an area, the fog returns after:

| Event | Time Until Fogged |
|-------|-------------------|
| Friendly unit left area | 2 seconds |
| Enemy unit spotted then vanished | 5 seconds (last known position ping) |
| Building destroyed in explored area | Instant (becomes revealed ruin) |
| Combat event location | 10 seconds (fades) |

---

## 3. Stealth & Fog Interaction

### 3.1 Stealthed Units in Fog

| Situation | Visible to Player? | Visible to Enemy? |
|-----------|-------------------|-------------------|
| Friendly stealthed | ✅ Yes (dimmed/faded) | ❌ No |
| Enemy stealthed (undetected) | ❌ No | N/A |
| Enemy stealthed (detected) | ✅ Yes (faint outline) | N/A |
| Friendly in stealth zone | ✅ Dimmed | ❌ If enemy outside zone |
| Ambush-ready units | ✅ Special icon | ❌ |

### 3.2 Detection System Integration

```
Stealthed Unit
      │
      ├── In open terrain: detection_range = 5m
      │
      ├── In forest zone: detection_range = 2m
      │
      ├── In stealth bush: detection_range = 1m (nearly invisible)
      │
      ├── Enemy watch tower nearby: detection_range × 2
      │
      ├── Unit moving: detection_range + 3m
      │
      └── Unit attacking: stealth broken, instantly revealed
```

---

## 4. Fog of War Data Structure

```json
{
  "map_width_tiles": 200,
  "map_height_tiles": 200,
  "tile_size": 1.0,
  "fog_grid": {
    "tiles": [
      {
        "x": 0, "z": 0,
        "state": "shrouded",  // shrouded | fogged | revealed | visible
        "last_visited_tick": 0,
        "permanent_features": null,
        "last_known_enemy": null
      }
    ]
  }
}
```

### 4.1 Fog State Per Tile

| State | Description | Minimap Color |
|-------|-------------|---------------|
| `shrouded` | Never visited | Dark/black |
| `fogged` | Visited but no current vision | Darkened terrain color |
| `revealed` | Currently in vision range | Full color |
| `permanently_revealed` | Watch tower or ability | Full color (permanent) |

---

## 5. Minimap Integration

| Element | Visible on Minimap? | Notes |
|---------|---------------------|-------|
| Explored terrain | ✅ Yes | Dimmed for fogged areas |
| Friendly units | ✅ Yes | Green dots |
| Enemy units (visible) | ✅ Yes | Red dots |
| Enemy units (fogged) | ⚠️ Last known | Fading red dot |
| Stealthed friendly | ✅ Yes | Dimmed green |
| Stealthed enemy | ❌ No | Only when detected |
| Buildings (friendly) | ✅ Yes | Green squares |
| Buildings (enemy known) | ✅ Yes | Red squares (last known) |
| Zone boundaries | ✅ Yes | Faint lines |
| Objective markers | ✅ Yes | Gold diamonds |

---

## 6. Performance Optimization

| Technique | Description |
|-----------|-------------|
| **Grid-based** | Tile grid (1m × 1m), not pixel-based |
| **Dirty region updates** | Only update tiles that changed this frame |
| **Throttled updates** | Perimeter vision updates at 10 Hz, not every frame |
| **Hierarchical** | Coarse grid (10m) for quick checks, fine grid (1m) for detail |
| **Instanced rendering** | Batch draw fog tiles for GPU efficiency |

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_Region_System.md` | Zone types and vision effects |
| `04_Faction_Relations.md` | How faction vision sharing works |
| `06_Campaign_Map_UI.md` | Campaign map fog display |
