# Asset List — 3D Models, Audio, UI, and VFX

> **Description:** Comprehensive asset inventory for production tracking — including 3D models, animations, audio files, UI elements, and visual effects.

---

## 1. Asset Categories

| Category | Code | Count | Priority |
|----------|------|-------|----------|
| 3D Units | A3D_U | 22 | High |
| 3D Buildings | A3D_B | 15 | High |
| 3D Heroes | A3D_H | 3 | High |
| 3D Environment | A3D_E | 20 | Medium |
| 3D Props | A3D_P | 12 | Low |
| Animations | AAN | 60+ | High |
| UI Screens | AUI_S | 12 | High |
| UI Elements | AUI_E | 40+ | Medium |
| SFX | ASFX | 80+ | High |
| Music | AMUS | 12 | Medium |
| Voiceover | AVO | 200+ | High |
| VFX | AVFX | 30 | Medium |
| Icons | AIC | 50+ | Medium |

---

## 2. 3D Unit Models

| ID | Name | Faction | Type | Polys | Variants | Animations Required |
|----|------|---------|------|-------|----------|-------------------|
| U01 | Swordman | Player | Melee | 3K | 2 (sword/shield) | Idle, Walk, Run, Attack, Block, Die |
| U02 | Archer | Player | Ranged | 3K | 1 | Idle, Walk, Shoot, Reload, Die |
| U03 | Spearman | Player | Melee | 3K | 1 | Idle, Walk, Attack, Block, Die |
| U04 | Light Cavalry | Player | Cavalry | 4K | 1 | Idle, Trot, Gallop, Attack, Die |
| U05 | Heavy Cavalry | Player | Cavalry | 5K | 1 | Idle, Trot, Gallop, Charge, Die |
| U06 | Siege Ram | Player | Siege | 6K | 1 | Push, Attack, Idle |
| U07 | Siege Tower | Player | Siege | 8K | 1 | Move, Deploy |
| U08 | Battering Ram | Player | Siege | 5K | 1 | Push, Attack |
| U09 | Byzantine Swordman | Enemy | Melee | 3K | 2 (shield variants) | Idle, Walk, Attack, Block, Die |
| U10 | Byzantine Archer | Enemy | Ranged | 3K | 1 | Idle, Walk, Shoot, Reload, Die |
| U11 | Byzantine Spearman | Enemy | Melee | 3K | 1 | Idle, Walk, Attack, Block, Die |
| U12 | Byzantine Cavalry | Enemy | Cavalry | 4K | 1 | Idle, Trot, Gallop, Charge, Die |
| U13 | Byzantine Commander | Enemy | Hero | 5K | 1 | Idle, Walk, Attack, Command, Die |
| U14 | Mongol Light Cavalry | Enemy | Cavalry | 4K | 1 | Idle, Trot, Gallop, Shoot, Die |
| U15 | Mongol Horse Archer | Enemy | Cavalry | 4K | 1 | Idle, Trot, Gallop, Shoot, Die |
| U16 | Mongol Swordsman | Enemy | Melee | 3K | 1 | Idle, Walk, Attack, Die |
| U17 | Peasant | Neutral | Civilian | 2K | 2 (male/female) | Idle, Walk, Work, Flee |
| U18 | Merchant | Neutral | Civilian | 2K | 1 | Idle, Walk, Trade |

---

## 3. Hero 3D Models

| ID | Name | Polys | Importance | Special Requirements |
|----|------|-------|------------|---------------------|
| H01 | Osman Bey | 8K | Critical | Distinctive red&white headgear, sword + shield, cape physics |
| H02 | Turgut Alp | 7K | Critical | Large frame, two-handed axe, heavy armor, beard |
| H03 | Buran | 6K | High | Slim build, bow + dagger, hooded cloak, agile stance |

**Hero Animations (each hero):**

| Animation | Priority | Notes |
|-----------|----------|-------|
| Idle (neutral) | High | Breathing, slight movement |
| Idle (combat ready) | High | Weapon drawn, alert |
| Walk | High | Stylized walk cycle |
| Run | High | Faster movement |
| Ability 01 (basic) | High | Basic attack animation |
| Ability 02 | High | Signature move |
| Ability 03 | High | Special ability |
| Ability 04 (ultimate) | High | Ultimate with VFX |
| Victory | Medium | Mission complete pose |
| Death | High | Dramatic death animation |
| Hurt/hit reaction | Medium | Flinch on taking damage |

---

## 4. Building 3D Models

