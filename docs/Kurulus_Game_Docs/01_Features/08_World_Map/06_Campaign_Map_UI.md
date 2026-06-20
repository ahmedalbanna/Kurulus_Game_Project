# Campaign Map UI — Between-Mission Navigation Screen

> **Description:** Defines the campaign map user interface — the hub screen where players select missions, view progress, manage upgrades, and access persistent systems.

---

## 1. Screen Layout

```
┌──────────────────────────────────────────────────────────────────┐
│  KURULUŞ — CAMPAIGN MAP                    ✦ SP: 8    💰 340   │
│ ┌──────────────────────────────────────────────────────────┐    │
│ │                                                          │    │
│ │        ⛰️  [Mission 05]       🌲  [Mission 06]          │    │
│ │              ❌ LOCKED              ❌ LOCKED             │    │
│ │                                                          │    │
│ │           ⛪  [Mission 03]                                │    │
│ │              ★★★ COMPLETE                                │    │
│ │                                                          │    │
│ │   🌊 [Mission 04]                                        │    │
│ │      ★★☆ AVAILABLE                                      │    │
│ │                                                          │    │
│ │          🏰  [Mission 01]  🏔️  [Mission 02]              │    │
│ │             ★★★ COMPLETE     ★★☆ COMPLETE                │    │
│ │                                                          │    │
│ └──────────────────────────────────────────────────────────┘    │
│                                                                │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│ │  TALENTS  │ │  ARMY    │ │  OPTIONS │ │  EXIT    │          │
│ └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└──────────────────────────────────────────────────────────────────┘
```

---

## 2. UI Elements

| Element | Description | Interaction |
|---------|-------------|-------------|
| **Region Node** | Mission location on map | Click → mission details panel |
| **Connection Line** | Path between regions | Shows progression order |
| **Lock Icon** | Mission not yet available | Hover shows unlock condition |
| **Star Rating** | 1-3 stars earned | Hover shows completion stats |
| **SP Counter** | Current Sovereignty Points | Click → talent tree |
| **Gold Counter** | Legacy gold | Hover shows breakdown |
| **Chapter Label** | Chapter 1/2/3 header | Collapsible section |
| **Bottom Bar** | Navigation buttons | Talents, Army, Options, Exit |

---

## 3. Mission Details Panel

When a mission node is clicked:

```
┌───────────────────────────────────────────────────────┐
│  ممر يار حصار (Yarhisar Pass)                          │
│  Chapter 1 — Mission 2/2                               │
│                                                        │
│  ★★☆ Completed in 12:34 — Casualties: 8               │
│                                                        │
│  ┌────────────────────────────────────┐                │
│  │ Briefing text:                     │                │
│  │ "After securing Kulucahisar,       │                │
│  │  Osman marches toward Yarhisar...  │                │
│  │  The pass is guarded by Byzantine   │                │
│  │  archers in fortified positions."  │                │
│  └────────────────────────────────────┘                │
│                                                        │
│  Objectives:                                           │
│  ☑ Capture the pass (completed)                        │
│  ☑ Eliminate Byzantine commander (completed)           │
│  ☐ Bonus: Complete with < 5 casualties (FAILED)        │
│                                                        │
│  Rewards: 2 SP  │  +134 Gold  │  Unlocked: Archer      │
│                                                        │
│  ╔══════════════════╗   ╔══════════════════╗          │
│  ║    PLAY MISSION  ║   ║  VIEW REPLAY     ║          │
│  ╚══════════════════╝   ╚══════════════════╝          │
└───────────────────────────────────────────────────────┘
```

---

## 4. Mission State Visuals

| State | Visual | Text |
|-------|--------|------|
| **Locked** | Greyed out + lock icon | "Complete Mission N to unlock" |
| **Available** | Glowing border | "Ready" |
| **In Progress** | Pulsing border | "In Progress" |
| **Completed (1★)** | Faded star | "Completed" |
| **Completed (2★)** | Two bright stars | "Well done!" |
| **Completed (3★)** | Three bright stars + glow | "Perfect!" |

---

## 5. Bottom Navigation Buttons

| Button | Opens | Description |
|--------|-------|-------------|
| **TALENTS** | Talent Tree Screen | Spend SP, view upgrades |
| **ARMY** | Persistent Army Viewer | View unlocked units, stats |
| **OPTIONS** | Settings Screen | Audio, graphics, controls |
| **EXIT** | Exit Confirmation | Save & quit or quit without saving |

---

## 6. Animations & Transitions

| Action | Animation |
|--------|-----------|
| Campaign map appear | Zoom in from sky, 1 second |
| Mission node hover | Pulsing glow, tooltip |
| Mission start | Zoom toward node → fade to loading screen |
| Mission complete return | Zoom out from loading → node lights up |
| Chapter transition | Map pans to new region, chapter title overlay |
| Talent tree open | Sliding panel from right, 0.3s |

---

## 7. Responsive Layout Rules

| Screen Width | Layout | Notes |
|--------------|--------|-------|
| ≥ 1920px | Full map visible, side panels | Desktop |
| 1366-1919px | Map with scroll, compact panels | Laptop |
| 1280-1365px | Map zoomed to fit, bottom sheet panels | Small laptop |
| < 1280px | Scrollable map, stacked panels | Minimum supported |

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Region_System.md` | What's displayed on the map |
| `03_Fog_of_War.md` | Campaign map fog display |
| `05_Strategic_Events.md` | Event popups on campaign map |
