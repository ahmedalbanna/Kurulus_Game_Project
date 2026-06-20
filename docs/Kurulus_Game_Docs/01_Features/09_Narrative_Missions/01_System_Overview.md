# Narrative Missions — System Overview

> **Description:** Defines the narrative mission framework — how missions are structured, how objectives work, how dialogue is triggered, and how branching narrative paths are managed.

---

## 1. System Overview

```
NARRATIVE MISSION SYSTEM
│
├── Mission Framework
│   ├── Mission structure (phases, acts)
│   ├── Objective system (primary / secondary / bonus)
│   └── Victory / defeat conditions
│
├── Dialogue System
│   ├── Dialogue trees and nodes
│   ├── Character assignment
│   ├── Conditional dialogue
│   └── Subtitles / VO integration
│
├── Cinematic Triggers
│   ├── Scripted sequence triggers
│   ├── Camera control
│   └── Player input lock
│
└── Branching Narrative
    ├── Choice points
    ├── Consequence tracking
    ├── Mission variant selection
    └── Replayability
```

---

## 2. Design Goals

| # | Goal | Rationale |
|---|------|-----------|
| 1 | **Clear mission structure** | Every mission has defined phases, objectives, and end conditions |
| 2 | **Narrative woven into gameplay** | Dialogue and story events happen during gameplay, not just before/after |
| 3 | **Player agency** | Choices in missions affect story and rewards |
| 4 | **Replayable** | Different choices lead to different outcomes and bonus objectives |
| 5 | **Cinematic without interrupting flow** | Scripted sequences are short and skippable |

---

## 3. File Reference

| File | Content |
|------|---------|
| `02_Mission_Structure.md` | Mission states, phases, victory/defeat conditions |
| `03_Objective_Types.md` | Complete catalog of objective types |
| `04_Dialogue_System.md` | Dialogue tree architecture, node types |
| `05_Cinematic_Triggers.md` | Triggered cutscenes, camera sequences |
| `06_Branching_Narrative.md` | Choice points, consequence tracking, variants |