| ID | Name | Faction | Polys | States |
|----|------|---------|-------|--------|
| B01 | Town Hall | Player | 6K | Construction, Active, Damaged, Ruined |
| B02 | Barracks | Player | 5K | Construction, Active, Damaged, Ruined |
| B03 | Archery Range | Player | 4K | Construction, Active, Damaged, Ruined |
| B04 | Stable | Player | 5K | Construction, Active, Damaged, Ruined |
| B05 | Blacksmith | Player | 4K | Construction, Active, Damaged, Ruined |
| B06 | Workshop | Player | 4K | Construction, Active, Damaged, Ruined |
| B07 | Market | Player | 5K | Construction, Active, Damaged, Ruined |
| B08 | House | Player | 3K | Construction, Active, Damaged, Ruined |
| B09 | Wall (straight) | Player | 2K | Construction, Active, Damaged, Broken |
| B10 | Wall (corner) | Player | 2K | Construction, Active, Damaged, Broken |
| B11 | Gate | Player | 4K | Open, Closed, Damaged, Broken |
| B12 | Tower | Player | 4K | Construction, Active, Damaged, Ruined |
| B13 | Byzantine Fortress | Enemy | 10K | Active, Damaged, Broken |
| B14 | Byzantine Tower | Enemy | 4K | Active, Damaged, Broken |
| B15 | Siege Camp | Mongol | 3K | Active |

---

## 5. Environment Assets

| ID | Name | Count | Polys Each | Notes |
|----|------|-------|------------|-------|
| E01 | Ground tile (grass) | 10 | 200 | Tiling, multiple variations |
| E02 | Ground tile (dirt) | 6 | 200 | Tiling |
| E03 | Ground tile (stone) | 6 | 200 | Tiling |
| E04 | Ground tile (snow) | 4 | 200 | Mission 05/06 |
| E05 | Tree (oak) | 3 | 1K | Forest areas |
| E06 | Tree (pine) | 2 | 1K | Mountain areas |
| E07 | Rock (large) | 4 | 800 | Scatter props |
| E08 | Rock (small) | 4 | 400 | Scatter props |
| E09 | Bush/grass clump | 5 | 300 | Scatter props — stealth cover |
| E10 | Tall grass | 3 | 500 | Stealth zones |
| E11 | River water | 2 | — | Animated surface shader |
| E12 | Path/road | 3 | — | Decorative |
| E13 | Fence (wooden) | 2 | 200 | Village decoration |
| E14 | Torch/firepit | 2 | 300 | Light source, animated |
| E15 | Tent | 3 | 800 | Camps |
| E16 | Cart/wagon | 2 | 1K | Scatter prop |
| E17 | Bridge (wooden) | 1 | 2K | River crossing |
| E18 | Bridge (stone) | 1 | 3K | Fortress entry |
| E19 | Banner (player) | 2 | 200 | Cloth physics |
| E20 | Banner (enemy) | 2 | 200 | Cloth physics |

---

## 6. UI Assets

### 6.1 Screens

| ID | Screen | Resolution | Elements |
|----|--------|------------|----------|
| UI01 | Main Menu | Full | Logo, buttons (New Game, Load, Settings, Quit) |
| UI02 | Settings | Full | Sliders (volume, speed), toggles, key bindings |
| UI03 | HUD | Overlay | Minimap, resources, unit info, abilities bar |
| UI04 | Pause Menu | Overlay | Resume, Save, Load, Settings, Quit to Menu |
| UI05 | Mission Briefing | Full | Objectives, starting units, map preview |
| UI06 | Victory Screen | Full | Score, stats, continue button |
| UI07 | Defeat Screen | Full | Retry, load save, quit |
| UI08 | Tech Tree | Full | 3 branches, node icons, descriptions |
| UI09 | World Map | Full | Regions, mission nodes, faction info |
| UI10 | Dialogue Box | Overlay | Speaker portrait, text, choices |
| UI11 | Cinematic | Full | Letterbox bars, subtitle area |
| UI12 | Loading Screen | Full | Progress bar, tips, lore text |

### 6.2 Icons

| Category | Count | Size |
|----------|-------|------|
| Unit portraits | 22 | 128×128 |
| Ability icons | 18 | 64×64 |
| Resource icons | 8 | 48×48 |
| Building icons | 15 | 64×64 |
| Tech tree node icons | 30 | 48×48 |
| UI buttons | 20 | 32×32 |
| Cursor icons | 8 | 32×32 |
| Minimap markers | 6 | 16×16 |

---

## 7. Audio Assets

### 7.1 Music Tracks

| ID | Name | Duration | Usage | Priority |
|----|------|----------|-------|----------|
| M01 | Main Theme | 2:30 | Menu | High |
| M02 | Exploration Calm | 3:00 | Early missions, peaceful | High |
| M03 | Combat Intensity | 2:30 | Combat state | High |
| M04 | Ambush Sting | 0:30 | Ambush trigger | Medium |
| M05 | Byzantine Theme | 2:00 | Byzantine missions, strategic | Medium |
| M06 | Mongol Theme | 2:00 | Mongol encounters | Medium |
| M07 | Victory Fanfare | 0:30 | Mission complete | Medium |
| M08 | Defeat Theme | 1:00 | Mission failure | Low |
| M09 | Stealth Theme | 2:00 | Stealth sections | Medium |
| M10 | Final Battle | 3:00 | Mission 06 | High |
| M11 | Chapter Transition | 1:00 | Cinematic interlude | Medium |
| M12 | World Map Theme | 2:00 | Strategic layer | Low |

