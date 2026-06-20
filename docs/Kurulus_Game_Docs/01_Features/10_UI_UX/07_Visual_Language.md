# Visual Language — UI Design System & Style Guide

> **Description:** Defines the visual design system for *Kuruluş* UI — colors, typography, icons, spacing, and component patterns.

---

## 1. Design Philosophy

| Principle | Description |
|-----------|-------------|
| **Thematic** | UI reflects the game's medieval Turkish/Arabic aesthetic |
| **Functional** | Every UI element has a clear purpose — no decoration without meaning |
| **Consistent** | Repeating patterns and predictable layouts |
| **Hierarchical** | Clear visual hierarchy guides the player's attention |
| **Responsive** | Adapts to different screen sizes and resolutions |

---

## 2. Color Palette

### 2.1 Core Colors

| Role | Hex | Name | Usage |
|------|-----|------|-------|
| Primary | `#C8A45C` | Gold | Headers, primary actions, hero elements |
| Secondary | `#8B4513` | Saddle Brown | Secondary elements, backgrounds |
| Accent | `#D4AF37` | Bright Gold | Highlights, active states |
| Background | `#1A1A2E` | Dark Navy | Panel backgrounds |
| Surface | `#16213E` | Deep Blue | Card backgrounds |
| Text Primary | `#F5F5F5` | Off White | Main text |
| Text Secondary | `#A0A0B0` | Gray Blue | Secondary text |
| Danger | `#C0392B` | Red | Errors, enemy, critical |
| Success | `#27AE60` | Green | Positive, friendly, completed |
| Warning | `#F39C12` | Orange | Alerts, caution |

### 2.2 Faction Colors

| Faction | Hex | Usage |
|---------|-----|-------|
| Player (Kayı) | `#2ECC71` | Green — unit outlines, team color |
| Byzantine | `#E74C3C` | Red — enemy units, territory |
| Mongol | `#8E44AD` | Purple — secondary enemy |
| Seljuk | `#3498DB` | Blue — ally |
| Neutral | `#95A5A6` | Gray — neutral units |

### 2.3 UI Semantic Colors

| Meaning | Hex | Usage |
|---------|-----|-------|
| HP Full | `#2ECC71` | Health bar full |
| HP Mid | `#F39C12` | Health bar mid (40-70%) |
| HP Low | `#E74C3C` | Health bar low (<40%) |
| Morale High | `#2ECC71` | Morale bar high |
| Morale Mid | `#F39C12` | Morale bar mid |
| Morale Low | `#E74C3C` | Morale bar low (near break) |
| Mana/Energy | `#3498DB` | Ability resource bar |

---

## 3. Typography

### 3.1 Font Family

| Role | Font | Fallback |
|------|------|----------|
| Headers (Latin) | Cinzel Decorative | Serif |
| Body (Latin) | Lato | Sans-serif |
| Arabic (UI) | Noto Naskh Arabic | Arabic Typesetting |
| Arabic (Decorative) | Scheherazade New | Serif |
| Monospace | JetBrains Mono | Courier New |

### 3.2 Font Sizes

| Element | Size (px) | Weight | Case |
|---------|-----------|--------|------|
| Game Title | 48 | Bold | Title |
| H1 | 32 | Bold | Title |
| H2 | 24 | Semi-Bold | Title |
| H3 | 20 | Semi-Bold | Sentence |
| Body | 16 | Regular | Sentence |
| Small | 14 | Regular | Sentence |
| Caption | 12 | Regular | Sentence |
| Tooltip | 13 | Regular | Sentence |
| Button | 16 | Bold | Title |
| Damage Number | 18-28 | Bold | N/A |
| Floating Text | 14 | Semi-Bold | N/A |

### 3.3 Arabic Typography Notes

- Arabic text uses **larger font size** (+2px) due to script complexity
- Line height for Arabic: 1.6× (vs 1.4× for Latin)
- Arabic supports **RTL** alignment natively
- Mixed Arabic/English text uses Unicode bidirectional algorithm

---

## 4. Iconography

