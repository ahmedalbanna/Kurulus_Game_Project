# Strategic Events — Dynamic World Changes

> **Description:** Defines the strategic event system that creates dynamic, unexpected situations during missions — adding replayability and forcing player adaptation.

---

## 1. Event Categories

| Category | Examples | Frequency |
|----------|----------|-----------|
| **Reinforcement** | Enemy reinforcements arrive, allied reinforcements arrive | Per mission (1-3) |
| **Environmental** | Weather change, fire spreads, terrain collapses | Occasional |
| **Faction** | Faction changes stance, new faction appears, betrayal | Per chapter |
| **Economy** | Resource discovery, trade caravan arrives, market prices shift | Per mission |
| **Narrative** | Story-triggered events, dialogue moments, scripted sequences | Fixed per mission |
| **Emergent** | Random skirmish between NPCs, wild animal attack, wandering monster | Random |

---

## 2. Fixed Strategic Events (Per Mission)

| Mission | Event | Trigger | Effect |
|---------|-------|---------|--------|
| **01** | Byzantine patrol route change | Player reaches castle gate | Patrol diverts to intercept |
| **01** | Prisoner break | Castle courtyard captured | Ally prisoners aid player |
| **02** | Bridge destruction | Player crosses halfway | Path splits, forces flanking |
| **02** | Seljuk scout arrival | Mission timer > 5 min | Map reveal, enemy positions shown |
| **03** | Mercenary betrayal | Player refuses payment | Mercenaries join enemy |
| **03** | Nightfall | Mission timer > 10 min | Visibility reduced, stealth bonus |
| **04** | Mongol vanguard | Player attacks main base | Mongols enter from north |
| **04** | Storm at coast | Mission timer > 8 min | Rain reduces ranged accuracy -30% |
| **05** | Reinforcements arrive | Player breaches outer wall | Elite Byzantine units spawn |
| **05** | Treason | Inner keep breached | Byzantine captain defects (info) |
| **06** | Alliance army arrives | Player reaches 50% objectives | Seljuk reinforcements join |
| **06** | Mongol flank attack | Player commits to main assault | Mongols attack from rear |

---

## 3. Random Emergent Events

### 3.1 Event Table

| Event | Probability | Condition | Effect |
|-------|-------------|-----------|--------|
| Wandering traders | 15% per 5 min | No active combat nearby | Trade window opens |
| Wild wolf pack | 10% per 5 min | Forest zone | Attacks nearest unit (any faction) |
| Scout skirmish | 20% per 5 min | Two factions both present in zone | Small auto-combat |
| Resources stash | 5% per zone explored | Player explores new zone | Discover 50-100 bonus resource |
| Refugee group | 8% per mission | Player controls a village | +5 population, +20 food |
| Earthquake | 3% per mission | Mountain zone | Buildings damaged 10-20% |
| Deserters | 10% per 10 min | Any faction morale < 30% | Units flee map edge |

### 3.2 Event Resolution

```
Event Triggered:
  "Wild wolf pack spotted near the eastern forest!"

Options:
  [1] Ignore — wolves attack nearest unit (possible losses)
  [2] Hunt — send 3+ units to kill wolves (bonus food, XP)
  [3] Avoid — reroute units around forest (takes 30s longer)
```

---

## 4. Event Trigger System

```json
{
  "event_id": "evt_03_mercenary_betrayal",
  "type": "narrative",
  "trigger_condition": {
    "type": "mission_state",
    "params": {
      "mission_id": "mission_03",
      "state": "player_refused_payment"
    }
  },
  "effects": [
    {
      "type": "faction_relation_change",
      "params": {
        "faction": "mercenaries",
        "change": -30
      }
    },
    {
      "type": "unit_transfer",
      "params": {
        "units": ["unit_mercenary_sword", "unit_mercenary_sword"],
        "from": "neutral",
        "to": "byzantine",
        "position": {"x": 45.0, "z": 30.0}
      }
    },
    {
      "type": "notification",
      "params": {
        "text_key": "event_mercenary_betrayal",
        "severity": "warning",
        "duration": 5.0
      }
    }
  ]
}
```

---

## 5. Event Cooldowns

| Event Type | Min Interval | Max per Mission |
|------------|-------------|-----------------|
| Environmental | 3 min | 2 |
| Reinforcement | 5 min | 3 |
| Random combat | 2 min | 5 |
| Resource | 4 min | 2 |
| Narrative | (scripted) | As defined |

---

## 6. Player Notification

```
┌───────────────────────────────────────────────────────┐
│ ⚡ STRATEGIC EVENT                                      │
│                                                         │
│ A Byzantine relief force has been spotted marching      │
│ from the north. They will arrive in 2 minutes.          │
│                                                         │
│ [🗺 Show on Map]  [✓ Acknowledge]                      │
└───────────────────────────────────────────────────────┘
```

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_Region_System.md` | Where events take place |
| `04_Faction_Relations.md` | How events affect relations |
| `06_Campaign_Map_UI.md` | Event display on campaign map |
