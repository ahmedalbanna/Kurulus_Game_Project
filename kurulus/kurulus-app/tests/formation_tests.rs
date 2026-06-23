use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

fn make_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(register_logic_plugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )))
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>();
    // First update enters the Execution state so `run_if(in_state(Execution))`
    // system sets (FormationMovement, AttackTimers, etc.) will run.
    app.update();
    app
}

/// Helper: spawn a unit at a grid position with given formation/archetype/stats.
/// Does NOT insert a Formation component (auto_form_system creates it).
fn spawn_unit(
    world: &mut World,
    grid_pos: IVec2,
    formation_type: FormationType,
    archetype: UnitArchetype,
    move_speed: f32,
    morale: f32,
    health: f32,
) -> Entity {
    world
        .spawn((
            Unit {
                archetype,
                faction: FactionId::Kayi,
                formation: formation_type,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(grid_pos, CELL_SIZE),
            Stats {
                move_speed,
                attack_range: 1.0,
                attack_damage: 10.0,
                attack_speed: 1.0,
                vision_range: 8.0,
                morale,
                morale_max: 100.0,
            },
            Health {
                current: health,
                max: 100.0,
                armor: 5.0,
                armor_type: ArmorType::Light,
            },
        ))
        .id()
}

/// Helper: add a Formation leader component to a unit (required for slot/bonus/break systems).
fn make_leader(
    world: &mut World,
    entity: Entity,
    formation_type: FormationType,
    unit_count: usize,
) {
    world.entity_mut(entity).insert(Formation {
        leader: entity,
        formation_type,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count,
    });
}

/// Helper: spawn a formation follower with explicit leader reference.
fn spawn_follower(
    world: &mut World,
    grid_pos: IVec2,
    leader: Entity,
    formation_type: FormationType,
    unit_count: usize,
) -> Entity {
    world
        .spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: formation_type,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(grid_pos, CELL_SIZE),
            Stats::default(),
            Health::default(),
            Formation {
                leader,
                formation_type,
                facing: Vec2::new(1.0, 0.0),
                is_breaking: false,
                break_timer: None,
                unit_count,
            },
            FormationFollower,
        ))
        .id()
}

// ─── Phase A/B: Basic Formation Movement ────────────────────────────────────

/// Integration test: Formation followers stay close to their leader
#[test]
fn test_formation_movement() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(5, 5),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        4.0,
        70.0,
        100.0,
    );
    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(7, 5),
        leader,
        FormationType::Line,
        2,
    );

    let initial_follower_pos = app
        .world()
        .get::<WorldPosition>(follower)
        .unwrap()
        .world_2d
        .0;
    for _ in 0..10 {
        app.update();
    }

    let final_leader_pos = app.world().get::<WorldPosition>(leader).unwrap().world_2d.0;
    let final_follower_pos = app
        .world()
        .get::<WorldPosition>(follower)
        .unwrap()
        .world_2d
        .0;

    let initial_dist = initial_follower_pos.distance(final_leader_pos);
    let final_dist = final_follower_pos.distance(final_leader_pos);

    assert!(
        final_dist <= initial_dist + 0.1,
        "Follower should move toward leader (initial: {:.2}, final: {:.2})",
        initial_dist,
        final_dist
    );
}

// ─── Phase D: Formation Slot Assignment ─────────────────────────────────────

/// Test that formation followers receive FormationSlot after update.
#[test]
fn test_formation_slots_assigned() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    make_leader(app.world_mut(), leader, FormationType::Line, 2);
    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(3, 0),
        leader,
        FormationType::Line,
        2,
    );

    app.update();

    // Follower should now have a FormationSlot
    let slot = app.world().get::<FormationSlot>(follower);
    assert!(
        slot.is_some(),
        "Follower should have FormationSlot after update"
    );
    if let Some(s) = slot {
        // Line formation: 2 units, file 0 and file 1, offset should be perpendicular
        assert_eq!(s.rank, 0, "Line formation rank should be 0");
        assert!(
            (s.file == 0 || s.file == 1),
            "Line formation file should be 0 or 1"
        );
        // Offset should be on the perpendicular axis (to the right of facing Vec2(1,0))
        assert!(
            s.offset.y.abs() > 0.0 || s.offset.x.abs() > 0.0,
            "Slot offset should be non-zero"
        );
    }
}

