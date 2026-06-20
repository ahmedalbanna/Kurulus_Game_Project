# Milestone Plan — Development Schedule & Deliverables

> **Description:** Milestone-based delivery schedule with clear deliverables, acceptance criteria, and resource allocation.

---

## 1. Milestone Overview

| Milestone | Phase | Target | Duration | Deliverable |
|-----------|-------|--------|----------|-------------|
| **M0 — Prototype** | P0 | Week 4 | 4 weeks | Playable map with character movement |
| **M1 — Core Combat** | P1 | Week 12 | 8 weeks | Full combat system, 2 heroes, basic AI |
| **M2 — Advanced Systems** | P2 | Week 18 | 6 weeks | Ambush/stealth, full AI, progression |
| **M3 — Content Complete** | P3 | Week 24 | 6 weeks | All 6 missions playable, world map |
| **M4 — Feature Freeze** | P4 | Week 28 | 4 weeks | UI/audio/art complete, balancing done |
| **M5 — Release Candidate** | P5 | Week 32 | 4 weeks | Tested, optimized, ready for release |

---

## 2. Milestone M0 — Prototype (Week 4)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M0.1 | Isometric camera | Pan, zoom, rotate functional |
| M0.2 | Tilemap grid | Place objects on grid, height support |
| M0.3 | Basic unit movement | Click to move, pathfinding via A* |
| M0.4 | Data pipeline | Config files load correctly (JSON/CSV) |
| M0.5 | Event system | Events fire and propagate |
| M0.6 | Save/load proof-of-concept | State serializes and deserializes |

### Exit Criteria

- ✅ Player can scroll around an isometric map
- ✅ Player can click a unit and move it to a destination
- ✅ All config files load without errors
- ✅ Game state can be saved and loaded
- ✅ Event bus processes basic game events

---

## 3. Milestone M1 — Core Combat (Week 12)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M1.1 | Melee combat | Units engage, deal/take damage, die |
| M1.2 | Ranged combat | Projectiles fire, hit detection works |
| M1.3 | Damage model | Armor/penetration/damage types applied |
| M1.4 | Unit formations | 3+ formation types, units stay in position |
| M1.5 | Hero — Osman | All 4 abilities functional with cooldowns |
| M1.6 | Hero — Turgut | All 4 abilities functional with cooldowns |
| M1.7 | Talent tree | At least 2 talent rows unlockable |
| M1.8 | Morale system | Morale affects combat effectiveness |
| M1.9 | Basic AI | AI units attack player, pathfind |
| M1.10 | Fog of war | War fog reveals/obscures correctly |
| M1.11 | Base building | Place buildings, gather resources |
| M1.12 | Economy loop | Resources generate, buildings cost resources |

### Exit Criteria

- ✅ Two heroes fully playable with ability systems
- ✅ AI units engage player and respond to threats
- ✅ Buildings can be constructed and produce units
- ✅ Economy loop functional (gather → spend → grow)
- ✅ All combat calculations correct per specs
- ✅ Formations work with at least 10 units per group

---

## 4. Milestone M2 — Advanced Systems (Week 18)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M2.1 | Stealth system | Units enter/exit stealth, stealth zones work |
| M2.2 | Ambush mechanics | Ambush bonus damage, surprise factor |
| M2.3 | Detection system | Line of sight, noise, distance detection work |
| M2.4 | Hero — Buran | All 4 abilities functional with cooldowns |
| M2.5 | Equipment system | Equip weapons/armor, stats update |
| M2.6 | Byzantine AI | Behaves defensively, uses shield wall |
| M2.7 | Mongol AI | Uses hit-and-run, feigned retreat |
| M2.8 | Strategic AI | AI builds economy, composes army |
| M2.9 | Difficulty scaling | 4 difficulty levels affect parameters |
| M2.10 | Tech tree | 3 paths (Blacksmith/Cavalry/Faith) with 5 tiers each |
| M2.11 | Sovereignty Points | SP earned and spent on unlocks |
| M2.12 | Persistent upgrades | Upgrades carry between missions |

### Exit Criteria

- ✅ Stealth and ambush systems fully functional
- ✅ All 3 heroes playable with complete ability kits
- ✅ Both AI factions behave distinctly per personality
- ✅ AI can win against a passive player on Medium
- ✅ Tech tree complete with all upgrades unlockable
- ✅ Campaign progression system works end-to-end

