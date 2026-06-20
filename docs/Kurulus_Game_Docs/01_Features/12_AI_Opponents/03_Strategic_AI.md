# Strategic AI — High-Level Planning & Resource Management

> **Description:** Defines the strategic AI system — how AI factions plan their economy, military production, expansion, and overall mission strategy.

---

## 1. Strategic AI Loop

```
STRATEGIC TICK (every 10 game seconds)
      │
      ├── 1. Evaluate state
      │     • Current resources
      │     • Army strength (total military power)
      │     • Territory controlled
      │     • Enemy (player) known positions
      │     • Active threats
      │
      ├── 2. Update goals
      │     • Short-term (next 60s): "Build 2 more spearmen"
      │     • Medium-term (next 5min): "Expand to gold mine"
      │     • Long-term (until done): "Eliminate player presence"
      │
      ├── 3. Generate plan
      │     • Series of actions to achieve goals
      │     • Assigns resources to each action
      │     • Prioritizes by urgency
      │
      └── 4. Execute plan
          • Issue build orders
          • Assign workers
          • Set rally points
          • Trigger attacks
```

---

## 2. AI Goal System

### 2.1 Goal Types

| Goal | Priority | Description |
|------|----------|-------------|
| **Survive** | 100 (highest) | Protect base, retreat when overwhelmed |
| **Expand** | 80 | Claim resource nodes, build forward bases |
| **Build Army** | 70 | Produce military units |
| **Attack Player** | 60 | Launch offensive operations |
| **Fortify** | 50 | Build walls, towers, defenses |
| **Research** | 40 | Upgrade units and buildings |
| **Economy** | 30 | Maintain resource income |
| **Scout** | 20 | Explore map, locate player |

### 2.2 Goal Selection Logic

```
IF (enemy units in base → SURVIVE priority 100)
ELSE IF (army < desired_strength → BUILD_ARMY)
ELSE IF (resources < threshold → ECONOMY)
ELSE IF (uncharted territory exists → SCOUT)
ELSE IF (defenses < threshold → FORTIFY)
ELSE → ATTACK_PLAYER
```

---

## 3. Economic AI

### 3.1 Resource Priorities

| Faction | Priority 1 | Priority 2 | Priority 3 |
|---------|------------|------------|------------|
| **Byzantine** | Food (sustain army) | Gold (upgrades) | Wood (buildings) |
| **Mongol** | Gold (cavalry) | Food (light units) | - (minimal building) |

### 3.2 Worker Allocation

```
BYZANTINE (20 worker cap):
  • Food: 6 (30%)
  • Gold: 6 (30%)
  • Wood: 5 (25%)
  • Build/Repair: 3 (15%)

MONGOL (no workers — pillaging economy):
  • Resources from raiding only
  • No buildings except camps
  • Units auto-generate over time
```

---

## 4. Military Production AI

### 4.1 Army Strength Calculation

```
ArmyPower = Σ (unit_count × unit_power_value)

Unit Power Values:
  Spearman: 1.0
  Swordsman: 1.5
  Archer: 1.2
  Cavalry: 2.0
  Horse Archer: 1.8
  Commander: 5.0
```

### 4.2 Production Rules

| Rule | Byzantine | Mongol |
|------|-----------|--------|
| Desired army size vs player | 1.5× player power | 2× player power (quality>quantity) |
| Unit mix check | Every 60s | Every 30s |
| Emergency production | When base attacked | When army depleted |
| Production building priority | Barracks > Archery > Stable | Stable > Barracks |

---

## 5. Strategic Action Queue

```json
{
  "faction": "byzantine",
  "plan": {
    "current_phase": "build_army",
    "actions": [
      {
        "id": "action_001",
        "type": "build_building",
        "building_id": "building_barracks",
        "position": {"x": 25, "z": 30},
        "priority": 80,
        "status": "in_progress",
        "progress": 0.45
      },
      {
        "id": "action_002",
        "type": "train_unit",
        "unit_id": "unit_spearman",
        "count": 3,
        "priority": 70,
        "status": "queued"
      },
      {
        "id": "action_003",
        "type": "attack_move",
        "target_zone": "zone_player_base",
        "min_army_size": 15,
        "priority": 60,
        "status": "waiting"
      }
    ]
  }
}
```

---

## 6. Expansion Logic

| Step | Byzantine | Mongol |
|------|-----------|--------|
| **Detect resource** | Claim nearby resources first | Identify player resources to raid |
| **Build forward** | Construct tower + wall | Send scout camp |
| **Secure** | Patrol 3 units around | Leave 1 scout |
| **Connect** | Build road to main base | No connection needed |

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_Faction_AI_Personalities.md` | Faction personality profiles |
| `04_Tactical_AI.md` | Tactical combat AI |
| `05_Difficulty_Scaling.md` | Difficulty adjustment |
