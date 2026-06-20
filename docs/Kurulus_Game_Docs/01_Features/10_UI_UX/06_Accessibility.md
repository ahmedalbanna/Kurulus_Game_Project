# Accessibility — Inclusive Design Features

> **Description:** Defines the accessibility features in *Kuruluş* to ensure the game is playable by a wide range of players with different needs and preferences.

---

## 1. Visual Accessibility

### 1.1 Text & Font

| Feature | Options | Description |
|---------|---------|-------------|
| Font Size | Small / Medium / Large / X-Large | Scales all UI text |
| Font Family | Default / Dyslexic-friendly | OpenDyslexic option |
| Line Spacing | 1.0× / 1.25× / 1.5× | Improves readability |
| Text Contrast | Normal / High / Maximum | WCAG AA/AAA compliance |

### 1.2 Color & Contrast

| Feature | Options | Description |
|---------|---------|-------------|
| Colorblind Mode | Off / Protanopia / Deuteranopia / Tritanopia | Adjusts UI colors |
| UI Contrast | Normal / High | Darker overlays, brighter borders |
| Outline Thickness | None / Thin / Thick | Unit selection outlines |
| Damage Numbers | Colored / Icon-coded | Shapes + colors for colorblind |

### 1.3 Visual Effects

| Feature | Options | Description |
|---------|---------|-------------|
| Screen Shake | On / Reduced / Off | Camera shake from explosions |
| Motion Effects | All / Reduced / Off | Camera transitions, particle effects |
| Flashing Lights | On / Reduced / Off | Epilepsy-safe mode |
| Brightness | Slider 0.5-1.5 | Screen brightness |
| UI Animation Speed | Normal / Slow / Off | Menu animation speed |

---

## 2. Audio Accessibility

| Feature | Options | Description |
|---------|---------|-------------|
| Master Volume | Slider 0-100 | Overall volume |
| SFX Volume | Slider 0-100 | Effects volume |
| Dialogue Volume | Slider 0-100 | Voiceover volume |
| Music Volume | Slider 0-100 | Background music |
| Mono Audio | On / Off | Merge stereo to mono |
| Subtitles | On / Off | Display spoken text |
| Subtitle Size | Normal / Large / X-Large | Subtitle text size |
| Subtitle Background | None / Dark / Solid | Readability |
| Speaker Name | Always / On first mention / Off | Show speaker name |
| Visual Combat Cues | On / Off | Visual + text cues for combat sounds |

---

## 3. Motor Accessibility

### 3.1 Input Options

| Feature | Options | Description |
|---------|---------|-------------|
| Camera Speed | Slider 0.1-2.0 | How fast camera moves |
| Edge Scroll | On / Off | Disable for accidental triggers |
| Edge Scroll Delay | 0 / 100 / 200 / 500 ms | Delay before edge scroll activates |
| Double Click Speed | Slow / Normal / Fast | Selection timing |
| Hold to Confirm | Off / 0.5s / 1.0s / 2.0s | For critical actions (delete, quit) |
| Sticky Keys | On / Off | Modifier keys (Shift, Ctrl) stay active |
| Toggle Abilities | Hold / Toggle | Ability activation mode |

### 3.2 Gameplay Assist

| Feature | Options | Description |
|---------|---------|-------------|
| Auto-Attack | On / Off | Units auto-attack enemies in range |
| Auto-Cast Abilities | On / Off | Heroes auto-cast abilities when ready |
| Simplified Controls | On / Off | Reduced commands, smart default |
| Movement Prediction | On / Off | Predicts path, shows before confirming |
| Pause on Event | Never / Combat / Any | Auto-pause on significant events |

---

## 4. Cognitive Accessibility

### 4.1 Gameplay Assistance

| Feature | Options | Description |
|---------|---------|-------------|
| Objective Guide | On / Off | Arrow pointing to next objective |
| Recommended Actions | On / Off | UI hints for what to do next |
| Auto-Manage Workers | On / Off | Workers automatically gather resources |
| Idle Worker Alert | On / Off | Notify when workers are idle |
| Formation Auto-Select | On / Off | Automatically select best formation |
| Combat Predictor | On / Off | Show predicted outcome before engaging |
| Simplified UI Mode | On / Off | Reduces HUD to essential elements |

### 4.2 Tutorial Features

| Feature | Description |
|---------|-------------|
| Interactive Tutorial | First mission guides every action |
| Contextual Tips | Tips appear based on current situation |
| Tip Library | Browse all gameplay tips from menu |
| Reminder Frequency | Often / Normal / Rare / Off |
| Video Tutorials | Short gameplay videos explaining mechanics |

---

## 5. Language Accessibility

| Feature | Options | Description |
|---------|---------|-------------|
| Game Language | العربية / English | UI and subtitles language |
| Subtitle Language | Same as game / Different | Separate subtitle language |
| Voiceover Language | Arabic / English | Audio dialogue language |
| UI Direction | LTR / RTL / Auto | Layout direction based on language |

---

## 6. Difficulty Accessibility

| Feature | Options | Description |
|---------|---------|-------------|
| Difficulty | Story / Easy / Medium / Hard | Overall difficulty preset |
| Enemy Aggression | Passive / Normal / Aggressive | AI behavior |
| Enemy Damage | 0.25× / 0.5× / 1.0× / 1.5× / 2.0× | Damage multiplier from enemies |
| Player Damage | 0.5× / 1.0× / 1.5× / 2.0× / 3.0× | Damage multiplier from player |
| Resource Rate | 1.0× / 1.5× / 2.0× / 3.0× | Resource gather speed |
| Enemy Count | Fewer / Normal / More | Number of enemy units |
| Invulnerable Heroes | On / Off | Heroes cannot die |
| No Morale | On / Off | Disable morale system |
| Infinite Resources | On / Off | Resources never deplete |
| Instant Build | On / Off | Construction and training instant |
| Fog of War | On / Off | Remove fog of war |

---

## 7. Accessibility Presets

| Preset | Changes |
|--------|---------|
| **High Contrast** | Max contrast, thick outlines, large text, subtitles on |
| **Colorblind** | Colorblind palette, shape-coded UI, colored text |
| **Low Motion** | Screen shake off, motion reduced, animations slow |
| **Story Mode** | Easy difficulty, resource bonus, heroes invulnerable |
| **Motor Assist** | Hold to confirm, sticky keys, simplified controls |
| **Custom** | User-defined combination of all options |

---

## 8. Save Accessibility Settings

- All accessibility settings are **saved per profile**
- Settings persist across sessions
- A dedicated **Accessibility Menu** section in Options
- Quick-access button from pause menu to accessibility

---

## 9. Related Files

| File | Content |
|------|---------|
| `02_HUD_Layout.md` | HUD elements affected by accessibility |
| `03_Menu_Structure.md` | Where accessibility options are located |
| `05_Input_Mapping.md` | Input accessibility features |
| `07_Visual_Language.md` | Visual accessibility considerations |
