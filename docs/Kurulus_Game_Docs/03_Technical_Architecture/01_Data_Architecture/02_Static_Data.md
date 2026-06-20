# Static Data — Balance Tables & Definitions

> **Description:** Catalog of all static (read-only) data used by *Kuruluş*. Static data never changes during gameplay — it defines the rules, limits, and identity of every game element.

---

## 1. Entity Definition Tables

### 1.1 Unit Definitions

Each unit archetype is defined by a single record:

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `unit_swordsman_basic` | Globally unique, kebab-case |
| `display_name_key` | string | `unit_swordsman_name` | Localization key |
| `description_key` | string | `unit_swordsman_desc` | Localization key |
| `archetype` | enum | `infantry`, `cavalry`, `ranged`, `siege`, `hero`, `support` | |
| `tier` | int (1-3) | `1` | Base → Upgraded → Elite |
| `base_stats` | StatBlock | `{hp: 100, ...}` | See StatBlock schema below |
| `abilities` | string[] | `["ability_charge", "ability_shield_wall"]` | References ability IDs |
| `upgrade_from` | string? | `null` | ID of lower tier, if upgradeable |
| `cost` | CostBlock | `{gold: 50, food: 20, ...}` | |
| `build_time` | float (sec) | `15.0` | |
| `population_cost` | int | `1` | Population slots consumed |
| `visual_scale` | float | `1.0` | Multiplier on base model size |
| `selection_radius` | float | `0.5` | Click selection sphere radius |
| `tags` | string[] | `["human", "organic", "melee"]` | For gameplay queries |

### 1.2 StatBlock Schema

```json
{
  "hp": 100,
  "hp_regen": 0.5,
  "move_speed": 3.5,
  "rotation_speed": 180.0,
  "attack_damage": 12,
  "attack_cooldown": 1.2,
  "attack_range": 1.5,
  "attack_type": "slashing",
  "armor": 5,
  "armor_type": "light",
  "morale_base": 70,
  "morale_break_threshold": 25,
  "vision_range": 15.0,
  "stealth_detect_range": 5.0,
  "weight": 1.0
}
```

### 1.3 Building Definitions

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `building_barracks` | Globally unique |
| `display_name_key` | string | `building_barracks_name` | |
| `description_key` | string | `building_barracks_desc` | |
| `category` | enum | `economic`, `military`, `defense`, `infrastructure`, `special` | |
| `tier` | int (1-3) | `1` | |
| `levels` | LevelDef[] | See below | Array of upgrade levels |
| `placement` | PlacementRule | `{requires_flat: true, min_dist_from_enemy: 20}` | |
| `tags` | string[] | `["produces_units", "garrison"]` | |

**LevelDef (per tier):**
```json
{
  "tier": 1,
  "hp": 500,
  "armor": 10,
  "cost": {"gold": 100, "wood": 150},
  "build_time": 30.0,
  "production_slots": 1,
  "garrison_slots": 3,
  "provides": {"population": 10},
  "upgrade_cost": {"gold": 75, "iron": 50},
  "upgrade_time": 20.0
}
```

---

## 2. Ability & Upgrade Tables

### 2.1 Ability Definitions

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `hero_osman_sword_storm` | |
| `hero_id` | string | `hero_osman` | Which hero owns this |
| `display_name_key` | string | `ability_sword_storm_name` | |
| `description_key` | string | `ability_sword_storm_desc` | |
| `type` | enum | `active`, `passive`, `toggle`, `triggered`, `ultimate` | |
| `cooldown` | float (sec) | `12.0` | For active abilities |
| `mana_cost` | int | `30` | |
| `range` | float | `5.0` | |
| `duration` | float (sec) | `4.0` | |
| `effect` | EffectBlock | See below | |
| `unlock_talent_id` | string? | `talent_osman_sword_storm` | |

**EffectBlock:**
```json
{
  "damage_per_hit": 25,
  "hit_interval": 0.5,
  "max_targets": 5,
  "radius": 3.0,
  "status_effects": ["stagger"],
  "visual_fx": "fx_sword_storm"
}
```

### 2.2 Talent/Upgrade Definitions

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `talent_osman_sword_storm` | |
| `tree` | enum | `blacksmith`, `cavalry`, `faith` | |
| `tier` | int (1-5) | `3` | Position in tree |
| `prerequisites` | string[] | `["talent_osman_warrior_spirit"]` | |
| `cost` | int (SP) | `3` | Sovereignty Points |
| `effect_description` | string | `Increases Sword Storm radius by 1m` | |
| `modifiers` | ModifierBlock[] | See below | |

