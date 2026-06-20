# Task Breakdown — Development Work Packages

> **Description:** Detailed task breakdown for implementing "Kuruluş" — organized by phase, system, and estimated effort.

---

## 1. Development Phases Overview

| Phase | Code | Duration | Systems |
|-------|------|----------|---------|
| **Foundation** | P0 | 4 weeks | Core loop, data layer, input, camera |
| **Core Systems** | P1 | 8 weeks | Combat, units, heroes, base building |
| **Advanced Systems** | P2 | 6 weeks | Ambush/stealth, AI opponents, progression |
| **Content & Campaign** | P3 | 6 weeks | Missions, dialogues, world map, narrative |
| **Polish & Production** | P4 | 4 weeks | UI/UX, audio, art finalization, balancing |
| **Testing & Release** | P5 | 4 weeks | QA, optimization, platform certification |

**Total estimated:** ~32 weeks (8 months)

---

## 2. Phase 0 — Foundation (Weeks 1-4)

### 2.1 Project Setup & Tooling (Week 1)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P0-001 | Create project repository & CI/CD | 4 | — |
| P0-002 | Configure engine/language toolchain | 8 | — |
| P0-003 | Set up asset pipeline (import/export) | 8 | — |
| P0-004 | Configure localization system (Arabic) | 6 | — |
| P0-005 | Set up logging & debugging framework | 4 | — |

### 2.2 Data & Config Layer (Week 2)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P0-006 | Define data schema & serialization | 12 | P0-001 |
| P0-007 | Implement config file loading (JSON/CSV) | 8 | P0-006 |
| P0-008 | Build balance data import system | 6 | P0-006 |
| P0-009 | Create localization data tables | 8 | P0-004 |

### 2.3 Event System & Messaging (Week 2-3)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P0-010 | Implement event bus | 8 | P0-001 |
| P0-011 | Define game event types | 6 | P0-010 |
| P0-012 | Unity Event/Listener pattern helpers | 4 | P0-010 |
| P0-013 | Event-driven UI binding system | 8 | P0-012 |

### 2.4 State Management (Week 3)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P0-014 | Implement game state manager | 10 | P0-006 |
| P0-015 | State persistence (save/load) | 12 | P0-014 |
| P0-016 | Undo/rollback system (strategic layer) | 6 | P0-014 |

### 2.5 Simulation Loop (Week 3-4)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P0-017 | Implement fixed-step game loop | 6 | P0-001 |
| P0-018 | Input buffer & command queue | 8 | P0-001 |
| P0-019 | Network/offline sync wrapper | 4 | P0-018 |
| P0-020 | Pause & time scale system | 4 | P0-017 |

---

## 3. Phase 1 — Core Systems (Weeks 5-12)

### 3.1 Camera & Input (Week 5)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-001 | Isometric camera (pan, zoom, rotate) | 12 | P0-001 |
| P1-002 | Mouse/keyboard input handling | 8 | P0-018 |
| P1-003 | Touch input support (optional) | 6 | P0-018 |
| P1-004 | Camera edge-scrolling & drag | 6 | P1-001 |

### 3.2 Map & Environment (Weeks 5-6)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-005 | Tilemap grid system | 10 | P0-001 |
| P1-006 | Terrain types & height support | 8 | P1-005 |
| P1-007 | Fog of war (basic) | 10 | P1-005 |
| P1-008 | Pathfinding (A*) | 12 | P1-005 |
| P1-009 | Navigation mesh (navmesh) | 8 | P1-008 |

### 3.3 Unit System (Weeks 6-8)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-010 | Unit data component & config | 8 | P0-006 |
| P1-011 | Unit movement & path following | 10 | P1-008 |
| P1-012 | Unit selection (click, box-select, group) | 10 | P1-002 |
| P1-013 | Unit formation system | 12 | P1-011 |
| P1-014 | Unit states (idle, move, attack, death) | 8 | P1-010 |
| P1-015 | Unit animation controller | 10 | P1-014 |
| P1-016 | Unit health & death | 6 | P1-010 |

### 3.4 Combat System (Weeks 7-9)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-017 | Melee combat logic | 10 | P1-010 |
| P1-018 | Ranged combat & projectiles | 12 | P1-010 |
| P1-019 | Damage model (armor, penetration, damage types) | 10 | P1-017 |
| P1-020 | Siege combat & building damage | 8 | P1-017 |
| P1-021 | Morale system | 8 | P1-017 |
| P1-022 | Friendly fire detection | 4 | P1-017 |

