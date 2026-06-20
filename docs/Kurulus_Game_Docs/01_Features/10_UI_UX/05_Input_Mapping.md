# Input Mapping — Key Bindings & Controls Reference

> **Description:** Complete reference for all player input bindings in *Kuruluş* — keyboard, mouse, and gamepad controls. All bindings are remappable.

---

## 1. Input Device Support

| Device | Support Level | Notes |
|--------|--------------|-------|
| Keyboard + Mouse | ✅ Primary | Full support |
| Gamepad (Xbox) | ✅ Full | Full support, UI adapted |
| Gamepad (PlayStation) | ✅ Full | Button prompts auto-detect |
| Steam Deck | ✅ Full | Optimized controls |
| Touch (RTS controls) | ❌ No | Not supported |

---

## 2. Keyboard + Mouse

### 2.1 Camera Controls

| Action | Default Key | Description |
|--------|-------------|-------------|
| Pan Camera | W/A/S/D or Arrow keys | Scroll camera in 4 directions |
| Edge Scroll | Mouse at screen edge | Auto-scroll when mouse at edge |
| Rotate Camera | Q / E | Rotate view left/right |
| Zoom In/Out | Scroll wheel | Zoom in/out smoothly |
| Zoom In | = (equals) | Zoom in step |
| Zoom Out | - (minus) | Zoom out step |
| Reset Camera | Middle mouse click | Reset to default angle |
| Follow Unit | Space | Follow selected unit |

### 2.2 Selection Controls

| Action | Default Key | Description |
|--------|-------------|-------------|
| Select Unit | Left click | Select single unit |
| Add to Selection | Shift + Left click | Toggle unit in selection |
| Selection Box | Left click drag | Box-select units |
| Deselect All | Esc | Clear selection |
| Select All Units | Ctrl + A | Select all friendly units |
| Select All Combat | Ctrl + C | Select all combat units |
| Select Idle Worker | F1 | Select next idle worker |
| Select Hero | F2 / F3 / F4 | Select Osman / Turgut / Buran |
| Cycle Selection | Tab | Cycle through unit types in selection |

### 2.3 Command Controls

| Action | Default Key | Description |
|--------|-------------|-------------|
| Move | Right click | Move to position |
| Attack Move | A + Right click | Move and engage enemies |
| Stop | S | Stop current action |
| Hold Position | H | Hold ground, attack in range |
| Patrol | P | Patrol between points |
| Force Attack | Ctrl + Right click | Attack ground/neutral target |
| Ability Slot 1 | Q | Use hero ability 1 |
| Ability Slot 2 | W | Use hero ability 2 |
| Ability Slot 3 | E | Use hero ability 3 |
| Ability Slot 4 (Ultimate) | R | Use hero ultimate |
| Cancel Ability | Esc | Cancel currently targeted ability |

### 2.4 Formation Controls

| Action | Default Key | Description |
|--------|-------------|-------------|
| Line Formation | F + 1 | Standard line formation |
| Wedge Formation | F + 2 | Wedge/arrow formation |
| Shield Wall | F + 3 | Dense defensive formation (melee) |
| Scatter Formation | F + 4 | Loose spread formation (ranged) |
| Circle Formation | F + 5 | Defensive circle |
| Column Formation | F + 6 | Travel column |
| Custom Formation | F + 7 | User-defined arrangement |

### 2.5 UI Shortcuts

| Action | Default Key | Description |
|--------|-------------|-------------|
| Pause / Resume | Esc | Toggle pause menu |
| Toggle Menu | Esc | Open/close pause menu |
| Quick Save | F5 | Save to quick slot |
| Quick Load | F8 | Load from quick slot |
| Open Talents | T | Open talent tree |
| Open Objectives | L | Open objective details |
| Open Options | O | Open settings |
| Screenshot | F12 | Capture screenshot |
| Toggle UI | Ctrl + U | Hide/show HUD (for screenshots) |
| Toggle Console | ~ (tilde) | Open developer console |
| Toggle Framerate | Ctrl + F | Show FPS counter |

### 2.6 Camera Bookmark

| Action | Default Key | Description |
|--------|-------------|-------------|
| Set Bookmark 1-5 | Ctrl + 1-5 | Save camera position |
| Go to Bookmark 1-5 | 1-5 | Jump to saved position |

### 2.7 Mouse Sensitivity

| Setting | Default | Range |
|---------|---------|-------|
| Camera Scroll Speed | 50% | 10% — 100% |
| Edge Scroll Size | 10 px | 2 — 50 px |
| Zoom Speed | 50% | 10% — 100% |
| Selection Box Delay | 150 ms | 50 — 500 ms |

---

## 3. Gamepad Controls

### 3.1 Xbox Layout

| Gameplay Action | Xbox Button |
|----------------|-------------|
| Primary action | A |
| Secondary action / Cancel | B |
| Select / Confirm | X |
| Ability Menu | Y |
| D-Pad: Select group | D-Pad Left/Right |
| D-Pad: Camera speed | D-Pad Up/Down |
| Left Stick | Move camera |
| Right Stick | Rotate camera |
| Left Trigger | Zoom out |
| Right Trigger | Zoom in |
| Left Bumper | Select all combat |
| Right Bumper | Select hero |
| Start | Pause menu |
| Back | Objectives |

### 3.2 Gamepad Cursor Mode

| Action | Button |
|--------|--------|
| Toggle cursor mode | Right Stick click |
| Cursor speed | Settings menu |
| Confirm action | A |

---

## 4. Rebindable Keys

All keyboard and mouse bindings can be remapped from the Options → Controls screen. Gamepad bindings are fixed (console standard).

### 4.1 Rebind UI

```
┌──────────────────────────────────────────────────┐
│  KEY BINDINGS                                      │
│                                                    │
│  Category: Camera Controls                         │
│                                                    │
│  [▼] Pan Camera Forward     W      [CHANGE]       │
│  [▼] Pan Camera Back        S      [CHANGE]       │
│  [▼] Pan Camera Left        A      [CHANGE]       │
│  [▼] Pan Camera Right       D      [CHANGE]       │
│  [▼] Rotate Camera Left     Q      [CHANGE]       │
│  [▼] Rotate Camera Right    E      [CHANGE]       │
│                                                    │
│  [               RESET TO DEFAULT               ] │
│  [                   APPLY                      ] │
└──────────────────────────────────────────────────┘
```

### 4.2 Conflict Detection

```
⚠ Warning: "Move" is already bound to Right Click.
Assigning "Select" to Right Click will:
  • Replace "Move" binding
  • Leave "Select" without a default binding

[✓ OVERRIDE] [✗ CANCEL]
```

---

## 5. Related Files

| File | Content |
|------|---------|
| `02_HUD_Layout.md` | UI elements that respond to input |
| `04_Information_Architecture.md` | How input reveals information |
| `06_Accessibility.md` | Input accessibility features |
