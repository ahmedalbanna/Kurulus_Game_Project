# Faction AI Personalities — Behavioral Identity Profiles

> **Description:** Defines the AI personality profiles for each enemy faction in *Kuruluş* — their strategic preferences, tactical tendencies, and behavioral characteristics.

---

## 1. AI Personality Architecture

```
AI MANAGER
│
├── FACTION AI: Byzantine
│   ├── Personality: "Fortress" (defensive, heavy)
│   ├── Strategy: Expand → Consolidate → Fortify
│   └── Tactics: Shield wall, ranged from walls, counter-attack
│
├── FACTION AI: Mongol
│   ├── Personality: "Horde" (aggressive, mobile)
│   ├── Strategy: Raid → Pillage → Retreat → Regroup
│   └── Tactics: Cavalry charges, feigned retreat, flanking
│
└── FACTION AI: Mercenary (if turned hostile)
    ├── Personality: "Opportunist" (adaptive, greedy)
    ├── Strategy: Attack weak points → Demand tribute
    └── Tactics: Hit-and-run, target resources
```

---

## 2. Byzantine AI Personality

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Defensive / Fortress | Prefers fortified positions, slow advance |
| **Aggression** | 3/10 | Only attacks when confident |
| **Expansion Rate** | 4/10 | Slow, methodical expansion |
| **Base Building** | 9/10 | Builds many defenses |
| **Unit Production** | 7/10 | Prioritizes army composition |
| **Retreat Threshold** | 40% HP | Retreats earlier than most |
| **Tech Progression** | 8/10 | Researches upgrades actively |
| **Economy Focus** | 6/10 | Moderate economy priority |

### 2.1 Byzantine Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Shield Wall Specialty** | Heavy infantry form shield walls when enemy approaches |
| **Garrison Preference** | Keeps 30% of army garrisoned in fortifications |
| **Ranged Advantage** | Positions archers on walls and high ground |
| **Reluctant to Chase** | Rarely leaves fortifications to chase |
| **Reinforcement Call** | Calls nearby patrols when engaged |
| **Sacrifice Commander** | Commander retreats when HP < 20%, leaving garrison |

### 2.2 Byzantine Army Composition

| Unit | Percentage | Role |
|------|------------|------|
| Spearman | 35% | Anti-cavalry, frontline |
| Swordsman | 25% | Melee damage, assault |
| Archer | 30% | Ranged damage, wall defense |
| Cavalry | 5% | Scouts, emergency response |
| Commander | 5% | Leadership, morale boost |

---

## 3. Mongol AI Personality

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Aggressive / Horde | Fast attacks, prefers mobility |
| **Aggression** | 9/10 | Attacks constantly |
| **Expansion Rate** | 8/10 | Rapid expansion, spread out |
| **Base Building** | 3/10 | Minimal bases, mobile camps |
| **Unit Production** | 9/10 | Spams cheap light units |
| **Retreat Threshold** | 60% HP | Retreats early to regroup |
| **Tech Progression** | 3/10 | Minimal upgrades |
| **Economy Focus** | 2/10 | Relies on pillaging |

### 3.1 Mongol Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Cavalry Focus** | 70% of army is cavalry |
| **Feigned Retreat** | Pretends to flee, then turns to attack pursuers |
| **Flanking** | Always tries to attack from sides/rear |
| **Resource Raiding** | Prioritizes attacking resource gatherers |
| **Divide and Conquer** | Sends small groups to split player's forces |
| **No Siege** | Cannot siege directly; relies on overwhelming speed |

### 3.2 Mongol Army Composition

| Unit | Percentage | Role |
|------|------------|------|
| Horse Archer | 45% | Mobile ranged, kiting |
| Light Cavalry | 30% | Melee charge, pursuit |
| Heavy Cavalry (elite) | 15% | Shock troops, commander bodyguard |
| Scout Rider | 10% | Vision, harassment |

---

## 4. AI Personality Comparison

| Aspect | Byzantine | Mongol |
|--------|-----------|--------|
| **Difficulty Curve** | Moderate early → Hard late | Hard early → Moderate late |
| **Player Counter** | Siege weapons break defenses | Ranged infantry + spearmen |
| **Preferred Terrain** | Fortified, narrow passes | Open plains, flanks |
| **Weakness** | Slow to react, predictable | Fragile units, low morale |
| **Strength** | Durable, fortified | Fast, unpredictable |
| **Encounter Chapter** | 1-6 | 3-6 |

---

## 5. Behavior Modification by Difficulty

| Difficulty | Byzantine Changes | Mongol Changes |
|------------|------------------|----------------|
| **Easy** | -30% army size, slow reactions, no elite units | -40% army size, poor flanking, no feigned retreat |
| **Medium** | Standard behavior | Standard behavior |
| **Hard** | +30% army size, faster reactions, shield wall always active | +40% army size, perfect flanking, feigned retreat + ambush |

---

## 6. Related Files

| File | Content |
|------|---------|
| `03_Strategic_AI.md` | High-level strategic planning |
| `04_Tactical_AI.md` | Tactical combat AI |
| `05_Difficulty_Scaling.md` | Difficulty adjustment system |
| `06_AI_Debugging_Tools.md` | AI debugging tools |
