# Risk Register — Project Risks & Mitigation Strategies

> **Description:** Comprehensive risk assessment for "Kuruluş" development — technical, design, production, and business risks with likelihood, impact, and mitigation plans.

---

## 1. Risk Scoring Matrix

| Score | Likelihood | Impact |
|-------|------------|--------|
| 5 | Almost certain | Critical — project failure |
| 4 | Likely | Major — significant delay/quality loss |
| 3 | Possible | Moderate — noticeable impact |
| 2 | Unlikely | Minor — small impact |
| 1 | Rare | Negligible — minimal impact |

**Risk Score = Likelihood × Impact** (range: 1-25)

| Risk Level | Score Range | Action Required |
|------------|-------------|-----------------|
| 🔴 Critical | 16-25 | Immediate mitigation plan |
| 🟡 High | 10-15 | Active monitoring + mitigation |
| 🟢 Medium | 5-9 | Regular review |
| ⚪ Low | 1-4 | Accept / monitor |

---

## 2. Technical Risks

| ID | Risk | Likelihood | Impact | Score | Mitigation |
|----|------|------------|--------|-------|------------|
| T01 | **Pathfinding performance degrades with 100+ units** | 4 | 4 | **🔴 16** | Grid-based A* with hierarchical pathfinding; navmesh baking for static geometry; limit path recalculation to 10 paths/frame |
| T02 | **Fog of war reveals too slow / expensive** | 3 | 3 | 🟡 9 | Use precomputed visibility textures; update only when units move; GPU-based reveal |
| T03 | **Save/load breaks with complex state** | 3 | 4 | 🟡 12 | Use deterministic serialization schema; version state format; write automated save/load tests |
| T04 | **Multi-threading conflicts in simulation loop** | 3 | 3 | 🟡 9 | Single-threaded simulation with worker threads for pathfinding only; use command pattern for thread-safe actions |
| T05 | **Unit formation glitches at formation edges** | 4 | 2 | 🟡 8 | Use spring-based position smoothing; clamp units to formation bounds; test with 50+ units |
| T06 | **Event bus memory leak (unsubscribed listeners)** | 4 | 3 | 🟡 12 | Weak references for UI listeners; force unsubscribe on object destroy; event profiler tool |
| T07 | **Memory budget exceeded on target platform** | 2 | 4 | 🟡 8 | Set asset budget early; LOD system; texture streaming; track memory per scene |
| T08 | **Animation blending causes visual glitches** | 3 | 2 | 🟢 6 | Use animation layers; test all transitions; blend durations in config |
| T09 | **Translation stutter in isometric movement** | 3 | 3 | 🟡 9 | Use velocity-based movement (not teleport); interpolation; camera smoothing |
| T10 | **Build size exceeds platform limit** | 2 | 3 | 🟢 6 | Texture compression; audio compression; asset bundles; strip unused code |

---

## 3. Design Risks

| ID | Risk | Likelihood | Impact | Score | Mitigation |
|----|------|------------|--------|-------|------------|
| D01 | **Stealth/ambush is not fun or frustrating** | 3 | 4 | **🔴 15** | Prototype stealth early (week 13); playtest with 5+ external players; tune detection thresholds |
| D02 | **Economy balance is broken (too easy/hard)** | 4 | 3 | 🟡 12 | All values in config CSV; daily balance testing in P4; use spreadsheet formulas to model snowball |
| D03 | **Mission 03 (Bilecik) ambush is unclear** | 3 | 3 | 🟡 9 | Tutorial hints during mission; objective markers; optional "hint" button |
| D04 | **AI is too predictable or too unfair** | 3 | 4 | 🟡 12 | Parameterized AI tuning; 4 difficulty levels; playtest with different skill levels |
| D05 | **Units feel unresponsive to commands** | 4 | 3 | 🟡 12 | Command buffer with queuing; instant feedback (audio/visual) on command receipt; test input lag |
| D06 | **Hero abilities overshadow units** | 3 | 2 | 🟢 6 | Heroes have cooldowns and resource costs; units remain cost-effective; balance pass |
| D07 | **Friendly fire causes player frustration** | 3 | 3 | 🟡 9 | Clear visual feedback for friendly fire; damage reduction for allies; option to toggle |
| D08 | **Morale system feels arbitrary** | 2 | 2 | 🟢 4 | Show morale bar always visible; clear tooltip on what affects morale; audio cues |
| D09 | **Progression (tech tree) feels grindy** | 3 | 3 | 🟡 9 | Multiple unlock paths; sovereignty points earned from achievements; pacing review |
| D10 | **World map strategic layer too shallow** | 2 | 3 | 🟢 6 | Add random events between missions; faction relation changes; optional side-missions |

