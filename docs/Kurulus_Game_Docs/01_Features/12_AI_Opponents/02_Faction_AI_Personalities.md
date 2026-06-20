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

---

## 7. الموسم السادس — New AI Personalities

### 7.1 Commander Lucas — The Professional (Byzantine Imperial Army)

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Tactician / Professional | Formations, discipline, tactical withdrawal |
| **Aggression** | 6/10 | Attacks only when odds ≥ 70% |
| **Expansion Rate** | 6/10 | Methodical expansion with supply lines |
| **Base Building** | 8/10 | Builds forward operating bases |
| **Unit Production** | 9/10 | Elite army composition |
| **Retreat Threshold** | 20% HP | Organized withdrawal |
| **Tech Progression** | 9/10 | All upgrades active |
| **Economy Focus** | 7/10 | Maintains supply lines |

#### 7.1.1 Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Tortoise Formation** | Shields overhead when under arrow fire (-80% ranged damage) |
| **Tactical Withdrawal** | Retreats with formations intact, loses only 10% during retreat |
| **Combined Arms** | Perfect coordination between infantry, archers, and cavalry |
| **Scout Screen** | Sends scouts 60s ahead of main army to detect ambushes |
| **Smoke Signals** | Calls reinforcements in 90 seconds via signal fires |
| **Flanking Protocol** | If pinned, sends cavalry to flank from both sides |

#### 7.1.2 Army Composition

| Unit | Percentage | Role |
|------|------------|------|
| Heavy Infantry | 30% | Frontline, shield wall |
| Longbow Archers | 25% | Ranged, 40m range |
| Cataphracts (Heavy Cavalry) | 20% | Shock charge, flanking |
| Medium Infantry | 15% | Flank protection |
| Scouts | 5% | Reconnaissance |
| Commander | 5% | Leadership |

---

### 7.2 Tekfur of Bursa — The Defender (Bursa Garrison)

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Fortress / Siege Defender | Best defensive AI in the game |
| **Aggression** | 3/10 | Only sorties when advantageous |
| **Expansion Rate** | 0/10 | Does not leave the city |
| **Base Building** | 10/10 | Maintains and repairs fortifications |
| **Unit Production** | 6/10 | Fixed garrison, cannot produce |
| **Retreat Threshold** | 0% HP | Fights to the death |
| **Tech Progression** | 10/10 | All siege defenses available (Greek fire, boiling oil) |

#### 7.2.1 Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Layered Defense** | 4 defense layers: outer wall → first courtyard → inner wall → keep |
| **Smart Repair** | Prioritizes: main gate > most-damaged wall > towers |
| **Greek Fire Reserve** | 3 uses: gate breach, courtyard crowd, final defense |
| **Sortie Timing** | Attacks when enemy is building siege engines or scattered |
| **Ammunition Conservation** | Towers have 100 arrows each, then switch to rocks |
| **Morale Immunity** | Never routs while Bursa banner flies |

#### 7.2.2 Garrison Composition

| Unit | Percentage | Role |
|------|------------|------|
| Wall Archers | 40% | Wall defense, siege engine targeting |
| Heavy Infantry (Courtyard) | 30% | Mobile reserve, breach response |
| Elite Guard (Keep) | 20% | Final defense line |
| Engineers | 10% | Wall/gate repair |

---

### 7.3 Catalan Company — The Shock Troops (Mercenaries)

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Aggressive / Shock | Devastating initial assault |
| **Aggression** | 10/10 | Always attacks, no hesitation |
| **Expansion Rate** | 2/10 | No base building |
| **Base Building** | 1/10 | Camps only |
| **Unit Production** | 1/10 | No production (fixed force) |
| **Retreat Threshold** | Conditional | Surrenders only if leader dies or losses > 80% |
| **Tech Progression** | 2/10 | Relies on ferocity, not tech |

#### 7.3.1 Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Shock Assault** | First 15 seconds of combat: +100% damage |
| **No Retreat** | Never retreats while leader is alive |
| **Wedge Formation** | Pierces enemy lines (+50% penetration damage) |
| **Bribeable** | 200 gold to switch sides (one-time offer) |
| **Leader-Dependent** | If Roger de Flor dies: 90% surrender chance at > 60% losses |

#### 7.3.2 Army Composition

| Unit | Percentage | Role |
|------|------------|------|
| Almogàvers (Light Infantry) | 65% | Shock infantry, fast assault |
| Javelin Skirmishers | 15% | Ranged support, anti-formation |
| Light Cavalry | 15% | Flanking, pursuit |
| Commander (Roger de Flor) | 5% | Leadership, "Catalan Cry" ability |

---

### 7.4 Byzantine Imperial Army — The Relief Force

| Attribute | Value | Description |
|-----------|-------|-------------|
| **Archetype** | Combined Arms / Elite | Multi-wave relief force |
| **Aggression** | 8/10 | Must break siege quickly |
| **Expansion Rate** | 4/10 | Moves directly to siege lines |
| **Base Building** | 5/10 | Supply depot only |
| **Unit Production** | 3/10 | Fixed force |
| **Retreat Threshold** | 50% | Retreats to regroup if losses exceed 50% |
| **Tech Progression** | 9/10 | Fully equipped |

#### 7.4.1 Behavioral Traits

| Trait | Description |
|-------|-------------|
| **Three-Wave Assault** | Wave 1: Cataphracts charge siege engines. Wave 2: Varangians attack infantry. Wave 3: Archers + spearmen cover |
| **Ambush Avoidance** | Scouts ahead 60s, flank guards, cautious march |
| **Combined Arms** | All unit types support each other perfectly |
| **Supply Line Dependent** | -30% effectiveness every 5 minutes if supply cut |
| **Expensive** | Elite units that Byzantium cannot replace easily |

#### 7.4.2 Army Composition

| Unit | Percentage | Role |
|------|------------|------|
| Cataphracts (Wave 1) | 25% | Shock charge, siege engine destruction |
| Varangian Guard (Wave 2) | 30% | Heavy infantry, elite melee |
| Longbow Archers (Wave 3) | 20% | Ranged cover |
| Spearmen | 20% | Anti-cavalry, flank protection |
| Scouts | 5% | Reconnaissance |

---

### 7.5 Siege AI Behaviors — Matrix

| Behavior | Tekfur of Bursa | Regular Fortress | Small Garrison | Mercenaries |
|----------|----------------|-----------------|----------------|-------------|
| **Garrison Distribution** | Perfect (40/30/20/10) | Good (35/35/20/10) | Random (50/25/15/10) | All on walls |
| **Wall Repair Speed** | Instant (smart priority) | Slow (10s) | Rarely | Never |
| **Sortie Frequency** | Smart timing | Random | Never | Aggressive (always) |
| **Greek Fire Usage** | Masterful (3 uses) | Single use | None | None |
| **Reinforcement Call** | Smoke signals (90s) | Messenger (180s) | Never | 300s |
| **Surrender Threshold** | Never | 80% losses | 60% losses | Leader death |
| **Final Stand** | Fights to death | Sometimes retreats | Surrenders | Fights or flees |

---

> **Season 6 Update**: 2026-06-20
