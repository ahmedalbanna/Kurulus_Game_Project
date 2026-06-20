# Tactical AI — Combat Decision-Making & Unit Control

> **Description:** Defines the tactical AI system — how individual units and groups make combat decisions, select targets, position themselves, and respond to threats.

---

## 1. Tactical Decision Loop

```
TACTICAL TICK (every simulation tick)
      │
      ├── 1. Sense
      │     • Detect enemies in detection range
      │     • Evaluate threat levels
      │     • Note environmental factors (terrain, stealth zones)
      │
      ├── 2. Assess
      │     • Current state (health, morale, position)
      │     • Enemy state (health, type, numbers)
      │     • Compare power ratios
      │
      ├── 3. Decide
      │     • Attack / Defend / Retreat / Flank / Hold
      │     • Select target
      │     • Choose ability to use (if available)
      │
      └── 4. Act
          • Execute chosen action
          • Update formation position
          • Emit AI events
```

---

## 2. Threat Evaluation

### 2.1 Threat Score Calculation

```
ThreatScore = 
    (enemy_damage_potential × 1.5) 
    + (enemy_proximity × 1.2) 
    + (enemy_army_power × 1.0) 
    + (enemy_morale × 0.8)
    - (ally_armor × 0.5)
    - (ally_morale × 0.3)
```

### 2.2 Target Priority

| Priority | Target Type | Condition |
|----------|-------------|-----------|
| **1 (highest)** | Enemy hero | Within range |
| **2** | Enemy ranged unit | Within range, exposed |
| **3** | Enemy siege unit | Visible |
| **4** | Enemy commander | Within range |
| **5** | Lowest HP enemy | Any |
| **6** | Nearest enemy | Default |
| **7** | Building | When attacking structure |

---

## 3. Formation & Positioning

### 3.1 Byzantine Tactical Behaviors

| Situation | Response |
|-----------|----------|
| **Player approaches base** | Units man walls, archers on wall, melee at gate |
| **Player breaches wall** | Form shield wall at breach point |
| **Player attacks with cavalry** | Spearmen to front, switch to anti-cavalry mode |
| **Player flanks** | Rotate formation, reposition archers |
| **Commander engaged** | Nearby units rush to protect |
| **Outnumbered** | Retreat to inner fortifications |
| **Siege weapons detected** | Send cavalry to flank and destroy siege |

### 3.2 Mongol Tactical Behaviors

| Situation | Response |
|-----------|----------|
| **Player army in open** | Cavalry charge from multiple directions |
| **Player forms shield wall** | Kite with horse archers, don't engage directly |
| **Player has many archers** | Spread formation, charge with cavalry |
| **Player pursues** | Feigned retreat → lead into ambush |
| **Player is low HP** | All-out attack, no mercy |
| **Player is fortified** | Raid resources instead, wait them out |

---

## 4. Ability Usage AI

| Ability | AI Trigger Condition | Target Priority |
|---------|---------------------|-----------------|
| Shield Wall | Enemy melee within 15m | Self (defensive) |
| Charge | Enemy group > 3 within charge range | Largest group |
| Arrow Volley | Player group > 5 in range | Dense formation |
| Feigned Retreat | Outnumbered 2:1 or worse | Self (survival) |
| Commander Rally | Morale < 30% in nearby allies | Self (buff aura) |

---

## 5. Retreat Logic

### 5.1 Retreat Trigger Conditions

| Condition | Retreat? | Notes |
|-----------|----------|-------|
| Unit HP < 20% | ✅ Yes | Individual retreat |
| Unit morale broken | ✅ Yes | Flee to nearest safe point |
| Army power ratio < 0.5 | ✅ Yes (group) | Tactical retreat |
| Commander HP < 20% | ✅ Yes (group) | Group retreat with commander |
| Player uses ultimate | ⚠️ Maybe | Evade if possible |
| Ally outnumbers nearby | ❌ No | Continue fighting |

### 5.2 Retreat Destination

| Faction | Retreat Destination |
|---------|-------------------|
| Byzantine | Nearest fortification, tower, or allied group |
| Mongol | Random direction (regroup 30s later) |

---

## 6. Ambush & Stealth AI

### 6.1 Byzantine Anti-Stealth

| Behavior | Description |
|----------|-------------|
| **Patrol routes** | Byzantine patrols follow fixed routes that cover stealth zones |
| **Torch bearers** | Certain units (torch bearers) reduce stealth effectiveness -50% in 8m radius |
| **Detection towers** | Towers auto-detect stealth units within 15m |
| **Alarm system** | One detection alerts all nearby patrols |

### 6.2 Mongol Stealth (Uses Stealth Offensively)

| Behavior | Description |
|----------|-------------|
| **Ambush setup** | Mongol units can hide in terrain (similar to player) |
| **Feigned retreat ambush** | One group retreats, hidden group attacks pursuers |
| **Night raids** | Stealth bonus at night, Mongols more active |

---

## 7. Tactical AI Configuration

```json
{
  "faction": "mongol",
  "tactical_settings": {
    "preferred_engagement_distance": 15.0,
    "flanking_angle": 45,
    "flanking_chance": 0.7,
    "shield_wall_threshold": 3,
    "retreat_hp_percent": 0.6,
    "group_attack_size": 5,
    "spread_formation_archer_threshold": 3,
    "feigned_retreat_chance": 0.4,
    "target_hero_priority": 0.8,
    "target_ranged_priority": 0.6
  }
}
```

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Faction_AI_Personalities.md` | Faction personality traits |
| `03_Strategic_AI.md` | High-level strategic planning |
| `05_Difficulty_Scaling.md` | Difficulty adjustment |
