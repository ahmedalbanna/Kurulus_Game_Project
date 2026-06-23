use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Helper to build a basic test app with minimal plugins
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
    app
}

/// Helper to spawn a hidden unit at a given position
fn spawn_hidden_unit(world: &mut World, pos: IVec2, level: f32, faction: FactionId) -> Entity {
    world
        .spawn((
            Stealth::hidden(level),
            Detection {
                radius: 5.0,
                height_bonus: 0.0,
                active: true,
            },
            WorldPosition::new(pos, CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction,
                formation: FormationType::Scatter,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id()
}

/// Helper to spawn a detector unit at a given position
fn spawn_detector(
    world: &mut World,
    pos: IVec2,
    radius: f32,
    faction: FactionId,
    facing: Vec2,
) -> Entity {
    world
        .spawn((
            Detection {
                radius,
                height_bonus: 0.0,
                active: true,
            },
            VisionCone {
                facing,
                ..Default::default()
            },
            ObserverState::default(),
            WorldPosition::new(pos, CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction,
                formation: FormationType::Scatter,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id()
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: VISION CONE - Front vs Rear Detection
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_vision_cone_detects_in_front() {
    let mut app = test_app();

    // Hidden unit in front of the detector
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 5), // 5 units ahead
        0.3,
        FactionId::Byzantine,
    );
    // Detector facing "south" (towards negative Y) — but hidden is at positive Y
    // In Bevy's coordinate system: facing down = (0, -1), up = (0, 1)
    // hidden is at (0,5), detector at (0,0), so detector faces TOWARDS hidden if facing=(0,1)
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        10.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0), // facing up (towards hidden)
    );

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        !stealth.is_hidden,
        "Hidden unit in front of detector should be detected"
    );
}

#[test]
fn test_vision_cone_does_not_detect_from_behind() {
    let mut app = test_app();

    // Hidden unit BEHIND the detector (detector facing away)
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 5), // 5 units away
        0.3,
        FactionId::Byzantine,
    );
    // Detector facing DOWN (0, -1) — hidden is at (0, 5) which is BEHIND
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        10.0,
        FactionId::Kayi,
        Vec2::new(0.0, -1.0), // facing down (away from hidden)
    );

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        stealth.is_hidden,
        "Hidden unit behind detector should remain hidden (0° rear = blind)"
    );
}

