# Difficulty Scaling — Adaptive Challenge System

> **Description:** Defines the difficulty scaling system — how the game adjusts challenge based on difficulty setting, player performance, and mission parameters.

---

## 1. Difficulty Levels

| Level | Label | Description | Recommended For |
|-------|-------|-------------|-----------------|
| 0 | **Story** | Minimal challenge, focus on narrative | New players, story-focused |
| 1 | **Easy** | Relaxed pace, forgiving | Casual RTS players |
| 2 | **Medium** | Balanced challenge | Default experience |
| 3 | **Hard** | Demanding, requires strategy | Experienced RTS players |

---

## 2. Difficulty Parameters

### 2.1 Global Multipliers

| Parameter | Story | Easy | Medium | Hard |
|-----------|-------|------|--------|------|
| Enemy attack damage | 0.5× | 0.75× | 1.0× | 1.3× |
| Enemy armor | 0.5× | 0.75× | 1.0× | 1.25× |
| Enemy unit HP | 0.6× | 0.8× | 1.0× | 1.2× |
| Enemy army size | 0.5× | 0.7× | 1.0× | 1.5× |
| Enemy production speed | 1.5× | 1.25× | 1.0× | 0.8× (faster) |
| Enemy resource income | 0.6× | 0.8× | 1.0× | 1.4× |
| Player attack damage | 1.5× | 1.25× | 1.0× | 0.9× |
| Player resource income | 1.5× | 1.25× | 1.0× | 0.85× |
| Morale effects on player | 0.5× | 0.75× | 1.0× | 1.1× |
| Morale effects on enemy | 1.5× | 1.25× | 1.0× | 0.9× |

### 2.2 AI Behavior Modifiers

| Behavior | Story | Easy | Medium | Hard |
|----------|-------|------|--------|------|
| AI reaction delay | +1.0s | +0.5s | 0s | -0.3s |
| AI flanking chance | 0% | 10% | 25% | 50% |
| AI retreat threshold | 20% HP | 30% HP | 40% HP | 50% HP |
| AI ability usage | Rare | Sometimes | Often | Optimal |
| AI base fortification | Minimal | Light | Standard | Heavy |
| AI scouting frequency | Never | Low | Medium | High |
| AI anti-stealth measures | None | Basic | Standard | Advanced |

---

## 3. Dynamic Difficulty Adjustment (Optional)

The game can optionally adjust difficulty based on player performance:

### 3.1 Adjustment Triggers

| Player Performance | Adjustment |
|--------------------|------------|
| Losing 3+ battles in a row | Reduce enemy army size by one tier |
| Winning all battles easily | Increase enemy aggression |
| Frequently pausing | Extend AI reaction delay |
| Taking heavy casualties | Reduce enemy attack damage |
| Completing objectives too fast | Increase enemy production speed |

### 3.2 Adjustment Rules

| Rule | Description |
|------|-------------|
| **No notification** | DDA changes are invisible to the player |
| **Per-mission only** | DDA resets between missions |
| **Clamped** | Cannot go below Story or above Hard |
| **Measured per checkpoint** | Evaluation every 3 minutes |

---

## 4. Per-Mission Difficulty Settings

| Mission | Default Difficulty | Notes |
|---------|-------------------|-------|
| 01 — Kulucahisar | Story/Easy | Tutorial mission, gentle intro |
| 02 — Yarhisar | Easy | First real challenge |
| 03 — Bilecik | Medium | Ambush required |
| 04 — Marmaracık | Medium | Multiple fronts |
| 05 — Byzantine Fortress | Medium/Hard | Large siege |
| 06 — Yenişehir | Hard | Final battle, all forces |

---

## 5. Difficulty Selection Screen

```
┌──────────────────────────────────────────────────────┐
│  SELECT DIFFICULTY                                     │
│                                                        │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐              │
│  │ STORY│  │ EASY │  │MEDIUM│  │ HARD │              │
│  └──────┘  └──────┘  └──────┘  └──────┘              │
│                                                        │
│  Recommended for:                                      │
│  • New to RTS games                                    │
│  • Focus on story                                      │
│  • Relaxed experience                                  │
│                                                        │
│  Modifiers:                                            │
│  ✓ Enemy damage: 0.5×                                  │
│  ✓ Enemy army: 0.5×                                    │
│  ✓ Player income: 1.5×                                 │
│                                                        │
│  [✓ CONFIRM]                                           │
└──────────────────────────────────────────────────────┘
```

---

## 6. Related Files

| File | Content |
|------|---------|
| `02_Faction_AI_Personalities.md` | How difficulty affects faction behavior |
| `03_Strategic_AI.md` | How difficulty changes strategy |
| `04_Tactical_AI.md` | How difficulty changes tactics |
