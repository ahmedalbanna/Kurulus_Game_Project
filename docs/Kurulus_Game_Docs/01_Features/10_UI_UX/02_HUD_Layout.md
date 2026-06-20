# HUD Layout — In-Game Interface Architecture

> **Description:** Defines the in-game Heads-Up Display (HUD) layout for *Kuruluş* — all on-screen elements during gameplay, their positions, and interactions.

---

## 1. HUD Zone Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  MINIMAP PANEL              TOP BAR                               │
│  ┌──────────┐  ┌─────────────────────────────────────────────┐   │
│  │          │  │  Resources: 💰340 🌲125 🍞78 ⛏️42           │   │
│  │  MINIMAP  │  │  Population: 18/30    SP: ✦8               │   │
│  │          │  │  Time: 12:34   Speed: ▶▶                    │   │
│  │          │  └─────────────────────────────────────────────┘   │
│  └──────────┘                                                    │
│                                                                   │
│                                                                   │
│                 GAME WORLD (Isometric View)                       │
│                                                                   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────┐        │
│  │  NOTIFICATIONS                                       │        │
│  │  ⚠ "Archer under attack!"                           │        │
│  │  ✅ "New objective: Capture the keep"                │        │
│  └──────────────────────────────────────────────────────┘        │
│                                                                   │
│ ──────────────────────────────────────────────────────────────────│
│ │ COMMAND BAR              │ SELECTION PANEL                      │
│ │ ┌────┐ ┌────┐ ┌────┐    │ ┌────────────────────┐              │
│ │ │ M  │ │ A  │ │ S  │    │ │ Unit: Swordsman ×3  │              │
│ │ │ove │ │ttck│ │top │    │ │ HP: ████████░░ 80%  │              │
│ │ └────┘ └────┘ └────┘    │ │ Abilities: [⚔️][🛡️] │              │
│ │ ┌────┐ ┌────┐ ┌────┐    │ └────────────────────┘              │
│ │ │ P  │ │ F  │ │ H  │    │ ┌────────────────────┐              │
│ │ │atrl│ │orm │ │old │    │ │ Objectives:         │              │
│ │ └────┘ └────┘ └────┘    │ │ ☐ Capture keep      │              │
│ └─────────────────────────┘ │ ☐ Defeat commander   │              │
│                              │ ★ Bonus: <5 deaths   │              │
│   ABILITY BAR                └────────────────────┘              │
│   ┌──┐ ┌──┐ ┌──┐ ┌──┐                                          │
│   │Q │ │W │ │E │ │R │  (hero abilities)                         │
│   └──┘ └──┘ └──┘ └──┘                                          │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. Element Details

### 2.1 Top Bar (Top Center)

| Element | Description |
|---------|-------------|
| **Resource Bar** | Shows gold, wood, food, iron with icons and amounts |
| **Population** | Current/max population count |
| **SP Counter** | Current Sovereignty Points (clickable → talent tree) |
| **Mission Timer** | Elapsed mission time |
| **Speed Control** | Pause ▶ Play ⏩ Fast Forward |

### 2.2 Minimap (Top Left)

| Aspect | Detail |
|--------|--------|
| **Size** | 200×200 px (scalable in options) |
| **Terrain** | Colored terrain overlay |
| **Units** | Green dots (friendly), Red dots (visible enemy) |
| **Fog** | Dark areas (unexplored), dim (explored) |
| **Vision** | Transparent circle (camera view) |
| **Click** | Pans camera to clicked location |
| **Right-click** | Ping location |
| **Drag** | Select area on minimap |

### 2.3 Command Bar (Bottom Left)

| Button | Default Key | Description |
|--------|------------|-------------|
| **Move** | M | Move selected units to target |
| **Attack** | A | Attack-move to target (engage enemies on way) |
| **Stop** | S | Stop current action |
| **Patrol** | P | Patrol between current position and target |
| **Formation** | F | Open formation menu |
| **Hold Position** | H | Hold ground, attack enemies in range |

### 2.4 Selection Panel (Bottom Right)

| Element | Description |
|---------|-------------|
| **Unit Info** | Unit name, count, icon |
| **Health Bar** | Current HP with damage flash |
| **Morale Bar** | Current morale (green → yellow → red) |
| **Abilities** | Available abilities for selected unit type |
| **Status Effects** | Icons for active buffs/debuffs |
| **Objectives Panel** | Current primary/secondary objectives |

### 2.5 Ability Bar (Above Command Bar)

| Slot | Hero Abilities (when hero selected) |
|------|-------------------------------------|
| **Q** | Hero Ability 1 (primary) |
| **W** | Hero Ability 2 (secondary) |
| **E** | Hero Ability 3 (utility) |
| **R** | Hero Ultimate (tier 5 talent required) |

---

## 3. Notification System

### 3.1 Notification Types

| Type | Color | Duration | Example |
|------|-------|----------|---------|
| Warning | ⚠️ Yellow/Orange | 5 sec | "Unit under attack" |
| Success | ✅ Green | 4 sec | "Objective complete" |
| Info | ℹ️ Blue | 3 sec | "New ability unlocked" |
| Error | ❌ Red | 5 sec | "Building destroyed" |
| Tip | 💡 White | 6 sec | "Tip: Use stealth in forests" |

### 3.2 Notification Stack

| Priority | Show Always? | Max Visible |
|----------|-------------|-------------|
| High (combat alert) | ✅ Yes | 3 |
| Medium (objective) | ✅ Yes | 2 |
| Low (tip, info) | ❌ No, replaced if busy | 1 |

---

## 4. Floating Text

| Type | Example |
|------|---------|
| **Damage Numbers** | `-12` (red, floating above damaged unit) |
| **Healing Numbers** | `+8` (green, floating above healed unit) |
| **Critical Hit** | `-35 CRIT!` (larger, orange, with flash) |
| **Blocked** | `BLOCKED` (gray, floating above armor) |
| **XP Gain** | `+XP` (blue) |
| **Gold Gain** | `+Gold` (gold color) |

---

## 5. HUD Customization

| Setting | Options | Description |
|---------|---------|-------------|
| **HUD Scale** | 0.7× — 1.3× | Scale all HUD elements |
| **Minimap Size** | Small / Medium / Large | Minimap dimensions |
| **Resource Bar** | Always / Auto-hide / Off | Resource display mode |
| **Health Bars** | Always / On selection / On damage | Health bar visibility |
| **Floating Text** | All / Damage only / Off | Combat text |
| **Notifications** | All / Alerts only / Off | Notification filtering |
| **Objective Tracker** | Expanded / Compact / Off | Objective display style |

---

## 6. Related Files

| File | Content |
|------|---------|
| `03_Menu_Structure.md` | Non-gameplay menu screens |
| `04_Information_Architecture.md` | Information hierarchy |
| `05_Input_Mapping.md` | Key bindings |
| `06_Accessibility.md` | Accessibility options |
| `07_Visual_Language.md` | Visual style guide |