### 3.5 Hero System (Weeks 8-10)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-023 | Hero base class & data config | 8 | P1-010 |
| P1-024 | Osman — hybrid leader abilities | 10 | P1-023 |
| P1-025 | Turgut — tank abilities | 10 | P1-023 |
| P1-026 | Buran — scout/archer abilities | 10 | P1-023 |
| P1-027 | Ability system (cooldowns, targeting, effects) | 12 | P1-023 |
| P1-028 | Talent tree UI & logic | 10 | P1-023 |
| P1-029 | Equipment system (weapons, armor, accessories) | 8 | P1-023 |
| P1-030 | Hero progression & XP | 6 | P1-023 |

### 3.6 Base Building (Weeks 9-12)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P1-031 | Building placement system | 10 | P1-005 |
| P1-032 | Building data & types catalog | 8 | P0-006 |
| P1-033 | Construction process & queue | 10 | P1-031 |
| P1-034 | Resource gathering & economy | 12 | P1-031 |
| P1-035 | Building states (construction, active, damaged) | 6 | P1-031 |
| P1-036 | Unit training from buildings | 8 | P1-031 |

---

## 4. Phase 2 — Advanced Systems (Weeks 13-18)

### 4.1 Ambush & Stealth (Weeks 13-14)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P2-001 | Stealth state & visibility system | 10 | P1-010 |
| P2-002 | Stealth zones (tall grass, forests, night) | 8 | P1-005 |
| P2-003 | Ambush trigger & execution | 10 | P2-001 |
| P2-004 | Detection system (line of sight, noise, distance) | 10 | P1-008 |
| P2-005 | Decoy & lure mechanics | 6 | P2-001 |
| P2-006 | Hit & run tactical bonus | 6 | P2-001 |

### 4.2 AI Opponents (Weeks 14-17)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P2-007 | AI controller base system | 8 | P0-010 |
| P2-008 | Byzantine AI personality | 10 | P2-007 |
| P2-009 | Mongol AI personality | 10 | P2-007 |
| P2-010 | Strategic AI (goals, economy, army composition) | 12 | P2-007 |
| P2-011 | Tactical AI (combat decisions, target selection) | 12 | P2-007 |
| P2-012 | AI pathfinding & movement coordination | 8 | P2-007 |
| P2-013 | Difficulty scaling system | 6 | P2-007 |
| P2-014 | AI debugging visualization | 6 | P2-007 |

### 4.3 Progression & Meta (Weeks 17-18)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P2-015 | Tech tree UI & logic | 10 | P1-030 |
| P2-016 | Sovereignty Points system | 6 | P2-015 |
| P2-017 | Persistent upgrades (carry between missions) | 8 | P2-015 |
| P2-018 | Unlock schedule & progression gates | 6 | P2-015 |

---

## 5. Phase 3 — Content & Campaign (Weeks 19-24)

### 5.1 Narrative Missions (Weeks 19-21)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P3-001 | Mission flow controller | 8 | P1-010 |
| P3-002 | Mission 01 — Kulucahisar (tutorial) | 16 | P3-001 |
| P3-003 | Mission 02 — Yarhisar | 16 | P3-001 |
| P3-004 | Mission 03 — Bilecik | 20 | P3-001 |
| P3-005 | Mission 04 — Marmaracık | 20 | P3-001 |
| P3-006 | Mission 05 — Byzantine Fortress | 24 | P3-001 |
| P3-007 | Mission 06 — Yenişehir | 24 | P3-001 |
| P3-008 | Objective system (primary, secondary, bonus) | 10 | P3-001 |
| P3-009 | Dialogue system & UI | 12 | P3-001 |
| P3-010 | Cinematic trigger system | 10 | P3-001 |
| P3-011 | Branching narrative logic | 8 | P3-010 |

### 5.2 World Map (Weeks 22-23)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P3-012 | Campaign world map (strategic layer) | 12 | P2-015 |
| P3-013 | Region system (control, bonuses, events) | 10 | P3-012 |
| P3-014 | Strategic fog of war | 6 | P1-007 |
| P3-015 | Faction relations & diplomacy | 8 | P3-012 |
| P3-016 | Strategic events & random encounters | 8 | P3-012 |
| P3-017 | Loading & transitioning between missions | 6 | P3-012 |