### 7.2 SFX Categories

| Category | Count | Examples |
|----------|-------|----------|
| Sword/clash | 8 | Sword hit, block, parry, whoosh |
| Bow | 6 | Shoot, arrow hit, arrow fly-by |
| Shield | 4 | Shield block, shield wall formation |
| Cavalry | 6 | Hoof steps (walk, trot, gallop), horse neigh |
| Building | 8 | Construction, repair, destruction, gate open/close |
| UI | 10 | Click, hover, confirm, cancel, error, notification |
| Ambient | 12 | Wind, birds, fire, water, forest, night |
| Impact | 6 | Arrow hit flesh, sword hit armor, blunt impact |
| Death | 4 | Human death sounds (2 male, 1 female, 1 horse) |
| Morale | 4 | Cheer, panic, rout, rally horn |
| Siege | 6 | Ram impact, catapult fire, wall break, tower deploy |
| Stealth | 4 | Grass rustle, dagger stab, stealth zone enter/exit |
| Weather | 4 | Rain, thunder, snow wind, storm |
| Notification | 4 | Objective complete, new objective, warning, mission start |

### 7.3 Voiceover

| Character | Lines | Priority |
|-----------|-------|----------|
| Osman Bey | 60+ | High |
| Turgut Alp | 30 | High |
| Buran | 20 | Medium |
| Byzantine Commander | 15 | Medium |
| Mongol Commander | 10 | Low |
| Narrator | 15 | Low |
| Peasants/units | 30 | Low |

---

## 8. Visual Effects (VFX)

| ID | Name | Type | Priority |
|----|------|------|----------|
| VFX01 | Sword hit spark | Particle | High |
| VFX02 | Arrow hit | Particle | High |
| VFX03 | Blood splatter | Particle | High |
| VFX04 | Shield block spark | Particle | Medium |
| VFX05 | Building construction dust | Particle | Medium |
| VFX06 | Building destruction | Particle + debris | High |
| VFX07 | Fire (torch, campfire) | Particle | Medium |
| VFX08 | Smoke (battlefield) | Particle | Medium |
| VFX09 | Morale rally glow | Shader + particle | Low |
| VFX10 | Stealth shimmer | Shader | Medium |
| VFX11 | Ambush burst | Particle | High |
| VFX12 | Ultimate ability flash | Particle + shader | High |
| VFX13 | Fog of war | Shader | Medium |
| VFX14 | Rain weather | Particle | Low |
| VFX15 | Snow weather | Particle | Low |
| VFX16 | Selection circle | Shader | Medium |
| VFX17 | Health bar fill | UI shader | Medium |
| VFX18 | Resource gather sparkle | Particle | Low |
| VFX19 | Siege ram impact | Particle + camera shake | High |
| VFX20 | Arrow volley | Particle | Medium |
| VFX21 | Cavalry charge dust | Particle | Medium |
| VFX22 | Commander aura | Shader | Low |
| VFX23 | Night vision | Post-processing | Low |
| VFX24 | Ranged attack tracer | Particle | Medium |
| VFX25 | Charge bonus trail | Particle | Low |
| VFX26 | Morale broken flee | Particle | Medium |
| VFX27 | Healing glow | Shader | Low |
| VFX28 | Speed boost trail | Particle | Low |
| VFX29 | Shield wall aura | Shader | Medium |
| VFX30 | Wall breach collapse | Particle + debris | High |

---

## 9. Asset Production Priority Matrix

| Priority | Description | Timeline |
|----------|-------------|----------|
| **P0 — Critical** | Game cannot function without these | Week 1-8 |
| **P1 — High** | Core gameplay requires these | Week 4-16 |
| **P2 — Medium** | Polish and variety | Week 12-24 |
| **P3 — Low** | Nice-to-have, can be placeholder | Week 20-28 |

---

## 10. Production Tracking

```csv
ID,Name,Type,Status,Assignee,Priority,Target Week,Notes
U01,Swordman,3D Unit,Draft,,P0,Week 2,Base model done
U02,Archer,3D Unit,Not Started,,P0,Week 2,
H01,Osman Bey,3D Hero,Not Started,,P0,Week 4,Requires concept approval
AAN01,Walk Cycle,Animation,Not Started,,P0,Week 2,All units
ASFX01,Sword Hit 1,SFX,Recorded,,P1,Week 6,
AMUS01,Main Theme,Music,Not Started,,P1,Week 8,Composer needed
AVO01,Osman Act1,Dialogue,Not Started,,P2,Week 20,Script ready
```

---

## 11. Related Files

| File | Content |
|------|---------|
| `01_Task_Breakdown.md` | Task list referencing assets |
| `02_Milestone_Plan.md` | Milestone schedule |
| `04_Risk_Register.md` | Asset-related risks |
| `05_Playtest_Notes.md` | Feedback on asset quality |
