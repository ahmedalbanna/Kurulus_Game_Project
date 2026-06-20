# Data Pipeline — Import, Validation & Hot-Reload

> **Description:** Defines the end-to-end journey of game data from authoring tools to runtime consumption. Covers import pipeline, validation rules, error handling, and developer hot-reload.

---

## 1. Pipeline Overview

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  AUTHORING    │    │   IMPORT      │    │  VALIDATION   │    │   RUNTIME    │
│  (Designer)   │───►│  (Build Tool) │───►│   (Checker)   │───►│   (Game)     │
│               │    │               │    │               │    │              │
│ • CSV/JSON    │    │ • Parse       │    │ • Schema      │    │ • Hash map   │
│ • Excel/GSheets│   │ • Normalize   │    │ • Range       │    │ • Tag index  │
│ • Text editor │    │ • Optimize    │    │ • Cross-ref   │    │ • Hot-reload │
│ • Version     │    │ • Pack        │    │ • Balance     │    │ • Dev overlay│
│   control     │    │               │    │               │    │              │
└──────────────┘    └──────────────┘    └──────────────┘    └──────────────┘
```

---

## 2. Authoring Phase

### 2.1 Authoring Tools

| Data Type | Recommended Tool | Format | Notes |
|-----------|-----------------|--------|-------|
| Balance tables (numbers) | Google Sheets / Excel | CSV | Designers love spreadsheets |
| Entity definitions | Any JSON editor | JSON | Structured, hierarchical |
| Dialogue trees | yEd / Graphviz / JSON | JSON (graph) | Visual graph → exported JSON |
| Talent trees | Dedicated editor or JSON | JSON | Nested tier structure |
| Localization | Google Sheets → export | CSV/JSON | One row per key, columns = locales |
| Mission definitions | JSON editor | JSON | Configures objectives, spawns |

### 2.2 Source Control

- **All data files** live in the repository under a `Data/` directory
- **CSV files** are the single source of truth for balance values
- **JSON files** are the single source of truth for structured definitions
- A **generated** directory (`.gitignore`) contains optimized runtime copies
- CI pipeline validates data on every commit (see §4)

---

## 3. Import Pipeline (Build Tool)

### 3.1 Build Steps

| Step | Description | Output |
|------|-------------|--------|
| 1. **Collect** | Scan data directory, resolve file list | File manifest |
| 2. **Parse** | Read each file into intermediate representation | Raw DataTable |
| 3. **Normalize** | Convert to canonical format (e.g. all times in seconds, all costs to int) | Normalized DataTable |
| 4. **Resolve References** | Replace string IDs with direct memory pointers / indices | Linked DataTable |
| 5. **Validate** | Run validation rules (§4); fail build on errors | Validation report |
| 6. **Optimize** | Strip comments, compress, reorder for cache | Optimized binary |
| 7. **Pack** | Write to target format (runtime JSON or binary) | Final asset file(s) |
| 8. **Index** | Build lookup hash maps, tag indices | Index file |

### 3.2 Build Outputs

```
Data/
├── source/                       # Authoring source files (version controlled)
│   ├── units.csv
│   ├── buildings.json
│   ├── heroes.json
│   └── ...
│
├── generated/                    # Build artifacts (gitignored)
│   ├── runtime_units.json
│   ├── runtime_buildings.bytes   (binary)
│   ├── indices.json
│   └── ...
│
├── schemas/                      # JSON Schema files (for validation & IDE autocomplete)
│   ├── unit_schema.json
│   ├── building_schema.json
│   └── ...
│
└── reports/                      # Build reports
    ├── validation_report.html
    └── balance_report.csv
```

### 3.3 Command-Line Interface

```bash
# Full rebuild
buildtool --rebuild

# Fast incremental (only changed files)
buildtool --incremental

# Validate only (no output)
buildtool --validate-only

