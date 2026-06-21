use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct StealthPlugin;

impl Plugin for StealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                stealth_detection_system.in_set(SimulationSet::StealthDetection),
                ambush_trigger_system.in_set(SimulationSet::AmbushTrigger),
                vision_calculation_system.in_set(SimulationSet::VisionCalculation),
            ),
        );
    }
}

/// Checks if hidden units are detected by nearby enemies with Detection
fn stealth_detection_system(
    mut stealth_query: Query<(Entity, &WorldPosition, &mut Stealth, &Detection)>,
    detectors: Query<(Entity, &WorldPosition, &Detection), (With<Unit>, Without<Stealth>)>,
    mut reveal_events: EventWriter<StealthRevealed>,
) {
    for (stealth_e, stealth_pos, mut stealth, _detection) in stealth_query.iter_mut() {
        if !stealth.is_hidden {
            continue;
        }

        for (detector_e, detector_pos, detector_detection) in detectors.iter() {
            if stealth_e == detector_e {
                continue;
            }

            let dist = stealth_pos.world_2d.0.distance(detector_pos.world_2d.0);
            let effective_radius = detector_detection.radius + detector_detection.height_bonus;

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

/// Triggers ambush bonuses when a hidden unit attacks from an AmbushZone
fn ambush_trigger_system(
    _attack_events: EventReader<AttackTriggered>,
    _ambush_zones: Query<&AmbushZone>,
    _stealth_units: Query<&Stealth>,
    _ambush_events: EventWriter<AmbushTriggered>,
) {
    // TODO: Implement ambush triggering logic
}

/// Calculates vision ranges for all units (used for fog of war and detection)
fn vision_calculation_system(_units: Query<(&WorldPosition, &Stats, Option<&HeightLevel>)>) {
    // TODO: Implement vision calculation with height bonuses
}
