# Events Catalog — Complete Event Type Reference

> **Description:** Comprehensive catalog of every game event type in *Kuruluş*. Each entry defines the event type string, category, payload fields, and which systems emit/listen.

---

## 1. Input Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Input_KeyDown` | Input | `{key_code: int, modifiers: string[]}` | Input Manager | Console, Debug |
| `Input_KeyUp` | Input | `{key_code: int, modifiers: string[]}` | Input Manager | (rarely listened) |
| `Input_MouseClick` | Input | `{button: int, position: Vector2, world_position: Vector3}` | Input Manager | Selection, Camera |
| `Input_MouseRelease` | Input | `{button: int, position: Vector2}` | Input Manager | Command System |
| `Input_MouseDrag` | Input | `{start_pos: Vector2, current_pos: Vector2, button: int}` | Input Manager | Selection Box, Camera Pan |
| `Input_ScrollWheel` | Input | `{delta: float, position: Vector2}` | Input Manager | Camera Zoom |
| `Input_Command` | Input | `{command_type: string, targets: string[], position: Vector3?, modifiers: string[]}` | Input Manager | Command System, Unit AI |
| `Input_SelectionChanged` | Input | `{selected_ids: string[], previous_ids: string[]}` | Selection Manager | UI Panel, Command Bar |
| `Input_HotkeyPressed` | Input | `{action: string, pressed: bool}` | Input Manager | All systems |

---

## 2. Gameplay Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Game_UnitSpawned` | Gameplay | `{unit_id: string, instance_id: string, position: Vector3, owner: string}` | Production System | UI, Audio, Stats |
| `Game_UnitKilled` | Gameplay | `{unit_id: string, instance_id: string, killer_id: string, position: Vector3, damage_type: string}` | Combat System | UI, Audio, AI, Stats, Objectives |
| `Game_UnitDamaged` | Gameplay | `{instance_id: string, damage: float, new_hp: float, source_id: string}` | Combat System | UI (healthbar), Audio |
| `Game_UnitHealed` | Gameplay | `{instance_id: string, amount: float, new_hp: float, healer_id: string}` | Ability System | UI, Audio |
| `Game_UnitMoved` | Gameplay | `{instance_id: string, from: Vector3, to: Vector3, speed: float}` | Movement System | (internal) |
| `Game_UnitEnteredZone` | Gameplay | `{instance_id: string, zone_id: string, unit_type: string}` | Spatial System | Stealth, Ambush, Triggers |
| `Game_UnitExitedZone` | Gameplay | `{instance_id: string, zone_id: string}` | Spatial System | Stealth, Ambush, Triggers |
| `Game_UnitMoraleChanged` | Gameplay | `{instance_id: string, old_value: float, new_value: float, reason: string}` | Morale System | UI, AI, Audio |
| `Game_UnitMoraleBroken` | Gameplay | `{instance_id: string, unit_type: string, position: Vector3}` | Morale System | AI (retreat), UI, Audio |
| `Game_UnitLeveledUp` | Gameplay | `{instance_id: string, hero_id: string, new_level: int, new_ability_unlocked: string?}` | Progression System | UI, Audio |
| `Game_BuildingCompleted` | Gameplay | `{building_id: string, instance_id: string, position: Vector3, tier: int}` | Construction System | UI, Audio, Economy, AI |
| `Game_BuildingDestroyed` | Gameplay | `{building_id: string, instance_id: string, position: Vector3, destroyer_id: string}` | Combat/Destruction | UI, Audio, AI, Objectives |
| `Game_BuildingUpgraded` | Gameplay | `{building_id: string, instance_id: string, old_tier: int, new_tier: int}` | Construction System | UI, Audio |
| `Game_BuildingGarrisoned` | Gameplay | `{building_id: string, instance_id: string, garrison_count: int}` | Unit System | UI |
| `Game_ResourceChanged` | Gameplay | `{resource_type: string, old_amount: int, new_amount: int, change: int, reason: string}` | Economy System | UI, Objectives |
| `Game_ResourceCapacityChanged` | Gameplay | `{resource_type: string, old_cap: int, new_cap: int, reason: string}` | Economy System | UI |
| `Game_ResearchCompleted` | Gameplay | `{tech_id: string, tech_name_key: string, cost: int}` | Research System | UI, Audio |
| `Game_PopulationChanged` | Gameplay | `{old_value: int, new_value: int, max: int}` | Economy System | UI |

---

