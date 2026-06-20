# Tech Tree — Technology & Upgrade Paths

> **Description:** Defines the technology tree system for *Kuruluş* — three distinct progress paths (Blacksmith, Cavalry, Faith) that unlock abilities, units, and bonuses across all missions.

---

## 1. Structure Overview

```
                    ┌──────────────────────────────────────────────────┐
                    │           THREE PROGRESSION TREES               │
                    │          (Shared across all missions)           │
                    │                                                  │
                    │  ┌──────────────┐  ┌──────────────┐  ┌────────┐ │
                    │  │  BLACKSMITH   │  │   CAVALRY     │  │  FAITH  │ │
                    │  │  (Power)     │  │  (Mobility)   │  │(Morale) │ │
                    │  └──────┬───────┘  └──────┬───────┘  └───┬────┘ │
                    │         │                 │               │      │
                    │    Tier 1 (2 SP)     Tier 1 (2 SP)    Tier 1 (2)│
                    │    Tier 2 (2 SP)     Tier 2 (2 SP)    Tier 2 (2)│
                    │    Tier 3 (3 SP)     Tier 3 (3 SP)    Tier 3 (3)│
                    │    Tier 4 (3 SP)     Tier 4 (3 SP)    Tier 4 (3)│
                    │    Tier 5 (4 SP)     Tier 5 (4 SP)    Tier 5 (4)│
                    │                      Total: 45 SP               │
                    │         (18 max earned — choose wisely)         │
                    └──────────────────────────────────────────────────┘
```

---

## 2. Blacksmith Tree (Power — Combat Effectiveness)

| Tier | SP Cost | Node Name | Effect |
|------|---------|-----------|--------|
| **T1** | 2 | **تقوية الأسلحة** (Weapon Reinforcement) | All melee units +10% attack damage |
| **T1** | 2 | **دروع معززة** (Reinforced Armor) | All units +5% armor |
| **T2** | 2 | **سيف طويل** (Longsword) | Osman's basic attacks deal +15% damage |
| **T2** | 2 | **سهام حادة** (Broadhead Arrows) | Ranged units +15% damage |
| **T3** | 3 | **عاصفة السيوف** (Sword Storm) | Osman unlocks Sword Storm ability |
| **T3** | 3 | **سلسلة دروع** (Chainmail) | All melee infantry +10 armor |
| **T4** | 3 | **نصل ملتهب** (Flaming Blade) | Osman's attacks deal fire DoT (5 dmg/3 sec) |
| **T4** | 3 | **قوس مركب** (Composite Bow) | Ranged units +20% range |
| **T5** | 4 | **سيف الدم** (Blood Sword) | Osman heals 20% of damage dealt |
| **T5** | 4 | **صانع أسلحة** (Master Smith) | Unit production cost -15% |

### 2.1 Blacksmith Modifiers Applied

| Talent | Target | Property | Change |
|--------|--------|----------|--------|
| Weapon Reinforcement | All melee units | attack_damage | +10% (multiplicative) |
| Reinforced Armor | All units | armor | +5 (additive) |
| Longsword | Hero Osman | attack_damage | +15% (multiplicative) |
| Broadhead Arrows | All ranged units | attack_damage | +15% (multiplicative) |
| Sword Storm | Hero Osman | Unlock ability | ability_sword_storm |
| Chainmail | Infantry units | armor | +10 (additive) |
| Flaming Blade | Hero Osman | attack_effects | Add fire_dot: 5 dmg, 3s duration |
| Composite Bow | Ranged units | attack_range | +20% (multiplicative) |
| Blood Sword | Hero Osman | lifesteal | 20% of damage |
| Master Smith | All production | cost_multiplier | 0.85 (15% reduction) |

---

## 3. Cavalry Tree (Mobility — Speed & Maneuver)

| Tier | SP Cost | Node Name | Effect |
|------|---------|-----------|--------|
| **T1** | 2 | **حدوات جديدة** (New Horseshoes) | All cavalry +10% move speed |
| **T1** | 2 | **خيالة خفيفة** (Light Cavalry) | Unlock cavalry scout unit |
| **T2** | 2 | **ركوب ماهر** (Expert Riding) | Cavalry rotation speed +25% |
| **T2** | 2 | **تكتيك الكر والفر** (Hit & Run) | Turgut unlocks Hit & Run ability |
| **T3** | 3 | **سيف طويل للخيالة** (Cavalry Lance) | Cavalry charge damage +25% |
| **T3** | 3 | **خيل عربية** (Arabian Horses) | All units +5% move speed |
| **T4** | 3 | **غارة سريعة** (Rapid Raid) | Cavalry attack speed +15% |
| **T4** | 3 | **درع الخيل** (Barding) | Cavalry armor +8 |
| **T5** | 4 | **سلاح الفرسان** (Knight Order) | Unlock heavy cavalry (elite unit) |
| **T5** | 4 | **مسيرة قسرية** (Forced March) | All units +10% move speed (stacks) |

### 3.1 Cavalry Modifiers Applied

