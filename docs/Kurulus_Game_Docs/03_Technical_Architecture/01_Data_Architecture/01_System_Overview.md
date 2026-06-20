# Data Architecture — System Overview

> **Description:** Defines the logical data layer for *Kuruluş* — how data flows between disk, runtime memory, game logic, and UI. Technology-agnostic; applies to any engine (Unity, Unreal, Godot, custom).
>
> **Folders:** `01_Data_Architecture/`

---

## 1. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Separation of static and dynamic data** | Static = balance tables, definitions (change per patch); Dynamic = runtime state (save/load). Never mix the two. |
| 2 | **Data-driven design** | All numeric values, thresholds, unlock conditions live in external config files — not in code. Tuning does not require recompilation. |
| 3 | **Deterministic serialization** | Save/load must produce bit-identical state for replay and debug. |
| 4 | **Locale-independent storage** | No hardcoded strings in data. Every text field is a string key referencing a localization table. |
| 5 | **Debug transparency** | Every data value must be inspectable at runtime via a developer console / debug overlay. |

---

## 2. Logical Data Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     GAME CLIENT                          │
│                                                          │
│  ┌─────────────────┐    ┌──────────────────────────┐    │
│  │   Static Data    │    │     Dynamic Data          │    │
│  │  (Read-only at   │    │   (Read/Write at          │    │
│  │   runtime)       │    │    runtime)               │    │
│  │                  │    │                           │    │
│  │ • Balance tables │    │ • Session state           │    │
│  │ • Unit defs      │    │ • Entity runtime stats    │    │
│  │ • Building defs  │    │ • Inventory / economy     │    │
│  │ • Hero abilities │    │ • World progress          │    │
│  │ • Dialogue trees │    │ • Player profile          │    │
│  │ • Localization   │    │ • Achievements            │    │
│  │ • Anim clips ref │    │ • Replay data             │    │
│  └────────┬─────────┘    └───────────┬───────────────┘    │
│           │                          │                     │
│           │          ┌───────────────┴────────┐            │
│           │          │    Data Access Layer     │            │
│           │          │   (Service / Manager)    │            │
│           │          │                          │            │
│           └──────────►  • StaticDB (read)       │            │
│                      │  • SaveManager (write)   │            │
│                      │  • RuntimeDB (volatile)  │            │
│                      └───────────┬──────────────┘            │
│                                  │                           │
│              ┌───────────────────┴──────────────────┐        │
│              │         Serialization Engine          │        │
│              │  (JSON / Binary / Custom Format)       │        │
│              └───────────────────┬──────────────────┘        │
│                                  │                           │
└──────────────────────────────────┼───────────────────────────┘
                                   │
                        ┌──────────┴──────────┐
                        │     Storage Layer    │
                        │  • Local file system │
                        │  • Cloud save (opt)  │
                        │  • PlayerPrefs / KVS │
                        └─────────────────────┘
```

---

## 3. Data Classification Matrix

| Category | Examples | Source | Access | Serialized? |
|----------|----------|--------|--------|-------------|
| **Balance Data** | Unit HP, build times, costs | JSON/CSV files | Read-only at runtime | No |
| **Definition Data** | Building catalog, ability defs | JSON files | Read-only at runtime | No |
| **Localization** | All display strings | String tables (JSON/CSV) | Read-only at runtime | No |
| **Session State** | Current mission progress, alive units | Memory | Read/Write | Yes (save) |
| **Persistent Profile** | Unlocks, total gold, achievements | Save file(s) | Read/Write | Yes (save) |
| **Replay Data** | Player input sequence, RNG seed | Memory → Disk | Write-only | Yes (replay) |
| **Config/Options** | Volume, graphics, key bindings | Config file / KVS | Read/Write | Yes (settings) |
| **Debug State** | Dev console vars, debug overlays | Memory | Read/Write | No |

---

## 4. Key Principles

### 4.1 Data-Driven Philosophy

- **No hard-coded numbers.** Every tunable value must be in a data file.
- **One source of truth.** A value exists in exactly one place. No duplication.
- **Fail fast.** Missing data files crash on startup with a clear error message (file name + field path).
- **Hot-reload (dev only).** In debug builds, reloading data files triggers reimport without restarting the game.

### 4.2 Versioning & Migration

| Scenario | Strategy |
|----------|----------|
| New field added | Default value in code handles missing field |
| Field removed | Migration script strips field on load |
| Value range changed | Clamp on load, log warning |
| Save file format bump | Version number in header → conditional deserialization |
| Balance patch | Only static data files change — save files unaffected |

### 4.3 Locale Independence

```
// ✅ Correct
"unit_swordsman_name" = "مبارز"  // Arabic locale
"unit_swordsman_name" = "Swordsman"  // English locale
"unit_swordsman_desc" = "وحدة مشاة خفيفة..."  // Arabic

// ❌ Wrong
"name" = "مبارز"  // Hardcoded Arabic — cannot localize
```

All strings in static data are **keys**, not values. The display layer always goes through the localization system.

---

## 5. Data Pipeline

```
[Designer] ──► CSV/JSON ──► [Import Tool] ──► Cleaned Data ──► [Runtime DB]
                         │                    │
                         ▼                    ▼
                    Validation Report    Backup / Versioned
```

| Step | Description |
|------|-------------|
| **Authoring** | Designer edits CSV or JSON files (Excel, Google Sheets, or text editor) |
| **Validation** | Import tool checks: ranges, required fields, cross-references, unique IDs |
| **Build** | Clean data is embedded in game build or loaded from streaming assets |
| **Runtime** | Data loaded into indexed lookup structures (hash maps / arrays) |
| **Debug** | Dev overlay can inspect any value by path or ID |

---

## 6. Data Integrity Rules

| Rule | Description | Violation Handling |
|------|-------------|-------------------|
| **Unique IDs** | Every entity/building/ability has a globally unique string ID | Crash on startup with duplicate ID listed |
| **Foreign Key Check** | References (e.g. building → resource) must resolve to existing entries | Log error, skip entry |
| **Range Check** | All numeric values must be within defined min/max | Clamp to range, log warning |
| **Required Fields** | Certain fields must be non-null/non-empty | Fail to load entry, log error |
| **Cross-Reference Integrity** | If a unit references an ability, that ability must exist | Fail to load referencing unit |

---

## 7. Technology Considerations

| Concern | Recommendation (Engine-Agnostic) |
|---------|----------------------------------|
| **Format** | JSON for human readability + version control. CSV for spreadsheet-heavy tables (balance). |
| **Compression** | For large data sets (dialogue trees): compress at build time, decompress on first load. |
| **Async Loading** | All static data loaded on a background thread during splash screen. Progress bar shown. |
| **Memory** | Static data stays loaded for entire session. Dynamic data freed on scene/mission transitions. |
| **Modding** | Data files in a well-documented `Data/` folder. Modder replaces file → game picks it up. |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `02_Static_Data.md` | Static data definitions — balance tables, entity catalogs |
| `03_Dynamic_Data.md` | Runtime state management, save/load, serialization |
| `04_Data_Pipeline.md` | Import, validation, hot-reload pipelines |
| `05_Configuration/01_System_Overview.md` | Config layer (options, CVars) |