## 3. Combat Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Combat_DamageDealt` | Combat | `{attacker_id: string, target_id: string, damage_amount: float, damage_type: string, is_critical: bool}` | Combat System | UI, Audio, Stats |
| `Combat_DamageAbsorbed` | Combat | `{target_id: string, absorbed: float, armor_value: float}` | Combat System | UI (floating text) |
| `Combat_HitLanded` | Combat | `{attacker_id: string, target_id: string, hit_type: string, position: Vector3}` | Combat System | Audio, VFX |
| `Combat_HitMissed` | Combat | `{attacker_id: string, target_id: string, miss_reason: string}` | Combat System | UI (floating text) |
| `Combat_CriticalHit` | Combat | `{attacker_id: string, target_id: string, multiplier: float}` | Combat System | UI, Audio, VFX |
| `Combat_StatusEffectApplied` | Combat | `{target_id: string, effect_id: string, duration: float, source_id: string}` | Ability System | UI, VFX |
| `Combat_StatusEffectExpired` | Combat | `{target_id: string, effect_id: string}` | Ability System | UI, VFX |
| `Combat_ArmorPierced` | Combat | `{target_id: string, armor_ignored: float}` | Combat System | UI, Audio |
| `Combat_RangedShotFired` | Combat | `{shooter_id: string, projectile_id: string, target_id: string, launch_position: Vector3}` | Ranged System | VFX, Audio |
| `Combat_SiegeHit` | Combat | `{siege_unit_id: string, target_building_id: string, damage: float, bonus_damage: float}` | Siege System | UI, Audio, VFX |
| `Combat_MeleeEngage` | Combat | `{attacker_id: string, defender_id: string, position: Vector3}` | Melee System | Audio, Animations |
| `Combat_MeleeDisengage` | Combat | `{attacker_id: string, defender_id: string, reason: string}` | Melee System | AI |
| `Combat_FriendlyFire` | Combat | `{attacker_id: string, victim_id: string, damage: float, is_penalty: bool}` | Combat System | UI (warning), Morale System |
| `Combat_RetreatStarted` | Combat | `{unit_id: string, from_position: Vector3, to_position: Vector3}` | AI / Morale | AI, Camera |

---

## 4. Ambush & Stealth Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Stealth_Entered` | Stealth | `{unit_id: string, stealth_zone_id: string, detection_rating: float}` | Stealth System | UI, Audio, VFX |
| `Stealth_Exited` | Stealth | `{unit_id: string, reason: string}` | Stealth System | UI, Audio, VFX |
| `Stealth_DetectionCheck` | Stealth | `{unit_id: string, detector_id: string, distance: float, detection_probability: float}` | Detection System | (internal) |
| `Stealth_UnitDetected` | Stealth | `{unit_id: string, detector_id: string, detection_method: string}` | Detection System | UI, AI, Audio |
| `Stealth_DetectionWarning` | Stealth | `{unit_id: string, threat_level: float, nearest_detector_id: string}` | Detection System | UI (stealth indicator) |
| `Ambush_Triggered` | Ambush | `{ambush_id: string, attacker_ids: string[], victim_ids: string[], position: Vector3}` | Ambush System | Combat, UI, Audio, Camera |
| `Ambush_Prepared` | Ambush | `{ambush_id: string, position: Vector3, radius: float, participating_units: string[]}` | Ambush System | UI, Stealth |
| `Ambush_Evaded` | Ambush | `{ambush_id: string, evading_unit_id: string, detection_method: string}` | Ambush System | UI, AI |
| `Ambush_DecoyActivated` | Ambush | `{decoy_id: string, position: Vector3, decoy_type: string, target_faction: string}` | Decoy System | AI, Audio |
| `Ambush_DecoyExpired` | Ambush | `{decoy_id: string, reason: string}` | Decoy System | AI |
| `Ambush_HitAndRun_Started` | Ambush | `{unit_ids: string[], target_id: string, position: Vector3}` | Hit & Run | AI, Camera |
| `Ambush_HitAndRun_Completed` | Ambush | `{unit_ids: string[], damage_dealt: float, casualties: int, evaded: bool}` | Hit & Run | UI (report), Stats |

---

## 5. AI Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `AI_StateChanged` | AI | `{entity_id: string, old_state: string, new_state: string, reason: string}` | AI System | Debug, Analytics |
| `AI_TargetAcquired` | AI | `{entity_id: string, target_id: string, target_type: string, priority: float}` | AI System | Combat, Audio |
| `AI_TargetLost` | AI | `{entity_id: string, target_id: string, reason: string}` | AI System | AI |
| `AI_ThreatEvaluated` | AI | `{entity_id: string, top_threat_id: string, threat_score: float, all_threats: string[]}` | AI System | (internal) |
| `AI_PlanGenerated` | AI | `{entity_id: string, plan_type: string, steps: string[], estimated_cost: float}` | AI Planner | Debug |
| `AI_ThreatDetected` | AI | `{entity_id: string, threat_id: string, threat_type: string, distance: float, position: Vector3}` | AI Sensor | AI, UI (minimap ping) |
| `AI_AmbushSuspected` | AI | `{entity_id: string, suspicion_level: float, zone_id: string}` | AI Sensor | AI (evasive behavior) |
| `AI_PatrolRouteComplete` | AI | `{entity_id: string, route_id: string}` | AI System | AI (re-route) |

---

