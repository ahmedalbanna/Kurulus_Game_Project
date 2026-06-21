use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct StealthPlugin;

impl Plugin for StealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                vision_calculation_system.in_set(SimulationSet::VisionCalculation),
                stealth_detection_system.in_set(SimulationSet::StealthDetection),
                ambush_trigger_system.in_set(SimulationSet::AmbushTrigger),
            ),
        );
    }
}

// ─── Vision calculation ─────────────────────────────────────────────────────

/// Updates `FogOfWar` and determines which entities are visible to each
/// faction based on unit vision ranges and height bonuses.
///
/// For Phase 2 we use a simplified model:
///   - Entities within `vision_range` of any allied unit are "visible"
///   - Height levels increase vision by 2× per level above Ground
fn vision_calculation_system(
    units: Query<(&WorldPosition, &Stats, &Unit, Option<&HeightLevel>)>,
    mut fog_tiles: Query<(&GridPos, &mut FogOfWar)>,
    _game_time: Res<GameTime>,
) {
    // Build set of visible grid cells for the player's faction
    let mut visible_cells: std::collections::HashSet<IVec2> =
        std::collections::HashSet::new();

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
            0.0 // enemy vision doesn't matter for player fog
        };

        if vision <= 0.0 {
            continue;
        }

        // Mark all cells within vision range as visible
        let gx = pos.grid.0.x;
        let gy = pos.grid.0.y;
        let range_i = vision.ceil() as i32;
        for dx in -range_i..=range_i {
            for dy in -range_i..=range_i {
                let d_sq = dx * dx + dy * dy;
                if (d_sq as f32) <= vision * vision {
                    visible_cells.insert(IVec2::new(gx + dx, gy + dy));
                }
            }
        }
    }

    // Update fog tiles
    for (grid_pos, mut fog) in fog_tiles.iter_mut() {
        let is_visible = visible_cells.contains(&grid_pos.0);
        if is_visible {
            fog.visible = true;
            fog.explored = true;
        }
    }
}

// ─── Stealth detection ──────────────────────────────────────────────────────

/// Checks if hidden units are detected by nearby enemies with Detection.
fn stealth_detection_system(
    mut stealth_query: Query<
        (Entity, &WorldPosition, &mut Stealth, Option<&Detection>),
        (With<Stealth>, Without<Dead>),
    >,
    detectors: Query<
        (Entity, &WorldPosition, &Detection),
        (With<Unit>, Without<Stealth>, Without<Dead>),
    >,
    mut reveal_events: EventWriter<StealthRevealed>,
) {
    for (stealth_e, stealth_pos, mut stealth, _self_detection) in stealth_query.iter_mut() {
        if !stealth.is_hidden {
            continue;
        }

        for (detector_e, detector_pos, detector_detection) in detectors.iter() {
            if stealth_e == detector_e {
                continue;
            }

            let dist = stealth_pos.world_2d.0.distance(detector_pos.world_2d.0);
            let effective_radius =
                detector_detection.radius + detector_detection.height_bonus;

            // Normalised detection chance: 0.0 (certain) → 1.0 (impossible)
            let detection_chance = (dist / effective_radius) * (1.0 - stealth.level);

            if detection_chance < 0.5 {
                stealth.is_hidden = false;
                stealth.hidden_timer = 0.0;
                reveal_events.send(StealthRevealed {
                    entity: stealth_e,
                    revealer: detector_e,
                });
            }
        }
    }
}

// ─── Ambush trigger ─────────────────────────────────────────────────────────

/// When a hidden unit attacks from inside an `AmbushZone`, apply bonus
/// damage and morale, then fire `AmbushTriggered`.
fn ambush_trigger_system(
    mut attack_events: EventReader<AttackTriggered>,
    positions: Query<&WorldPosition>,
    ambush_zones: Query<(Entity, &AmbushZone, &WorldPosition)>,
    stealth_units: Query<&Stealth>,
    mut damage_events: EventWriter<DamageDealt>,
    mut ambush_events: EventWriter<AmbushTriggered>,
) {
    for attack in attack_events.read() {
        // Check if attacker is hidden
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
        'zone_loop: for (zone_e, zone, zone_pos) in ambush_zones.iter() {
            let dist = attacker_pos
                .world_2d
                .0
                .distance(zone_pos.world_2d.0);
            if dist <= zone.radius {
                bonus.damage_multiplier = zone.bonus_damage.max(1.0);
                bonus.morale_bonus = zone.bonus_morale.max(0.0);
                bonus.ambush_zone = Some(zone_e);
                break 'zone_loop;
            }
        }

        if let Some(zone_entity) = bonus.ambush_zone {
            // Apply bonus damage via a new DamageDealt event
            // The original attack already fired; we fire an additional event
            // with the bonus damage factored in.
            let original_damage = 10.0; // estimated — real damage is in the system
            let bonus_damage = original_damage * (bonus.damage_multiplier - 1.0);

            if bonus_damage > 0.0 {
                damage_events.send(DamageDealt {
                    attacker: attack.attacker,
                    target: attack.target,
                    amount: bonus_damage,
                    damage_type: DamageType::Melee,
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
