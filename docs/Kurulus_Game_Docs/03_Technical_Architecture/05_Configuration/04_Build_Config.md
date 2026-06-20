# Build Configuration — Platform, Build Variants & Feature Toggles

> **Description:** Defines the build-time configuration system for *Kuruluş* — how platform-specific settings, build variants (debug/release/demo), and feature toggles are managed.

---

## 1. Build Variants

| Variant | Purpose | Optimization | Debug Tools | Cheats | Performance |
|---------|---------|-------------|-------------|--------|-------------|
| **Debug** | Development | None | ✅ Full | ✅ Enabled | Low |
| **Development** | Internal testing | Light | ✅ Console + overlays | ✅ Enabled | Medium |
| **Release** | Public launch | Full | ❌ None | ❌ Disabled | High |
| **Demo** | Public demo/trial | Full | ❌ None | ❌ Disabled | High |

---

## 2. Platform Configuration

### 2.1 Target Platforms

| Platform | Architecture | Graphics API | Notes |
|----------|-------------|--------------|-------|
| Windows (Steam) | x64 | DirectX 11/12 | Primary platform |
| Windows (Standalone) | x64 | DirectX 11/12 | DRM-free |
| macOS (App Store) | x64, ARM | Metal | Apple Silicon native |
| Linux (Steam) | x64 | Vulkan | Proton or native |
| Steam Deck | ARM/x86 | Vulkan | Controller input optimized |

### 2.2 Platform-Specific Defaults

```json
{
  "windows": {
    "graphics.resolution.width": 1920,
    "graphics.resolution.height": 1080,
    "graphics.quality.shadow_quality": "high",
    "graphics.quality.texture_quality": "high",
    "graphics.anti_aliasing": "msaa_4x",
    "graphics.fps_limit": 60,
    "system.io.mouse_sensitivity": 0.5,
    "system.storage.save_path": "%APPDATA%/Kurulus/"
  },
  "macos": {
    "graphics.resolution.width": 1512,
    "graphics.resolution.height": 982,
    "graphics.quality.shadow_quality": "medium",
    "graphics.quality.texture_quality": "high",
    "graphics.anti_aliasing": "fxaa",
    "graphics.fps_limit": 60,
    "system.io.mouse_sensitivity": 0.5,
    "system.storage.save_path": "~/Library/Application Support/Kurulus/"
  },
  "linux": {
    "graphics.resolution.width": 1920,
    "graphics.resolution.height": 1080,
    "graphics.quality.shadow_quality": "medium",
    "graphics.quality.texture_quality": "medium",
    "graphics.anti_aliasing": "fxaa",
    "graphics.fps_limit": 60,
    "system.io.mouse_sensitivity": 0.5,
    "system.storage.save_path": "~/.local/share/Kurulus/"
  },
  "steam_deck": {
    "graphics.resolution.width": 1280,
    "graphics.resolution.height": 800,
    "graphics.quality.shadow_quality": "low",
    "graphics.quality.texture_quality": "medium",
    "graphics.anti_aliasing": "fxaa",
    "graphics.fps_limit": 30,
    "graphics.v_sync": true,
    "control.camera.edge_scroll": false,
    "system.io.mouse_sensitivity": 0.3
  }
}
```

---

## 3. Feature Toggles

### 3.1 Defining Feature Toggles

```json
{
  "features": {
    "ambush_system": {
      "enabled": true,
      "debug_mode": false,
      "description": "Ambush and stealth mechanics"
    },
    "advanced_formation": {
      "enabled": true,
      "debug_mode": false,
      "description": "Advanced formation controls (shield wall, wedge)"
    },
    "dynamic_weather": {
      "enabled": false,
      "debug_mode": true,
      "description": "Dynamic weather system (WIP for post-launch)"
    },
    "replay_system": {
      "enabled": true,
      "debug_mode": true,
      "description": "Input recording and replay"
    },
    "modding_support": {
      "enabled": false,
      "debug_mode": false,
      "description": "External data file modding (post-launch)"
    },
    "achievements": {
      "enabled": true,
      "debug_mode": false,
      "description": "Achievement system"
    },
    "controller_support": {
      "enabled": true,
      "debug_mode": false,
      "description": "Gamepad/controller input"
    },
    "arabic_localization": {
      "enabled": true,
      "debug_mode": false,
      "description": "Arabic language support"
    },
    "analytics": {
      "enabled": true,
      "debug_mode": false,
      "description": "Anonymous usage analytics (opt-out)"
    },
    "tutorial_system": {
      "enabled": true,
      "debug_mode": false,
      "description": "In-game tutorial and hints"
    }
  }
}
```