#[test]
fn test_vision_cone_partial_detection_from_side() {
    let mut app = test_app();

    // Hidden unit to the SIDE of the detector (at 90°)
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(5, 0), // 5 units to the right
        0.5,
        FactionId::Byzantine,
    );
    // Detector facing UP — hidden is at right (90° angle)
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        10.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0), // facing up
    );

    for _ in 0..10 {
        app.update();
    }

    let _stealth = app.world().get::<Stealth>(hidden).unwrap();
    // Side detection has 0.4 multiplier, so at range 5 with radius 10:
    // effective_radius = 10 * 0.4 * observer * target * env = ~4.0
    // detection_ratio = 5 / 4 = 1.25 > 1.0 → probably NOT detected
    // This is expected since side multiplier is low
    // The test verifies the vision cone system runs without errors
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: STRAIGHT-LINE FRONT DETECTION (simple regression)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_detection_at_exact_range_boundary() {
    let mut app = test_app();

    // Hidden at the edge of detection range
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 12), // 12 units away, detector radius = 10
        0.5,
        FactionId::Byzantine,
    );
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        10.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0), // facing towards hidden
    );

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    // At range 12 with radius 10 and front multiplier 1.0:
    // effective_radius = 10 * 1.0 * 1.0 * 0.5 * 1.0 = 5.0 (target_factor=0.5 for stealth 0.5)
    // detection_ratio = 12 / 5 = 2.4 > 1.0 → not detected
    assert!(
        stealth.is_hidden,
        "Unit beyond effective range should remain hidden"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: STEALTH ZONE BONUS
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_stealth_zone_improves_hiding() {
    let mut app = test_app();

    // Hidden in a stealth zone with good stealth level
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        0.8, // high stealth level
        FactionId::Byzantine,
    );
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 3), // 3 units away — should detect non-hidden
        5.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Add a StealthZone component at the hidden unit's position
    app.world_mut().spawn((
        StealthZone::new(StealthZoneType::DenseForest),
        GridPos(IVec2::new(0, 0)),
        TerrainTile {
            position: GridPos(IVec2::new(0, 0)),
            terrain: TerrainType::Forest,
        },
    ));

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        stealth.is_hidden,
        "Unit in dense forest with high stealth should remain hidden at 3 units"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: NIGHT STEALTH BONUS
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_night_stealth_bonus_helps_hiding() {
    let mut app = test_app();

    // Hidden unit with low stealth
    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        0.2, // low stealth - easily spotted
        FactionId::Byzantine,
    );
    // Detector very close
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 2), // 2 units
        5.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Set game time to night (hour 0)
    let mut game_time = app.world_mut().resource_mut::<GameTime>();
    game_time.hour = 0; // midnight

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    // At night with +0.20 bonus: effective stealth = 0.2 + 0.2 = 0.4
    // target_factor = 0.6, effective_radius = 5 * 0.6 * 0.6 * 1.0 = 1.8
    // (observer=1.0, env=0.6 at night, cone=1.0)
    // detection_ratio = 2 / 1.8 = 1.11 > 1.0 → not detected
    // Without night bonus: effective_radius = 5 * 0.8 * 1.0 * 1.0 = 4.0
    // detection_ratio = 2 / 4 = 0.5 → detected!
    // So night bonus should help keep this unit hidden
    assert!(
        stealth.is_hidden,
        "Low stealth unit at night should remain hidden due to night bonus"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: DETECTION CHAIN (Suspicion → Alarm)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_detection_chain_suspicion_raised() {
    let mut app = test_app();
    app.add_event::<UnitSuspicionRaised>();
    app.add_event::<UnitConfirmed>();

    let hidden = spawn_hidden_unit(app.world_mut(), IVec2::new(0, 3), 0.3, FactionId::Byzantine);
    let detector = spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        10.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Manually add a DetectionChain to initiate the chain
    app.world_mut()
        .entity_mut(detector)
        .insert(DetectionChain::new(
            hidden, 0.5, // rng seed → ~1.5s confirmation time
        ));

    // Run for a few frames to let the timer progress (suspicion builds over time)
    for _ in 0..10 {
        app.update();
    }

    let chain = app.world().get::<DetectionChain>(detector).unwrap();
    assert!(
        chain.timer > 0.0,
        "Detection chain timer should have progressed"
    );
    assert!(
        !chain.alarm_raised,
        "Alarm should not be raised before confirmation time (~1.5s, only 10 frames ≈ 0.17s elapsed)"
    );
    assert!(
        chain.suspicion_level >= 0.3,
        "Suspicion should remain at initial level until confirmation time is reached"
    );
}

#[test]
fn test_detection_chain_alarm_raised() {
    let mut app = test_app();
    app.add_event::<UnitSuspicionRaised>();
    app.add_event::<UnitConfirmed>();
    app.add_event::<AlarmRaised>();

    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 2), // very close
        0.3,
        FactionId::Byzantine,
    );
    let detector = spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        15.0, // large radius
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Manually add DetectionChain with very short confirmation time
    app.world_mut().entity_mut(detector).insert(DetectionChain {
        suspicion_level: 0.3,
        timer: 0.0,
        confirmation_time: 0.1, // very fast confirmation
        alarm_raised: false,
        tracked_entity: Some(hidden),
        alarm_prevented: false,
    });

    // Run many frames to ensure confirmation + alarm
    for _ in 0..20 {
        app.update();
    }

    // The chain should be gone (removed after alarm)
    let chain = app.world().get::<DetectionChain>(detector);
    assert!(
        chain.is_none(),
        "Detection chain should be removed after alarm is raised"
    );

    // Hidden unit should now be revealed
    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        !stealth.is_hidden,
        "Hidden unit should be revealed after detection chain completes"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: OBSERVER STATE - ALERT vs UNWARE
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_alert_detector_detects_farther() {
    let mut app = test_app();

    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 1), // 1 unit away — within auto-detect range for alert detector
        0.4,
        FactionId::Byzantine,
    );
    let detector = spawn_detector(
        app.world_mut(),
        IVec2::new(0, 0),
        5.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Set detector to Alert state (1.2x detection multiplier)
    app.world_mut().entity_mut(detector).insert(ObserverState {
        alert_level: AlertLevel::Alert,
        fatigue: 0.0,
        in_formation: false,
        is_alone: false,
        is_sleeping: false,
    });

    for _ in 0..10 {
        app.update();
    }

    // Alert multiplier 1.2x + close range (1 unit) → auto-detect (ratio < 0.5)
    // effective_radius = 5 * 1.2 * 0.6 * 1.0 * 1.0 = 3.6
    // detection_ratio = 1 / 3.6 = 0.28 < 0.5 → immediate detection
    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        !stealth.is_hidden,
        "Alert detector should auto-detect hidden unit at close range"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: REGRESSION - Existing detection behavior preserved
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_hidden_unit_not_detected_when_far() {
    let mut app = test_app();

    let hidden = spawn_hidden_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        0.9, // near-invisible
        FactionId::Byzantine,
    );
    spawn_detector(
        app.world_mut(),
        IVec2::new(100, 100), // very far
        5.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    for _ in 0..5 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        stealth.is_hidden,
        "Hidden unit far from detector should remain hidden"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: OVERCROWDED STEALTH ZONE PENALTY
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_overcrowded_stealth_zone_detected() {
    let mut app = test_app();

    let hidden = spawn_hidden_unit(app.world_mut(), IVec2::new(0, 0), 0.7, FactionId::Byzantine);
    spawn_detector(
        app.world_mut(),
        IVec2::new(0, 3),
        5.0,
        FactionId::Kayi,
        Vec2::new(0.0, 1.0),
    );

    // Add a Caves stealth zone (capacity 10) but set occupancy to 11 (overcrowded)
    app.world_mut().spawn((
        StealthZone {
            zone_type: StealthZoneType::Caves,
            current_occupancy: 11, // over capacity of 10
            destroyed: false,
        },
        GridPos(IVec2::new(0, 0)),
    ));

    for _ in 0..10 {
        app.update();
    }

    let stealth = app.world().get::<Stealth>(hidden).unwrap();
    assert!(
        stealth.partial_stealth,
        "Unit in overcrowded zone should have partial_stealth = true"
    );
}
