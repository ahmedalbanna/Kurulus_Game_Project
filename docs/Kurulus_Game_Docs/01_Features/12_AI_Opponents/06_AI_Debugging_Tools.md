# AI Debugging Tools — Developer Utilities for AI Testing

> **Description:** Defines debugging and visualization tools for AI behavior during development — making AI decisions transparent and testable.

---

## 1. AI State Visualization

### 1.1 Unit State Labels

When debug mode is active, each AI unit displays:

```
┌─────────────────┐
│ Spearman #042   │
│ State: ATTACK   │
│ HP: 67/100      │
│ Morale: 58/70   │
│ Target: #015    │
│ Threat: 0.72    │
└─────────────────┘
```

### 1.2 Color-Coded States

| State | Color | Description |
|-------|-------|-------------|
| IDLE | Gray | No action |
| MOVE | Blue | Moving to position |
| ATTACK | Red | Engaged in combat |
| FLEE | Yellow | Retreating |
| STEALTH | Purple (dim) | Hidden |
| ABILITY_USE | Orange | Using special ability |
| PATROL | Light Blue | Following patrol route |
| GARRISONED | White | Inside building |

---

## 2. AI Decision Log

```json
{
  "timestamp": 452.5,
  "entity_id": "byz_spearman_042",
  "decision_type": "target_selection",
  "input": {
    "nearby_enemies": [
      {"id": "player_sword_012", "distance": 8.0, "hp": 45, "threat": 0.6},
      {"id": "player_archer_005", "distance": 12.0, "hp": 30, "threat": 0.4},
      {"id": "hero_osman", "distance": 15.0, "hp": 200, "threat": 1.0}
    ],
    "own_hp": 67,
    "own_morale": 58
  },
  "decision": {
    "selected_target": "player_sword_012",
    "reason": "Highest threat within attack range",
    "score": 0.6,
    "alternative": "hero_osman (0.8 but out of range)"
  }
}
```

---

## 3. Debug Commands

| Command | Description |
|---------|-------------|
| `ai.debug_show_state` | Toggle AI state labels on all AI units |
| `ai.debug_show_threat` | Show threat scores for all entities |
| `ai.debug_show_path` | Show AI pathfinding lines |
| `ai.debug_show_goals` | Show current AI goal for each faction |
| `ai.debug_log_level [0-3]` | Set AI logging verbosity |
| `ai.force_goal <faction> <goal>` | Force AI faction to adopt a specific goal |
| `ai.disable` | Disable all AI |
| `ai.spawn_wave <faction> <unit> <count>` | Spawn attack wave |
| `ai.pause_ai` | Pause AI decision-making (toggle) |
| `ai.step` | Advance AI by one decision tick |

---

## 4. Debug Overlay

```
┌──────────────────────────────────────────────────────────┐
│  AI DEBUG OVERLAY                                         │
│                                                            │
│  Faction: Byzantine    State: EXPAND                       │
│  Army Power: 142       Economy: 340g 120w 80f             │
│  Current Goal: Build army (priority 70)                   │
│                                                            │
│  Decision Queue:                                          │
│  [▶] Build Barracks (45% complete)                        │
│  [▶] Train 3 Spearmen (1/3 complete)                     │
│  [⏸] Attack player base (waiting: 5 more units needed)   │
│                                                            │
│  ┌──────┬──────┬──────┬──────┬──────┬──────┬──────┐      │
│  │ IDLE │ MOVE │ ATK  │ FLEE │ STEL │ ABIL │ GARR │      │
│  │  12  │  20  │  15  │  3   │  0   │  2   │  8   │      │
│  └──────┴──────┴──────┴──────┴──────┴──────┴──────┘      │
└──────────────────────────────────────────────────────────┘
```

---

## 5. Testing Scenarios

| Scenario | Description | How to Test |
|----------|-------------|-------------|
| **Direct engagement** | AI vs AI in open field | `ai.spawn_wave byzantine unit_spearman 10` at position A, `ai.spawn_wave mongol unit_horse_archer 8` at position B |
| **Base defense** | AI defends against player assault | Start mission, `ai.force_goal byzantine defend_base` |
| **Flanking test** | AI attempts flank | Spawn player group, AI group with flanking enabled |
| **Retreat test** | AI retreats correctly | Set enemy HP to 30%, observe retreat behavior |
| **Economy stress** | AI manages resources under pressure | Give AI high resources, observe building priority |
| **Stealth/ambush** | AI detects and reacts to stealth | Place stealthed units near AI patrol |

---

## 6. AI Performance Monitoring

| Metric | Target | Warning Threshold |
|--------|--------|-------------------|
| AI decision time per entity | < 0.05 ms | > 0.1 ms |
| AI tick total time | < 3.0 ms | > 5.0 ms |
| Pathfinding queries per tick | < 50 | > 100 |
| AI memory usage | < 5 MB | > 15 MB |
| AI update frequency | Every tick | Skipping > 10% of ticks |

---

## 7. AI Behavior Test Suite

```python
# Pseudo test cases

def test_byzantine_shield_wall():
    """Verify Byzantine units form shield wall when enemy approaches"""
    ai = create_ai("byzantine", difficulty="medium")
    units = [create_unit("spearman") for _ in range(5)]
    enemy = create_unit("player_swordsman", position=(20, 0))

    ai.tick()
    assert units[0].state == "SHIELD_WALL"
    assert all(u.distance_to(u.formation_center) < 2.0 for u in units)

def test_mongol_feigned_retreat():
    """Verify Mongol units feign retreat when outnumbered"""
    ai = create_ai("mongol", difficulty="hard")
    units = [create_unit("light_cavalry") for _ in range(3)]
    enemies = [create_unit("player_swordsman") for _ in range(6)]

    ai.tick()
    assert units[0].state == "FLEE"
    assert ai.get_flag("feigned_retreat_active") == True
```

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Faction_AI_Personalities.md` | Faction personality profiles |
| `03_Strategic_AI.md` | Strategic planning |
| `04_Tactical_AI.md` | Tactical combat decision-making |
| `05_Difficulty_Scaling.md` | Difficulty parameters |
