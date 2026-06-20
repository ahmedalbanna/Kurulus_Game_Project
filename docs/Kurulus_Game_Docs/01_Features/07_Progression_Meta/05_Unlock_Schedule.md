# Unlock Schedule — Per-Mission Progression Gate

> **Description:** Defines exactly which units, buildings, abilities, and features unlock at each mission, and the recommended player progression curve.

---

## 1. Master Unlock Table

| Item | Type | Unlock Condition | Available From |
|------|------|-----------------|----------------|
| Barracks | Building | Default | Mission 01 |
| Swordsman (Basic) | Unit | Barracks built | Mission 01 |
| Lumber Mill | Building | Default | Mission 01 |
| Wood gather | Mechanic | Default | Mission 01 |
| Gold Mine | Building | Default | Mission 01 |
| **Archery Range** | Building | Mission 01 complete | Mission 02 |
| **Archer (Basic)** | Unit | Archery Range built | Mission 02 |
| **Walls (Wood)** | Building | Mission 01 complete | Mission 02 |
| **Gate** | Building | Walls available | Mission 02 |
| **Smithy** | Building | Mission 02 complete | Mission 03 |
| **Swordsman (Upgraded)** | Unit | Smithy built | Mission 03 |
| **Watch Tower** | Building | Mission 02 complete | Mission 03 |
| **Market** | Building | Mission 03 complete | Mission 04 |
| **Spearman** | Unit | Barracks Tier 2 | Mission 04 |
| **Stable** | Building | Mission 04 complete | Mission 05 |
| **Cavalry Scout** | Unit | Stable built OR Talent Cavalry T1 | Mission 05 |
| **Fortress Walls (Stone)** | Building | Mission 04 complete | Mission 05 |
| **Heavy Cavalry** | Unit | Stable Tier 2 + Talent Cavalry T5 | Mission 06 |
| **Catapult** | Unit | Smithy Tier 2 | Mission 06 |
| **Siege Tower** | Unit | Smithy Tier 2 | Mission 06 |
| **Hero Ultimate Abilities** | Mechanic | Faith T5 | Mission 05+ |

---

## 2. Chapter-by-Chapter Unlock Flow

### Chapter 1: Spark (Missions 01-02)

```
Available from start:
  ┌──────────────────────────────────────────┐
  │ Heroes: Osman (all missions)              │
  │ Units: Swordsman (Basic)                  │
  │ Buildings: Barracks, Lumber Mill,         │
  │           Gold Mine, House                 │
  │ Mechanics: Move, Attack, Patrol,          │
  │            Basic formations               │
  │ Stealth: Grass stealth zones              │
  └──────────────────────────────────────────┘

Unlocks after Mission 01:
  ┌──────────────────────────────────────────┐
  │ Archery Range building                    │
  │ Archer (Basic) unit                       │
  │ Wooden Walls + Gate                       │
  │ Ambush mechanic unlocked                  │
  │ Hero: Turgut joins                        │
  └──────────────────────────────────────────┘
```

### Chapter 2: Guerrilla (Missions 03-04)

```
Unlocks after Mission 02:
  ┌──────────────────────────────────────────┐
  │ Smithy building                           │
  │ Swordsman (Upgraded) unit                 │
  │ Watch Tower building                      │
  │ Decoy system unlocked                     │
  │ Hero: Buran joins                         │
  │ Talent Trees: Tier 2 available            │
  └──────────────────────────────────────────┘

Unlocks after Mission 03:
  ┌──────────────────────────────────────────┐
  │ Market building                           │
  │ Trade mechanic (resource exchange)        │
  │ Hit & Run tactics unlocked                │
  │ Formation: Shield Wall                    │
  └──────────────────────────────────────────┘
```

### Chapter 3: Conquest (Missions 05-06)

```
Unlocks after Mission 04:
  ┌──────────────────────────────────────────┐
  │ Stable building                           │
  │ Spearman unit                             │
  │ Stone Walls + Stone Gate                  │
  │ Siege weapons unlocked (build at Smithy)  │
  │ Talent Trees: Tier 4 available            │
  └──────────────────────────────────────────┘

Unlocks after Mission 05:
  ┌──────────────────────────────────────────┐
  │ Cavalry Scout unit (or talent unlock)     │
  │ Heavy Cavalry (talent gated)              │
  │ Catapult, Siege Tower                     │
  │ Hero Ultimates (Faith T5)                 │
  │ Talent Trees: Tier 5 available            │
  └──────────────────────────────────────────┘
```

---

## 3. New Feature Introduction Schedule

| Mission | New Features Introduced |
|---------|------------------------|
| **01** | Movement, attack, basic RTS controls, stealth (grass), hero abilities |
| **02** | Ranged units, walls/gates, ambush system, Turgut hero |
| **03** | Unit upgrades, watch towers, decoys, Buran hero, advanced formations |
| **04** | Economy buildings (Market), trade, hit & run, enemy hero encounters |
| **05** | Cavalry, stone defenses, siege weapons, ultimate abilities, Mongol faction |
| **06** | All features available, full army, massive siege battle |

---

## 4. Difficulty Curve via Unlocks

```
DIFFICULTY
    ▲
    │                                  ★ Mission 06 (all tools)
    │                               ★  Mission 05 (cavalry + siege)
    │                           ★     Mission 04 (trade + tactics)
    │                      ★        Mission 03 (upgrades + decoys)
    │                 ★           Mission 02 (ranged + ambush)
    │            ★              Mission 01 (basic RTS)
    └─────────────────────────────────────────────────► MISSION
        01    02    03    04    05    06
```

The unlock schedule ensures that:
- **Mission 01** is simple (basic RTS)
- **Mission 02** introduces the core X-Factor (ambush)
- **Mission 03-04** expand tactical options
- **Mission 05-06** give the player all tools for complex battles

---

## 5. Related Files

| File | Content |
|------|---------|
| `02_Tech_Tree.md` | Talent tree details |
| `03_Sovereignty_Points.md` | SP earning schedule |
| `04_Persistent_Upgrades.md` | Cross-mission persistence |
