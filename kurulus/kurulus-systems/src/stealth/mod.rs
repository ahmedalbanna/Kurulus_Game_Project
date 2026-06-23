use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct StealthPlugin;

impl Plugin for StealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                // Phase 4a: Pre-detection modifiers (terrain, night, noise)
                terrain_stealth_system.in_set(SimulationSet::VisionCalculation),
                night_stealth_system.in_set(SimulationSet::VisionCalculation),
                noise_calculation_system.in_set(SimulationSet::VisionCalculation),
                // Phase 4b: Vision & Detection
                vision_calculation_system.in_set(SimulationSet::VisionCalculation),
                ambush_auto_hide_system.in_set(SimulationSet::StealthDetection),
                stealth_detection_system.in_set(SimulationSet::StealthDetection),
                // Phase 4c: Ambush
                ambush_trigger_system.in_set(SimulationSet::AmbushTrigger),
                ambush_placement_system.in_set(SimulationSet::AmbushTrigger),
            ),
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TERRAIN STEALTH MODIFIER
// ═════════════════════════════════════════════════════════════════════════════

/// The effective stealth level (base + terrain + night − noise − size) is
/// computed inside `ambush_auto_hide_system` and `stealth_detection_system`.
/// This system exists as a hook for future visual feedback (e.g. a UI icon
/// showing the current terrain bonus) and is deliberately a no-op for now.
fn terrain_stealth_system(
    _stealth_query: Query<(&WorldPosition, &Stealth), (With<Stealth>, Without<Dead>)>,
    _terrain: Res<TerrainGrid>,
) {
}

// ═════════════════════════════════════════════════════════════════════════════
//  NIGHT STEALTH BONUS
// ═════════════════════════════════════════════════════════════════════════════

/// The night stealth bonus (+0.20) is applied inside
/// `ambush_auto_hide_system` and `stealth_detection_system`.
/// This system exists as a hook for future visual effects (e.g. dimming the
/// scene overlay) and is deliberately a no-op for now.
fn night_stealth_system(
    _stealth_query: Query<&Stealth, (With<Stealth>, Without<Dead>)>,
    _game_time: Res<GameTime>,
) {
}

// ═════════════════════════════════════════════════════════════════════════════
//  NOISE CALCULATION
// ═════════════════════════════════════════════════════════════════════════════

