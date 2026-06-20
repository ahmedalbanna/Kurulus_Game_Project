# State Management — System Overview

> **Description:** Defines how *Kuruluş* manages its global and local state — from top-level application states (menu, gameplay) down to individual entity states (idle, moving, attacking). Technology-agnostic state machine architecture.

---

## 1. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Predictable state transitions** | Every state change is explicit, logged, and reversible where appropriate |
| 2 | **Hierarchical states** | Global app state, scene/level state, and entity state form a clear hierarchy |
| 3 | **Save/load transparency** | The entire state tree can be serialized and restored deterministically |
| 4 | **Debug visibility** | Current state of any entity or system visible in debug overlay |
| 5 | **State isolation** | Systems in different states don't interfere — input has different meaning in menu vs gameplay |

---

## 2. State Layers

```
┌─────────────────────────────────────────────────────────┐
│                    APPLICATION STATE                      │
│  (Top-level: Loading, MainMenu, Options, Credits, Quit)  │
└────────────────────────┬────────────────────────────────┘
                         │ owns
                         ▼
┌─────────────────────────────────────────────────────────┐
│                     SCENE STATE                           │
│  (Per-level: MissionIntro, Gameplay, MissionComplete,    │
│   Defeat, PauseMenu, Dialogue)                           │
└────────────────────────┬────────────────────────────────┘
                         │ owns
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   GAMEPLAY STATE                          │
│  (GameMode: Campaign, Skirmish, Tutorial)                │
└────────────────────────┬────────────────────────────────┘
                         │ owns
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   ENTITY STATES                           │
│  (Per unit/building: Idle, Move, Attack, Retreat, Build, │
│   Garrisoned, Dead)                                      │
└─────────────────────────────────────────────────────────┘
```

---

## 3. Application State Machine

```
                    ┌──────────┐
                    │  BOOT    │
                    └────┬─────┘
                         ▼
                   ┌──────────┐
            ┌──────│ LOADING  │──────┐
            │      └────┬─────┘      │
            │           ▼            │
            │     ┌──────────┐       │
            │     │  MAIN    │       │
            │     │  MENU    │       │
            │     └──┬─┬─┬──┘       │
            │        │ │ │          │
            │  ┌─────┘ │ └──────┐   │
            │  ▼       ▼        ▼   │
            │ ┌────┐ ┌──────┐ ┌───┐ │
            │ │OPT.│ │MISSION│ │CRE│ │
            │ └──┬─┘ │SELECT│ │DIT│ │
            │    │   └──┬───┘ └───┘ │
            │    └──┐   │           │
            │       ▼   ▼           │
            │    ┌──────────┐       │
            │    │ LOADING  │       │
            │    │ MISSION  │───────┘
            │    └────┬─────┘
            │         ▼
            │    ┌──────────┐
            └────│GAMEPLAY  │
                 └──┬───┬──┘
                    │   │
              ┌─────┘   └─────┐
              ▼               ▼
         ┌────────┐     ┌────────┐
         │MISSION │     │DEFEAT/ │
         │COMPLETE│     │ GAME   │
         └───┬────┘     │ OVER   │
             │          └───┬────┘
             └──────┬───────┘
                    ▼
              ┌──────────┐
              │  RETURN  │
              │ TO MENU  │──────────► MAIN MENU
              └──────────┘
```

### 3.1 Transition Rules

| From | To | Trigger | Side Effects |
|------|----|---------|--------------|
| BOOT | LOADING | System init complete | Start loading static data |
| LOADING | MAIN_MENU | Data loaded | Release loading screen |
| MAIN_MENU | MISSION_SELECT | "Play" pressed | Animate transition |
| MAIN_MENU | OPTIONS | "Options" pressed | Pause music? |
| MISSION_SELECT | LOADING_MISSION | Mission chosen | Show loading screen + tip |
| LOADING_MISSION | GAMEPLAY | Mission loaded | Start simulation |
| GAMEPLAY | PAUSE_MENU | Escape pressed | Pause simulation, time = 0 |
| PAUSE_MENU | GAMEPLAY | Resume pressed | Unpause simulation |
| GAMEPLAY | MISSION_COMPLETE | All objectives done | Victory sequence, stats |
| GAMEPLAY | DEFEAT | Defeat condition met | Defeat sequence |
| MISSION_COMPLETE | MAIN_MENU | "Continue" | Save profile, load menu |
| DEFEAT | LOADING_MISSION | "Retry" | Reload mission from start |
| DEFEAT | MAIN_MENU | "Quit" | Return to menu |