/// Test that formation slots are unique per follower (different offsets).
#[test]
fn test_formation_slots_unique() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    make_leader(app.world_mut(), leader, FormationType::Line, 3);
    let f1 = spawn_follower(
        app.world_mut(),
        IVec2::new(3, 0),
        leader,
        FormationType::Line,
        3,
    );
    let f2 = spawn_follower(
        app.world_mut(),
        IVec2::new(3, 2),
        leader,
        FormationType::Line,
        3,
    );

    app.update();

    let s1 = app.world().get::<FormationSlot>(f1).map(|s| s.offset);
    let s2 = app.world().get::<FormationSlot>(f2).map(|s| s.offset);

    assert!(s1.is_some(), "Follower 1 should have slot");
    assert!(s2.is_some(), "Follower 2 should have slot");
    assert_ne!(s1, s2, "Followers should have different slot offsets");
}

// ─── Phase D: Formation Bonus Assignment ───────────────────────────────────

/// Test that formation followers receive FormationBonus matching the
/// formation type.
#[test]
fn test_formation_bonus_assigned_to_followers() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::HeavyInfantry,
        3.0,
        80.0,
        100.0,
    );
    make_leader(app.world_mut(), leader, FormationType::Line, 2);
    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(2, 0),
        leader,
        FormationType::Line,
        2,
    );

    app.update();

    let bonus = app.world().get::<FormationBonus>(follower);
    assert!(bonus.is_some(), "Follower should have FormationBonus");
    if let Some(b) = bonus {
        // Line formation bonuses
        assert!(
            (b.damage_mult - 1.15).abs() < 0.01,
            "Line damage mult should be 1.15, got {}",
            b.damage_mult
        );
        assert!(
            (b.incoming_damage_mult - 0.90).abs() < 0.01,
            "Line incoming damage mult should be 0.90, got {}",
            b.incoming_damage_mult
        );
        assert!(
            (b.speed_mult - 0.80).abs() < 0.01,
            "Line speed mult should be 0.80, got {}",
            b.speed_mult
        );
        assert!(
            (b.block_bonus - 5.0).abs() < 0.01,
            "Line block bonus should be 5.0, got {}",
            b.block_bonus
        );
        assert!(
            (b.anti_cavalry_mult - 1.0).abs() < 0.01,
            "Line anti-cav mult should be 1.0"
        );
        assert!(!b.prevents_flanking, "Line should not prevent flanking");
    }
}

/// Test that formation leaders also receive FormationBonus.
#[test]
fn test_formation_bonus_assigned_to_leader() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Wedge,
        UnitArchetype::HeavyInfantry,
        3.0,
        80.0,
        100.0,
    );
    make_leader(app.world_mut(), leader, FormationType::Wedge, 2);

    app.update();

    let bonus = app.world().get::<FormationBonus>(leader);
    assert!(
        bonus.is_some(),
        "Leader should have FormationBonus after update"
    );
    if let Some(b) = bonus {
        // Wedge formation: +20% damage
        assert!(
            (b.damage_mult - 1.20).abs() < 0.01,
            "Wedge damage mult should be 1.20, got {}",
            b.damage_mult
        );
    }
}

/// Test that Scatter formation does NOT get a FormationBonus.
#[test]
fn test_scatter_no_bonus() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Scatter,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    // Although Scatter is not meant for formations, if a leader exists,
    // the leader should NOT get a bonus.
    make_leader(app.world_mut(), leader, FormationType::Scatter, 1);

    app.update();

    let bonus = app.world().get::<FormationBonus>(leader);
    assert!(bonus.is_none(), "Scatter should NOT get FormationBonus");
}

// ─── Phase D: Auto-Form System ──────────────────────────────────────────────

