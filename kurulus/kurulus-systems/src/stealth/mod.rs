use bevy::prelude::*;
use kurulus_core::prelude::*;
use std::collections::HashMap;

pub struct StealthPlugin;

impl Plugin for StealthPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbushConfig::default()).add_systems(
            FixedUpdate,
            (
                // Phase 4a: Pre-detection modifiers (terrain, night, noise)
                terrain_stealth_system.in_set(SimulationSet::VisionCalculation),
                night_stealth_system.in_set(SimulationSet::VisionCalculation),
                noise_calculation_system.in_set(SimulationSet::VisionCalculation),
                // Phase 4b: Vision & Detection
                vision_calculation_system.in_set(SimulationSet::VisionCalculation),
                detection_chain_system.in_set(SimulationSet::StealthDetection),
                ambush_auto_hide_system.in_set(SimulationSet::StealthDetection),
                stealth_detection_system.in_set(SimulationSet::StealthDetection),
                // Phase 4c: Ambush
                ambush_phase_system.in_set(SimulationSet::AmbushTrigger),
                ready_state_enforcement_system.in_set(SimulationSet::AmbushTrigger),
                ambush_trigger_system.in_set(SimulationSet::AmbushTrigger),
                ambush_placement_system.in_set(SimulationSet::AmbushTrigger),
            ),
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TERRAIN STEALTH MODIFIER (ENHANCED)
// ═════════════════════════════════════════════════════════════════════════════

/// Computes and applies terrain stealth modifiers based on `StealthZone` tiles.
fn terrain_stealth_system(
    mut stealth_query: Query<(&WorldPosition, &mut Stealth)>,
    stealth_zones: Query<(&GridPos, &StealthZone)>,
    terrain: Res<TerrainGrid>,
) {
    let zone_map: std::collections::HashMap<IVec2, &StealthZone> =
        stealth_zones.iter().map(|(g, z)| (g.0, z)).collect();

    for (pos, mut stealth) in stealth_query.iter_mut() {
        let cell = pos.grid.0;
        let terrain_mod = terrain.stealth_modifier(cell);

        if let Some(zone) = zone_map.get(&cell) {
            if zone.destroyed {
                stealth.zone_type = None;
                stealth.partial_stealth = false;
                continue;
            }
            stealth.zone_type = Some(zone.zone_type);
            stealth.partial_stealth = zone.is_overcrowded();

            let zone_bonus = zone.zone_type.stealth_bonus();
            let effective_terrain = terrain_mod.max(zone_bonus);
            stealth.level = (stealth.level + effective_terrain).clamp(0.0, 1.0);
        } else {
            stealth.zone_type = None;
            stealth.partial_stealth = false;
            stealth.level = (stealth.level + terrain_mod).clamp(0.0, 1.0);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  NIGHT STEALTH BONUS (ENHANCED)
// ═════════════════════════════════════════════════════════════════════════════

/// Applies graduated night stealth bonus based on day/night cycle.
fn night_stealth_system(
    mut stealth_query: Query<&mut Stealth, (With<Stealth>, Without<Dead>)>,
    game_time: Res<GameTime>,
) {
    let night_mod: f32 = if game_time.is_night() {
        if game_time.hour < 6 || game_time.hour >= 20 {
            0.20
        } else if game_time.hour >= 18 {
            0.10
        } else {
            0.0
        }
    } else if game_time.hour >= 6 && game_time.hour < 8 {
        0.10
    } else {
        0.0
    };

    if night_mod > 0.0 {
        for mut stealth in stealth_query.iter_mut() {
            stealth.level = (stealth.level + night_mod).clamp(0.0, 1.0);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  NOISE CALCULATION (ENHANCED)
// ═════════════════════════════════════════════════════════════════════════════

/// Calculates noise level from movement speed, using archetype-based thresholds.
fn noise_calculation_system(
    mut units: Query<(&Stats, &Unit, &mut NoiseLevel), (With<Unit>, Without<Dead>)>,
) {
    for (stats, unit, mut noise) in units.iter_mut() {
        let max_speed: f32 = match unit.archetype {
            UnitArchetype::Scout => 4.5,
            UnitArchetype::BaitRider => 6.0, // GDD: fastest unit
            UnitArchetype::LightInfantry => 3.5,
            UnitArchetype::Archer => 3.0,
            UnitArchetype::Worker => 2.5,
            UnitArchetype::LightCavalry => 5.5,
            UnitArchetype::HeavyInfantry => 2.5,
            UnitArchetype::HeavyCavalry => 4.0,
            UnitArchetype::SiegeCrew => 2.0,
        };

        let speed_ratio: f32 = if max_speed > 0.0 {
            (stats.move_speed / max_speed).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let is_cavalry = matches!(
            unit.archetype,
            UnitArchetype::LightCavalry | UnitArchetype::HeavyCavalry | UnitArchetype::BaitRider
        );
        let cavalry_base: f32 = if is_cavalry { 0.2 } else { 0.0 };

        if speed_ratio < 0.5 {
            noise.noise = (0.1_f32 + cavalry_base + speed_ratio * 0.2).clamp(0.0, 1.0);
        } else {
            noise.noise = (0.3_f32 + cavalry_base + speed_ratio * 0.4).clamp(0.0, 1.0);
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  VISION CALCULATION (ENHANCED with VisionCone)
// ═════════════════════════════════════════════════════════════════════════════

/// Updates fog of war using vision cones, height bonuses, and terrain blocking.
fn vision_calculation_system(
    units: Query<(
        &WorldPosition,
        &Stats,
        &Unit,
        Option<&HeightLevel>,
        Option<&VisionCone>,
    )>,
    mut fog_tiles: Query<(&GridPos, &mut FogOfWar)>,
    terrain: Res<TerrainGrid>,
) {
    let mut visible_cells: std::collections::HashSet<IVec2> = std::collections::HashSet::new();

    for (pos, stats, unit, height, vision_cone) in units.iter() {
        let is_player_unit = unit.faction == FactionId::Kayi;
        if !is_player_unit {
            continue;
        }

        let height_bonus: f32 = match height {
            Some(HeightLevel::Ground) | None => 1.0,
            Some(HeightLevel::Wall) => 2.0,
            Some(HeightLevel::Tower) => 3.0,
            Some(HeightLevel::Mountain) => 4.0,
        };

        let base_vision = stats.vision_range * height_bonus;
        if base_vision <= 0.0 {
            continue;
        }

        let gx = pos.grid.0.x;
        let gy = pos.grid.0.y;
        let range_i = base_vision.ceil() as i32;

        let cone = vision_cone.map_or_else(|| VisionCone::default(), |c| c.clone());

        for dx in -range_i..=range_i {
            for dy in -range_i..=range_i {
                let d_sq = (dx * dx + dy * dy) as f32;
                if d_sq > base_vision * base_vision {
                    continue;
                }
                let cell = IVec2::new(gx + dx, gy + dy);

                if terrain.blocks_vision(cell) && (dx.abs() > 1 || dy.abs() > 1) {
                    continue;
                }

                let target_vec = Vec2::new(dx as f32, dy as f32);
                let angle: f32 =
                    if cone.facing.length_squared() > 0.0 && target_vec.length_squared() > 0.0 {
                        // angle_to returns a signed angle in radians; convert to degrees absolute
                        target_vec.angle_to(cone.facing).abs().to_degrees()
                    } else {
                        0.0
                    };
                let cone_mult = cone.detection_mult(angle);

                if cone_mult > 0.0 {
                    let effective_range = base_vision * cone_mult;
                    if d_sq <= effective_range * effective_range {
                        visible_cells.insert(cell);
                    }
                }
            }
        }
    }

    for (grid_pos, mut fog) in fog_tiles.iter_mut() {
        if visible_cells.contains(&grid_pos.0) {
            fog.visible = true;
            fog.explored = true;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  DETECTION CHAIN SYSTEM (NEW)
// ═════════════════════════════════════════════════════════════════════════════

/// Manages the Suspicion → Confirmation → Alarm progression for detection.
fn detection_chain_system(
    mut commands: Commands,
    mut chains: Query<(Entity, &mut DetectionChain)>,
    stealth_query: Query<(Entity, &WorldPosition, &Stealth), (With<Stealth>, Without<Dead>)>,
    detectors: Query<
        (
            Entity,
            &WorldPosition,
            &Detection,
            Option<&VisionCone>,
            Option<&ObserverState>,
        ),
        (With<Unit>, Without<Stealth>, Without<Dead>),
    >,
    time: Res<Time>,
    config: Res<AmbushConfig>,
    mut suspicion_events: EventWriter<UnitSuspicionRaised>,
    mut confirm_events: EventWriter<UnitConfirmed>,
    mut alarm_events: EventWriter<AlarmRaised>,
    mut reveal_events: EventWriter<StealthRevealed>,
) {
    let dt = time.delta_secs();
    let detector_list: Vec<(
        Entity,
        &WorldPosition,
        &Detection,
        Option<&VisionCone>,
        Option<&ObserverState>,
    )> = detectors
        .iter()
        .filter(|(_, _, det, _, _)| det.active)
        .collect();

    for (chain_e, mut chain) in chains.iter_mut() {
        if chain.alarm_raised || chain.alarm_prevented {
            continue;
        }

        let Some(tracked) = chain.tracked_entity else {
            continue;
        };

        let still_detected = detector_list.iter().any(|(_det_e, det_pos, det, _, _)| {
            // Allow self-detection — the chain owner uses its own detection to track
            let Ok(stealth_pos) = stealth_query.get(tracked) else {
                return false;
            };
            let dist = stealth_pos.1.world_2d.0.distance(det_pos.world_2d.0);
            let effective_radius = det.radius + det.height_bonus;
            dist <= effective_radius * 1.5
        });

        if !still_detected {
            chain.suspicion_level = (chain.suspicion_level - dt * 2.0).max(0.0);
            chain.timer = 0.0;
            if chain.suspicion_level <= 0.0 {
                commands.entity(chain_e).remove::<DetectionChain>();
            }
            continue;
        }

        chain.timer += dt;

        if chain.timer >= chain.confirmation_time {
            chain.suspicion_level = 1.0;
            if let Ok((_, _, _stealth)) = stealth_query.get(tracked) {
                // Confirmed detection — stealth state will be updated by
                // stealth_detection_system which runs in the same set
            }

            // Use direct entity access via commands or separate query
            confirm_events.send(UnitConfirmed {
                observer: chain_e,
                target: tracked,
            });
            reveal_events.send(StealthRevealed {
                entity: tracked,
                revealer: chain_e,
            });

            chain.alarm_raised = true;
            if let Ok((_, det_pos, _, _, _)) = detectors.get(chain_e) {
                alarm_events.send(AlarmRaised {
                    raiser: chain_e,
                    position: det_pos.world_2d.0,
                    radius: config.detection.alarm_radius,
                    duration: config.detection.alarm_duration,
                });
            }

            commands.entity(chain_e).remove::<DetectionChain>();
        } else {
            let progress = chain.timer / chain.confirmation_time;
            chain.suspicion_level = 0.3 + progress * 0.7;
            suspicion_events.send(UnitSuspicionRaised {
                observer: chain_e,
                suspected: tracked,
                suspicion_level: chain.suspicion_level,
            });
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH AUTO-HIDE (ENHANCED)
// ═════════════════════════════════════════════════════════════════════════════

/// Units with `Stealth` inside an `AmbushZone` are automatically hidden.
/// Enhanced with zone entry timer (0.5s → 1.5s → 2.0s progression).
fn ambush_auto_hide_system(
    mut stealth_query: Query<(
        Entity,
        &WorldPosition,
        &mut Stealth,
        Option<&NoiseLevel>,
        Option<&UnitVisibility>,
    )>,
    ambush_zones: Query<(Entity, &AmbushZone, &WorldPosition)>,
    terrain: Res<TerrainGrid>,
    game_time: Res<GameTime>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let zone_list: Vec<(Entity, &AmbushZone, &WorldPosition)> = ambush_zones
        .iter()
        .filter(|(_, zone, _)| zone.remaining > 0.0)
        .collect();

    for (_entity, pos, mut stealth, noise_opt, vis_opt) in stealth_query.iter_mut() {
        let terrain_mod = terrain.stealth_modifier(pos.grid.0);
        let night_mod: f32 = if game_time.is_night() { 0.20 } else { 0.0 };
        let noise_factor = noise_opt.map(|n| n.noise * 0.5).unwrap_or(0.0);
        let size_factor = vis_opt.map(|v| v.footprint * 0.3).unwrap_or(0.0);
        let effective_level =
            (stealth.level + terrain_mod + night_mod - noise_factor - size_factor).clamp(0.0, 1.0);

        stealth.level = effective_level;

        let mut inside_zone = false;
        for (zone_e, zone, zone_pos) in &zone_list {
            let dist = pos.world_2d.0.distance(zone_pos.world_2d.0);
            if dist <= zone.radius {
                inside_zone = true;
                stealth.ambush_zone = Some(*zone_e);
                break;
            }
        }

        if inside_zone {
            // GDD entry progression: enter (0→0.5s) → settle (0.5→1.5s) → hidden (1.5→2.0s)
            if stealth.entry_timer < 2.0 {
                stealth.entry_timer += dt;
            }

            // Fully hidden once timer reaches 2.0s
            if stealth.entry_timer >= 2.0 && !stealth.is_hidden {
                stealth.is_hidden = true;
                stealth.hidden_timer = zone_list
                    .first()
                    .map(|(_, z, _)| z.duration)
                    .unwrap_or(30.0);
            }
        } else if stealth.ambush_zone.is_some() {
            // Left the zone — start exit decay from current entry_timer value
            let _prev_zone = stealth.ambush_zone.take();
            if stealth.is_hidden {
                stealth.entry_timer -= dt * 2.0; // exit twice as fast as entry
                if stealth.entry_timer <= -0.5 {
                    stealth.is_hidden = false;
                    stealth.hidden_timer = 0.0;
                }
            } else {
                stealth.entry_timer = 0.0;
            }
        } else if stealth.is_hidden && stealth.zone_type.is_none() {
            // Out in the open — unhide after brief delay
            stealth.entry_timer -= dt * 2.0;
            if stealth.entry_timer <= -0.5 {
                stealth.is_hidden = false;
                stealth.hidden_timer = 0.0;
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STEALTH DETECTION (REWRITTEN with VisionCone, ObserverState)
// ═════════════════════════════════════════════════════════════════════════════

/// Enhanced detection with vision cones, observer factors, target factors.
fn stealth_detection_system(
    grid: Res<SpatialGrid>,
    mut stealth_query: Query<
        (
            Entity,
            &WorldPosition,
            &mut Stealth,
            Option<&NoiseLevel>,
            Option<&UnitVisibility>,
        ),
        (With<Stealth>, Without<Dead>),
    >,
    detectors: Query<
        (
            Entity,
            &WorldPosition,
            &Detection,
            Option<&VisionCone>,
            Option<&ObserverState>,
        ),
        (With<Unit>, Without<Stealth>, Without<Dead>),
    >,
    terrain: Res<TerrainGrid>,
    game_time: Res<GameTime>,
    _config: Res<AmbushConfig>,
    mut reveal_events: EventWriter<StealthRevealed>,
) {
    // Snapshot active detector data into a HashMap for O(1) lookups.
    // Store a cloned VisionCone so we can use detection_mult() faithfully.
    let detector_map: HashMap<
        Entity,
        (
            GridPos,
            f32,        // detection radius + height bonus (effective base radius)
            Vec2,       // world_2d position
            VisionCone, // cloned cone for accurate detection_mult()
            f32,        // observer multiplier
        ),
    > = detectors
        .iter()
        .filter(|(_, _, det, _, _)| det.active)
        .map(|(e, pos, det, cone_opt, obs_opt)| {
            let cone = cone_opt.cloned().unwrap_or_default();
            let observer_factor = obs_opt
                .map(|obs| obs.effective_detection_mult())
                .unwrap_or(1.0);
            (
                e,
                (
                    pos.grid,
                    det.radius + det.height_bonus,
                    pos.world_2d.0,
                    cone,
                    observer_factor,
                ),
            )
        })
        .collect();

    for (stealth_e, stealth_pos, mut stealth, noise_opt, vis_opt) in stealth_query.iter_mut() {
        if !stealth.is_hidden {
            continue;
        }

        let terrain_mod = terrain.stealth_modifier(stealth_pos.grid.0);
        let zone_bonus = stealth.zone_type.map(|z| z.stealth_bonus()).unwrap_or(0.0);
        let night_mod: f32 = if game_time.is_night() { 0.20 } else { 0.0 };
        let noise_factor = noise_opt.map(|n| n.noise * 0.3).unwrap_or(0.0);
        let size_factor = vis_opt.map(|v| v.footprint * 0.3).unwrap_or(0.0);

        let best_terrain = terrain_mod.max(zone_bonus);
        let effective_stealth =
            (stealth.level + best_terrain + night_mod - noise_factor - size_factor).clamp(0.0, 1.0);

        let target_factor = 1.0 - effective_stealth;
        let st_pos_2d = stealth_pos.world_2d.0;

        // Use SpatialGrid to find nearby entities, then check only those that
        // are active detectors
        let nearby = grid.entities_in_range(stealth_pos.grid, 50.0); // generous outer radius

        for &candidate in &nearby {
            if candidate == stealth_e {
                continue;
            }
            let Some(&(det_grid_pos, det_base_radius, det_pos_2d, ref cone, observer_factor)) =
                detector_map.get(&candidate)
            else {
                continue;
            };

            let d_grid = stealth_pos.grid.0.distance_squared(det_grid_pos.0) as f32;
            if d_grid > 2500.0 {
                // 50² — matches outer radius above
                continue;
            }

            let dist = st_pos_2d.distance(det_pos_2d);

            let cone_mult = if cone.facing.length_squared() > 0.0 {
                let to_target = st_pos_2d - det_pos_2d;
                let angle = if to_target.length_squared() > 0.0 {
                    to_target.angle_to(cone.facing).abs().to_degrees()
                } else {
                    0.0
                };
                cone.detection_mult(angle)
            } else {
                1.0
            };

            if cone_mult <= 0.0 {
                continue;
            }

            let env_factor: f32 = if game_time.is_night() { 0.6 } else { 1.0 };

            let effective_radius =
                det_base_radius * observer_factor * target_factor * env_factor * cone_mult;

            if effective_radius <= 0.0 {
                continue;
            }

            let detection_ratio = dist / effective_radius;

            if detection_ratio < 0.5 {
                stealth.is_hidden = false;
                stealth.hidden_timer = 0.0;
                reveal_events.send(StealthRevealed {
                    entity: stealth_e,
                    revealer: candidate,
                });
                break;
            } else if detection_ratio < 1.0 {
                let detection_chance = 1.0 - (detection_ratio * 0.5) - (effective_stealth * 0.3);
                if detection_chance > 0.5 {
                    stealth.is_hidden = false;
                    stealth.hidden_timer = 0.0;
                    reveal_events.send(StealthRevealed {
                        entity: stealth_e,
                        revealer: candidate,
                    });
                    break;
                }
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH TRIGGER (enhanced with morale panic)
// ═════════════════════════════════════════════════════════════════════════════

/// When a hidden unit attacks from inside an `AmbushZone`, apply bonus
/// damage and trigger morale panic on the target.
fn ambush_trigger_system(
    mut commands: Commands,
    mut attack_events: EventReader<AttackTriggered>,
    positions: Query<&WorldPosition>,
    ambush_zones: Query<(Entity, &AmbushZone, &WorldPosition)>,
    stealth_units: Query<&Stealth>,
    weapons: Query<&MeleeWeapon>,
    mut damage_events: EventWriter<DamageDealt>,
    mut ambush_events: EventWriter<AmbushTriggered>,
    mut morale_events: EventWriter<MoraleCheckTriggered>,
    config: Res<AmbushConfig>,
) {
    for attack in attack_events.read() {
        let Ok(stealth) = stealth_units.get(attack.attacker) else {
            continue;
        };
        if !stealth.is_hidden {
            continue;
        }

        let Ok(attacker_pos) = positions.get(attack.attacker) else {
            continue;
        };

        let mut bonus = AmbushBonus::default();
        for (zone_e, zone, zone_pos) in ambush_zones.iter() {
            let dist = attacker_pos.world_2d.0.distance(zone_pos.world_2d.0);
            if dist <= zone.radius {
                bonus.damage_multiplier = zone.bonus_damage.max(1.0);
                bonus.morale_bonus = zone.bonus_morale.max(0.0);
                bonus.ambush_zone = Some(zone_e);
                break;
            }
        }

        if let Some(zone_entity) = bonus.ambush_zone {
            let weapon_damage = weapons
                .get(attack.attacker)
                .map(|w| w.damage)
                .unwrap_or(10.0);

            let bonus_damage = weapon_damage * (bonus.damage_multiplier - 1.0);

            if bonus_damage > 0.0 {
                damage_events.send(DamageDealt {
                    attacker: attack.attacker,
                    target: attack.target,
                    amount: bonus_damage,
                    damage_type: DamageType::Pierce,
                    ..Default::default()
                });
            }

            ambush_events.send(AmbushTriggered {
                ambusher: attack.attacker,
                target: attack.target,
                damage_bonus: bonus.damage_multiplier,
                zone: zone_entity,
            });

            morale_events.send(MoraleCheckTriggered {
                entity: attack.target,
                trigger: MoraleCheckTrigger::AmbushReceived,
                panic_multiplier: config.effects.panic_morale_check_mult,
            });

            // GDD: Formation break (3s after ambush)
            commands.entity(attack.target).insert(FormationBreak {
                remaining: config.effects.formation_break_duration,
                total_duration: config.effects.formation_break_duration,
                panic: false,
            });

            // GDD: Shock (2s stun — target cannot attack/respond)
            commands.entity(attack.target).insert(Shock {
                remaining: config.effects.shock_duration,
                total_duration: config.effects.shock_duration,
            });
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH PLACEMENT (enhanced with terrain validation)
// ═════════════════════════════════════════════════════════════════════════════

/// Handles `PlaceAmbushOrder` events with terrain validation.
fn ambush_placement_system(
    mut placement_events: EventReader<PlaceAmbushOrder>,
    mut commands: Commands,
    mut stockpile: ResMut<ResourceStockpile>,
    terrain: Res<TerrainGrid>,
    stealth_zones: Query<(&GridPos, &StealthZone)>,
    config: Res<AmbushConfig>,
) {
    let zone_map: std::collections::HashMap<IVec2, &StealthZone> =
        stealth_zones.iter().map(|(g, z)| (g.0, z)).collect();

    for order in placement_events.read() {
        let cost = config.effects.placement_cost_gold;

        if stockpile.gold < cost {
            continue;
        }

        let cell = order.position.0;
        let terrain_type = terrain.get(cell);

        let valid_terrain = matches!(
            terrain_type,
            TerrainType::Forest | TerrainType::Mountain | TerrainType::Building
        );
        let has_stealth_zone = zone_map.contains_key(&cell);

        if !valid_terrain && !has_stealth_zone {
            continue;
        }

        stockpile.gold -= cost;

        let pos = WorldPosition::new(order.position.0, 1.0);
        commands.spawn((
            AmbushZone {
                owner: order.entity,
                radius: config.effects.default_zone_radius,
                bonus_damage: config.effects.default_bonus_damage,
                bonus_morale: config.effects.default_bonus_morale,
                duration: config.effects.default_zone_duration,
                remaining: config.effects.default_zone_duration,
            },
            pos,
            AmbushMarker {
                position: order.position,
                placed_by: order.entity,
            },
        ));
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH PHASE SYSTEM (NEW — Phase 1)
// ═════════════════════════════════════════════════════════════════════════════

/// Drives the `AmbushState` state machine: Idle → Setup → Ready → Triggered
/// → Executing → Cooldown → Idle. Emits `AmbushPhaseChanged` events on every
/// transition. Auto-triggers for `AutoFirstContact` trigger type when an enemy
/// enters the ambush zone's radius.
fn ambush_phase_system(
    mut query: Query<(
        Entity,
        &Unit,
        &mut AmbushState,
        &mut Stealth,
        &WorldPosition,
    )>,
    ambush_zones: Query<(Entity, &AmbushZone, &WorldPosition)>,
    enemies: Query<(&Unit, &WorldPosition), (Without<AmbushState>, Without<Dead>)>,
    config: Res<AmbushConfig>,
    time: Res<Time>,
    mut phase_events: EventWriter<AmbushPhaseChanged>,
) {
    let dt = time.delta_secs();

    for (entity, unit, mut state, mut stealth, _pos) in query.iter_mut() {
        let old_phase = state.phase;

        match state.phase {
            AmbushPhase::Idle => {
                // Transition to Setup when placed in an ambush zone with active stealth
                if stealth.ambush_zone.is_some() && stealth.entry_timer > 0.0 {
                    state.phase = AmbushPhase::Setup;
                }
            }
            AmbushPhase::Setup => {
                // Wait for the 2.0s entry timer to complete, then become ready
                if stealth.entry_timer >= 2.0 && stealth.is_hidden {
                    state.phase = AmbushPhase::Ready;
                    state.ready_since = 0.0;
                    stealth.is_ready = true;
                }
                // If the zone was removed or the unit was revealed, abort setup
                if stealth.ambush_zone.is_none() || !stealth.is_hidden {
                    state.phase = AmbushPhase::Idle;
                }
            }
            AmbushPhase::Ready => {
                state.ready_since += dt;

                // Only auto-trigger for AutoFirstContact type
                let should_trigger = match state.trigger_type {
                    TriggerType::AutoFirstContact => {
                        // Check if any enemy unit is inside the ambush zone radius
                        // Note: zone entity is stored on Stealth.ambush_zone (set by ambush_auto_hide_system)
                        if let Some(zone_e) = stealth.ambush_zone {
                            if let Ok((_, zone_data, zone_pos)) = ambush_zones.get(zone_e) {
                                enemies.iter().any(|(e_unit, e_pos)| {
                                    e_unit.faction != unit.faction
                                        && zone_pos.world_2d.0.distance(e_pos.world_2d.0)
                                            <= zone_data.radius
                                })
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    }
                    TriggerType::Manual | TriggerType::Staged | TriggerType::Solo => false,
                };

                if should_trigger {
                    state.phase = AmbushPhase::Triggered;
                }

                // If revealed, abort ambush
                if !stealth.is_hidden {
                    state.discovered = true;
                    state.phase = AmbushPhase::Idle;
                }
            }
            AmbushPhase::Triggered => {
                // Immediately enter execution phase with bonuses active
                state.bonus_damage_remaining = config.effects.damage_duration;
                state.shock_remaining = config.effects.shock_duration;
                stealth.is_ready = false;
                state.phase = AmbushPhase::Executing;
            }
            AmbushPhase::Executing => {
                // Countdown bonus damage window and shock duration
                state.bonus_damage_remaining -= dt;
                state.shock_remaining -= dt;

                if state.bonus_damage_remaining <= 0.0 {
                    state.phase = AmbushPhase::Cooldown;
                }
            }
            AmbushPhase::Cooldown => {
                // Cooldown timer before re-entering Idle
                state.ready_since += dt;
                if state.ready_since >= 5.0 {
                    state.phase = AmbushPhase::Idle;
                }
            }
        }

        if old_phase != state.phase {
            phase_events.send(AmbushPhaseChanged {
                entity,
                old_phase,
                new_phase: state.phase,
            });
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  READY STATE ENFORCEMENT SYSTEM (NEW — Phase 1)
// ═════════════════════════════════════════════════════════════════════════════

/// Prevents auto-attack for units in the Ready ambush state by removing their
/// `AttackTarget` component. This enforces the GDD rule that ambushers in
/// Ready state suppress all attacks until the trigger condition is met.
fn ready_state_enforcement_system(
    mut commands: Commands,
    ready_units: Query<(Entity, &Stealth, Option<&AttackTarget>)>,
) {
    for (entity, stealth, attack_target) in ready_units.iter() {
        if stealth.is_ready && attack_target.is_some() {
            commands.entity(entity).remove::<AttackTarget>();
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HELPERS
// ═════════════════════════════════════════════════════════════════════════════

struct AmbushBonus {
    damage_multiplier: f32,
    morale_bonus: f32,
    ambush_zone: Option<Entity>,
}

impl Default for AmbushBonus {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            morale_bonus: 0.0,
            ambush_zone: None,
        }
    }
}
