# Sovereignty Points — Meta-Currency & Talent Spending

> **Description:** Defines the Sovereignty Point (SP) system — the meta-currency earned by completing missions and spent on permanent talent tree upgrades. SP is the primary cross-mission progression driver.

---

## 1. SP Economy Overview

```
MISSION COMPLETION
      │
      ├── Victory: 2 SP base (minimum)
      │     └── +1 SP if 3 stars ⭐⭐⭐
      │     └── +1 SP if bonus objective completed
      │
      ├── First-time completion bonus: +1 SP
      │
      ├── Achievements: 0-2 SP (one-time)
      │
      └── Total SP earnable: 18-24 SP
               │
               ▼
         SP POOL (shared)
               │
               ▼
         TALENT TREES
         ┌──────────────┬──────────────┬──────────────┐
         │  Blacksmith   │   Cavalry    │    Faith      │
         │  (5 tiers)    │   (5 tiers)  │   (5 tiers)   │
         └──────────────┴──────────────┴──────────────┘
```

---

## 2. SP Earning Table

| Mission | Base SP | ⭐⭐⭐ Bonus | Bonus Obj | Total Potential |
|---------|---------|------------|-----------|-----------------|
| 01 — Kulucahisar | 2 | +1 | +1 | 4 |
| 02 — Yarhisar | 2 | +1 | +1 | 4 |
| 03 — Bilecik | 2 | +1 | +1 | 4 |
| 04 — Marmaracık | 2 | +1 | +1 | 4 |
| 05 — Byzantine Fortress | 2 | +1 | +1 | 4 |
| 06 — Yenişehir | 2 | +1 | +1 | 4 |
| **Total** | **12** | **+6** | **+6** | **24** |

### 2.1 Achievement SP Rewards

| Achievement | SP Reward | Condition |
|-------------|-----------|-----------|
| أول الغزو (First Conquest) | 1 | Complete Chapter 1 |
| قلب الأناضول (Heart of Anatolia) | 1 | Complete Chapter 2 |
| الدولة (The State) | 2 | Complete Chapter 3 |
| المهندس (The Architect) | 1 | Build 50 buildings across all missions |
| القائد (The Commander) | 1 | Never lose a unit in a mission (any difficulty) |
| الصياد (The Hunter) | 1 | Perform 20 ambushes total |
| غير المهزوم (Undefeated) | 2 | Complete all missions with 3 stars |
| **Total** | **9** | |

### 2.2 Practical Maximum

| Source | SP |
|--------|----|
| All missions 3★ + bonus | 24 |
| Achievements | 9 |
| **Hard cap** | **33** |
| **Realistic (normal play)** | **18-24** |

---

## 3. SP Spending Rules

| Rule | Detail |
|------|--------|
| **Non-refundable** | SP spent cannot be refunded. Permanent choice. |
| **Unlock order** | Must purchase sequentially within a tree tier-by-tier |
| **Prerequisites** | Each tier requires minimum number of purchases in previous tiers |
| **No cap per tree** | Can spend all SP in a single tree (but limited by tier unlock requirements) |

### 3.1 Cost by Tier

| Tier | Cost per Node | Nodes per Tier | Min Prior Purchases |
|------|---------------|----------------|-------------------|
| T1 | 2 SP | 2 | 0 |
| T2 | 2 SP | 2 | 1 |
| T3 | 3 SP | 2 | 2 |
| T4 | 3 SP | 2 | 3 |
| T5 | 4 SP | 2 | 4 |

---

## 4. SP UI Display

### 4.1 HUD Element

```
┌─────────────────────────────────────────────┐
│  ✦ Sovereignty Points: 8                    │
│  ┌──────┐ ┌──────┐ ┌──────┐              │
│  │ 🔨   │ │ 🐎   │ │ ☪    │              │
│  │ 2/14 │ │ 4/14 │ │ 2/14 │              │
│  └──────┘ └──────┘ └──────┘              │
│                                            │
│  Current SP: 8                             │
│  Total Earned: 14                          │
│  Next SP at: Mission completion            │
└─────────────────────────────────────────────┘
```

### 4.2 Purchase Confirmation

```
┌─────────────────────────────────────────┐
│  ✦ Confirm Purchase                     │
│                                         │
│  تقوية الأسلحة (Weapon Reinforcement)   │
│  All melee units +10% attack damage      │
│                                         │
│  Cost: 2 SP  │  Current SP: 8           │
│  Remaining after: 6                     │
│                                         │
│  [✓ Confirm]   [✗ Cancel]              │
│                                         │
│  ⚠ This purchase cannot be refunded.   │
└─────────────────────────────────────────┘
```

---

## 5. SP Cheat/Debug (Developer)

| Command | Effect |
|---------|--------|
| `sp.add <amount>` | Add SP to pool |
| `sp.set <amount>` | Set SP to exact value |
| `sp.reset_trees` | Refund all SP for reallocation |
| `sp.unlock_all` | Unlock all talents (cheat, disables achievements) |

---

## 6. Related Files

| File | Content |
|------|---------|
| `02_Tech_Tree.md` | What SP is spent on |
| `04_Persistent_Upgrades.md` | How SP upgrades persist |
| `05_Unlock_Schedule.md` | When SP becomes available |