### 3.2 Feature Toggle Resolution

```
Build variant: Release
  │
  ├── features.ambush_system.enabled = true   → ✅ Included
  ├── features.dynamic_weather.enabled = false → ❌ Excluded
  │
  └── features.replay_system.debug_mode = true → ❌ Excluded (debug only in Release)
```

---

## 4. Build Script Integration

```bash
# Build commands (pseudocode)

# Debug build (with full debug tools)
build --variant debug --platform windows

# Development build (light optimization, debug tools)
build --variant development --platform windows

# Release build (full optimization, no debug)
build --variant release --platform windows

# Demo build
build --variant demo --platform windows --feature-toggles demo_features.json

# Steam Deck specific
build --variant release --platform steam_deck --profile steam_deck_perf

# With custom feature toggles
build --variant development --features custom_features.json
```

---

## 5. Build Configuration Files

### 5.1 Build Config Schema

```json
{
  "build_version": "1.0.0.5421",
  "build_variant": "release",
  "build_date": "2026-03-15T10:30:00Z",
  "git_hash": "a1b2c3d4e5f6...",
  "git_branch": "main",

  "platform": {
    "id": "windows",
    "architecture": "x64",
    "graphics_api": "directx_12",
    "renderer": "forward+"
  },

  "optimization": {
    "enable_assertions": false,
    "enable_profiling": false,
    "optimization_level": "speed",
    "strip_debug_symbols": true,
    "link_time_optimization": true
  },

  "features_file": "config/features_release.json",
  "locale_files": ["locales/ar.json", "locales/en.json"],

  "output": {
    "executable_name": "Kurulus.exe",
    "icon": "assets/icons/app_icon.ico",
    "product_name": "Kuruluş",
    "company_name": "[Studio Name]",
    "copyright": "© 2026 [Studio Name]"
  }
}
```

### 5.2 Build Artifacts

```
Build Output (Release):
├── Kurulus.exe
├── Kurulus.pdb                  (debug symbols, optional)
├── Data/
│   ├── generated/               (optimized runtime data)
│   ├── locales/                 (locale files)
│   └── config/                  (config files)
├── Assets/
│   └── ...                      (packed assets)
├── README.txt
└── credits.txt
```

---

## 6. Environment Detection

At startup, the game detects the runtime environment:

| Detection | Purpose |
|-----------|---------|
| **Steam running** | Enable Steamworks integration |
| **Platform OS** | Load platform-specific defaults |
| **GPU capabilities** | Determine max graphics settings |
| **Available RAM** | Adjust texture budget, streaming |
| **Monitor count** | Multi-monitor support |
| **Controller connected** | Enable controller UI hints |
| **Portable mode** | Check if running from USB (save in local folder) |

```python
def detect_environment():
    env = {
        "steam_active": detect_steam(),
        "platform": detect_os(),
        "gpu": detect_gpu_capabilities(),
        "ram_mb": psutil.virtual_memory().total // (1024*1024),
        "monitor_count": len(get_monitors()),
        "controller_connected": any(joystick for joystick in get_joysticks()),
        "portable_mode": check_portable_mode(),
        "debugger_attached": is_debugger_present()
    }
    return env
```

---

## 7. Config Validation at Startup

```python
def validate_build_config():
    errors = []

    # Check required files exist
    required_files = [
        "Data/generated/units.json",
        "Data/generated/buildings.json",
        "Data/generated/heroes.json",
        "Data/locales/ar.json",
        "Data/config/platform_config.json"
    ]
    for path in required_files:
        if not file_exists(path):
            errors.append(f"Missing required file: {path}")

    # Check feature toggles for consistency
    if not features.ambush_system.enabled and mission_02.requires_ambush:
        errors.append("Mission 02 requires ambush system but it's disabled")

    # Check locale completeness
    for mission in mission_definitions:
        if not locale_has_key(f"mission_{mission.id}_name"):
            errors.append(f"Missing locale key for mission {mission.id}")

    if errors:
        log_errors(errors)
        if build_variant == "debug":
            show_error_dialog(errors)
        else:
            # Graceful fallback in release
            log_warning("Build validation found issues (continuing with fallbacks)")
```

---

## 8. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Configuration architecture |
| `02_Runtime_Config.md` | Runtime CVars that complement build config |
| `03_Localization_Config.md` | Locale files bundled in build |
