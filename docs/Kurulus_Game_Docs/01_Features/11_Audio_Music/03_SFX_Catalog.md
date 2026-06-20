# SFX Catalog — Sound Effects Reference

> **Description:** Complete catalog of all sound effects used in *Kuruluş*, organized by category.

---

## 1. UI Sounds

| SFX ID | Description | Priority | Notes |
|--------|-------------|----------|-------|
| `sfx_ui_button_hover` | Button hover | Low | Short tick |
| `sfx_ui_button_click` | Button confirm | Medium | Short click |
| `sfx_ui_button_back` | Cancel/back | Medium | Different pitch |
| `sfx_ui_panel_open` | Panel/menu open | Medium | Whoosh |
| `sfx_ui_panel_close` | Panel/menu close | Medium | Reverse whoosh |
| `sfx_ui_notification_alert` | Warning notification | High | Distinct alert |
| `sfx_ui_notification_success` | Completion notification | High | Rising chime |
| `sfx_ui_notification_info` | Info notification | Medium | Soft ping |
| `sfx_ui_error` | Error dialog | High | Error buzz |
| `sfx_ui_tooltip_show` | Tooltip appears | Very low | Subtle |
| `sfx_ui_drag_select` | Selection box drag | Low | Rubber band |
| `sfx_ui_talent_purchase` | Talent unlocked | High | Ascending chime |
| `sfx_ui_stars_earned` | Stars animation | High | Per-star chime |

---

## 2. Unit Sounds

| SFX ID | Description | Priority | Notes |
|--------|-------------|----------|-------|
| `sfx_unit_select_swordsman` | Swordsman selected | Medium | "نعم!" (Yes!) |
| `sfx_unit_select_archer` | Archer selected | Medium | "أنا جاهز" (Ready) |
| `sfx_unit_select_cavalry` | Cavalry selected | Medium | Horse snort |
| `sfx_unit_select_hero` | Hero selected | High | Hero-specific line |
| `sfx_unit_move` | Move order acknowledged | Medium | "إلى الأمام" (Forward) |
| `sfx_unit_attack` | Attack order | Medium | "هجوم!" (Attack!) |
| `sfx_unit_death_swordsman` | Unit dies | High | Impact + grunt |
| `sfx_unit_death_hero` | Hero dies | Highest | Special impact |
| `sfx_unit_morale_break` | Morale broken | High | Panic shout |
| `sfx_unit_morale_rally` | Morale recovered | Medium | Cheer |
| `sfx_unit_level_up` | Hero levels up | High | Ascending fanfare |

---

## 3. Combat Sounds

| SFX ID | Description | Priority | Variants |
|--------|-------------|----------|----------|
| `sfx_combat_sword_hit` | Sword connects | High | 3 pitches |
| `sfx_combat_sword_clash` | Sword parried | High | 3 pitches |
| `sfx_combat_sword_miss` | Swing misses | Medium | Whoosh |
| `sfx_combat_arrow_fire` | Bow released | High | 2 variants |
| `sfx_combat_arrow_impact` | Arrow hits | High | Flesh + shield |
| `sfx_combat_shield_block` | Shield blocks | High | Thud |
| `sfx_combat_armor_hit` | Hits armor | Medium | Clang |
| `sfx_combat_critical_hit` | Critical hit | High | Enhanced impact |
| `sfx_combat_siege_catapult` | Catapult fires | Highest | Heavy creak + whoosh |
| `sfx_combat_siege_impact` | Siege projectile hits | Highest | Explosion |
| `sfx_combat_fire_hit` | Fire damage | Medium | Sizzle |
| `sfx_combat_poison_hit` | Poison damage | Medium | Acid sizzle |
| `sfx_combat_heal` | Healing | Medium | Warm chime |

---

## 4. Ability Sounds

