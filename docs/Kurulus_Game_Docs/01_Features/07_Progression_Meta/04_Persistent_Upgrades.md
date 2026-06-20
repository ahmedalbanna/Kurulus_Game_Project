# Persistent Upgrades — Cross-Mission Progression

> **Description:** Defines what carries over between missions, how persistent upgrades are applied, and the rules for transferring progress from mission to mission.

---

## 1. What Carries Over

### 1.1 Full Persistence (Always Carries Over)

| Element | Detail |
|---------|--------|
| **Sovereignty Points** | All earned SP carry over. Unspent SP available next mission. |
| **Talent Tree Purchases** | All purchased talents remain active for all future missions. |
| **Achievements** | Unlocked achievements persist permanently. |
| **Total Statistics** | Kill counts, build counts, etc. accumulate across missions. |
| **Settings** | Graphics, audio, controls, language — persisted in profile. |

### 1.2 Partial Persistence (Carries with Rules)

| Element | Carry Rule |
|---------|-----------|
| **Gold** | 20% of gold earned in mission carries over (reward, not starting gold) |
| **Wood** | 20% of wood earned carries over |
| **Unit Unlocks** | Once unlocked (via talent tree or mission reward), permanently available |
| **Building Unlocks** | Once unlocked, permanently available |
| **Hero Levels** | Heroes reset to base level each mission (balance) |

### 1.3 No Persistence (Per-Mission Only)

| Element | Reason |
|---------|--------|
| Units and buildings | Fresh start each mission for balance |
| Mission-specific resources | Distinct economy per mission |
| Unit experience/kills | Balance — each mission is a self-contained scenario |
| Map progress/fog | New map each mission |
| Mission-specific objectives | Self-contained per mission |

---

## 2. Persistent Gold Calculation

```
Mission Reward Gold = 20% of (Gold Mined + Gold Looted + Gold Rewarded)

Example:
  Mission 02 completion:
    - Gold mined: 450
    - Gold looted: 120
    - Gold rewarded: 100
    - Total: 670
    - Carry over (20%): 134

  Player starts Mission 03 with:
    - Base starting gold: 200
    - Persistent gold: 134
    - Total starting gold: 334
```

### 2.1 Persistence Caps

| Resource | Cap | Notes |
|----------|-----|-------|
| Legacy Gold | 10,000 | Maximum that can accumulate |
| Legacy Wood | 5,000 | Maximum that can accumulate |
| SP | 33 (theoretical) | Cannot exceed total earned |

---

## 3. Unlock Progression

### 3.1 Unlock Sources

| Source | What It Unlocks | Example |
|--------|-----------------|---------|
| **Talent Tree Purchase** | Abilities, modifiers, units | Light Cavalry at Cavalry T1 |
| **Mission Completion** | Buildings, units, mission itself | Archery Range unlocked after Mission 01 |
| **Achievement** | Cosmetic rewards, bonus SP | "The Architect" achievement |
| **Chapter Completion** | Next chapter missions | Chapter 1 → Chapter 2 |

### 3.2 Unlock Tracking

```json
{
  "unlocked_units": [
    {"id": "unit_swordsman_basic", "source": "default"},
    {"id": "unit_archer_basic", "source": "mission_01_reward"},
    {"id": "unit_cavalry_scout", "source": "talent_cavalry_t1"}
  ],
  "unlocked_buildings": [
    {"id": "building_barracks", "source": "default"},
    {"id": "building_archery_range", "source": "mission_01_reward"}
  ],
  "unlocked_abilities": [
    {"id": "ability_sword_storm", "source": "talent_blacksmith_t3"},
    {"id": "ability_hit_and_run", "source": "talent_cavalry_t2"}
  ]
}
```

---

## 4. Upgrade Application Flow

```
1. MISSION COMPLETE
      │
2. Calculate persistent rewards
      │
3. Save to profile:
      │  ├── SP earned → add to SP pool
      │  ├── Gold 20% → add to legacy gold
      │  ├── Wood 20% → add to legacy wood
      │  ├── Unlocks → add to unlock registry
      │  └── Stats → update cumulative stats
      │
4. NEXT MISSION START
      │
5. Load persistent profile
      │
6. Apply all talent tree modifiers to base game state
      │
7. Apply persistent starting resources
      │
8. Apply unlocked units/buildings to production menus
      │
9. Begin mission
```

---

## 5. Upgrade Persistence Validation

| Rule | Enforcement |
|------|-------------|
| **Balance integrity** | Persistent gold cannot exceed cap |
| **Unlock consistency** | If a prerequisite unlock is missing, dependent unlocks are disabled |
| **Version migration** | If game version changed, persistent data is migrated |
| **Cheat detection** | If cheats were used, persistent rewards for that mission are flagged (no achievement) |

---

## 6. Related Files

| File | Content |
|------|---------|
| `02_Tech_Tree.md` | What talents are available for purchase |
| `03_Sovereignty_Points.md` | How SP is earned and spent |
| `05_Unlock_Schedule.md` | Per-mission unlock progression |
