# Menu Structure — Non-Gameplay Screens & Navigation

> **Description:** Defines the complete menu system — all screens outside of gameplay, navigation flow, and screen specifications.

---

## 1. Menu Navigation Flow

```
BOOT (splash)
  │
  ▼
MAIN MENU
  ├── [PLAY] → Campaign Map
  ├── [LOAD] → Load Game
  ├── [OPTIONS] → Options Menu
  │                ├── Graphics
  │                ├── Audio
  │                ├── Controls
  │                └── Language
  ├── [CREDITS] → Credits
  └── [QUIT] → Exit Confirmation
         │
         ├── Campaign Map
         │     ├── [MISSION NODE] → Mission Briefing → Play
         │     │                      └── [BACK]
         │     ├── [TALENTS] → Talent Tree Screen
         │     └── [ARMY] → Army Viewer
         │
         └── Load Game
               ├── [SAVE SLOT] → Load Confirmation → Gameplay
               └── [BACK]
```

---

## 2. Screen Specifications

### 2.1 Splash Screen

| Element | Detail |
|---------|--------|
| **Logo** | Studio logo (3 sec) |
| **Fade** | Fade to black (0.5 sec) |
| **Next** | Auto-transition to Main Menu |
| **Skip** | Any key skips immediately |
| **Background** | Black with subtle particle effect |

### 2.2 Main Menu

| Element | Position | Description |
|---------|----------|-------------|
| Game Logo | Center-top | "Kuruluş" title with subtitle |
| New Game | Center | Starts campaign from beginning |
| Continue | Center | Resumes from last save (if exists) |
| Load Game | Center | Opens save file browser |
| Options | Center | Settings menu |
| Credits | Bottom-left | Scrolls credit text |
| Quit | Bottom-right | Exit confirmation |
| **Background** | Full screen | Animated isometric scene (auto-rotating) |
| **Music** | - | Menu theme (looping) |

### 2.3 Options Menu

#### Graphics Tab

| Option | Type | Values |
|--------|------|--------|
| Preset | Dropdown | Low / Medium / High / Ultra / Custom |
| Resolution | Dropdown | 1280×720 — 3840×2160 |
| Fullscreen | Toggle | On / Off |
| V-Sync | Toggle | On / Off |
| Shadow Quality | Dropdown | Off / Low / Medium / High |
| Texture Quality | Dropdown | Low / Medium / High |
| Anti-Aliasing | Dropdown | Off / FXAA / MSAA 2× / MSAA 4× |
| View Distance | Slider | Low — Ultra (5 steps) |
| Brightness | Slider | 0.5 — 1.5 |

#### Audio Tab

| Option | Type | Values |
|--------|------|--------|
| Master Volume | Slider | 0 — 100 |
| Music Volume | Slider | 0 — 100 |
| SFX Volume | Slider | 0 — 100 |
| Voiceover Volume | Slider | 0 — 100 |
| Ambient Volume | Slider | 0 — 100 |

#### Controls Tab

| Option | Type | Values |
|--------|------|--------|
| Camera Speed | Slider | 0.1 — 2.0 |
| Edge Scroll | Toggle | On / Off |
| Invert Y | Toggle | On / Off |
| Mouse Sensitivity | Slider | 0.1 — 2.0 |
| Key Bindings | Button | Opens rebinding screen |

#### Language Tab

| Option | Type | Values |
|--------|------|--------|
| Language | Dropdown | العربية / English |
| Subtitles | Toggle | On / Off |

---

## 3. Menu Transitions

| Transition | Duration | Type |
|------------|----------|------|
| Main Menu → Campaign Map | 1.0s | Fade through black |
| Campaign Map → Mission Briefing | 0.5s | Slide from right |
| Mission Briefing ↔ Campaign Map | 0.3s | Slide back |
| Campaign Map → Gameplay | 1.5s | Fade to loading → zoom-in |
| Gameplay → Pause Menu | 0.2s | Overlay slide down |
| Pause Menu → Gameplay | 0.2s | Overlay slide up |
| Any → Main Menu | 0.5s | Fade to black + fade in |
| Options ↔ Previous Menu | 0.3s | Slide from left |

---

## 4. Pause Menu Overlay

```
┌──────────────────────────────────────────────────────┐
│                     ⏸ PAUSED                          │
│                                                       │
│                     [▶ RESUME]                        │
│                     [💾 SAVE]                         │
│                     [📂 LOAD]                         │
│                     [⚙ OPTIONS]                      │
│                     [🏠 QUIT TO MENU]                 │
│                                                       │
│             Time: 12:34  |  Units: 18/30              │
└──────────────────────────────────────────────────────┘
```

---

## 5. Confirmation Dialogs

### 5.1 Quit Confirmation

```
┌──────────────────────────────────────────────┐
│  Exit to Menu?                                │
│  Any unsaved progress will be lost.            │
│                                                │
│  [✓ SAVE & EXIT]  [✗ EXIT WITHOUT SAVING]    │
│  [BACK]                                       │
└──────────────────────────────────────────────┘
```

### 5.2 Load Confirmation

```
┌──────────────────────────────────────────────┐
│  Load this save?                               │
│  Slot 1: "Yarhisar Pass — 12:34 elapsed"      │
│  Date: 2026-03-15 14:30                       │
│                                                │
│  [✓ LOAD]  [✗ CANCEL]                        │
└──────────────────────────────────────────────┘
```

---

## 6. Related Files

| File | Content |
|------|---------|
| `02_HUD_Layout.md` | In-game HUD |
| `04_Information_Architecture.md` | Information hierarchy |
| `05_Input_Mapping.md` | Menu navigation keys |
| `07_Visual_Language.md` | Menu visual style |
