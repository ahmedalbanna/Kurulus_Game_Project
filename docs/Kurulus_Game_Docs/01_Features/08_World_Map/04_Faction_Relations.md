# Faction Relations — Diplomacy, Allegiance & Interactions

> **Description:** Defines the faction relationship system for *Kuruluş* — how factions interact, what determines their behavior toward each other, and how player actions affect faction standing.

---

## 1. Faction Overview

| Faction | Role | Alignment | Starting Relation to Player |
|---------|------|-----------|---------------------------|
| **Kayı Tribe (Player)** | Protagonist | Playable | - |
| **Byzantine Empire** | Primary antagonist | Hostile | -50 (War) |
| **Mongol Empire** | Secondary antagonist | Hostile | -80 (War — appears Ch3) |
| **Seljuk Sultanate** | Ally (distant) | Friendly | +40 |
| **Peasant Villages** | Neutral / Friendly | Passive | +20 |
| **Mercenary Bands** | Neutral | Pass-through | 0 |

---

## 2. Relation Levels

| Level | Value Range | Behavior |
|-------|-------------|----------|
| **War** | -100 to -50 | Attack on sight, no communication |
| **Hostile** | -49 to -10 | Will attack if provoked, avoid if possible |
| **Neutral** | -9 to +9 | Ignore each other, no interaction |
| **Friendly** | +10 to +49 | Allow passage, trade available |
| **Ally** | +50 to +100 | Share vision, joint operations |

---

## 3. Relation Modifiers

### 3.1 Player Actions Affecting Relations

| Action | Target Faction | Change |
|--------|---------------|--------|
| Attack a unit | Any | -5 per unit killed |
| Destroy a building | Any | -10 per building |
| Complete a mission | Enemy | -20 (escalation) |
| Free a village | Byzantine | -15 (to Byz), +10 (to Village) |
| Trade with faction | Any | +1 per trade |
| Accept a quest from faction | Any | +5 |
| Complete a quest for faction | Any | +10 |
| Return captives | Any | +8 |
| Pay tribute | Any | +15 |

### 3.2 Automatic Relation Drift

| Situation | Change | Rate |
|-----------|--------|------|
| Neighboring territories | -1 | Per mission (tension) |
| Shared enemy | +2 | Per mission (enemy of my enemy) |
| Time without contact | +0.5 | Per mission (drift toward neutral) |
| Player grows stronger | -2 | Per mission (NPCs feel threatened) |

---

## 4. Faction Behavior by Relation

### 4.1 War (Byzantine — Default)

```
- Patrols aggressively through region
- Attacks player units on sight
- Reinforces threatened positions
- Builds defensive structures
- Doesn't accept communication
```

### 4.2 Neutral (Mercenaries — Default)

```
- Ignores player unless attacked
- Can be hired for gold
- Will defend themselves if attacked
- May offer information for payment
- Can switch sides if paid enough
```

### 4.3 Friendly (Villages — Default)

```
- Allow player passage through their territory
- Offer basic supplies (small gold/food gift)
- Provide shelter for wounded units
- Warn about enemy movements nearby
- Request help against bandits/raiders
```

### 4.4 Ally (Seljuks — Story)

```
- Share vision of explored areas
- Send reinforcement units when requested
- Coordinate attacks on shared enemies
- Offer trade discounts
- Provide military technology access
```

---

## 5. Relation Transition Rules

| From | To | Condition |
|------|----|-----------|
| War | Hostile | Player avoids conflict for 2 missions |
| Hostile | Neutral | Player performs friendly action (e.g., return captives) |
| Neutral | Friendly | Player completes 2+ quests for faction |
| Friendly | Ally | Player completes major quest + relation > +50 |
| Friendly | Neutral | Player attacks faction units |
| Friendly | Hostile | Player kills faction leader / destroys capital |
| Any | War | Player attacks faction without provocation |

---

## 6. Relation Display (UI)

```
┌──────────────────────────────────────────────────┐
│  FACTION RELATIONS                                  │
│                                                     │
│  Byzantine Empire     ●●●●●○○○○○  [WAR]    -50     │
│  Mongol Empire        ●●●●●●●●○○  [WAR]    -80     │
│  Seljuk Sultanate     ●○○○○○○○○○  [ALLY]   +40     │
│  Village of Yarhisar  ●●○○○○○○○○  [FRIEND] +15     │
│  Mercenary Band       ●○○○○○○○○○  [NEUTRAL] +3     │
│                                                     │
│  [DETAILS] [HISTORY]                                │
└──────────────────────────────────────────────────┘
```

---

## 7. Mission-Specific Relations

| Mission | Active Factions | Starting Relation | Notes |
|---------|----------------|-------------------|-------|
| 01 — Kulucahisar | Byzantines, Villagers | War, Friendly | Simple two-faction |
| 02 — Yarhisar | Byzantines, Seljuks | War, Neutral | Seljuks provide intel |
| 03 — Bilecik | Byzantines, Mercenaries | War, Neutral | Hire mercenaries |
| 04 — Marmaracık | Byzantines, Mongols (intro) | War, Hostile | Mongol scouts appear |
| 05 — Byzantine Fortress | Byzantines, Seljuks | War, Allied | Joint assault |
| 06 — Yenişehir | All factions | War(×2), Allied, Variable | Full war scenario |

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Region_System.md` | Which faction controls which region |
| `03_Fog_of_War.md` | Faction vision sharing |
| `05_Strategic_Events.md` | Events that change relations |