## 6. UI Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `UI_SelectionChanged` | UI | `{selected_entities: string[], selection_count: int, primary_type: string}` | Selection Manager | Panel, Command Bar, Info |
| `UI_PanelOpened` | UI | `{panel_id: string, context_data: object}` | UI System | Analytics, Audio |
| `UI_PanelClosed` | UI | `{panel_id: string}` | UI System | Game State |
| `UI_TooltipRequested` | UI | `{target_id: string, target_type: string, screen_position: Vector2}` | UI System | Tooltip System |
| `UI_TooltipDismissed` | UI | `{target_id: string}` | UI System | Tooltip System |
| `UI_NotificationQueued` | UI | `{notification_id: string, severity: string, message_key: string, duration: float}` | Any System | Notification Manager |
| `UI_NotificationDismissed` | UI | `{notification_id: string}` | UI System | (cleanup) |
| `UI_MiniMapPing` | UI | `{position: Vector3, ping_type: string, source: string}` | Any System | MiniMap, Audio |
| `UI_DialogLineDisplayed` | UI | `{character_id: string, line_key: string, duration: float}` | Dialogue System | Subtitles, Audio |
| `UI_DialogLineCompleted` | UI | `{character_id: string, line_key: string}` | Dialogue System | (flow control) |

---

## 7. Audio Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Audio_PlaySFX` | Audio | `{sfx_id: string, position: Vector3?, volume_mult: float, pitch_mult: float}` | Any System | Audio Manager |
| `Audio_PlayVoiceover` | Audio | `{vo_id: string, character_id: string, subtitle_key: string, priority: int}` | Dialogue System | Audio Manager, Subtitles |
| `Audio_PlayMusic` | Audio | `{music_id: string, fade_time: float, loop: bool}` | Game State / Triggers | Audio Manager |
| `Audio_StopMusic` | Audio | `{fade_time: float}` | Game State / Triggers | Audio Manager |
| `Audio_SetMusicIntensity` | Audio | `{intensity_level: int (0-3), blend_time: float}` | Combat / Stealth | Audio Manager |
| `Audio_PlayAmbient` | Audio | `{ambient_id: string, position: Vector3?, blend_time: float}` | Zone System | Audio Manager |
| `Audio_SetRTPC` | Audio | `{rtpc_name: string, value: float}` | Any System | Audio Manager (Wwise parameter) |

---

## 8. System Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `System_GameStarted` | System | `{save_file: string?, is_new_game: bool}` | Game Manager | All systems (init) |
| `System_GamePaused` | System | `{reason: string}` | Game Manager | Simulation, Audio |
| `System_GameResumed` | System | `{}` | Game Manager | Simulation, Audio |
| `System_GameSaved` | System | `{slot_index: int, success: bool, save_size_bytes: int}` | Save System | UI |
| `System_GameLoaded` | System | `{slot_index: int, mission_id: string, success: bool}` | Save System | All systems (rebuild) |
| `System_SceneTransition` | System | `{from_scene: string, to_scene: string, transition_type: string}` | Scene Manager | All systems |
| `System_DataReloaded` | System | `{data_sources: string[], entries_modified: int}` | Data Pipeline | UI (refresh) |
| `System_ApplicationQuit` | System | `{save_before_quit: bool}` | Application | Save System |
| `System_Error` | System | `{error_code: string, message: string, severity: string}` | Any System | UI (error dialog), Log |
| `System_AchievementUnlocked` | System | `{achievement_id: string, display_name_key: string}` | Achievement System | UI, Audio, Stats |

---

## 9. Objective/Mission Events

| Event Type | Category | Payload | Emitter | Listeners |
|------------|----------|---------|---------|-----------|
| `Mission_ObjectiveUpdated` | Mission | `{objective_id: string, old_state: string, new_state: string}` | Objective System | UI, Audio |
| `Mission_ObjectiveCompleted` | Mission | `{objective_id: string, display_text_key: string}` | Objective System | UI, Audio |
| `Mission_ObjectiveFailed` | Mission | `{objective_id: string, reason_key: string}` | Objective System | UI, Audio |
| `Mission_AllObjectivesComplete` | Mission | `{mission_id: string, time_elapsed: float, casualties: int}` | Objective System | Victory Sequence |
| `Mission_DefeatConditionMet` | Mission | `{reason_key: string}` | Objective System | Defeat Sequence |
| `Mission_CheckpointReached` | Mission | `{checkpoint_id: string, description_key: string}` | Mission System | Save, UI |

---

## 10. Event Statistics

| Metric | Value |
|--------|-------|
| Total distinct event types | ~95 |
| Average payload size | ~48 bytes |
| Most frequently emitted event | `Game_UnitMoved` (every frame per moving entity) |
| Most listened-to event | `Game_UnitKilled` (~8 listeners) |
| Largest payload | `AI_PlanGenerated` |
| Critical path events (must be fast) | `Combat_DamageDealt`, `Input_*` |

---

## 11. Related Files

| File | Description |
|------|-------------|
| `01_System_Overview.md` | Event bus architecture |
| `03_Trigger_Conditions.md` | Rule-based trigger system |
| `04_Event_Propagation.md` | Priority, chaining, deferred events |