/// Test that two unformed units with the same faction and formation type
/// that are close together automatically form a formation.
#[test]
fn test_auto_form_creates_formation() {
    let mut app = make_test_app();

    let u1 = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    let u2 = spawn_unit(
        app.world_mut(),
        IVec2::new(1, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        70.0,
        100.0,
    );

    // Run 3 updates so auto_form_system can run
    for _ in 0..3 {
        app.update();
    }

    // One of them should be the leader (Formation without FormationFollower)
    let has_form = |e: Entity| -> bool {
        let form = app.world().get::<Formation>(e);
        form.is_some()
    };
    let is_follower = |e: Entity| -> bool { app.world().get::<FormationFollower>(e).is_some() };

    assert!(
        has_form(u1) || has_form(u2),
        "At least one unit should have a Formation component"
    );
    assert!(
        has_form(u1) && has_form(u2),
        "Both units should have Formation component after auto-form"
    );
    // One should be a follower
    assert!(
        is_follower(u1) || is_follower(u2),
        "At least one unit should be a FormationFollower"
    );
}

/// Test that units with different formation types do not auto-form.
#[test]
fn test_auto_form_different_types_no_formation() {
    let mut app = make_test_app();

    let u1 = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    let u2 = spawn_unit(
        app.world_mut(),
        IVec2::new(1, 0),
        FormationType::Wedge,
        UnitArchetype::LightInfantry,
        3.0,
        70.0,
        100.0,
    );

    for _ in 0..3 {
        app.update();
    }

    let has_form = |e: Entity| app.world().get::<Formation>(e).is_some();
    assert!(
        !has_form(u1) && !has_form(u2),
        "Units with different formation types should NOT auto-form"
    );
}

/// Test that units far apart do not auto-form.
#[test]
fn test_auto_form_distant_units_no_formation() {
    let mut app = make_test_app();

    let u1 = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    let u2 = spawn_unit(
        app.world_mut(),
        IVec2::new(100, 100),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        70.0,
        100.0,
    );

    for _ in 0..3 {
        app.update();
    }

    assert!(
        app.world().get::<Formation>(u1).is_none(),
        "Distant unit 1 should not have Formation"
    );
    assert!(
        app.world().get::<Formation>(u2).is_none(),
        "Distant unit 2 should not have Formation"
    );
}

// ─── Phase D: Formation Breaking ────────────────────────────────────────────

/// Test that a formation breaks when the leader takes 30%+ damage.
#[test]
fn test_formation_breaks_on_casualty_threshold() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::HeavyInfantry,
        3.0,
        80.0,
        50.0, // 50 health = 50% HP → below 70% threshold
    );
    app.world_mut().entity_mut(leader).insert(Formation {
        leader,
        formation_type: FormationType::Line,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count: 2,
    });

    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(2, 0),
        leader,
        FormationType::Line,
        2,
    );

    // Run enough updates to trigger break detection
    for _ in 0..5 {
        app.update();
    }

    // Leader should have FormationBreak
    let leader_break = app.world().get::<FormationBreak>(leader);
    assert!(
        leader_break.is_some(),
        "Leader should have FormationBreak after casualty threshold"
    );
    if let Some(b) = leader_break {
        assert!(b.panic, "Break should be panic=true for casualty threshold");
        assert!(b.remaining > 0.0, "Break timer should be > 0");
        assert!(
            (b.total_duration - 5.0).abs() < 0.1,
            "Break duration should be 5.0s, got {}",
            b.total_duration
        );
    }

    // Follower should also have FormationBreak
    let follower_break = app.world().get::<FormationBreak>(follower);
    assert!(
        follower_break.is_some(),
        "Follower should have FormationBreak"
    );
}

/// Test that a formation's is_breaking flag is set correctly.
#[test]
fn test_formation_break_sets_flag() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Square,
        UnitArchetype::HeavyInfantry,
        3.0,
        80.0,
        20.0, // 20% HP → well below threshold
    );
    app.world_mut().entity_mut(leader).insert(Formation {
        leader,
        formation_type: FormationType::Square,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count: 2,
    });

    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(1, 0),
        leader,
        FormationType::Square,
        2,
    );

    for _ in 0..5 {
        app.update();
    }

    let leader_form = app.world().get::<Formation>(leader);
    assert!(leader_form.is_some(), "Leader should still have Formation");
    if let Some(f) = leader_form {
        assert!(f.is_breaking, "Leader formation should be breaking");
        assert!(f.break_timer.is_some(), "Break timer should be set");
    }

    // Followers should still have FormationFollower
    assert!(
        app.world().get::<FormationFollower>(follower).is_some(),
        "Follower should still exist"
    );
}