---

## 5. Milestone M3 — Content Complete (Week 24)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M3.1 | Mission 01 — Kulucahisar | Tutorial, completable, teaches basic mechanics |
| M3.2 | Mission 02 — Yarhisar | Stealth intro, completable |
| M3.3 | Mission 03 — Bilecik | Ambush mission, completable |
| M3.4 | Mission 04 — Marmaracık | Multi-front battle, completable |
| M3.5 | Mission 05 — Byzantine Fortress | Siege mission, completable |
| M3.6 | Mission 06 — Yenişehir | Final battle, completable |
| M3.7 | Cinematic triggers | Chapter transitions trigger correctly |
| M3.8 | Dialogue system | All dialogues display, choices matter |
| M3.9 | World map | Regions visible, mission selection works |
| M3.10 | Faction relations | Diplomacy affects available missions |
| M3.11 | Strategic events | Random events appear and affect gameplay |

### Exit Criteria

- ✅ All 6 missions playable from start to finish
- ✅ No softlocks or progression blockers
- ✅ All dialogue lines display correctly in Arabic
- ✅ Cinematic transitions work between chapters
- ✅ World map functional for mission selection
- ✅ Save/load works across missions (persistent upgrades)

---

## 6. Milestone M4 — Feature Freeze (Week 28)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M4.1 | HUD complete | All UI elements functional |
| M4.2 | Menu system | Main menu, pause menu, settings all work |
| M4.3 | Input rebinding | All keyboard/mouse inputs rebindable |
| M4.4 | Colorblind modes | 3 colorblind filter options |
| M4.5 | Music system | Dynamic music transitions between states |
| M4.6 | SFX integration | All actions have corresponding sound effects |
| M4.7 | Voiceover | All mission VO in place (placeholder OK) |
| M4.8 | Art finalization | All 3D models final or final placeholder |
| M4.9 | VFX complete | Combat, building, ability effects in place |
| M4.10 | Balance pass | All combat/economy values finalized |

### Exit Criteria

- ✅ No new features added after this milestone
- ✅ UI is complete and polished
- ✅ All audio systems functional
- ✅ Art is final or near-final
- ✅ All balance values finalized in config files
- ✅ Game is fully playable from start to end

---

## 7. Milestone M5 — Release Candidate (Week 32)

### Deliverables

| # | Deliverable | Acceptance Criteria |
|---|-------------|-------------------|
| M5.1 | Full playthrough | All missions completable without crashes |
| M5.2 | Performance targets | 60 FPS on target spec, <10s loading |
| M5.3 | Bug tracker zero | All P0/P1 bugs resolved |
| M5.4 | Store assets | Screenshots, trailer, description ready |
| M5.5 | Build pipeline | Automated builds for target platform(s) |
| M5.6 | Compliance pass | Legal, age rating, platform requirements met |

### Exit Criteria

- ✅ All acceptance criteria met
- ✅ 60 FPS on minimum specification hardware
- ✅ All critical bugs fixed
- ✅ Build passes platform certification
- ✅ Ready for public release

---

## 8. Gantt Summary

```
Week  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
P0    [██████████████████]
P1                        [████████████████████████████████]
P2                                                         [████████████████████████]
P3                                                                                   [████
P4
P5

Week 19 20 21 22 23 24 25 26 27 28 29 30 31 32
P3    ██████████████████████]
P4                         [████████████████████]
P5                                             [████████████████]
```

---

## 9. Resource Allocation

| Phase | Dev 1 (Systems) | Dev 2 (Content) | Dev 3 (Art/Audio) |
|-------|----------------|-----------------|-------------------|
| P0 | Core architecture | — | — |
| P1 | Combat, units, heroes | Base building, economy | Prototype models |
| P2 | AI, stealth, progression | — | — |
| P3 | Mission framework | Missions 01-06 | Environment art |
| P4 | Balancing, optimization | UI implementation | Final art, audio |
| P5 | Testing, bug fixing | Testing, localization | Store assets |

---

## 10. Related Files

| File | Content |
|------|---------|
| `01_Task_Breakdown.md` | Detailed task list with estimates |
| `03_Asset_List.md` | Asset production tracking |
| `04_Risk_Register.md` | Risk assessment |
| `05_Playtest_Notes.md` | Playtesting methodology |
