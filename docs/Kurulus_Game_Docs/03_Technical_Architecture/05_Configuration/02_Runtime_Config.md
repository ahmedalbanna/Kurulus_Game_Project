# Runtime Configuration — CVars, Developer Console & Cheats

> **Description:** Defines the runtime-configurable variables (CVars), developer console system, and cheat/debug overrides available in *Kuruluş*. These allow developers, testers, and (optionally) modders to tweak game behavior without recompilation or restart.

---

## 1. CVar System

### 1.1 CVar Classification

| Class | Persistence | Available In | Example |
|-------|-------------|--------------|---------|
| **Startup** | Config file | Release + Debug | `graphics.resolution.width` |
| **Runtime** | Config file | Release + Debug | `audio.volume.master_volume` |
| **Developer** | Not persisted | Debug only | `debug.show_fps` |
| **Cheat** | Not persisted | Debug only | `debug.god_mode` |
| **Tuning** | Config file (optional) | Release + Debug | `gameplay.difficulty.damage_multiplier` |

### 1.2 CVar Storage

```python
class CVar:
    def __init__(self, key: str, value, metadata: CVarMetadata):
        self.key = key
        self._value = value
        self.metadata = metadata  # type, range, description, flags

    @property
    def value(self):
        return self._value

    @value.setter
    def value(self, new_value):
        # Type check
        if not self._validate_type(new_value):
            raise CVarTypeError(self.key, self.metadata.type, new_value)
        # Range check
        if self.metadata.min is not None and new_value < self.metadata.min:
            new_value = self.metadata.min  # Clamp
        if self.metadata.max is not None and new_value > self.metadata.max:
            new_value = self.metadata.max  # Clamp
        # Set
        self._value = new_value
        # Notify
        config_manager.notify(self.key, new_value)
```

### 1.3 Complete CVar Reference

#### Graphics CVars

| CVar | Type | Default | Min | Max | Restart |
|------|------|---------|-----|-----|---------|
| `graphics.resolution.width` | int | 1920 | 640 | 7680 | Yes |
| `graphics.resolution.height` | int | 1080 | 480 | 4320 | Yes |
| `graphics.fullscreen` | bool | true | - | - | Yes |
| `graphics.v_sync` | bool | true | - | - | No |
| `graphics.quality.shadow_quality` | enum | "high" | - | - | No |
| `graphics.quality.texture_quality` | enum | "high" | - | - | Yes |
| `graphics.quality.anti_aliasing` | enum | "fxaa" | - | - | Yes |
| `graphics.quality.post_processing` | bool | true | - | - | No |
| `graphics.quality.view_distance` | enum | "medium" | - | - | No |
| `graphics.quality.particle_quality` | enum | "medium" | - | - | No |
| `graphics.fps_limit` | int | 60 | 30 | 240 | No |
| `graphics.brightness` | float | 1.0 | 0.5 | 1.5 | No |

#### Audio CVars

| CVar | Type | Default | Min | Max |
|------|------|---------|-----|-----|
| `audio.volume.master_volume` | float | 0.8 | 0.0 | 1.0 |
| `audio.volume.music_volume` | float | 0.7 | 0.0 | 1.0 |
| `audio.volume.sfx_volume` | float | 1.0 | 0.0 | 1.0 |
| `audio.volume.voiceover_volume` | float | 1.0 | 0.0 | 1.0 |
| `audio.volume.ambient_volume` | float | 0.8 | 0.0 | 1.0 |
| `audio.volume.ui_volume` | float | 0.6 | 0.0 | 1.0 |
| `audio.output.device` | string | "default" | - | - |

#### Control CVars

| CVar | Type | Default | Min | Max |
|------|------|---------|-----|-----|
| `control.camera.speed` | float | 0.5 | 0.1 | 2.0 |
| `control.camera.edge_scroll` | bool | true | - | - |
| `control.camera.edge_scroll_size` | int | 10 | 2 | 50 |
| `control.mouse.invert_y` | bool | false | - | - |
| `control.mouse.sensitivity` | float | 0.5 | 0.1 | 2.0 |
| `control.scroll_wheel.zoom_invert` | bool | false | - | - |

#### Gameplay CVars

| CVar | Type | Default | Min | Max | Notes |
|------|------|---------|-----|-----|-------|
| `gameplay.difficulty.level` | enum | "medium" | - | - | easy/medium/hard |
| `gameplay.difficulty.damage_to_player` | float | 1.0 | 0.5 | 2.0 | Multiplier |
| `gameplay.difficulty.damage_to_ai` | float | 1.0 | 0.5 | 2.0 | Multiplier |
| `gameplay.difficulty.ai_aggressiveness` | float | 1.0 | 0.0 | 2.0 | Multiplier |
| `gameplay.speed.time_scale` | float | 1.0 | 0.0 | 4.0 | Pause at 0 |

#### Debug CVars