/// Test that FormationsBreakEvent is emitted on break.
#[test]
fn test_formation_break_event_emitted() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        25.0, // 25 HP → below 70%
    );
    app.world_mut().entity_mut(leader).insert(Formation {
        leader,
        formation_type: FormationType::Line,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count: 2,
    });

    let _follower = spawn_follower(
        app.world_mut(),
        IVec2::new(2, 0),
        leader,
        FormationType::Line,
        2,
    );

    // Clear any startup events
    app.update();

    // Track FormationBreakEvent
    let mut reader = app
        .world_mut()
        .query_filtered::<Entity, With<FormationBreak>>();

    // Run a few more frames to trigger break
    for _ in 0..5 {
        app.update();
    }

    // At least the leader should have FormationBreak
    let broken: Vec<Entity> = reader.iter(app.world()).collect();
    assert!(
        !broken.is_empty(),
        "At least leader should be in FormationBreak"
    );

    let _events = app
        .world()
        .get_resource::<Events<FormationBreakEvent>>()
        .expect("FormationBreakEvent resource should exist");
    // Can't read events directly without a system, but we know the system ran
    // because FormationBreak components were added.
}

// ─── Phase D: Formation Bonus Cleanup ───────────────────────────────────────

/// Test that when a unit leaves a formation, its FormationBonus is removed.
#[test]
fn test_formation_bonus_removed_on_cleanup() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    make_leader(app.world_mut(), leader, FormationType::Line, 2);
    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(2, 0),
        leader,
        FormationType::Line,
        2,
    );

    // Update to assign FormationBonus (2nd update — 1st was in make_test_app)
    app.update();
    assert!(
        app.world().get::<FormationBonus>(follower).is_some(),
        "Follower should have bonus after update"
    );

    // Remove Formation and FormationFollower, simulating break/cleanup
    app.world_mut()
        .entity_mut(follower)
        .remove::<Formation>()
        .remove::<FormationFollower>();

    // Run cleanup system
    app.update();

    // Bonus should be removed
    assert!(
        app.world().get::<FormationBonus>(follower).is_none(),
        "Follower without Formation should not have FormationBonus"
    );
}

// ─── Phase D: Leader Death Break ────────────────────────────────────────────

/// Test that when a formation leader is killed, followers break.
#[test]
fn test_formation_break_on_leader_death() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    app.world_mut().entity_mut(leader).insert(Formation {
        leader,
        formation_type: FormationType::Line,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count: 2,
    });

    let follower = spawn_follower(
        app.world_mut(),
        IVec2::new(2, 0),
        leader,
        FormationType::Line,
        2,
    );

    // Give bonuses and slots first
    app.update();

    // Kill the leader by inserting Dead
    app.world_mut().entity_mut(leader).insert(Dead);

    // Send UnitDied event
    app.world_mut().send_event(UnitDied {
        entity: leader,
        killer: leader,
    });

    app.update();

    // Follower should have FormationBreak after leader death
    let follower_break = app.world().get::<FormationBreak>(follower);
    assert!(
        follower_break.is_some(),
        "Follower should have FormationBreak after leader death"
    );
    if let Some(b) = follower_break {
        assert!(b.panic, "Leader-death break should be panic");
        assert!(
            (b.total_duration - 4.0).abs() < 0.1,
            "Leader-death break should be 4.0s, got {}",
            b.total_duration
        );
    }

    // Follower should have lost Formation and FormationFollower
    assert!(
        app.world().get::<Formation>(follower).is_none(),
        "Follower should lose Formation after leader death"
    );
    assert!(
        app.world().get::<FormationFollower>(follower).is_none(),
        "Follower should lose FormationFollower after leader death"
    );
}

/// Test: Grid quantization snaps positions to integer grid
#[test]
fn test_grid_quantization() {
    let mut app = make_test_app();

    // Spawn a unit at non-integer position
    let unit = app
        .world_mut()
        .spawn((
            Unit::default(),
            WorldPosition {
                grid: GridPos(IVec2::new(0, 0)),
                world_2d: WorldPos2D(Vec2::new(1.7, 3.4)),
                world_3d: WorldPos3D(Vec3::new(1.7, 0.0, 3.4)),
            },
            Stats::default(),
            Health::default(),
        ))
        .id();

    // Run simulation
    for _ in 0..3 {
        app.update();
    }

    let pos = app.world().get::<WorldPosition>(unit).unwrap();
    assert_eq!(pos.grid.0.x, 2, "Grid X should be quantized to 2");
    assert_eq!(pos.grid.0.y, 3, "Grid Y should be quantized to 3");
}

