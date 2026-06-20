# Information Architecture — Data Hierarchy & Display Rules

> **Description:** Defines how information is organized, prioritized, and displayed to the player — ensuring clarity, reducing cognitive load, and maintaining focus on gameplay.

---

## 1. Information Priority Pyramid

```
                      🔴  CRITICAL
                    Combat alerts, unit death,
                  building destroyed, objective fail
                🟡  IMPORTANT
            Objective updates, resource low,
          new threat detected, ability ready
        🟢  INFORMATIONAL
      Unit selected, building complete, research done,
    resource income, idle units
  🔵  BACKGROUND
Ambient notifications, lore text, tips, statistics
```

---

## 2. Display Rules

### 2.1 What to Show When

| Situation | Show | Hide |
|-----------|------|------|
| **Nothing selected** | Resources, minimap, objectives, idle worker count | Selection panel, ability bar, command bar (dimmed) |
| **Single unit selected** | Unit info, HP, abilities, commands | Multi-unit info |
| **Multiple units selected** | Count, average HP, group commands | Individual abilities (unless same type) |
| **Building selected** | HP, production queue, upgrade options | Unit abilities |
| **Combat occurring** | Unit HP flashing, damage numbers, alert | Tips, ambient info |
| **Paused** | Pause menu overlay | All gameplay HUD (dimmed) |
| **Dialogue active** | Dialogue box, speaker portrait | HUD (dimmed, not hidden) |
| **Cinematic playing** | Subtitles only | Full HUD |

### 2.2 Tooltip Rules

| Element | Tooltip Content | Delay |
|---------|----------------|-------|
| Unit portrait | Unit name, description, stats | 0.3s |
| Ability icon | Ability name, cooldown, mana cost, description | 0.3s |
| Resource icon | Resource name, income rate, capacity | 0.5s |
| Minimap ping | Location name if revealed | 0.5s |
| Objective text | Full objective description | 0.2s |
| Building icon | Building name, tier, production | 0.3s |
| Status effect | Effect name, remaining duration | 0.3s |

---

## 3. Data Aggregation Rules

### 3.1 Multi-Unit Selection

```
2-3 units:     List all unit types with counts
4+ same type:  "Swordsman ×4" with single portrait
Mixed types:   "3 units" with first 3 portraits shown
```

### 3.2 Selection Categories

| Selection | Shows |
|-----------|-------|
| All same type | Unit portrait + count + aggregate HP |
| Mixed combat units | Multiple portraits (max 3) + total count |
| Mixed with workers | Combat units prioritized; worker count shown separately |
| Heroes only | Hero portrait(s) + individual HP + abilities |
| Buildings only | Building portrait + HP + production |

---

## 4. Color Coding System

| Color | Meaning | Use |
|-------|---------|-----|
| 🟢 Green | Friendly, safe, positive | Friendly units, HP full, completed objectives |
| 🟡 Yellow | Warning, caution | Mid HP, attention required, neutral |
| 🟠 Orange | Dangerous, urgent | Low HP, enemy detected nearby |
| 🔴 Red | Critical, hostile | Critical HP, enemy units, failed objectives |
| 🔵 Blue | Information, allied NPC | Allied units, informational text |
| 🟣 Purple | Special, unique | Hero units, rare items, special abilities |
| ⚪ White/Gray | Neutral, inactive | Diminished UI, explored fog areas |
| ⚫ Black | Hidden, unknown | Unexplored fog areas |

---

## 5. Information Refresh Rates

| Data | Refresh Rate | Method |
|------|-------------|--------|
| Unit HP | Every frame (smooth) | Continuous update |
| Resources | Every 0.5s | Increment display |
| Minimap | Every 0.25s | Batch update |
| Objective progress | On change | Event-driven |
| Ability cooldowns | Every frame | Continuous |
| Notifications | On event | Event-driven |
| Population count | On change | Event-driven |
| Income rates | Every 1s | Calculated |

---

## 6. Floating Text Priority

| Priority | Content | Max on Screen |
|----------|---------|---------------|
| 1 (highest) | Critical damage / healing | 10 |
| 2 | Status effect application | 5 |
| 3 | Resource gain (workers) | 3 |
| 4 | XP/level up | 2 |
| 5 (lowest) | Ambient combat barks | 3 |

---

## 7. Related Files

| File | Content |
|------|---------|
| `02_HUD_Layout.md` | Where information is displayed |
| `03_Menu_Structure.md` | Menu information organization |
| `05_Input_Mapping.md` | How to access information |
| `07_Visual_Language.md` | Visual style for information display |