### 5.3 Voiceover & Localization (Week 24)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P3-018 | Voiceover implementation & timing | 12 | P3-009 |
| P3-019 | Arabic text & UI localization | 8 | P0-004 |
| P3-020 | Unit VO lines integration | 6 | P1-014 |

---

## 6. Phase 4 — Polish & Production (Weeks 25-28)

### 6.1 UI/UX (Weeks 25-26)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P4-001 | HUD layout & implementation | 16 | P0-013 |
| P4-002 | Menu system (main, pause, settings) | 10 | P4-001 |
| P4-003 | Information architecture & tooltips | 8 | P4-001 |
| P4-004 | Input mapping & rebinding UI | 6 | P1-002 |
| P4-005 | Accessibility features (colorblind, font size) | 8 | P4-001 |
| P4-006 | Visual design polish & animations | 12 | P4-001 |

### 6.2 Audio (Week 27)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P4-007 | Music system implementation | 8 | P0-001 |
| P4-008 | SFX system & trigger integration | 10 | P0-013 |
| P4-009 | Dynamic audio mixing (combat intensity) | 8 | P4-007 |
| P4-010 | VO playback & subtitle sync | 6 | P3-018 |

### 6.3 Art & Visuals (Weeks 26-28)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P4-011 | 3D model finalization (units, buildings, heroes) | 40 | P0-003 |
| P4-012 | Animation finalization & blending | 20 | P1-015 |
| P4-013 | Environment art & terrain textures | 30 | P1-005 |
| P4-014 | UI art & icons | 16 | P4-001 |
| P4-015 | VFX (particles, combat effects, magic) | 16 | P1-017 |
| P4-016 | Lighting & post-processing | 10 | P0-002 |

### 6.4 Balancing (Week 28)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P4-017 | Combat balance pass | 16 | P1-019 |
| P4-018 | Economy balance pass | 12 | P1-034 |
| P4-019 | AI difficulty tuning | 12 | P2-013 |
| P4-020 | Mission difficulty tuning | 10 | P3-001 |

---

## 7. Phase 5 — Testing & Release (Weeks 29-32)

### 7.1 Quality Assurance (Weeks 29-30)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P5-001 | Core loop playtesting | 20 | P0-020 |
| P5-002 | Combat & unit balancing QA | 16 | P4-017 |
| P5-003 | Campaign playthrough (all missions) | 24 | P3-001 |
| P5-004 | AI behavior testing | 16 | P2-014 |
| P5-005 | Localization & text QA | 8 | P3-019 |
| P5-006 | Accessibility testing | 8 | P4-005 |

### 7.2 Performance & Optimization (Week 31)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P5-007 | Profiling & optimization pass | 20 | P0-002 |
| P5-008 | Memory & loading time optimization | 12 | P5-007 |
| P5-009 | Build size reduction | 8 | P5-007 |
| P5-010 | Platform-specific optimization | 12 | P5-007 |

### 7.3 Release Preparation (Week 32)

| ID | Task | Est. Hours | Depends On |
|----|------|------------|------------|
| P5-011 | Store page assets (screenshots, trailer) | 16 | P4-016 |
| P5-012 | Build pipeline & versioning | 8 | P5-007 |
| P5-013 | Legal & compliance review | 4 | — |
| P5-014 | Release candidate & deployment | 8 | P5-011 |

---

## 8. Effort Summary

| Phase | Work Hours | Duration | Full-Time Devs |
|-------|------------|----------|----------------|
| P0 — Foundation | 134 | 4 weeks | 1 |
| P1 — Core Systems | 316 | 8 weeks | 2 |
| P2 — Advanced Systems | 108 | 6 weeks | 2 |
| P3 — Content & Campaign | 292 | 6 weeks | 2-3 |
| P4 — Polish & Production | 296 | 4 weeks | 3 |
| P5 — Testing & Release | 188 | 4 weeks | 2-3 |
| **Total** | **~1,334** | **32 weeks** | **1-3 devs** |

---

## 9. Related Files

| File | Content |
|------|---------|
| `02_Milestone_Plan.md` | Milestone schedule & deliverables |
| `03_Asset_List.md` | Asset production tracking |
| `04_Risk_Register.md` | Risk assessment & mitigation |
| `05_Playtest_Notes.md` | Playtesting methodology & templates |
