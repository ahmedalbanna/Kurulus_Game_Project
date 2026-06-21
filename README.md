# KURULUŞ: FOUNDATIONS OF AN EMPIRE

**A single-player isometric RTS inspired by the epic story of Osman Bey and the founding of the Ottoman Empire.**

> Genre: Real-Time Strategy (Isometric) — Single Player  
> Target Platform: PC (controller expansion planned)  
> Art Style: Low-Poly Stylized 3D (Phase 2) / 2D Sprite (Phase 1)  
> Engine: [Bevy 0.15](https://bevyengine.org/) (Rust ECS)  
> Estimated Campaign: 30–40 hours across 5 seasons, 49 missions

---

## Game Overview

Lead the Kayı Tribe as **Osman Bey** through 13th-century Anatolia. Balance guerrilla warfare, tribal politics, and state-building in a living world where every decision determines your people's survival.

### Core Loop

```
Plan (build, upgrade, train, research)
  → Execute (move army, ambush, siege, defend)
    → Harvest (resources, loot, XP, sovereignty points)
      → (back to Plan)
```

### Five Pillars

| Pillar | Essence |
|--------|---------|
| **Tribe Sovereignty** | You are the Bey — decisions carry real weight |
| **Guerrilla Tactics** | Ambushes are the game's DNA |
| **Tangible Progression** | Every upgrade is visible on screen |
| **Living World** | Dynamic events, day/night cycle, seasons |
| **Historical Soul** | Epic inspired by the spirit of history |

---

## Architecture

The project is organized as a **Bevy workspace** with 7 crates inside `kurulus/`:

```
.
├── setup.sh                  ← Unified setup script (deps + Rust + build + MCP)
├── opencode.json             ← MCP server configuration for AI-assisted dev
├── scripts/
│   └── setup-mcp.sh          ← MCP tools setup script
├── docs/                     ← Game design docs & production plan
└── kurulus/
    ├── Cargo.toml              ← Workspace root
    ├── kurulus-core/           ← Core types, ECS components, events, resources, states
    ├── kurulus-config/         ← RON asset loader for game config files
    ├── kurulus-systems/        ← All game logic systems (22 plugins)
    ├── kurulus-render-2d/      ← 2D rendering pipeline (Phase 1)
    ├── kurulus-render-3d/      ← 3D rendering pipeline (Phase 2)
    ├── kurulus-app/            ← Application entry point & plugin registration
    └── kurulus-editor/         ← Mission editor tool (WIP)
```

### Crate Dependencies

```
kurulus-app ──────────┬── kurulus-render-2d (optional)
                       ├── kurulus-render-3d (optional)
                       ├── kurulus-systems
                       ├── kurulus-config
                       └── kurulus-core
                              ↑
                         (referenced by all crates)
```

### ECS Architecture (Phase-based Simulation)

The simulation runs in 9 ordered system sets:

| Phase | Systems | Purpose |
|-------|---------|---------|
| 1 | `InputProcessing` → `CommandQueueing` | Player & AI input |
| 2 | `AIDecision` → `AICommandIssuing` | AI decision-making |
| 3 | `Pathfinding` → `FormationMovement` → `GridQuantization` | Movement |
| 4 | `VisionCalculation` → `StealthDetection` → `AmbushTrigger` | Detection |
| 5 | `AttackTimers` → `MeleeResolution` → `RangedResolution` → `SiegeResolution` → `DamageApplication` → `MoraleUpdate` → `DeathProcessing` | Combat |
| 6 | `ResourceProduction` → `ConstructionProgress` → `ResearchProgress` → `XPGain` | Economy |
| 7 | `MissionObjectiveCheck` → `FactionRelationUpdate` → `WorldEventProcessing` | World |
| 8 | `DelegationProcessing` → `TrustUpdate` → `SynergyCalculation` | Co-Rule |
| 9 | `ComponentCleanup` → `PositionSync` | Cleanup |

---

## Game Systems (22 Logic Plugins)

| # | System | Description |
|---|--------|-------------|
| 1 | **Core Loop** | Game phase state machine (Build → Combat → Economy) |
| 2 | **World Map** | Region/faction context, territory control, fog of war |
| 3 | **AI** | Byzantine (defensive/fortified) and Mongol (fast/aggressive) |
| 4 | **Economy** | 4 primary resources (Wood, Iron, Food, Gold) + secondary resources |
| 5 | **Unit Command** | Move, attack, formations (Line/Column/Square/Crescent/Spread) |
| 6 | **Combat** | 5 damage types, morale system, friendly fire, weapon types |
| 7 | **Stealth & Ambush** | Terrain hiding, 5 ambush types (Frontal/Flank/Rear/Pincer/Circle) |
| 8 | **Hero** | Leveling, ability trees, gear (weapon/armor/accessory) |
| 9 | **Base Building** | 12 building types, 3 visual tiers per building |
| 10 | **Progression** | 3-path tech tree (15 tiers), sovereignty points, prestige |
| 11 | **Narrative** | Mission objectives, dialogue triggers, branching story |
| 12 | **Diplomacy** | Faction relations, treaties, war declarations |
| 13 | **Beylik Politics** | Inter-tribal relations (Season 3+) |
| 14 | **Next-Gen Heroes** | Orhan, Alaeddin, Fatma as playable heroes (Season 5) |
| 15 | **Romance & Loyalty** | Emotional metrics, companion bonds |
| 16 | **Succession & Legacy** | Heir system, dynasty mechanics |
| 17 | **Siege Warfare** | Siege engines, wall breach, multi-phase assault |
| 18 | **Imperial Admin** | State institutions, governance, justice system |
| 19 | **Naval Warfare** | Ship combat, coastal control |
| 20 | **Heir Co-Rule** | Delegation, trust matrix, synergy between rulers |
| 21 | **Audio Logic** | Event-to-sound mapping, dynamic music |
| 22 | **UI Logic** | HUD, minimap, panel system (data-binding, no bevy_ui rendering) |

### Render Pipeline (Feature-gated)

- `default` → `render-2d` — 2D sprite rendering with spritesheet, camera, debug overlays
- `render-3d` — 3D PBR rendering with terrain, animation, camera
- `headless-test` — Headless test harness (no rendering)

---

## Campaign Structure

```
Season 1 — SPARK (M1–M6)        Launch season
Season 2 — SUCCESSION (M7–M11)  Extension
Season 3 — BEYLIK WARS (M12–M21) Extension
Season 4 — EMPIRE (M22–M34)     Extension
Season 5 — LEGACY (M35–M49)     Finale
```

Full mission details are in `docs/Kurulus_Game_Docs/02_Campaign/`.

---

## Getting Started

### Quick Install (Recommended)

The project provides a unified setup script that handles everything — system dependencies, Rust toolchain, project build, and optional MCP developer tools:

```bash
# Full setup (checks system deps, installs Rust if needed, builds project,
# and prompts to install MCP debugging tools)
./setup.sh

# Quick setup — skip MCP tools, just system deps + Rust + build
./setup.sh --quick

# CI mode — non-interactive, full install
./setup.sh --ci

# Check prerequisites only (no modifications)
./setup.sh --check
```

### Prerequisites (Manual)

If you prefer to set things up by hand:

- Rust toolchain (stable, **edition 2021**) — install via [rustup](https://rustup.rs/)
- [Bevy system dependencies](https://bevyengine.org/learn/quick-start/getting-started/#set-up):
  - **Debian/Ubuntu:** `sudo apt install libasound2-dev libudev-dev libx11-dev libx11-xcb-dev libxcb-randr0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxcb-xinput-dev libxkbcommon-dev libwayland-dev`
  - **Fedora:** `sudo dnf install alsa-lib-devel systemd-devel libX11-devel libxcb-devel libxkbcommon-devel wayland-devel`
  - **Arch:** `sudo pacman -S alsa-lib systemd-libs libx11 libxcb libxkbcommon wayland`
  - **openSUSE:** `sudo zypper install alsa-devel systemd-devel libX11-devel libxcb-devel libxkbcommon-devel wayland-devel`
  - **Alpine:** `sudo apk add alsa-lib-dev eudev-dev libx11-dev libxcb-dev libxkbcommon-dev wayland-dev`

### Build & Run

```bash
# Default (2D rendering)
cd kurulus
cargo run

# 3D rendering
cargo run --features render-3d --no-default-features

# Headless test harness
cargo run --features headless-test --no-default-features
```

### Run Tests

```bash
cd kurulus
cargo test
```

### Build Only (no execution)

```bash
cd kurulus
cargo check
```

---

## AI-Assisted Development (MCP)

هذا المشروع مزود بأدوات **Model Context Protocol (MCP)** مخصصة لمحرك **Bevy**، والتي تسمح لمساعد الأكواد **OpenCode** بالتحكم في اللعبة وفحصها وتصحيحها وهي تعمل في الخلفية.

### الأدوات المتاحة

| الأداة | الوصف |
|--------|-------|
| [`bevy_brp_mcp`](https://crates.io/crates/bevy_brp_mcp) | خادم MCP للتحكم عن بُعد عبر **BRP (Bevy Remote Protocol)** — يتيح فحص وتعديل الـ Entities والـ Components والـ Resources أثناء التشغيل |
| [`bevy-debugger-mcp`](https://crates.io/crates/bevy_debugger_mcp) | مصحح أخطاء ذكي — يسمح بطرح أسئلة بلغة طبيعية عن حالة اللعبة (مثل: "لماذا تنخفض الروح المعنوية؟") |

### الإعداد السريع

```bash
# الطريقة الموصى بها — إعداد كل شيء (التبعيات النظامية + Rust + الأدوات)
./setup.sh

# أو لتثبيت أدوات MCP فقط
./scripts/setup-mcp.sh
```

### الإعداد اليدوي

```bash
# 1. تثبيت أدوات MCP عبر Cargo
cargo install bevy_brp_mcp bevy_debugger_mcp

# 2. تشغيل اللعبة مع تفعيل التحكم عن بُعد
cd kurulus
cargo run -p kurulus-app --features remote
```

### بنية الإعداد

```
setup.sh                         ← سكريبت الإعداد الشامل (تبعيات النظام + Rust + بناء + أدوات MCP)
scripts/setup-mcp.sh             ← سكريبت تثبيت وتحقق لأدوات MCP فقط
opencode.json                    ← تكوين خادمي MCP للمشروع
```

**ملف `opencode.json`** في جذر المشروع يُعرّف الخادمين لـ OpenCode:

```json
{
  "mcp": {
    "bevy-brp": {
      "type": "local",
      "command": ["bevy_brp_mcp"],
      "enabled": true,
      "description": "Bevy Remote Protocol MCP — فحص وتعديل ECS في الوقت الحقيقي"
    },
    "bevy-debugger": {
      "type": "local",
      "command": ["bevy-debugger-mcp"],
      "enabled": true,
      "description": "AI-assisted debugging — تحليل سلوك اللعبة بلغة طبيعية"
    }
  }
}
```

### تفعيل BRP في كود Rust

تمت إضافة `RemotePlugin` و `RemoteHttpPlugin` بشكل شرطي خلف ميزة `remote` في ملف `plugins.rs`:

```rust
#[cfg(feature = "remote")]
{
    use bevy::remote::RemotePlugin;
    use bevy::remote::http::RemoteHttpPlugin;
    app.add_plugins(RemotePlugin::default())
       .add_plugins(RemoteHttpPlugin::default());
}
```

### ماذا يمكنك فعله بعد الإعداد؟

بعد تشغيل اللعبة مع `--features remote` وفتح OpenCode في جذر المشروع:

| الطلب | النتيجة |
|-------|---------|
| "أظهر لي جميع الكيانات في ساحة المعركة" | استعلام ECS وعرض الكائنات |
| "تحقق من صحة (Health) وحدات كايي" | قراءة Component الـ `Health` لكل وحدة |
| "زِد نقاط الروح المعنوية للجندي رقم 5" | تعديل Resource مباشر أثناء التشغيل |
| "لماذا فشل نظام التخفي؟" | تحليل سجلات النظام وتحديد المشكلة |
| "راقب سلوك نظام `CombatPlugin`" | مراقبة Component Watching في الوقت الحقيقي |

### التحقق من الإعداد

```bash
# فحص النظام بالكامل (تبعيات النظام + Rust + بناء + MCP)
./setup.sh --check

# أو فحص أدوات MCP فقط
./scripts/setup-mcp.sh --check
```

الإخراج المتوقع (من `./setup.sh --check`):

```
  ✓ All Bevy system dependencies found
  ✓ rustc 1.96.0
  ✓ cargo found at /root/.cargo/bin/cargo
  ✓ Rust version meets Bevy requirement (>= 1.77)
  ✓ Build succeeded (default features)
  ✓ remote feature compiles
  ✓ opencode.json found
  ✓ MCP server 'bevy-brp' configured
  ✓ MCP server 'bevy-debugger' configured
```

### روابط مفيدة

- [Bevy Remote Protocol (BRP)](https://docs.rs/bevy/0.15/bevy/remote/index.html) — التوثيق الرسمي لبروتوكول التحكم عن بُعد
- [MCP Servers for Rust Developers](https://www.youtube.com/watch?v=0pYIAMlEmR0) — فيديو يشرح دور خوادم MCP في تحليل الشيفرات البرمجية لمشاريع Rust
- [bevy_brp على GitHub](https://github.com/natepiano/bevy_brp) — المستودع الرسمي للأدوات

---

## Configuration

Game data is stored as **RON** files in `kurulus/kurulus-config/assets/`:

```
assets/
├── balance/       # Economy balance tables
├── buildings/     # Building definitions
├── heroes/        # Hero stats & ability trees
├── missions/      # Mission definitions
├── tech/          # Tech tree data
├── units/         # Unit archetypes & stats
└── world/         # World map regions & factions
```

The custom `RonAssetLoader` in `kurulus-config` handles loading these at runtime.

---

## Documentation

| Document | Location |
|----------|----------|
| Game Design Document (Arabic/English) | `docs/GDD.md` |
| Core Design Bible | `docs/Kurulus_Game_Docs/00_Core_Design/` |
| Feature Specs (22 systems) | `docs/Kurulus_Game_Docs/01_Features/` |
| Campaign & Mission Design | `docs/Kurulus_Game_Docs/02_Campaign/` |
| Technical Architecture | `docs/Kurulus_Game_Docs/03_Technical_Architecture/` |
| Reference Materials | `docs/Kurulus_Game_Docs/04_Reference/` |
| Production Plan & Roadmap | `docs/Kurulus_Game_Docs/05_Production/` |

---

## Development Status

See `docs/Kurulus_Game_Docs/05_Production/` for the full production plan and sprint tracking.

---

## License

MIT
