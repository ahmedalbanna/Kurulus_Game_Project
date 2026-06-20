# Dynamic Audio — Mixing, 3D Audio & Adaptive Systems

> **Description:** Defines the dynamic audio mixing system — how sounds interact, 3D positioning, bus routing, and adaptive audio behaviors.

---

## 1. Audio Bus Architecture

```
MASTER BUS
│
├── MUSIC BUS
│   ├── Exploration Layer
│   ├── Combat Layer
│   └── Narrative Layer
│
├── SFX BUS
│   ├── Combat SFX (highest priority)
│   ├── UI SFX
│   ├── Unit SFX
│   ├── Ability SFX
│   └── Ambience SFX
│
├── VOICEOVER BUS
│   ├── Story VO
│   ├── Combat VO
│   └── Ambient VO
│
└── AMBIENCE BUS
    ├── Weather
    ├── Environment
    └── Distant sounds
```

---

## 2. Ducking Rules

| Priority | Source | Duck Target | Duck Amount | Fade |
|----------|--------|-------------|-------------|------|
| **Highest** | Story VO | Music | -6 dB | 0.3s |
| | Story VO | Ambience | -4 dB | 0.3s |
| | Story VO | Combat SFX | -3 dB | 0.2s |
| **High** | Combat SFX | Music (during combat) | -3 dB | 0.5s |
| | Cinematic | All non-essential | -8 dB | 0.5s |
| **Medium** | UI SFX | None | - | - |
| | Ability SFX | Music | -2 dB | 0.2s |
| **Low** | Ambient | None | - | - |

---

## 3. 3D Audio Settings

| Parameter | Value |
|-----------|-------|
| Audio Engine | 3D positional (Wwise / FMOD concept) |
| Attenuation | Logarithmic |
| Min Distance | 2m (full volume) |
| Max Distance | 50m (inaudible beyond) |
| Doppler | Disabled (RTS, not needed) |
| Occlusion | Simple raycast (buildings block) |
| Obstruction | Gradual (partial cover reduces volume) |
| Reverb Zones | Per terrain/map zone |

### 3.1 Attenuation by Sound Type

| Sound Type | Min Distance | Max Distance | Roll-off |
|------------|-------------|-------------|----------|
| Combat SFX | 5m | 60m | Medium |
| Unit barks | 3m | 30m | Fast |
| Ability SFX | 10m | 80m | Slow |
| Ambient | 2m | 40m | Medium |
| VO (combat) | 5m | 25m | Fast |
| Distant battle | 0m (world) | 200m | Very slow |

---

## 4. Reverb Zones

| Zone Type | Reverb | Decay Time | Description |
|-----------|--------|------------|-------------|
| Open field | Low | 0.3s | Minimal echo |
| Forest | Medium | 0.6s | Natural diffusion |
| Mountain pass | High | 1.2s | Echo between cliffs |
| Fortress interior | Very High | 2.0s | Stone hall echo |
| Village | Low | 0.4s | Open but enclosed |
| Cave/tunnel | Extreme | 3.0s | Long decay |
| Coast/open water | Low | 0.2s | Open, minimal reverb |

---

## 5. Audio Priority & Polyphony

| Priority Band | Max Voices | Preemption |
|---------------|------------|------------|
| **Critical** (VO, alert) | 4 | Cannot be preempted |
| **High** (combat SFX, abilities) | 8 | Preempts medium/low |
| **Medium** (unit SFX, UI) | 12 | Preempts low |
| **Low** (ambient, music layers) | 8 | Lowest priority |
| **Total** | **32** | |

---

## 6. Adaptive Audio Behaviors

### 6.1 Combat Intensity

| Intensity Level | Enemy Units Near Player | Audio Changes |
|----------------|------------------------|---------------|
| 0 — Peaceful | 0 | Full exploration mix |
| 1 — Alert | 1-3 spotted | Music rhythm layer adds, combat SFX range increased |
| 2 — Skirmish | 4-10 engaged | Full combat music, battle SFX, combat VO |
| 3 — Battle | 11-30 engaged | Maximum intensity, siege sounds, commander VO |
| 4 — Siege/Boss | 30+ | Cinematic intensity, boss-specific audio, all layers max |

### 6.2 Time of Day Filtering

| Time | EQ Change | Volume Change |
|------|-----------|---------------|
| Dawn | Boost highs (+2dB) | +1dB |
| Day | Flat | 0dB |
| Dusk | Cut highs (-2dB), boost lows | -1dB |
| Night | Heavy cut highs (-6dB), boost lows (+3dB) | -3dB |

---

## 7. Audio Memory Budget

| Category | Budget |
|----------|--------|
| Loaded music (streaming) | 30 MB |
| Loaded SFX (compressed) | 15 MB |
| Loaded VO (streaming) | 20 MB |
| Audio engine overhead | 10 MB |
| **Total** | **75 MB** |

---

## 8. Related Files

| File | Content |
|------|---------|
| `02_Music_System.md` | Music system |
| `03_SFX_Catalog.md` | Sound effects |
| `04_Voiceover_Guide.md` | Voiceover guide |
