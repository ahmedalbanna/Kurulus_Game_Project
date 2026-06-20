# Game States — Application & Scene State Definitions

> **Description:** Complete reference for all application-level and scene-level states in *Kuruluş*. Defines each state's purpose, allowed transitions, active systems, and side effects.

---

## 1. Application States

### `APP_BOOT`

| Property | Value |
|----------|-------|
| **Purpose** | Cold start — system initialization |
| **Duration** | ~0.5-2 sec |
| **Active Systems** | Core services, file system, logging |
| **Input** | None |
| **UI** | None (black screen) |
| **Audio** | None |
| **Transition To** | `APP_LOADING` (when core services ready) |
| **Side Effects** | Initialize event bus, state manager, data manager |

### `APP_LOADING`

| Property | Value |
|----------|-------|
| **Purpose** | Load static game data, check save files |
| **Duration** | 3-10 seconds |
| **Active Systems** | Data Manager (async load), Save Manager (scan) |
| **Input** | None |
| **UI** | Splash screen with loading bar + tip text |
| **Audio** | Intro music (starts loading) |
| **Transition To** | `APP_MAIN_MENU` (on success), `APP_ERROR` (on failure) |
| **Side Effects** | Validate data integrity, write crash log from previous session if any |

### `APP_MAIN_MENU`

| Property | Value |
|----------|-------|
| **Purpose** | Game hub — new game, continue, options, credits, quit |
| **Duration** | Indefinite |
| **Active Systems** | UI (menu), Audio (music), Save Manager (scan) |
| **Input** | UI navigation (mouse + keyboard) |
| **UI** | Main menu screen with animated background |
| **Audio** | Menu music (looping) |
| **Transitions** | `→ APP_MISSION_SELECT`, `→ APP_OPTIONS`, `→ APP_CREDITS`, `→ APP_QUIT` |

### `APP_MISSION_SELECT`

| Property | Value |
|----------|-------|
| **Purpose** | Choose mission / chapter / difficulty |
| **Duration** | Indefinite |
| **Active Systems** | UI, Save Manager |
| **UI** | Campaign map screen, mission cards |
| **Transitions** | `→ APP_LOADING_MISSION`, `→ APP_MAIN_MENU` (back) |

### `APP_LOADING_MISSION`

| Property | Value |
|----------|-------|
| **Purpose** | Load mission assets, build world, init gameplay systems |
| **Duration** | 5-20 seconds (depends on map complexity) |
| **Active Systems** | Asset loader, world builder, gameplay init |
| **Input** | None |
| **UI** | Loading screen with mission briefing text + art |
| **Audio** | Loading ambient |
| **Transition To** | `SCENE_MISSION_INTRO` (on success), `APP_ERROR` (on failure) |

### `APP_OPTIONS`

| Property | Value |
|----------|-------|
| **Purpose** | Settings — graphics, audio, controls, language |
| **UI** | Tabbed settings panel |
| **Transitions** | `→ APP_MAIN_MENU` (back/save) |
| **Side Effects** | Save settings on exit |

### `APP_CREDITS`

| Property | Value |
|----------|-------|
| **Purpose** | Display credits sequence |
| **UI** | Scrolling credits text |
| **Transitions** | `→ APP_MAIN_MENU` (end or skip) |

### `APP_ERROR`

| Property | Value |
|----------|-------|
| **Purpose** | Fatal error display |
| **UI** | Error dialog with error code, description, and "Restart" / "Quit" buttons |
| **Transitions** | `→ APP_BOOT` (restart), `→ APP_QUIT` |
| **Side Effects** | Write error report to disk |

### `APP_QUIT`

| Property | Value |
|----------|-------|
| **Purpose** | Graceful shutdown |
| **Duration** | ~1-2 seconds |
| **Side Effects** | Auto-save if applicable, flush logs, release resources |

---

## 2. Scene States (Within Gameplay)

### `SCENE_MISSION_INTRO`

| Property | Value |
|----------|-------|
| **Purpose** | Opening cinematic + briefing for the mission |
| **Duration** | 20-60 seconds |
| **Simulation** | Frozen |
| **Input** | Disabled (skip key allowed) |
| **Camera** | Locked to cinematic sequence |
| **UI** | Subtitles, briefing text overlay |
| **Audio** | VO, music, ambient |
| **Transition To** | `SCENE_GAMEPLAY` (on end or skip) |
| **Side Effects** | Initialize mission state, spawn starting units (hidden) |

### `SCENE_GAMEPLAY` (Core State)

| Property | Value |
|----------|-------|
| **Purpose** | Main RTS gameplay |
| **Duration** | 5-45 minutes |
| **Simulation** | Running at full speed |
| **Input** | Full gameplay controls |
| **Camera** | Full player control |
| **UI** | Full HUD (minimap, resources, selection panel, command bar) |
| **Audio** | Full dynamic audio |
| **Transitions** | `→ SCENE_PAUSED`, `→ SCENE_DIALOGUE`, `→ SCENE_CINEMATIC`, `→ SCENE_MISSION_COMPLETE`, `→ SCENE_DEFEAT` |

### `SCENE_PAUSED`

| Property | Value |
|----------|-------|
| **Purpose** | Game paused — menu overlay |
| **Simulation** | Frozen (time = 0) |
| **Input** | Menu navigation only |
| **Camera** | Frozen (greyed out) |
| **UI** | Pause menu overlay (Resume, Save, Load, Options, Quit to Menu) |
| **Audio** | Ambient low volume or paused |
| **Transitions** | `→ SCENE_GAMEPLAY` (resume), `→ APP_MAIN_MENU` (quit) |

### `SCENE_DIALOGUE`

