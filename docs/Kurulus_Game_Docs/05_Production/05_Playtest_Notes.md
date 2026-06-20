# Playtest Notes — Testing Methodology & Feedback Tracking

> **Description:** Defines the playtesting methodology, feedback collection templates, and iteration process for "Kuruluş."

---

## 1. Playtest Phases

| Phase | When | Focus | Testers | Format |
|-------|------|-------|---------|--------|
| **Alpha 1** | Week 8-10 | Core combat, hero abilities | Internal (2-3) | Weekly structured session |
| **Alpha 2** | Week 16-18 | AI, stealth, progression | Internal + 2 external | Weekly structured session |
| **Beta 1** | Week 22-24 | Full campaign, balancing | 5-10 external | Guided testing |
| **Beta 2** | Week 28-30 | Polish, UI, audio, performance | 10-20 external | Open testing |
| **Release Candidate** | Week 32 | Final checks, bug hunt | Internal | Bug bash |

---

## 2. Tester Profiles

| Profile | Skill Level | Genre Preference | Value |
|---------|-------------|------------------|-------|
| **RTS Veteran** | Expert (plays AoE2/SC2) | Competitive | Balance, AI difficulty, mechanics depth |
| **Casual RTS** | Intermediate | Campaign-focused | Pacing, fun, clarity |
| **Strategy Gamer** | Intermediate (plays 4X/tactics) | Tactical depth | Ambush/stealth, hero system |
| **New to RTS** | Beginner | Tutorial, accessibility | Onboarding, UI clarity |
| **Historical Fan** | Variable | Story, authenticity | Narrative, setting immersion |

---

## 3. Playtest Session Template

```markdown
# Playtest Session — [Session #]

Date: _______________
Tester: _____________
Profile: ____________
Build Version: _______
Duration: ___________

## 1. Pre-Test Questionnaire

1. Experience with RTS games: [None / Beginner / Intermediate / Expert]
2. Experience with isometric games: [None / Some / A lot]
3. What are you most excited to try? ___________________

## 2. Test Focus Areas (check which were tested)

[ ] Tutorial / Onboarding
[ ] Combat — Melee
[ ] Combat — Ranged
[ ] Combat — Siege
[ ] Hero Abilities
[ ] Unit Control & Formations
[ ] Base Building
[ ] Resource Management
[ ] Stealth / Ambush
[ ] AI Behavior
[ ] Mission Flow
[ ] UI / HUD
[ ] Audio
[ ] Performance

## 3. Session Metrics (observed or recorded)

| Metric | Value | Notes |
|--------|-------|-------|
| Time to complete mission | _____ | |
| Units lost | _____ | |
| Resources gathered | _____ | |
| Buildings constructed | _____ | |
| Abilities used | _____ | |
| Times paused | _____ | |
| Crashes | _____ | Describe: |
| Softlocks | _____ | Describe: |

## 4. Feedback — What worked well?

1. ________________________________________________
2. ________________________________________________
3. ________________________________________________

## 5. Feedback — What needs improvement?

| Issue | Severity (1-5) | Suggested Fix |
|-------|----------------|---------------|
| 1.    |                |               |
| 2.    |                |               |
| 3.    |                |               |

## 6. Bug Reports

| Bug ID | Description | Steps to Reproduce | Severity | Status |
|--------|-------------|-------------------|----------|--------|
| BUG-001 | | | | |

## 7. Open Questions / Comments

________________________________________________
________________________________________________
________________________________________________

## 8. Post-Test Rating (1-5)

- Fun: _____
- Difficulty: _____
- Clarity: _____
- Performance: _____

---
```

---

## 4. Feedback Prioritization Matrix

| Severity | Label | Response | Examples |
|----------|-------|----------|----------|
| **5 — Critical** | Game-breaking | Fix immediately, block release | Crash, save corruption, softlock |
| **4 — Major** | Severe impact | Fix within current phase | Unit stuck, objective impossible, balance broken |
| **3 — Moderate** | Noticeable | Fix in next milestone | AI too easy, UI confusion, awkward controls |
| **2 — Minor** | Annoying | Fix if time allows | Text misalignment, sound volume, animation glitch |
| **1 — Suggestion** | Wishlist | Consider for future | "Would be cool if..." |

---

## 5. Playtest Session Log

| Date | Build | Tester(s) | Focus | Key Findings | Action Items |
|------|-------|-----------|-------|-------------|--------------|
| — | — | — | — | — | — |

*(Start filling from first session)*

---

## 6. Common Issues Tracking

| Issue Code | Description | First Reported | Occurrences | Status |
|------------|-------------|----------------|-------------|--------|
| CI-001 | Units don't respond to move command when in combat | — | — | Open |
| CI-002 | Archer projectiles go through walls | — | — | Open |
| CI-003 | Mission 03 objective marker wrong position | — | — | Open |
| CI-004 | AI doesn't build economy (stuck at 10 units) | — | — | Open |
| CI-005 | Save file corrupt after using ability | — | — | Open |

---

## 7. Balance Testing Checklist

| System | Test Cases |
|--------|------------|
| **Melee combat** | Swordman vs Spearman (equal numbers); Swordman vs Cavalry (counter) |
| **Ranged combat** | Archer vs Swordman (siege); Archer vs Archer (mirror) |
| **Economy** | Build order speed; resource bottleneck check |
| **Hero abilities** | Each ability used 10x; cooldown timer accuracy |
| **Tech tree** | All 5 tiers unlockable; cost-to-benefit ratio |
| **AI difficulty** | Player idle for 10 min on each difficulty; AI army size check |
| **Missions** | Speedrun (30 min); completionist (all objectives); pacifist (minimal kills) |

---

## 8. Iteration Cycle

```
Playtest Session
      │
      ▼
Collect Feedback
      │
      ▼
Categorize (Bug / Balance / Polish / Feature Request)
      │
      ▼
Prioritize (Severity × Frequency)
      │
      ▼
Implement Changes
      │
      ▼
Regression Test
      │
      ▼
Next Playtest Session
```

---

## 9. Related Files

| File | Content |
|------|---------|
| `01_Task_Breakdown.md` | Tasks for implementing fixes |
| `02_Milestone_Plan.md` | Milestones with test phases |
| `04_Risk_Register.md` | Risks validated via playtesting |