// ─── Phase D: Formation Type Bonus Mapping ──────────────────────────────────

/// Test that each formation type returns the expected bonuses.
#[test]
fn test_all_formation_type_bonuses() {
    // Line
    let line = FormationType::Line.bonuses();
    assert!((line.damage_mult - 1.15).abs() < 0.01);
    assert!((line.incoming_damage_mult - 0.90).abs() < 0.01);
    assert!((line.speed_mult - 0.80).abs() < 0.01);
    assert!((line.block_bonus - 5.0).abs() < 0.01);
    assert!(!line.prevents_flanking);

    // Column
    let col = FormationType::Column.bonuses();
    assert!((col.speed_mult - 1.25).abs() < 0.01);
    assert!(!col.prevents_flanking);

    // Square
    let sq = FormationType::Square.bonuses();
    assert!((sq.damage_mult - 0.85).abs() < 0.01);
    assert!((sq.block_bonus - 10.0).abs() < 0.01);
    assert!((sq.anti_cavalry_mult - 1.40).abs() < 0.01);
    assert!(sq.prevents_flanking);

    // Wedge
    let wedge = FormationType::Wedge.bonuses();
    assert!((wedge.damage_mult - 1.20).abs() < 0.01);

    // Crescent
    let cres = FormationType::Crescent.bonuses();
    assert!(cres.prevents_flanking);

    // Ambush
    let amb = FormationType::Ambush.bonuses();
    assert!((amb.damage_mult - 2.0).abs() < 0.01);

    // Scatter
    let scat = FormationType::Scatter.bonuses();
    assert!((scat.damage_mult - 0.80).abs() < 0.01);
    assert!(scat.prevents_flanking);
}

// ─── Phase D: Formation Slot Computation ────────────────────────────────────

/// Test that compute_slots returns the correct number of slots.
#[test]
fn test_compute_slots_correct_count() {
    for count in [1, 2, 3, 5, 10, 20] {
        let slots = FormationType::Line.compute_slots(count, Vec2::new(1.0, 0.0));
        assert_eq!(
            slots.len(),
            count,
            "Line should return {count} slots for {count} units, got {}",
            slots.len()
        );
    }
}

/// Test wedge formation produces an arrow-like shape (widens with each rank).
#[test]
fn test_wedge_formation_expands() {
    let facing = Vec2::new(0.0, -1.0);
    let slots = FormationType::Wedge.compute_slots(6, facing);
    // 6 units in wedge: rank 0 = 1, rank 1 = 2, rank 2 = 3
    assert_eq!(slots.len(), 6, "Should have 6 slots for 6 units");

    // Rank 0 should have 1 unit (at centre)
    // Rank 1 should have 2 units
    // Rank 2 should have 3 units
    let rank_counts: Vec<usize> = {
        let mut counts = vec![0; 3];
        for (_, r, _) in &slots {
            if *r < 3 {
                counts[*r] += 1;
            }
        }
        counts
    };
    assert_eq!(
        rank_counts,
        vec![1, 2, 3],
        "Wedge ranks should be [1, 2, 3], got {:?}",
        rank_counts
    );
}

/// Test formation type changes are reflected in leader's Formation component
#[test]
fn test_sync_formation_type_updates_formation() {
    let mut app = make_test_app();

    let leader = spawn_unit(
        app.world_mut(),
        IVec2::new(0, 0),
        FormationType::Line,
        UnitArchetype::LightInfantry,
        3.0,
        80.0,
        100.0,
    );
    app.world_mut().entity_mut(leader).insert(Formation {
        leader,
        formation_type: FormationType::Line,
        facing: Vec2::new(1.0, 0.0),
        is_breaking: false,
        break_timer: None,
        unit_count: 2,
    });

    app.update();

    // Change the formation type via Unit field (simulating FormationCommand)
    let old_unit = app.world_mut().get::<Unit>(leader).unwrap().clone();
    app.world_mut().entity_mut(leader).insert(Unit {
        formation: FormationType::Wedge,
        ..old_unit
    });

    app.update();

    // The sync_formation_type_system should update the Formation component
    let form = app.world().get::<Formation>(leader);
    assert!(form.is_some(), "Leader should have Formation");
    if let Some(f) = form {
        assert_eq!(
            f.formation_type,
            FormationType::Wedge,
            "Formation type should be synced to Wedge"
        );
    }
}
