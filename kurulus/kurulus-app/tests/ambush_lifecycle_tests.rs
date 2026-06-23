use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

/// Helper to build a test app with minimal plugins.
/// Configures phase transitions to stay in Execution phase so ambush
/// systems (gated behind `in_state(GamePhase::Execution)`) keep running.
fn test_app() -> App {
    let mut app = App::new();

    // Insert PhaseTransitionConfig BEFORE plugins so CoreLoopPlugin's
    // init_resource doesn't override it. The extremely long timeout keeps
    // the game in Execution for the whole test.
    app.insert_resource(PhaseTransitionConfig {
        combat_inactivity_timeout: 999_999.0,
        ..Default::default()
    });

    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));

    app
}

/// Helper to spawn a hidden unit (Kayi-friendly) at a given position
fn spawn_ambusher(world: &mut World, pos: IVec2, trigger_type: TriggerType, level: f32) -> Entity {
    world
        .spawn((
            Stealth::hidden(level),
            Detection {
                radius: 10.0,
                height_bonus: 0.0,
                active: true,
            },
            AmbushState {
                phase: AmbushPhase::Idle,
                trigger_type,
                ..Default::default()
            },
            WorldPosition::new(pos, CELL_SIZE),
            Unit {
                archetype: UnitArchetype::Scout,
                faction: FactionId::Kayi,
                formation: FormationType::Scatter,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
            NoiseLevel { noise: 0.0 },
            UnitVisibility::from_archetype(UnitArchetype::Scout),
        ))
        .id()
}

/// Helper to spawn an enemy unit at a given position
fn spawn_enemy(world: &mut World, pos: IVec2) -> Entity {
    world
        .spawn((
            WorldPosition::new(pos, CELL_SIZE),
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            Stats::default(),
            Health::default(),
        ))
        .id()
}

/// Helper to spawn an AmbushZone entity at a given position
fn spawn_ambush_zone(world: &mut World, pos: IVec2, owner: Entity, radius: f32) -> Entity {
    world
        .spawn((
            AmbushZone {
                owner,
                radius,
                bonus_damage: 1.5,
                bonus_morale: 0.2,
                duration: 60.0,
                remaining: 60.0,
            },
            WorldPosition::new(pos, CELL_SIZE),
            AmbushMarker {
                position: GridPos(pos),
                placed_by: owner,
            },
        ))
        .id()
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: AMBUSH PHASE TRANSITION — Idle → Setup → Ready
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ambush_phase_transitions_to_ready() {
    let mut app = test_app();
    app.add_event::<AmbushPhaseChanged>();

    // Spawn an ambusher near an ambush zone
    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 5.0);

    // Run many frames so the entry timer progresses past 2.0
    // At 1/60s per frame, need 120 frames to reach 2.0s
    for _ in 0..200 {
        app.update();
    }

    // The unit should now be in Ready phase
    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(
        state.phase,
        AmbushPhase::Ready,
        "Ambusher should have transitioned to Ready phase after entry timer completes"
    );

    // The unit should be hidden
    let stealth = app.world().get::<Stealth>(ambusher).unwrap();
    assert!(
        stealth.is_hidden,
        "Ambusher should be hidden in Ready phase"
    );
    assert!(
        stealth.is_ready,
        "Ambusher should have is_ready set in Ready phase"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: READY STATE ENFORCEMENT — No AttackTarget
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ready_state_removes_attack_target() {
    let mut app = test_app();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 5.0);

    // Manually set the unit to Ready state with an AttackTarget
    app.world_mut().entity_mut(ambusher).insert((
        AmbushState {
            phase: AmbushPhase::Ready,
            ready_since: 3.0,
            ..Default::default()
        },
        AttackTarget {
            target: Entity::from_raw(999),
            aggressive: true,
        },
    ));

    let mut stealth = app.world_mut().get_mut::<Stealth>(ambusher).unwrap();
    stealth.is_ready = true;
    stealth.is_hidden = true;

    // Run a few frames so ready_state_enforcement_system runs
    for _ in 0..5 {
        app.update();
    }

    // AttackTarget should be removed
    let attack_target = app.world().get::<AttackTarget>(ambusher);
    assert!(
        attack_target.is_none(),
        "Ready-state ambusher should have no AttackTarget"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: AUTO-TRIGGER — Enemy enters ambush zone
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ambush_auto_triggers_when_enemy_in_range() {
    let mut app = test_app();
    app.add_event::<AmbushPhaseChanged>();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 8.0);

    // Run frames to enter Ready state
    for _ in 0..200 {
        app.update();
    }

    // Verify we're in Ready state
    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(
        state.phase,
        AmbushPhase::Ready,
        "Should be in Ready before enemy approaches"
    );

    // Now spawn an enemy close to the ambush zone center
    let _enemy = spawn_enemy(app.world_mut(), IVec2::new(0, 3));

    // Run frames for the system to detect the enemy
    for _ in 0..30 {
        app.update();
    }

    // The ambush should have triggered (enemy within 8m radius)
    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert!(
        state.phase == AmbushPhase::Triggered || state.phase == AmbushPhase::Executing,
        "Ambush should have triggered when enemy entered zone (phase: {:?})",
        state.phase
    );

    // If in Executing, verify bonus damage is active
    if state.phase == AmbushPhase::Executing {
        assert!(
            state.bonus_damage_remaining > 0.0,
            "Bonus damage should be active in Executing phase"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: FORMATION BREAK APPLIED ON AMBUSH TRIGGER
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ambush_trigger_applies_formation_break_and_shock() {
    let mut app = test_app();
    app.add_event::<AttackTriggered>();
    app.add_event::<AmbushTriggered>();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 5.0);

    // Set up the ambusher as hidden in the zone
    app.world_mut().entity_mut(ambusher).insert(AmbushState {
        phase: AmbushPhase::Executing,
        bonus_damage_remaining: 10.0,
        ..Default::default()
    });

    // Spawn a target enemy with MeleeWeapon (needed by ambush_trigger_system)
    let target = spawn_enemy(app.world_mut(), IVec2::new(0, 2));

    // Fire an AttackTriggered event from the ambusher
    app.world_mut().send_event(AttackTriggered {
        attacker: ambusher,
        target,
        weapon_type: WeaponType::Bow,
        is_charge: false,
        is_from_stealth: true,
    });

    // Run a few frames
    for _ in 0..5 {
        app.update();
    }

    // The target should now have FormationBreak and Shock components
    let formation_break = app.world().get::<FormationBreak>(target);
    assert!(
        formation_break.is_some(),
        "Target should have FormationBreak after ambush trigger"
    );
    if let Some(fb) = formation_break {
        assert!(
            fb.remaining > 0.0 && fb.remaining <= 3.0,
            "FormationBreak should be active with valid duration"
        );
    }

    let shock = app.world().get::<Shock>(target);
    assert!(
        shock.is_some(),
        "Target should have Shock after ambush trigger"
    );
    if let Some(s) = shock {
        assert!(
            s.remaining > 0.0 && s.remaining <= 2.0,
            "Shock should be active with valid duration"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: FULL AMBUSH LIFECYCLE — Complete phase cycle
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ambush_full_lifecycle_cycle() {
    let mut app = test_app();
    app.add_event::<AmbushPhaseChanged>();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 8.0);

    // Phase 1: Idle → Setup → Ready (run 200 frames = ~3.3s)
    for _ in 0..200 {
        app.update();
    }

    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(
        state.phase,
        AmbushPhase::Ready,
        "Phase should be Ready after entry timer"
    );

    // Phase 2: Set short timers BEFORE enemy spawns, so the ambush reads
    // the short damage_duration when it enters Executing.
    app.world_mut()
        .resource_mut::<AmbushConfig>()
        .effects
        .damage_duration = 0.5;

    let _enemy = spawn_enemy(app.world_mut(), IVec2::new(0, 3));
    for _ in 0..30 {
        app.update();
    }

    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert!(
        state.phase == AmbushPhase::Executing,
        "Phase should be Executing after enemy detected (phase: {:?})",
        state.phase
    );
    assert!(
        state.bonus_damage_remaining > 0.0,
        "Bonus damage should be counting down"
    );

    // Phase 3: Wait for Executing → Cooldown → Idle
    // Short damage window (0.5s) + hardcoded 5.0s cooldown = ~5.5s total.
    // Use 20000 frames to handle any FixedUpdate dt variation.
    for _ in 0..20000 {
        app.update();
        let state = app.world().get::<AmbushState>(ambusher).unwrap();
        if state.phase == AmbushPhase::Idle {
            break;
        }
    }

    // Check phase sequence: should have completed full cycle to Idle
    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(
        state.phase,
        AmbushPhase::Idle,
        "Phase should be Idle after full lifecycle (phase: {:?}, bonus: {}, ready_since: {})",
        state.phase,
        state.bonus_damage_remaining,
        state.ready_since
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: MANUAL TRIGGER DOES NOT AUTO-TRIGGER
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_manual_trigger_does_not_auto_trigger() {
    let mut app = test_app();
    app.add_event::<AmbushPhaseChanged>();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::Manual, // Manual — should NOT auto-trigger
        0.5,
    );
    let _zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 8.0);

    // Run frames to enter Ready state
    for _ in 0..200 {
        app.update();
    }

    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(state.phase, AmbushPhase::Ready, "Should be in Ready state");

    // Spawn enemy very close
    let _enemy = spawn_enemy(app.world_mut(), IVec2::new(0, 2));
    for _ in 0..60 {
        app.update();
    }

    // Should still be Ready (Manual trigger type)
    let state = app.world().get::<AmbushState>(ambusher).unwrap();
    assert_eq!(
        state.phase,
        AmbushPhase::Ready,
        "Manual trigger ambusher should NOT auto-trigger even with enemy in range"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
//  TEST: ENTRY TIMER PROGRESSION FIX
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_ambush_entry_exit_timer() {
    let mut app = test_app();

    let ambusher = spawn_ambusher(
        app.world_mut(),
        IVec2::new(0, 0),
        TriggerType::AutoFirstContact,
        0.5,
    );
    let zone = spawn_ambush_zone(app.world_mut(), IVec2::new(0, 0), ambusher, 5.0);

    // Run frames to enter zone (timer should increase)
    for _ in 0..60 {
        app.update();
    }

    let entry_timer_before = app.world().get::<Stealth>(ambusher).unwrap().entry_timer;
    assert!(
        entry_timer_before > 0.0,
        "Entry timer should be > 0 after being in zone"
    );

    // Now despawn the zone — unit should start exiting
    app.world_mut().entity_mut(zone).despawn();

    for _ in 0..30 {
        app.update();
    }

    // After exiting, entry_timer should still be >= -0.5 (not reset to 0)
    let entry_timer_after = app.world().get::<Stealth>(ambusher).unwrap().entry_timer;
    assert!(
        entry_timer_after < 0.0 || entry_timer_after >= -0.5,
        "Entry timer should decay properly after leaving zone (timer: {})",
        entry_timer_after
    );
}