/// Calculates noise level from movement speed.
/// Units with higher `move_speed` generate more noise.
/// Noise is stored on the `NoiseLevel` component.
fn noise_calculation_system(
    mut units: Query<(&Stats, &mut NoiseLevel), (With<Unit>, Without<Dead>)>,
) {
    for (stats, mut noise) in units.iter_mut() {
        // Noise scales linearly with move speed: 0 at speed 0, 1.0 at speed 10+
        noise.noise = (stats.move_speed / 10.0).clamp(0.0, 1.0);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  VISION CALCULATION (enhanced with terrain blocking)
// ═════════════════════════════════════════════════════════════════════════════

/// Updates `FogOfWar` and determines which entities are visible to each
/// faction based on unit vision ranges, height bonuses, and terrain blocking.
fn vision_calculation_system(
    units: Query<(&WorldPosition, &Stats, &Unit, Option<&HeightLevel>)>,
    mut fog_tiles: Query<(&GridPos, &mut FogOfWar)>,
    terrain: Res<TerrainGrid>,
) {
    let mut visible_cells: std::collections::HashSet<IVec2> = std::collections::HashSet::new();

    for (pos, stats, unit, height) in units.iter() {
        let vision = if unit.faction == FactionId::Kayi {
            let height_bonus = match height {
                Some(HeightLevel::Ground) | None => 1.0,
                Some(HeightLevel::Wall) => 2.0,
                Some(HeightLevel::Tower) => 3.0,
                Some(HeightLevel::Mountain) => 4.0,
            };
            stats.vision_range * height_bonus
        } else {
            0.0
        };

        if vision <= 0.0 {
            continue;
        }

        let gx = pos.grid.0.x;
        let gy = pos.grid.0.y;
        let range_i = vision.ceil() as i32;
        for dx in -range_i..=range_i {
            for dy in -range_i..=range_i {
                let d_sq = dx * dx + dy * dy;
                if (d_sq as f32) > vision * vision {
                    continue;
                }
                let cell = IVec2::new(gx + dx, gy + dy);

                // Terrain vision blocking: if mountain or building blocks the ray,
                // don't mark cells behind it (simplified: check cell itself)
                if terrain.blocks_vision(cell) && (dx.abs() > 1 || dy.abs() > 1) {
                    continue;
                }

                visible_cells.insert(cell);
            }
        }
    }

    for (grid_pos, mut fog) in fog_tiles.iter_mut() {
        let is_visible = visible_cells.contains(&grid_pos.0);
        if is_visible {
            fog.visible = true;
            fog.explored = true;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH AUTO-HIDE
// ═════════════════════════════════════════════════════════════════════════════

/// Units with `Stealth` inside an `AmbushZone` are automatically hidden.
/// Units outside any zone or too far from all zones are revealed.
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
) {
    let zone_list: Vec<(Entity, &AmbushZone, &WorldPosition)> = ambush_zones
        .iter()
        .filter(|(_, zone, _)| zone.remaining > 0.0)
        .collect();

    for (_entity, pos, mut stealth, noise_opt, vis_opt) in stealth_query.iter_mut() {
        // Calculate effective stealth considering all modifiers
        let terrain_mod = terrain.stealth_modifier(pos.grid.0);
        let night_mod = if game_time.is_night() { 0.20 } else { 0.0 };
        let noise_factor = noise_opt.map(|n| n.noise * 0.5).unwrap_or(0.0);
        let size_factor = vis_opt.map(|v| v.footprint * 0.3).unwrap_or(0.0);
        let effective_level =
            (stealth.level + terrain_mod + night_mod - noise_factor - size_factor).clamp(0.0, 1.0);

        // Update the level for detection system
        stealth.level = effective_level;

        // Check if inside any ambush zone
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
            // Auto-hide inside ambush zones (even if previously revealed)
            if !stealth.is_hidden {
                stealth.is_hidden = true;
                stealth.hidden_timer = zone_list
                    .first()
                    .map(|(_, z, _)| z.duration)
                    .unwrap_or(30.0);
            }
        } else if stealth.ambush_zone.is_some() {
            // Left the zone — reveal
            stealth.ambush_zone = None;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STEALTH DETECTION (enhanced with noise, unit size, terrain, night)
// ═════════════════════════════════════════════════════════════════════════════

/// Enhanced detection that accounts for terrain, night, noise, and unit size.
fn stealth_detection_system(
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
        (Entity, &WorldPosition, &Detection),
        (With<Unit>, Without<Stealth>, Without<Dead>),
    >,
    terrain: Res<TerrainGrid>,
    game_time: Res<GameTime>,
    mut reveal_events: EventWriter<StealthRevealed>,
) {
    let detector_list: Vec<(Entity, &WorldPosition, &Detection)> =
        detectors.iter().filter(|(_, _, det)| det.active).collect();

    for (stealth_e, stealth_pos, mut stealth, noise_opt, vis_opt) in stealth_query.iter_mut() {
        if !stealth.is_hidden {
            continue;
        }

        // Calculate all modifiers for this stealth unit
        let terrain_mod = terrain.stealth_modifier(stealth_pos.grid.0);
        let night_mod = if game_time.is_night() { 0.20 } else { 0.0 };
        let noise_factor = noise_opt.map(|n| n.noise * 0.5).unwrap_or(0.0);
        let size_factor = vis_opt.map(|v| v.footprint * 0.3).unwrap_or(0.0);

        // Effective stealth (0.0 = easily spotted, 1.0 = nearly invisible)
        let effective_stealth =
            (stealth.level + terrain_mod + night_mod - noise_factor - size_factor).clamp(0.0, 1.0);

        for (detector_e, detector_pos, detector_detection) in &detector_list {
            if *detector_e == stealth_e {
                continue;
            }

            let dist = stealth_pos.world_2d.0.distance(detector_pos.world_2d.0);
            let effective_radius = detector_detection.radius + detector_detection.height_bonus;

            if effective_radius <= 0.0 {
                continue;
            }

            // Normalised detection chance: 0.0 (certain) → large (impossible)
            // When dist >> radius, detection_chance is large → impossible to detect.
            // Higher stealth = lower numerator = harder to detect.
            let detection_chance = (dist / effective_radius) * (1.0 - effective_stealth);

            if detection_chance < 0.5 {
                stealth.is_hidden = false;
                stealth.hidden_timer = 0.0;
                reveal_events.send(StealthRevealed {
                    entity: stealth_e,
                    revealer: *detector_e,
                });
                break; // Only one reveal needed
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH TRIGGER (fixed — uses actual MeleeWeapon damage)
// ═════════════════════════════════════════════════════════════════════════════

/// When a hidden unit attacks from inside an `AmbushZone`, apply bonus
/// damage based on the attacker's actual weapon, then fire `AmbushTriggered`.
fn ambush_trigger_system(
    mut attack_events: EventReader<AttackTriggered>,
    positions: Query<&WorldPosition>,
    ambush_zones: Query<(Entity, &AmbushZone, &WorldPosition)>,
    stealth_units: Query<&Stealth>,
    weapons: Query<&MeleeWeapon>,
    mut damage_events: EventWriter<DamageDealt>,
    mut ambush_events: EventWriter<AmbushTriggered>,
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

        // Check if attacker is inside an AmbushZone
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
            // Read actual weapon damage instead of hardcoding 10.0
            let weapon_damage = weapons
                .get(attack.attacker)
                .map(|w| w.damage)
                .unwrap_or(10.0); // fallback

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
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  AMBUSH PLACEMENT
// ═════════════════════════════════════════════════════════════════════════════

/// Handles `PlaceAmbushOrder` events: spawns an AmbushZone entity
/// and consumes a resource (gold or special resource).
fn ambush_placement_system(
    mut placement_events: EventReader<PlaceAmbushOrder>,
    mut commands: Commands,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    for order in placement_events.read() {
        // Cost: 30 gold to place an ambush zone
        let ambush_cost = 30.0;

        if stockpile.gold < ambush_cost {
            continue;
        }
        stockpile.gold -= ambush_cost;

        // Spawn the ambush zone
        let pos = WorldPosition::new(order.position.0, 1.0);
        commands.spawn((
            AmbushZone {
                owner: order.entity,
                radius: 4.0,
                bonus_damage: 1.5, // +50% damage
                bonus_morale: 0.2,
                duration: 60.0,
                remaining: 60.0,
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