---

## 4. Production Risks

| ID | Risk | Likelihood | Impact | Score | Mitigation |
|----|------|------------|--------|-------|------------|
| P01 | **Art asset pipeline bottlenecks (models not ready)** | 4 | 4 | **🔴 16** | Start with blockout/placeholder models; parallelize modeling; outsource if needed |
| P02 | **Voiceover recording delayed** | 3 | 3 | 🟡 9 | Record placeholder VO early; final recording can be swapped in later |
| P03 | **Team member leaves mid-project** | 2 | 4 | 🟡 8 | Cross-train; document architecture decisions; maintain code quality |
| P04 | **Feature creep: stakeholders want more** | 4 | 3 | 🟡 12 | Strict scope control; feature freeze at M4; defer non-critical to post-release |
| P05 | **Platform certification fails** | 2 | 4 | 🟡 8 | Research platform requirements early; compliance checklist; test submission process |
| P06 | **Crunch in weeks 28-32 (end of project)** | 4 | 3 | 🟡 12 | Build buffer into schedule; prioritize essential features; don't over-scope M5 |
| P07 | **Animation quality inconsistent across units** | 3 | 2 | 🟢 6 | Style guide for animations; same rig structure for all humanoids; review pass |
| P08 | **Localization (Arabic) text overflow in UI** | 3 | 3 | 🟡 9 | Design UI for RTL; test with longest text strings; dynamic font sizing |
| P09 | **Soundtrack tracks not ready for milestone** | 2 | 2 | 🟢 4 | Temp tracks from royalty-free libraries; swap in final tracks later |
| P10 | **Build server / CI/CD unreliable** | 2 | 3 | 🟢 6 | Dedicated build machine; automated daily builds; test on clean hardware |

---

## 5. Business Risks

| ID | Risk | Likelihood | Impact | Score | Mitigation |
|----|------|------------|--------|-------|------------|
| B01 | **Target audience smaller than expected** | 3 | 3 | 🟡 9 | Focus on niche (historical RTS + ambush mechanic); community engagement during development |
| B02 | **Revenue insufficient to cover costs** | 3 | 4 | 🟡 12 | Keep scope achievable; consider early access; DLC/add-on potential |
| B03 | **IP/trademark conflict with "Kuruluş" title** | 2 | 3 | 🟢 6 | Search trademark databases; have backup title options |
| B04 | **Negative review bombs due to historical inaccuracy** | 2 | 2 | 🟢 4 | Clear "inspired by history, not a simulation" disclaimer; focus on fun over accuracy |
| B05 | **Competing game releases same month** | 2 | 3 | 🟢 6 | Flexible release date; monitor major releases; build wishlist early |

---

## 6. Top 5 Critical Risks (Priority Action)

| Rank | ID | Risk | Score | Action Owner | Review Cadence |
|------|----|------|-------|-------------|----------------|
| 1 | T01 | Pathfinding performance | 🔴 16 | Lead Programmer | Weekly during P1 |
| 2 | P01 | Art pipeline bottleneck | 🔴 16 | Art Lead | Weekly starting P0 |
| 3 | D01 | Stealth not fun | 🔴 15 | Game Designer | Bi-weekly during P2 |
| 4 | T06 | Event bus memory leak | 🟡 12 | Lead Programmer | Monthly |
| 5 | D04 | AI unfair/predictable | 🟡 12 | Game Designer | Monthly during P2-P3 |

---

## 7. Risk Tracking Log

| Date | ID | Score (Prev) | Score (New) | Trend | Note |
|------|----|-------------|-------------|-------|------|
| Week 1 | T01 | — | 16 | → | Initial assessment |
| Week 4 | D01 | — | 15 | → | Prototype needed to validate |
| Week 8 | P01 | — | 16 | → | Start modeling by week 2 |

---

## 8. Related Files

| File | Content |
|------|---------|
| `01_Task_Breakdown.md` | Tasks affected by risks |
| `02_Milestone_Plan.md` | Milestones with risk checkpoints |
| `05_Playtest_Notes.md` | Testing feedback as risk validation |
