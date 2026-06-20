# Configuration System — System Overview

> **Description:** Defines the layered configuration architecture for *Kuruluş* — how settings, CVars, platform configs, and localization data are structured, loaded, and overridden at different levels.

---

## 1. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Layered overrides** | Default values → platform config → user settings → runtime CVars → command-line overrides |
| 2 | **Hot-reloadable** | Developer CVars can be changed at runtime without restarting |
| 3 | **Locale-independent** | All user-facing strings come from localization tables, not config |
| 4 | **Type safety** | Each config value has a defined type, range, and description |
| 5 | **Debug visibility** | All config values inspectable and modifiable via developer console |

---

## 2. Configuration Layered Model

```
Priority (higher wins)
    │
    ▲  1.  DEFAULT VALUES          (hardcoded in config schema)
    │  2.  PLATFORM DEFAULTS       (platform-specific overrides: Windows, macOS, Linux, Mobile)
    │  3.  BUILD-TIME CONFIG       (embedded during build: "release", "debug", "demo")
    │  4.  USER SETTINGS           (persisted settings file: options, key bindings)
    │  5.  RUNTIME CVARS           (developer console, hot-reloadable)
    │  6.  COMMAND-LINE ARGS       (launch arguments: --windowed, --debug, --seed=12345)
    │  7.  CHEAT/TEST OVERRIDES    (debug builds only: god mode, speed hack)
    ▼
    │   RESOLVED VALUE: the highest-priority set value
```

### 2.1 Resolution Example

```python
# Config key: "graphics.shadow_quality"
# Layer resolution:

Layer 1 (Default):      "high"          ← embeded default
Layer 2 (Platform):     None            ← no platform override
Layer 3 (Build):        None            ← no build override
Layer 4 (User settings): "medium"       ← user set this in options
Layer 5 (CVar):         "low"           ← developer set via console for performance testing
Layer 6 (CLI):          None            ← no command-line arg

RESOLVED: "low"  (from Layer 5)
```

---

## 3. Configuration Schema

### 3.1 Config Key Convention

```
# Naming
category.subcategory.key_name

# Examples
graphics.quality.shadow_quality
audio.volume.master_volume
gameplay.difficulty.difficulty_level
debug.visual.show_fps
system.io.mouse_sensitivity
```

### 3.2 Config Entry Schema

```json
{
  "key": "graphics.quality.shadow_quality",
  "type": "enum",
  "options": ["off", "low", "medium", "high", "ultra"],
  "default": "high",
  "platform_defaults": {
    "mobile": "medium",
    "low_end_pc": "low"
  },
  "description": "Controls shadow resolution and draw distance",
  "category": "Graphics",
  "runtime_mutable": true,
  "requires_restart": false,
  "min": null,
  "max": null,
  "tags": ["performance", "quality"]
}
```

### 3.3 Config Value Types

| Type | Example | Validation |
|------|---------|------------|
| `bool` | `true` | Must be true/false |
| `int` | `60` | Must be within min/max |
| `float` | `0.8` | Must be within min/max, clamped |
| `string` | `"player_name"` | Max length, allowed chars |
| `enum` | `"medium"` | Must be one of predefined options |
| `path` | `"/saves/profile.json"` | Must be valid filesystem path |
| `keybind` | `{"key": "W", "modifiers": []}` | Must be valid key + modifiers |
| `color` | `"#FF8800"` | Must be valid hex color |

---

## 4. Configuration Manager