| Talent | Target | Property | Change |
|--------|--------|----------|--------|
| New Horseshoes | All cavalry | move_speed | +10% (multiplicative) |
| Light Cavalry | Player unlocks | unit_cavalry_scout | Available for recruitment |
| Expert Riding | All cavalry | rotation_speed | +25% (multiplicative) |
| Hit & Run | Hero Turgut | Unlock ability | ability_hit_and_run |
| Cavalry Lance | All cavalry | charge_damage | +25% (multiplicative) |
| Arabian Horses | All units | move_speed | +5% (multiplicative) |
| Rapid Raid | All cavalry | attack_cooldown | -15% (faster attacks) |
| Barding | All cavalry | armor | +8 (additive) |
| Knight Order | Player unlocks | unit_cavalry_heavy | Available for recruitment |
| Forced March | All units | move_speed | +10% (multiplicative) |

---

## 4. Faith Tree (Morale — Resilience & Leadership)

| Tier | SP Cost | Node Name | Effect |
|------|---------|-----------|--------|
| **T1** | 2 | **صلاة الفجر** (Dawn Prayer) | All units +10 max morale |
| **T1** | 2 | **بيعة** (Oath of Allegiance) | Units fight harder near heroes (+15% dmg in 10m) |
| **T2** | 2 | **دعاء النصر** (Victory Prayer) | Morale recovery +20% faster |
| **T2** | 2 | **راية الإيمان** (Banner of Faith) | Buran unlocks Banner of Faith (morale aura) |
| **T3** | 3 | **شهيد** (Martyrdom) | Units at <20% HP gain +25% damage |
| **T3** | 3 | **وحدة الصف** (Unity of Ranks) | Formation bonuses +50% effectiveness |
| **T4** | 3 | **بركة** (Blessing) | Heroes regenerate 2 HP/sec out of combat |
| **T4** | 3 | **خطبة** (Sermon) | Morale break threshold reduced by 10 |
| **T5** | 4 | **إلهام إلهي** (Divine Inspiration) | Unlock hero ultimate abilities |
| **T5** | 4 | **جيش لا يقهر** (Invincible Army) | Units below 10% HP cannot morale break |

### 4.1 Faith Modifiers Applied

| Talent | Target | Property | Change |
|--------|--------|----------|--------|
| Dawn Prayer | All units | max_morale | +10 (additive) |
| Oath of Allegiance | Units near heroes | attack_damage | +15% when within 10m of hero |
| Victory Prayer | All units | morale_recovery_rate | +20% (multiplicative) |
| Banner of Faith | Hero Buran | Unlock ability | ability_banner_faith |
| Martyrdom | All units with HP < 20% | attack_damage | +25% (multiplicative) |
| Unity of Ranks | Formations | formation_bonus | +50% effectiveness |
| Blessing | Heroes | hp_regen_out_of_combat | +2 HP/sec |
| Sermon | All units | morale_break_threshold | -10 (harder to break) |
| Divine Inspiration | Heroes | Unlock ultimate | Tier 5 ultimate abilities |
| Invincible Army | Units at <10% HP | morale_break | Immune |

---

## 5. Tech Tree Unlock Flow

```
SP EARNED: 3 per mission × 6 missions = 18 SP total
                   │
                   ▼
        ┌──────────────────┐
        │  Choose a tree    │
        │  (Blacksmith,     │
        │   Cavalry, Faith) │
        └──────────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  Tier 1 nodes     │── Must buy at least 1 to unlock Tier 2
        │  (2 SP each)      │
        └──────────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  Tier 2 nodes     │── Must have 1 T1 node in same tree
        │  (2 SP each)      │
        └──────────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  Tier 3 nodes     │── Must have 2 nodes in previous tiers
        │  (3 SP each)      │
        └──────────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  Tier 4 nodes     │── Must have 3 nodes in previous tiers
        │  (3 SP each)      │
        └──────────────────┘
                   │
                   ▼
        ┌──────────────────┐
        │  Tier 5 nodes     │── Must have 4 nodes in previous tiers
        │  (4 SP each)      │
        └──────────────────┘
```

---

## 6. Tree Synergies

| Combination | Synergy |
|-------------|---------|
| **Blacksmith T3 + Cavalry T3** | Sword Storm + Cavalry Lance = devastating charge then AoE clean-up |
| **Faith T2 + Cavalry T5** | Banner of Faith + Knight Order = elite cavalry with morale immunity |
| **Faith T4 + Blacksmith T5** | Sermon + Blood Sword = Osman never breaks morale, heals while fighting |
| **Cavalry T1 + T5** | Light Cavalry early, Heavy Cavalry late = full cavalry army viable |
| **Blacksmith T4 + Faith T3** | Composite Bow + Martyrdom = ranged units deadly at low HP |

---

## 7. Full Unlock Path Costs

| Strategy | SP Cost | Nodes | Description |
|----------|---------|-------|-------------|
| **One tree mastery** | 14 SP | 5-6 nodes max | Drill deep in one tree |
| **Balanced** | 6 + 6 + 6 | 3 per tree | Moderate spread |
| **Blacksmith + Cavalry** | 10 + 8 | 4 + 3 | Aggressive military |
| **Faith + Blacksmith** | 10 + 8 | 4 + 3 | Durable + powerful |
| **Cavalry + Faith** | 10 + 8 | 4 + 3 | Mobile + resilient |
| **Dabbler** | 4 + 4 + 4 | 2 per tree | Jack of all trades |

---

## 8. Related Files

| File | Content |
|------|---------|
| `03_Sovereignty_Points.md` | How SP is earned and spent |
| `04_Persistent_Upgrades.md` | Cross-mission upgrade persistence |
| `05_Unlock_Schedule.md` | What unlocks when per mission |