| Property | Value |
|----------|-------|
| **Purpose** | Character dialogue / conversation |
| **Duration** | 5-30 seconds |
| **Simulation** | Running (time passes, AI acts, no player input) |
| **Input** | Disabled (except advance dialogue key) |
| **Camera** | Locked to speaker or cinematic framing |
| **UI** | Dialogue box with portrait, text, speaker name |
| **Audio** | VO prioritized, combat audio ducked |
| **Transitions** | `→ SCENE_GAMEPLAY` (on dialogue end), `→ SCENE_CINEMATIC` |

### `SCENE_CINEMATIC`

| Property | Value |
|----------|-------|
| **Purpose** | Scripted sequence (camera moves, world events, no player control) |
| **Simulation** | Running (scripted) |
| **Input** | Disabled (skip allowed) |
| **Camera** | Locked to sequence |
| **UI** | Subtitles only (HUD hidden) |
| **Audio** | Full cinematic mix |
| **Transitions** | `→ SCENE_GAMEPLAY` (on end), `→ SCENE_DIALOGUE` |

### `SCENE_MISSION_COMPLETE`

| Property | Value |
|----------|-------|
| **Purpose** | Victory — show results, rewards, stats |
| **Duration** | 5-15 seconds (player dismisses) |
| **Simulation** | Frozen (last frame shown) |
| **Input** | "Continue" button only |
| **UI** | Victory screen: mission stats, stars earned, rewards, "Next Mission" |
| **Audio** | Victory fanfare |
| **Side Effects** | Auto-save profile, unlock next mission, tally persistent rewards |
| **Transitions** | `→ APP_MAIN_MENU`, `→ APP_LOADING_MISSION` (next mission) |

### `SCENE_DEFEAT`

| Property | Value |
|----------|-------|
| **Purpose** | Defeat — show reason, offer retry |
| **Duration** | 5-15 seconds |
| **Simulation** | Frozen |
| **Input** | "Retry" or "Quit" buttons |
| **UI** | Defeat screen: reason, tip for next attempt, stats |
| **Audio** | Defeat sting |
| **Side Effects** | (none, no auto-save on defeat) |
| **Transitions** | `→ APP_LOADING_MISSION` (retry), `→ APP_MAIN_MENU` (quit) |

---

## 3. Composite State Table

```
State                Sim  Input  Camera    UI           Audio            Transitions
─────────────────────────────────────────────────────────────────────────────────────────
APP_BOOT             off  off    -         none         none             → LOADING
APP_LOADING          off  off    -         splash+bar   intro music      → MAIN_MENU, ERROR
APP_MAIN_MENU        off  ui     -         menu screen  menu music       → MISSION_SELECT, OPTIONS, CREDITS, QUIT
APP_MISSION_SELECT   off  ui     -         mission ui   menu music       → LOADING_MISSION, MAIN_MENU
APP_LOADING_MISSION  off  off    -         loading scr  loading ambient  → SCENE_MISSION_INTRO, ERROR
APP_OPTIONS          off  ui     -         settings     menu music       → MAIN_MENU
APP_CREDITS          off  off    -         credits      credits music    → MAIN_MENU
APP_ERROR            off  off    -         error dialog silence          → BOOT, QUIT
APP_QUIT             off  off    -         fade out     fade out         → (exit)
─────────────────────────────────────────────────────────────────────────────────────────
SCENE_MISSION_INTRO  off  off    locked    subtitles    VO+cine mix      → GAMEPLAY
SCENE_GAMEPLAY       on   full   full      full HUD     dynamic mix      → PAUSED, DIALOGUE, CINEMATIC, COMPLETE, DEFEAT
SCENE_PAUSED         off  menu   frozen    pause menu   ducked           → GAMEPLAY, MAIN_MENU
SCENE_DIALOGUE       on   advance locked   dialogue box VO+cine mix      → GAMEPLAY, CINEMATIC
SCENE_CINEMATIC      on   skip   locked    subtitles    cine mix         → GAMEPLAY, DIALOGUE
SCENE_MISSION_COMPLETE off continue frozen  victory ui   fanfare         → MAIN_MENU, LOADING_MISSION
SCENE_DEFEAT         off  retry  frozen    defeat ui    defeat sting     → LOADING_MISSION, MAIN_MENU
```

---

## 4. State Enter/Exit Hooks

### 4.1 Scene_GAMEPLAY — Enter

```
1. Enable simulation (start game clock)
2. Enable all input bindings
3. Unlock camera controls
4. Show HUD elements (fade in, 0.5s)
5. Resume dynamic audio mix
6. Emit: "Scene_GameplayStarted"
7. If first enter → trigger mission intro objectives
```

### 4.2 Scene_GAMEPLAY — Exit (to PAUSED)

```
1. Capture simulation timestamp (for resume)
2. Disable gameplay input bindings
3. Freeze camera
4. Show pause menu overlay (0.2s fade)
5. Duck audio mix (0.3s transition)
6. Emit: "System_GamePaused"
```

---

## 5. State Transition Validation

| Rule | Enforcement |
|------|-------------|
| **Disallowed transition** | Log error + ignore the request (e.g., trying to go from BOOT → GAMEPLAY) |
| **Transition in progress** | Block new transitions until current one completes |
| **Nested scene conflict** | Can't enter SCENE_DIALOGUE while already in SCENE_CINEMATIC |
| **State stack limit** | Max 3 stacked scene states (e.g., GAMEPLAY → DIALOGUE → CINEMATIC) |

---

## 6. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | State machine architecture |
| `03_Persistent_State.md` | Cross-session state (profile) |
| `04_Runtime_State.md` | Entity state machines |