---

## 4. Scene State Machine

Each mission has its own scene state machine:

```
MISSION_INTRO (cinematic, dialogue)
      │
      ▼
  GAMEPLAY (core RTS simulation)
      │
      ├── PAUSED (menu open)
      │
      ├── DIALOGUE (character conversation, time still passes?)
      │
      ├── CINEMATIC (scripted sequence, player input disabled)
      │
      ├── AMBUSH_SEQUENCE (ambush plays out, camera focuses)
      │
      └── BOSS_ENCOUNTER (special combat rules)
              │
              ▼
      MISSION_COMPLETE / DEFEAT
```

### 4.1 State Masks

Each state has a mask that defines what systems are active:

| State | Simulation | Input | UI | Audio | Camera Control |
|-------|-----------|-------|----|-------|---------------|
| MISSION_INTRO | Frozen | Disabled | Subtitles only | Full | Locked (cinematic) |
| GAMEPLAY | Running | Full | Full | Full | Full |
| PAUSED | Frozen | Menu only | Menu on top | Ambient only | Frozen |
| DIALOGUE | Running* | Disabled | Dialogue UI | VO + ambient | Locked to speaker |
| CINEMATIC | Frozen | Disabled | Subtitles | Full | Locked to sequence |
| * DIALOGUE: time passes, AI runs, but player cannot issue commands |

---

## 5. Entity State Machine

Every unit and building has a **finite state machine (FSM)** governing its behavior:

### 5.1 Unit States

```
                    ┌─────────┐
                    │  IDLE   │◄──────────────────────┐
                    └────┬────┘                        │
                         │                             │
              ┌──────────┼──────────┐                  │
              ▼          ▼          ▼                  │
         ┌────────┐ ┌────────┐ ┌────────┐             │
         │ MOVE   │ │ ATTACK │ │BUILD/  │             │
         └───┬────┘ └───┬────┘ │GATHER  │             │
             │          │      └───┬────┘             │
             │          │          │                   │
             │          ▼          │                   │
             │     ┌────────┐      │                   │
             │     │ ABILITY│      │                   │
             │     │ USE    │      │                   │
             │     └───┬────┘      │                   │
             │         │           │                   │
             ├─────────┼───────────┘                   │
             │         ▼                               │
             │    ┌──────────┐                          │
             │    │ RETREAT/ │                          │
             │    │ MORALE   │──────────────────────────┤
             │    │ BROKEN   │                          │
             │    └────┬─────┘                          │
             │         │                               │
             ▼         ▼                               │
        ┌────────┐ ┌────────┐                           │
        │ FLEEING│ │STEALTH │                           │
        └───┬────┘ └───┬────┘                           │
            │          │                               │
            ▼          ▼                               │
        ┌────────┐                                      │
        │  DEAD  │──────────────────────────────────────┘
        └────────┘
```

### 5.2 Building States

```
IDLE ──► UNDER_CONSTRUCTION ──► OPERATIONAL ──► UPGRADING ──► OPERATIONAL
  │                                 │
  └─────────────────────────────────┴──► DESTROYED
                                         │
                                         ▼
                                      RUBBLE
```

---

## 6. State Transition Rules

| Rule | Description |
|------|-------------|
| **Explicit transitions only** | No "any state can transition to any other" — transitions must be defined |
| **Enter/Exit callbacks** | Every state has `on_enter()` and `on_exit()` for setup/teardown |
| **Guard conditions** | Transitions can be blocked if guard conditions fail (e.g., can't attack while stealthed) |
| **Transition animations** | State changes may trigger visual/audio transitions (e.g., entering stealth → fade effect) |
| **Nested state inheritance** | If parent state is PAUSED, all child states are implicitly paused |

---

## 7. State Debugging

| Tool | Description |
|------|-------------|
| **State Viewer** | Tree view of all active states (app → scene → entity) |
| **Transition Log** | Time-stamped log of every state change: `[+12.3s] hero_osman: IDLE → MOVE` |
| **Force State** | Console: `set_state hero_osman DEAD` (with safety override) |
| **State Graph** | Visual FSM graph for any entity type, highlighting current state |
| **Transition Guard Check** | Why did a transition fail? Show guard condition evaluation |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `02_Game_States.md` | Complete app and scene state definitions |
| `03_Persistent_State.md` | Cross-session state (profile, unlocks) |
| `04_Runtime_State.md` | Entity state implementation details |
| `01_Data_Architecture/03_Dynamic_Data.md` | Save/load integration |