| SFX ID | Description | Duration |
|--------|-------------|----------|
| `sfx_ability_sword_storm` | Osman's Sword Storm | 2.0s |
| `sfx_ability_warcry` | Osman's War Cry | 1.0s |
| `sfx_ability_shield_bash` | Turgut's Shield Bash | 0.5s |
| `sfx_ability_charge` | Turgut's Charge | 1.5s |
| `sfx_ability_hit_and_run` | Turgut's Hit & Run | 1.0s |
| `sfx_ability_volley` | Buran's Arrow Volley | 2.0s |
| `sfx_ability_snipe` | Buran's Snipe | 0.5s |
| `sfx_ability_decoy` | Buran's Decoy | 1.0s |
| `sfx_ability_banner_faith` | Buran's Banner | 1.5s |
| `sfx_ability_ultimate_osman` | Osman's Ultimate | 3.0s |
| `sfx_ability_ultimate_turgut` | Turgut's Ultimate | 3.0s |
| `sfx_ability_ultimate_buran` | Buran's Ultimate | 3.0s |

---

## 5. Ambush & Stealth Sounds

| SFX ID | Description | Priority | Notes |
|--------|-------------|----------|-------|
| `sfx_stealth_enter` | Enter stealth | Medium | Subtle fade |
| `sfx_stealth_exit` | Exit stealth | Medium | Reveal sound |
| `sfx_stealth_detection_warning` | About to be detected | High | Heartbeat-like |
| `sfx_stealth_detected` | Fully detected | High | Alert sound |
| `sfx_ambush_trigger` | Ambush initiated | Highest | Horn blast |
| `sfx_ambush_surprise` | Enemy surprised | High | Startled shout |
| `sfx_ambush_evaded` | Ambush evaded | Medium | "فاته!" (Missed!) |
| `sfx_decoy_activate` | Decoy deployed | Medium | Distant noise |
| `sfx_decoy_distract` | Enemy distracted | Medium | "ما هذا؟" |

---

## 6. Building & Economy Sounds

| SFX ID | Description | Priority |
|--------|-------------|----------|
| `sfx_build_place` | Building placement | Medium |
| `sfx_build_construction` | Construction loop | Low (ambient) |
| `sfx_build_complete` | Construction complete | High |
| `sfx_build_upgrade` | Upgrade starts | Medium |
| `sfx_build_upgrade_complete` | Upgrade complete | High |
| `sfx_build_destroyed` | Building destroyed | Highest |
| `sfx_economy_gold` | Gold collected | Low |
| `sfx_economy_wood` | Wood chopped | Low |
| `sfx_economy_food` | Food gathered | Low |
| `sfx_economy_trade` | Trade completed | Medium |
| `sfx_recruit_unit` | Unit trained | Medium |

---

## 7. Ambient Sounds

| SFX ID | Description | Category |
|--------|-------------|----------|
| `sfx_ambient_wind_light` | Light wind | Weather |
| `sfx_ambient_wind_strong` | Strong wind | Weather |
| `sfx_ambient_rain_light` | Light rain | Weather |
| `sfx_ambient_rain_heavy` | Heavy rain | Weather |
| `sfx_ambient_thunder` | Thunder rumble | Weather |
| `sfx_ambient_night_crickets` | Night insects | Time |
| `sfx_ambient_forest_birds` | Day birds | Time |
| `sfx_ambient_river` | River flowing | Water |
| `sfx_ambient_ocean` | Ocean waves | Water |
| `sfx_ambient_fire` | Fire crackling | Fire |
| `sfx_ambient_village` | Village activity | Settlement |
| `sfx_ambient_distant_battle` | Distant combat | War |

---

## 8. SFX Technical Specs

| Specification | Value |
|--------------|-------|
| Format | OGG Vorbis |
| Sample Rate | 44.1 kHz |
| Bit Depth | 16-bit |
| Channels | Mono (positional) / Stereo (UI, ambient) |
| Max Polyphony | 32 simultaneous voices |
| Priority Levels | VeryLow / Low / Medium / High / Highest |

---

## 9. Related Files

| File | Content |
|------|---------|
| `02_Music_System.md` | Music system |
| `04_Voiceover_Guide.md` | VO recording guide |
| `05_Dynamic_Audio.md` | Dynamic mixing |
