use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct SiegeWarfarePlugin;

impl Plugin for SiegeWarfarePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SiegeWallDamaged>().add_systems(
            FixedUpdate,
            (
                // Phase 5c: Siege operations run during SiegeResolution
                siege_engine_operation_system.in_set(SimulationSet::SiegeResolution),
                wall_damage_system.in_set(SimulationSet::SiegeResolution),
                breach_detection_system.in_set(SimulationSet::SiegeResolution),
            ),
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  SIEGE ENGINE OPERATION (Phase E)
// ═════════════════════════════════════════════════════════════════════════════

/// Handles siege engine reload, targeting, and firing.
///
/// Each frame:
/// 1. Decrements `reload_timer` by delta
/// 2. When timer reaches 0 and a target exists, fires
/// 3. Calculates wall damage based on engine type and ammo
/// 4. Sends `SiegeWallDamaged` for application
/// 5. Resets reload timer to the engine's reload interval
///
/// Damage values (GDD Siege Weapons table):
///   Trebuchet: 150 dmg, 8s reload
///   Catapult:  100 dmg, 6s reload
///   Ballista:   60 dmg, 4s reload
///   BatteringRam: 200 dmg, 10s reload (melee range, gates only)
///   SiegeTower:  40 dmg, 12s reload (transports troops)
///   Mantlet:     10 dmg, 3s reload (cover for archers)
fn siege_engine_operation_system(
    time: Res<Time>,
    mut engines: Query<(Entity, &mut SiegeEngine, &WorldPosition)>,
    mut wall_damage_events: EventWriter<SiegeWallDamaged>,
) {
    let delta = time.delta_secs();

    for (engine_e, mut engine, _pos) in engines.iter_mut() {
        // Tick reload timer
        engine.reload_timer -= delta;

        // Fire when reloaded and a target exists
        if engine.reload_timer <= 0.0 {
            if let Some(target) = engine.target {
                let damage = engine_damage(engine.engine_type, engine.ammo);

                wall_damage_events.send(SiegeWallDamaged {
                    wall: target,
                    damage,
                    source: engine_e,
                    is_breached: false,
                });

                // Reset reload timer
                engine.reload_timer = reload_time(engine.engine_type);
            } else {
                // No target — hold at 0 ready to fire
                engine.reload_timer = 0.0;
            }
        }
    }
}

/// Returns the base damage for an engine/ammo combination.
fn engine_damage(engine_type: SiegeEngineType, _ammo: SiegeAmmoType) -> f32 {
    match engine_type {
        SiegeEngineType::Trebuchet => 150.0,
        SiegeEngineType::Catapult => 100.0,
        SiegeEngineType::Ballista => 60.0,
        SiegeEngineType::BatteringRam => 200.0,
        SiegeEngineType::SiegeTower => 40.0,
        SiegeEngineType::Mantlet => 10.0,
    }
}

/// Returns the base reload time for an engine type.
fn reload_time(engine_type: SiegeEngineType) -> f32 {
    match engine_type {
        SiegeEngineType::Trebuchet => 8.0,
        SiegeEngineType::Catapult => 6.0,
        SiegeEngineType::Ballista => 4.0,
        SiegeEngineType::BatteringRam => 10.0,
        SiegeEngineType::SiegeTower => 12.0,
        SiegeEngineType::Mantlet => 3.0,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  WALL DAMAGE SYSTEM (Phase E)
// ═════════════════════════════════════════════════════════════════════════════

/// Applies damage to wall segments from `SiegeWallDamaged` events.
///
/// When a wall segment's health drops to 0 or below:
///   - Sets `is_breached = true`
///   - Sets `breach_size` proportional to the overkill damage
fn wall_damage_system(
    mut wall_damage_events: EventReader<SiegeWallDamaged>,
    mut walls: Query<&mut WallSegment>,
) {
    for event in wall_damage_events.read() {
        if let Ok(mut wall) = walls.get_mut(event.wall) {
            wall.health -= event.damage;

            if wall.health <= 0.0 && !wall.is_breached {
                wall.is_breached = true;
                // Breach size = fraction of max health below zero (capped at 1.0)
                wall.breach_size = (wall.health.abs() / wall.max_health).clamp(0.2, 1.0);
                wall.health = 0.0;
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  BREACH DETECTION SYSTEM (Phase E)
// ═════════════════════════════════════════════════════════════════════════════

/// Detects breached wall segments and manages `BreachPoint` components.
///
/// For each breached wall:
///   - If a `BreachPoint` already exists, updates passability
///   - If no `BreachPoint` exists, spawns one
///
/// A breach is passable when `breach_size >= 0.5`.
fn breach_detection_system(
    mut commands: Commands,
    walls: Query<(Entity, &WallSegment, Option<&BreachPoint>)>,
) {
    for (entity, wall, existing_breach) in walls.iter() {
        if wall.is_breached {
            let is_passable = wall.breach_size >= 0.5;

            match existing_breach {
                Some(breach) => {
                    // Update existing breach
                    if breach.is_passable != is_passable {
                        commands.entity(entity).insert(BreachPoint {
                            wall_segment: entity,
                            size: wall.breach_size,
                            is_passable,
                        });
                    }
                }
                None => {
                    // Spawn new BreachPoint
                    commands.entity(entity).insert(BreachPoint {
                        wall_segment: entity,
                        size: wall.breach_size,
                        is_passable,
                    });
                }
            }
        } else {
            // Wall not breached — remove any lingering BreachPoint
            if existing_breach.is_some() {
                commands.entity(entity).remove::<BreachPoint>();
            }
        }
    }
}