**ModifierBlock:**
```json
{
  "target_id": "ability_sword_storm",
  "property": "radius",
  "operation": "add",
  "value": 1.0
}
```

---

## 3. Economy Tables

### 3.1 Resource Definitions

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `resource_gold` | |
| `display_name_key` | string | `resource_gold_name` | |
| `category` | enum | `primary`, `secondary`, `special` | |
| `base_capacity` | int | `500` | Max storage at game start |
| `gather_rate` | float (units/sec) | `2.0` | Per gatherer |
| `decay_rate` | float (units/sec) | `0` | Perishable resources only |
| `icon` | string | `ico_resource_gold` | |

### 3.2 Production Recipes

Each item (unit, building, upgrade) references a cost block:

```json
{
  "recipe_id": "build_barracks_t1",
  "output_id": "building_barracks",
  "output_type": "building",
  "costs": {
    "resource_gold": 100,
    "resource_wood": 150,
    "resource_iron": 0,
    "resource_food": 0
  },
  "time_seconds": 30.0,
  "requires": [],
  "unlocked_by": "mission_1_complete"
}
```

---

## 4. Mission & Campaign Data

### 4.1 Mission Definitions

| Field | Type | Example | Notes |
|-------|------|---------|-------|
| `id` | string | `mission_01_kulucahisar` | |
| `chapter` | int | `1` | |
| `display_name_key` | string | `mission_01_name` | |
| `description_key` | string | `mission_01_desc` | |
| `objective_groups` | ObjectiveGroup[] | See below | |
| `starting_units` | string[] | `["hero_osman", "unit_swordsman_basic"]` | |
| `starting_resources` | CostBlock | `{gold: 100, wood: 50}` | |
| `victory_conditions` | string[] | `["capture_flag", "eliminate_enemy_commander"]` | |
| `defeat_conditions` | string[] | `["heroes_all_dead"]` | |
| `map_id` | string | `map_kulucahisar` | |

### 4.2 Objective Definitions

```json
{
  "objective_id": "obj_01_capture_flag",
  "type": "capture_point",
  "display_text_key": "obj_01_capture_flag_text",
  "target_id": "capture_point_keep",
  "completed_when": "player_owns_zone",
  "optional": false,
  "order": 1
}
```

---

## 5. Static Data File Manifest

| File Name | Contents | Estimate Size |
|-----------|----------|--------------|
| `units.json` | All unit definitions (12-15 entries) | ~15 KB |
| `buildings.json` | All building definitions (15-20 entries × 3 tiers) | ~25 KB |
| `heroes.json` | 3 hero definitions + 12 abilities | ~10 KB |
| `abilities.json` | All abilities (hero + unit + enemy) | ~20 KB |
| `talents.json` | 3 talent trees × 15 nodes | ~18 KB |
| `resources.json` | 8 resource definitions | ~3 KB |
| `missions.json` | 6 mission definitions | ~15 KB |
| `recipes.json` | All production recipes | ~12 KB |
| `objectives.json` | All objective definitions | ~8 KB |
| `loot_tables.json` | Drop tables for enemies | ~6 KB |
| `terrain_types.json` | Terrain definitions + movement costs | ~4 KB |
| `weather_patterns.json` | Weather effects (optional) | ~3 KB |

**Total estimated static data:** ~140 KB uncompressed JSON.

---

## 6. Schema Validation Rules

| Rule | Implementation |
|------|---------------|
| **ID Format** | Must match `^[a-z][a-z0-9_]*$` |
| **Value Ranges** | All numeric fields define `min` and `max` in schema; validator clamps out-of-range |
| **Required Fields** | Schema marks required vs optional; missing required field = hard error |
| **Cross-Refs** | Any ID reference (e.g. `unlock_talent_id`) must exist in its target table |
| **Unique Constraints** | Composite keys (e.g. building_id + tier) must be unique |
| **Cost Sum Check** | Zero-cost items flagged as warnings (intentional? e.g. basic units) |

---

## 7. Runtime Data Structures

| Pattern | Used For | Why |
|---------|----------|-----|
| **HashMap (ID → Object)** | All entity definitions | O(1) lookup by string ID |
| **Tag Index** | Gameplay queries ("find all buildings with tag `military`") | Pre-built inverted index |
| **Array (ordered)** | Talent trees, mission objectives | Order matters for display |
| **Bidirectional Map** | Display name key ↔ localized string | Avoid repeated lookups |
| **Sparse Array** | Grid-based data (terrain cost map) | Memory-efficient for large maps |

---

## 8. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Architecture principles, data classification |
| `03_Dynamic_Data.md` | How static data is consumed at runtime |
| `04_Data_Pipeline.md` | Import, validation, hot-reload |