| CVar | Type | Default | Description |
|------|------|---------|-------------|
| `debug.visual.show_fps` | bool | false | Show FPS counter |
| `debug.visual.show_grid` | bool | false | Show navigation grid |
| `debug.visual.show_stealth_zones` | bool | false | Visualize stealth zones |
| `debug.visual.show_ai_state` | bool | false | Show AI state labels |
| `debug.visual.show_pathfinding` | bool | false | Show pathfinding lines |
| `debug.visual.show_collision` | bool | false | Show collision boxes |
| `debug.visual.show_hitboxes` | bool | false | Show attack hitboxes |
| `debug.visual.show_zone_boundaries` | bool | false | Show trigger zones |
| `debug.visual.show_fow` | bool | false | Show fog of war data |
| `debug.gameplay.god_mode` | bool | false | Player units invulnerable |
| `debug.gameplay.unlimited_resources` | bool | false | Infinite resources |
| `debug.gameplay.reveal_map` | bool | false | Remove fog of war |
| `debug.gameplay.no_ai` | bool | false | Disable AI opponents |
| `debug.gameplay.instant_build` | bool | false | Instant construction |
| `debug.gameplay.speed_multiplier` | float | 1.0 | 0.1-10.0 game speed |

---

## 2. Developer Console

### 2.1 Console UI

| Aspect | Design |
|--------|--------|
| **Toggle key** | `~` (backtick / tilde) |
| **Position** | Top of screen, overlays game |
| **Background** | Semi-transparent dark overlay |
| **Font** | Monospace, 14px |
| **History** | Last 100 commands, scrollable |
| **Autocomplete** | Tab-complete CVar names and command names |
| **Multi-line** | Shift+Enter for multi-line scripts |

### 2.2 Console Commands Reference

| Command | Syntax | Description |
|---------|--------|-------------|
| `get` | `get <cvar>` | Get current CVar value |
| `set` | `set <cvar> <value>` | Set CVar value |
| `list` | `list <pattern>` | List CVars matching pattern (e.g., `graphics.*`) |
| `reset` | `reset <cvar>` | Reset CVar to default |
| `toggle` | `toggle <cvar>` | Toggle boolean CVar |
| `watch` | `watch <cvar>` | Monitor CVar for changes |
| `exec` | `exec <filename>` | Execute commands from file |
| `bind` | `bind <key> <command>` | Bind key to console command |
| `unbind` | `unbind <key>` | Remove key binding |
| `help` | `help [command]` | Show help for command |
| `spawn_unit` | `spawn_unit <unit_id> [x] [z]` | Spawn a unit (debug) |
| `complete_objective` | `complete_objective <obj_id>` | Force-complete objective |
| `trigger_event` | `trigger_event <event_type>` | Emit a game event |
| `save_state` | `save_state <filename>` | Save current state snapshot |
| `load_state` | `load_state <filename>` | Load state snapshot |
| `quit` | `quit` | Exit game |

### 2.3 Console Command Parser

```python
class ConsoleParser:
    def parse(self, input_string: str) -> Command:
        parts = shlex.split(input_string)
        if not parts:
            return None

        command_name = parts[0].lower()
        args = parts[1:]

        if command_name == "get":
            return GetCommand(args[0])
        elif command_name == "set":
            return SetCommand(args[0], " ".join(args[1:]))
        elif command_name == "list":
            return ListCommand(args[0] if args else "*")
        # ... etc
```

---

## 3. Cheat System (Debug Builds)

### 3.1 Cheat Activation

```
Method 1: Console command
  > debug.god_mode = true

Method 2: Key combination
  Ctrl+Shift+G → toggle god mode

Method 3: Menu toggle (debug menu)
  Debug Tools → Cheats → Enable God Mode

Method 4: Command-line argument
  --god-mode
```

### 3.2 Cheat List

| Cheat | CVar | Effect |
|-------|------|--------|
| God Mode | `debug.gameplay.god_mode` | Player units take no damage |
| Unlimited Resources | `debug.gameplay.unlimited_resources` | Resources never deplete |
| Reveal Map | `debug.gameplay.reveal_map` | Remove fog of war |
| No AI | `debug.gameplay.no_ai` | AI factions inactive |
| Instant Build | `debug.gameplay.instant_build` | Construction/production instant |
| Speed Hack | `debug.gameplay.speed_multiplier` | Game runs at N× speed |
| Kill All | `spawn_kill all` | Kill all entities of type |
| Complete Mission | `debug.complete_mission` | Force victory |

### 3.3 Cheat Protection (Release Builds)

- All `debug.*` CVars are **stripped** from release builds
- Console commands that modify gameplay are **disabled**
- Any attempt to access debug CVars logs a security warning
- Achievements are **not awarded** if cheats were active (even in debug — detected by CVar flag)

---

## 4. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Configuration architecture overview |
| `03_Localization_Config.md` | Locale and string table configuration |
| `04_Build_Config.md` | Platform and build-specific configuration |