### 4.1 Icon Style

| Aspect | Specification |
|--------|---------------|
| Style | Flat, filled, geometric |
| Stroke | None (filled) |
| Corner Radius | 2px |
| Size (UI) | 20×20px, 24×24px, 32×32px |
| Size (HUD) | 28×28px |
| Color | Monochromatic (tinted by role) |
| Background | None (transparent) |

### 4.2 Icon Categories

| Category | Example Icons |
|----------|--------------|
| Resources | Gold coin, wood log, bread, iron ingot |
| Units | Sword, bow, shield, horse, siege tower |
| Buildings | House, wall, tower, factory, farm |
| Abilities | Storm, shield, arrow, healing, fire |
| Commands | Move (arrow), Attack (crossed swords), Stop (square) |
| Formations | Line (|||), Wedge (>>>), Circle (O) |
| Status | Poison, fire, slow, stun, stealth |
| UI | Settings, save, load, quit, back, close |
| Objectives | Star, flag, skull, clock, shield |

---

## 5. Spacing & Layout

### 5.1 Spacing Scale

| Token | Size | Usage |
|-------|------|-------|
| `space-xxs` | 2px | Micro spacing |
| `space-xs` | 4px | Tight spacing |
| `space-sm` | 8px | Related elements |
| `space-md` | 12px | Default spacing |
| `space-lg` | 16px | Section spacing |
| `space-xl` | 24px | Panel padding |
| `space-xxl` | 32px | Page margins |

### 5.2 Panel Styles

| Panel | Background | Border | Corner Radius | Shadow |
|-------|------------|--------|---------------|--------|
| Main Panel | `#1A1A2E` | `#C8A45C` 1px | 4px | None |
| Card | `#16213E` | `#2C3E50` 1px | 4px | Small |
| Tooltip | `#1A1A2E` | `#C8A45C` 1px | 2px | Medium |
| Modal | `#1A1A2E` | `#C8A45C` 2px | 8px | Large |
| Notification | `#16213E` | Per type | 4px | Small |

---

## 6. Component Patterns

### 6.1 Buttons

```
Primary:   [▣ GOLD BACKGROUND] "PLAY" [Gold text, dark bg]
Secondary: [▣ DARK BACKGROUND] "CANCEL" [Light text, border]
Tertiary:  "TEXT LINK" [Gold underline on hover]
Disabled:  [▣ GRAY BACKGROUND] "LOCKED" [Gray text]
Icon:      [🔘] [24×24 icon, no label]
```

### 6.2 Progress Bars

```
Health Bar:    ████████░░ 80% (green→yellow→red)
Morale Bar:    ██████░░░░ 60% (green→yellow→red)
Build Bar:     ████░░░░░░ 40% (blue)
Experience:    ███████░░░ 70% (gold)
Timer:         ██████░░░░ 60% (white)
```

### 6.3 Selection Circles

```
Friendly:  🟢 Green circle (2px thick, pulsating)
Enemy:     🔴 Red circle (2px thick)
Ally:      🔵 Blue circle (2px thick)
Hero:      🟣 Purple circle (3px thick, special glow)
Stealthed: ⚪ Dimmed circle (1px, semi-transparent)
```

---

## 7. Motion & Animation

| Element | Animation | Duration | Easing |
|---------|-----------|----------|--------|
| Panel open | Fade in + slide up | 200ms | ease-out |
| Panel close | Fade out | 150ms | ease-in |
| Tooltip show | Fade in | 100ms | ease-out |
| Notification | Slide in from right | 300ms | ease-out |
| Button hover | Brightness increase | 100ms | ease-out |
| Button click | Scale 0.95 | 50ms | ease-in |
| Health bar change | Smooth fill change | 200ms | linear |
| Unit selection | Circle pulse | 500ms | ease-in-out |
| Screen transition | Crossfade | 300ms | ease-in-out |

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_HUD_Layout.md` | HUD applying this visual language |
| `03_Menu_Structure.md` | Menu screens applying this style |
| `06_Accessibility.md` | Visual accessibility considerations |