# Export to specific format
buildtool --format binary --output ./build/data/
```

---

## 4. Validation Rules

### 4.1 Validation Categories

| Category | Severity | Example Violation |
|----------|----------|-------------------|
| **Schema** | Error | Missing required field `base_stats.hp` |
| **Range** | Error | Unit HP set to -50 (below min 1) |
| **Cross-Reference** | Error | Ability references nonexistent `buff_xyz` |
| **Unique ID** | Error | Duplicate unit ID `unit_swordsman_basic` |
| **Balance** | Warning | Swordsman costs more gold than elite version |
| **Performance** | Warning | Building model has 50,000 triangles |
| **Style** | Warning | ID uses spaces instead of underscores |

### 4.2 Validation Report Example

```json
{
  "status": "FAILED",
  "timestamp": 1704067200,
  "files_checked": 24,
  "errors": [
    {
      "file": "units.csv",
      "line": 42,
      "field": "unit_archer_basic.hp",
      "severity": "error",
      "message": "HP (0) is below minimum (1)"
    }
  ],
  "warnings": [
    {
      "file": "buildings.json",
      "path": "$.buildings[3].levels[0].cost.gold",
      "severity": "warning",
      "message": "Gold cost (0) — intentional free building?"
    }
  ]
}
```

### 4.3 CI Integration

```yaml
# Pseudocode CI pipeline step
on: push to branches [main, develop]
steps:
  - checkout
  - run: buildtool --validate-only
  - if: validation contains errors
      fail build with report
  - run: buildtool --rebuild
  - archive: generated/
```

---

## 5. Runtime Data Loading

### 5.1 Loading Sequence

```
1. Game starts
2. Display splash screen
3. Begin async load of static data (background thread)
4. Show loading progress (e.g. "Loading game data... 45%")
5. Validate runtime integrity (checksum generated vs embedded)
6. Initialize DataManager
7. Build runtime lookup structures
8. Signal "Data Ready" → proceed to main menu
```

### 5.2 Data Access API (Logical)

```python
# Pseudocode for the data access interface

class DataManager:
    # Lookup by ID
    def get_unit(unit_id: str) -> UnitDef
    def get_building(building_id: str) -> BuildingDef
    def get_ability(ability_id: str) -> AbilityDef

    # Query by tag
    def find_units_by_tag(tag: str) -> List[UnitDef]
    def find_buildings_by_tag(tag: str) -> List[BuildingDef]

    # Economy
    def get_recipe(output_id: str) -> Recipe
    def get_upgrade_cost(building_id: str, from_tier: int, to_tier: int) -> CostBlock

    # Locale
    def get_localized_string(key: str, locale: str) -> str

    # Hot-reload
    def reload_data_source(source_name: str) -> bool
```

---

## 6. Hot-Reload System (Developer Mode)

### 6.1 Trigger Mechanisms

| Trigger | Description |
|---------|-------------|
| **File watcher** | On change in `Data/source/`, auto-reload affected tables |
| **Console command** | `reload_data units` — reload specific table |
| **Drag-and-drop** | Drop JSON file onto game window → import and reload |
| **Build tool re-run** | Re-run buildtool while game is running → game detects new `generated/` files |

### 6.2 Hot-Reload Protocol

```
Game running...
  │
  ├── File watcher detects: Data/source/units.csv changed
  │
  ├── 1. Lock data access (prevent reads during swap)
  ├── 2. Re-parse modified file
  ├── 3. Run validation on new data
  ├── 4. If validation fails → log error, keep old data, unlock
  ├── 5. If validation passes → swap pointer to new data table
  ├── 6. Notify dependent systems (e.g., UI tooltips, unit stat displays)
  ├── 7. Log: "Data reloaded: units.csv (5 entries, 1 modified)"
  └── 8. Unlock data access
```

### 6.3 Constraints

- Hot-reload only modifies **static data** (definitions, balance)
- **Dynamic/runtime state** is never modified by hot-reload
- Hot-reload is **disabled in release builds** (production)
- UI components that display data values refresh on next tick

---

## 7. Error Handling Strategy

| Scenario | Response |
|----------|----------|
| Data file missing | Show error dialog with file name. Do not crash. Offer to reinstall. |
| Validation error in release build | Log error, use fallback default values, warn user on next launch |
| Corrupted save file | Attempt repair from backup `.bak`; if both fail, offer to start new game |
| Data version mismatch | Show "This save file is from an older/newer version" — block load or attempt migration |
| Hot-reload failure | Keep old data, log detailed error, notify developer (console notification) |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Data architecture classifications |
| `02_Static_Data.md` | Definition of all static data tables |
| `03_Dynamic_Data.md` | Save/load and runtime state |
| `05_Configuration/02_Runtime_Config.md` | CVars and runtime configuration |