```python
class ConfigManager:
    def __init__(self):
        self.layers: list[ConfigLayer] = [
            DefaultLayer(),
            PlatformLayer(),
            BuildLayer(),
            UserSettingsLayer("settings.json"),
            CVarLayer(),
            CommandLineLayer()
        ]
        self.cache: dict[str, ConfigEntry] = {}
        self.listeners: dict[str, list[Callable]] = {}

    def get(self, key: str, fallback=None):
        # Check cache first
        if key in self.cache:
            return self.cache[key]

        # Resolve through layers (high to low priority)
        for layer in reversed(self.layers):
            value = layer.get(key)
            if value is not None:
                self.cache[key] = value
                return value

        return fallback

    def set(self, key: str, value, layer_index: int = 4):
        """Set value at a specific layer (default: CVar layer = 4)"""
        self.layers[layer_index].set(key, value)
        self.cache.pop(key, None)  # invalidate cache

        # Notify listeners
        if key in self.listeners:
            for callback in self.listeners[key]:
                callback(key, value)

    def set_user_setting(self, key: str, value):
        """Convenience: set at user settings layer and persist"""
        self.set(key, value, layer_index=3)
        self.persist_user_settings()

    def persist_user_settings(self):
        self.layers[3].save()  # UserSettingsLayer.save()

    def subscribe(self, key: str, callback: Callable):
        """Register callback for when a config value changes"""
        if key not in self.listeners:
            self.listeners[key] = []
        self.listeners[key].append(callback)

    def get_all(self, category: str = None) -> dict:
        """Get all resolved config values, optionally filtered by category"""
        result = {}
        # In production, iterate all known keys from schema
        for key in self.get_all_keys():
            if category and not key.startswith(category):
                continue
            result[key] = self.get(key)
        return result
```

---

## 5. Config File Locations

| Config Type | File Name | Location | Format |
|-------------|-----------|----------|--------|
| User Settings | `settings.json` | User data directory | JSON |
| Key Bindings | `keybindings.json` | User data directory | JSON |
| Graphics Presets | `presets.json` | Game install directory | JSON |
| CVars | `cvars.cfg` | User data directory | Text (key=value) |
| Platform Config | `platform_config.json` | Game install directory | JSON |
| Build Config | `build_config.json` | Embedded (Resources/) | JSON |

### 5.1 User Data Directory

```
Windows:  %APPDATA%/Kurulus/
macOS:    ~/Library/Application Support/Kurulus/
Linux:    ~/.local/share/Kurulus/
```

---

## 6. Developer Console CVars

### 6.1 Built-in CVars

| CVar | Type | Default | Description |
|------|------|---------|-------------|
| `debug.show_fps` | bool | false | Toggle FPS counter |
| `debug.show_grid` | bool | false | Show navigation grid |
| `debug.show_stealth_zones` | bool | false | Visualize stealth zones |
| `debug.show_ai_state` | bool | false | Show AI state labels on units |
| `debug.god_mode` | bool | false | Player units invulnerable |
| `debug.unlimited_resources` | bool | false | Infinite resources |
| `debug.reveal_map` | bool | false | Remove fog of war |
| `debug.speed_multiplier` | float | 1.0 | Game speed multiplier (0.1-10) |
| `debug.trigger_ambush` | trigger | - | Manually trigger ambush |
| `debug.complete_mission` | trigger | - | Force mission victory |

### 6.2 Console Commands

```bash
# Get a value
> get graphics.shadow_quality
→ "high"

# Set a value
> set graphics.shadow_quality low
→ OK (requires restart: false)

# List all CVars in a category
> list graphics.*
→ graphics.quality.shadow_quality = "high"
→ graphics.quality.texture_quality = "ultra"
→ graphics.quality.anti_aliasing = "msaa_4x"

# Reset to defaults
> reset graphics.quality.shadow_quality
→ OK

# Watch a value for changes
> watch audio.volume.master_volume
→ monitoring audio.volume.master_volume...
```

---

## 7. Performance: Config Access

| Metric | Target |
|--------|--------|
| Config get (cached) | < 50 ns |
| Config get (uncached, first access) | < 1 µs |
| Config set + notify | < 10 µs |
| Config layer resolution | < 5 µs |
| Persist user settings | < 5 ms |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `02_Runtime_Config.md` | CVars, cheats, developer console |
| `03_Localization_Config.md` | String tables, locale support |
| `04_Build_Config.md` | Platform and build configuration |
| `01_Data_Architecture/04_Data_Pipeline.md` | Config file hot-reload |
