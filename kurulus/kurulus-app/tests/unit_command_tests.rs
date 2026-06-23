use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use kurulus_core::prelude::*;
use kurulus_systems::register_logic_plugins;
use std::time::Duration;

// ─── Selection Tests ───────────────────────────────────────────────────────

/// Single click on a unit selects it (Selected component + SelectionState).
#[test]
fn test_single_select() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn a player unit
    let unit = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // First update to initialize event readers
    app.update();

    // Emit a select event targeting this unit
    app.world_mut().send_event(SelectEvent {
        entity: Some(unit),
        modifiers: SelectModifiers::default(),
    });

    // Run simulation to process event
    app.update();

    // Verify unit is selected
    let selection = app.world().resource::<SelectionState>();
    assert!(
        selection.selected.contains(&unit),
        "Selected unit should be in SelectionState"
    );
    assert!(
        app.world().get::<Selected>(unit).is_some(),
        "Selected unit should have Selected component"
    );
    assert_eq!(
        selection.selected.len(),
        1,
        "Exactly one entity should be selected"
    );
}

/// Clicking on empty ground deselects all entities.
#[test]
fn test_click_empty_ground_deselects() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn a unit and pre-select it
    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Manually select it (simulating prior selection)
    let mut selection = app.world_mut().resource_mut::<SelectionState>();
    selection.selected = vec![unit];
    app.world_mut().entity_mut(unit).insert(Selected);

    // First update to initialize event readers
    app.update();

    // Emit a select event with None (click on empty ground)
    app.world_mut().send_event(SelectEvent {
        entity: None,
        modifiers: SelectModifiers::default(),
    });

    app.update();

    // Verify unit is deselected
    let selection = app.world().resource::<SelectionState>();
    assert!(
        selection.selected.is_empty(),
        "Selection should be empty after clicking empty ground"
    );
    assert!(
        app.world().get::<Selected>(unit).is_none(),
        "Unit should not have Selected component after deselect"
    );
}

/// Box select adds multiple units to selection.
#[test]
fn test_box_select() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn three player units
    let mut ids = Vec::new();
    for i in 0..3 {
        let unit = app.world_mut().spawn((
            Unit::default(),
            WorldPosition::new(IVec2::new(i, 0), CELL_SIZE),
            Stats::default(),
            Health::default(),
        )).id();
        ids.push(unit);
    }

    // First update to initialize event readers
    app.update();

    // Emit a box select event containing all three
    app.world_mut().send_event(BoxSelectEvent {
        entities: ids.clone(),
        additive: false,
    });

    app.update();

    // Verify all three are selected
    let selection = app.world().resource::<SelectionState>();
    assert_eq!(selection.selected.len(), 3, "All three units should be selected");
    for id in &ids {
        assert!(
            selection.selected.contains(id),
            "Each unit should be in SelectionState"
        );
        assert!(
            app.world().get::<Selected>(*id).is_some(),
            "Each unit should have Selected component"
        );
    }
}

/// Shift+click toggles a unit in/out of selection.
#[test]
fn test_shift_click_toggle() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn two units
    let unit_a = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();
    let unit_b = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Step 0: Initialize event readers
    app.update();

    // Step 1: Select unit_a (normal click)
    app.world_mut().send_event(SelectEvent {
        entity: Some(unit_a),
        modifiers: SelectModifiers::default(),
    });
    app.update();
    assert_eq!(
        app.world().resource::<SelectionState>().selected.len(),
        1,
        "One unit selected after first click"
    );
    assert!(app.world().get::<Selected>(unit_a).is_some());

    // Step 2: Shift+click unit_b to add it
    app.world_mut().send_event(SelectEvent {
        entity: Some(unit_b),
        modifiers: SelectModifiers { shift: true, ..default() },
    });
    app.update();
    assert_eq!(
        app.world().resource::<SelectionState>().selected.len(),
        2,
        "Both units selected after shift+click"
    );
    assert!(app.world().get::<Selected>(unit_b).is_some());

    // Step 3: Shift+click unit_a to remove it
    app.world_mut().send_event(SelectEvent {
        entity: Some(unit_a),
        modifiers: SelectModifiers { shift: true, ..default() },
    });
    app.update();
    let selection = app.world().resource::<SelectionState>();
    assert_eq!(selection.selected.len(), 1, "Only unit_b remains after shift-toggle off");
    assert!(!selection.selected.contains(&unit_a), "unit_a should be deselected");
    assert!(selection.selected.contains(&unit_b), "unit_b should remain selected");
}

// ─── Control Group Tests ───────────────────────────────────────────────────

/// Create a control group (Ctrl+1), then recall it (1).
#[test]
fn test_control_group_create_recall() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn two units
    let unit_a = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();
    let unit_b = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Step 0: Initialize event readers
    app.update();

    // Step 1: Select both units
    app.world_mut().send_event(BoxSelectEvent {
        entities: vec![unit_a, unit_b],
        additive: false,
    });
    app.update();
    assert_eq!(
        app.world().resource::<SelectionState>().selected.len(),
        2,
        "Both units selected"
    );

    // Step 2: Create control group 1
    app.world_mut().send_event(ControlGroupEvent {
        group: 1,
        action: ControlGroupAction::Create,
    });
    app.update();

    let selection = app.world().resource::<SelectionState>();
    assert!(
        selection.control_groups.contains_key(&1),
        "Control group 1 should exist"
    );
    assert_eq!(
        selection.control_groups.get(&1).unwrap().len(),
        2,
        "Control group 1 should contain 2 units"
    );

    // Step 3: Deselect all
    app.world_mut().send_event(SelectEvent {
        entity: None,
        modifiers: SelectModifiers::default(),
    });
    app.update();
    assert!(
        app.world().resource::<SelectionState>().selected.is_empty(),
        "Selection cleared"
    );

    // Step 4: Recall control group 1
    app.world_mut().send_event(ControlGroupEvent {
        group: 1,
        action: ControlGroupAction::Recall,
    });
    app.update();

    let selection = app.world().resource::<SelectionState>();
    assert_eq!(selection.selected.len(), 2, "Both units restored from group");
    assert!(selection.selected.contains(&unit_a), "unit_a restored");
    assert!(selection.selected.contains(&unit_b), "unit_b restored");
    assert!(app.world().get::<Selected>(unit_a).is_some());
    assert!(app.world().get::<Selected>(unit_b).is_some());
}

/// Append a control group to an existing selection (Shift+number).
#[test]
fn test_control_group_append() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit_a = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();
    let unit_b = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Store unit_a in group 1 manually
    {
        let mut selection = app.world_mut().resource_mut::<SelectionState>();
        selection.control_groups.insert(1, vec![unit_a]);
    }

    // First update to initialize event readers
    app.update();

    // Select unit_b
    app.world_mut().send_event(SelectEvent {
        entity: Some(unit_b),
        modifiers: SelectModifiers::default(),
    });
    app.update();
    assert_eq!(
        app.world().resource::<SelectionState>().selected.len(),
        1,
        "Only unit_b selected"
    );

    // Append group 1
    app.world_mut().send_event(ControlGroupEvent {
        group: 1,
        action: ControlGroupAction::Append,
    });
    app.update();

    let selection = app.world().resource::<SelectionState>();
    assert_eq!(selection.selected.len(), 2, "Both units selected after append");
    assert!(selection.selected.contains(&unit_a), "unit_a should be in selection");
    assert!(selection.selected.contains(&unit_b), "unit_b should be in selection");
}

// ─── Dead Entity Cleanup ───────────────────────────────────────────────────

/// When a selected unit dies, it should be removed from SelectionState
/// and lose its Selected component.
#[test]
fn test_selected_dead_entity_cleanup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Spawn a unit and select it
    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        Selected,
    )).id();

    {
        let mut selection = app.world_mut().resource_mut::<SelectionState>();
        selection.selected = vec![unit];
    }

    // Also store it in a control group
    {
        let mut selection = app.world_mut().resource_mut::<SelectionState>();
        selection.control_groups.insert(3, vec![unit]);
    }

    // Mark the unit as dead
    app.world_mut().entity_mut(unit).insert(Dead);

    // Run multiple ticks to ensure cleanup fires
    for _ in 0..3 {
        app.update();
    }

    // Verify cleanup
    let selection = app.world().resource::<SelectionState>();
    assert!(
        !selection.selected.contains(&unit),
        "Dead unit should be removed from selection"
    );
    // Control group should also be cleaned
    if let Some(group) = selection.control_groups.get(&3) {
        assert!(!group.contains(&unit), "Dead unit should be removed from control groups");
    }
}

// ─── Phase B: Core Command Tests ────────────────────────────────────────────

/// Attack order inserts an AttackTarget component.
#[test]
fn test_attack_order_sets_target() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let attacker = app.world_mut().spawn((
        Unit { faction: FactionId::Kayi, ..default() },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();
    let defender = app.world_mut().spawn((
        Unit { faction: FactionId::Byzantine, ..default() },
        WorldPosition::new(IVec2::new(5, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    app.update();

    app.world_mut().send_event(AttackOrder {
        entity: attacker,
        target: defender,
        aggressive: true,
    });
    app.update();

    let target = app.world().get::<AttackTarget>(attacker);
    assert!(target.is_some(), "Attacker should have AttackTarget");
    assert_eq!(target.unwrap().target, defender, "Target should be defender");
    assert!(target.unwrap().aggressive, "Attack should be aggressive");
}

/// Stop command removes movement and attack components.
#[test]
fn test_stop_command_clears_components() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        Destination { target: GridPos(IVec2::new(10, 0)) },
        AttackTarget { target: Entity::PLACEHOLDER, aggressive: false },
        HoldPosition,
    )).id();

    app.update();

    app.world_mut().send_event(StopCommand { entity: unit });
    app.update();

    assert!(app.world().get::<Destination>(unit).is_none(), "Destination should be removed");
    assert!(app.world().get::<AttackTarget>(unit).is_none(), "AttackTarget should be removed");
    assert!(app.world().get::<HoldPosition>(unit).is_none(), "HoldPosition should be removed");
}

/// Hold command sets HoldPosition and clears movement.
#[test]
fn test_hold_command_engages_hold() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        Destination { target: GridPos(IVec2::new(10, 0)) },
    )).id();

    app.update();

    app.world_mut().send_event(HoldCommand { entity: unit });
    app.update();

    assert!(app.world().get::<HoldPosition>(unit).is_some(), "Should have HoldPosition");
    assert!(app.world().get::<Destination>(unit).is_none(), "Destination should be cleared");
}

/// Formation command updates Unit.formation.
#[test]
fn test_formation_command_changes_type() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::HeavyInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    app.update();

    app.world_mut().send_event(FormationCommand {
        entity: unit,
        formation: FormationType::Wedge,
    });
    app.update();

    let unit_comp = app.world().get::<Unit>(unit);
    assert!(unit_comp.is_some(), "Unit component should exist");
    assert_eq!(unit_comp.unwrap().formation, FormationType::Wedge, "Formation should be Wedge");
    // Other fields should be preserved
    assert_eq!(unit_comp.unwrap().archetype, UnitArchetype::HeavyInfantry, "Archetype preserved");
    assert_eq!(unit_comp.unwrap().stance, CombatStance::Aggressive, "Stance preserved");
}

/// Stance command updates Unit.stance.
#[test]
fn test_stance_command_changes_stance() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit {
            archetype: UnitArchetype::Archer,
            faction: FactionId::Kayi,
            formation: FormationType::Scatter,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    app.update();

    app.world_mut().send_event(StanceChangeCommand {
        entity: unit,
        stance: CombatStance::Aggressive,
    });
    app.update();

    let unit_comp = app.world().get::<Unit>(unit);
    assert!(unit_comp.is_some(), "Unit component should exist");
    assert_eq!(unit_comp.unwrap().stance, CombatStance::Aggressive, "Stance should be Aggressive");
    // Other fields preserved
    assert_eq!(unit_comp.unwrap().archetype, UnitArchetype::Archer, "Archetype preserved");
    assert_eq!(unit_comp.unwrap().formation, FormationType::Scatter, "Formation preserved");
}

/// Scatter command inserts a Destination away from current position.
#[test]
fn test_scatter_command_moves_unit() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    app.update();

    app.world_mut().send_event(ScatterCommand { entity: unit });
    app.update();

    let dest = app.world().get::<Destination>(unit);
    assert!(dest.is_some(), "Scatter should produce a Destination");
    // The destination should be different from (0,0) by 2–4 cells
    let target = dest.unwrap().target.0;
    let distance = ((target.x as f32).powi(2) + (target.y as f32).powi(2)).sqrt();
    assert!(distance >= 1.5, "Scatter distance should be >= ~2 cells (got {distance})");
}

/// Retreat command clears attack target and sets a Destination.
#[test]
fn test_retreat_command_clears_attack_and_moves() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        AttackTarget { target: Entity::PLACEHOLDER, aggressive: false },
    )).id();

    app.update();

    app.world_mut().send_event(RetreatCommand {
        entity: unit,
        direction: Some(Vec2::new(0.0, -1.0)), // retreat south
    });
    app.update();

    // Attack target should be removed
    assert!(app.world().get::<AttackTarget>(unit).is_none(), "AttackTarget should be cleared");

    // Destination should be set in the retreat direction
    let dest = app.world().get::<Destination>(unit);
    assert!(dest.is_some(), "Retreat should produce a Destination");
    // With direction (0, -1), the destination should be south (negative Y)
    assert!(dest.unwrap().target.0.y < 0, "Should retreat south (negative Y)");
}

/// Attack-Move inserts both Destination and AttackTargetMarker.
#[test]
fn test_attack_move_sets_destination_and_marker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    let unit = app.world_mut().spawn((
        Unit::default(),
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    app.update();

    app.world_mut().send_event(AttackMoveOrder {
        entity: unit,
        target: GridPos(IVec2::new(10, 10)),
    });
    app.update();

    let dest = app.world().get::<Destination>(unit);
    assert!(dest.is_some(), "Attack-move should produce a Destination");
    assert_eq!(dest.unwrap().target.0, IVec2::new(10, 10), "Destination should be (10,10)");

    let marker = app.world().get::<AttackTargetMarker>(unit);
    assert!(marker.is_some(), "Attack-move should have AttackTargetMarker");
}

/// Run enough updates for all systems to fire on a fresh App.
/// State-gated sets (AIDecision, AttackTimers, etc.) need 2 frames:
/// frame 1 settles state initialization, frame 2 runs the gated systems.
fn run_execution_frame(app: &mut App) {
    app.update();
    app.update();
}

// ─── Phase C: Engagement Tests ─────────────────────────────────────────────

/// Engagement movement sets Destination toward the AttackTarget.
#[test]
fn test_engagement_moves_toward_target() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Attacker at (0,0), target at (10,0) — well outside attack_range (1.0)
    let attacker = app.world_mut().spawn((
        Unit { faction: FactionId::Kayi, ..default() },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        AttackTarget {
            target: Entity::PLACEHOLDER,
            aggressive: true,
        },
    )).id();
    let defender = app.world_mut().spawn((
        Unit { faction: FactionId::Byzantine, ..default() },
        WorldPosition::new(IVec2::new(10, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Set the real target
    app.world_mut().entity_mut(attacker).insert(AttackTarget {
        target: defender,
        aggressive: true,
    });

    // Need 2 frames: engagement_movement_system is in CommandQueueing
    // which needs 1 frame for state init before it fires reliably.
    run_execution_frame(&mut app);

    let dest = app.world().get::<Destination>(attacker);
    assert!(dest.is_some(), "Engagement should set Destination toward target");
    // Destination should be toward (10,0)
    assert_eq!(dest.unwrap().target.0, IVec2::new(10, 0), "Destination should match target position");
}

/// Engagement does NOT set Destination when already in attack range.
#[test]
fn test_engagement_does_not_move_when_in_range() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Attacker at (0,0), target at (0,1) — within default attack_range (1.0)
    let attacker = app.world_mut().spawn((
        Unit { faction: FactionId::Kayi, ..default() },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats {
            attack_range: 2.0,
            ..default()
        },
        Health::default(),
        AttackTarget {
            target: Entity::PLACEHOLDER,
            aggressive: true,
        },
    )).id();
    let defender = app.world_mut().spawn((
        Unit { faction: FactionId::Byzantine, ..default() },
        WorldPosition::new(IVec2::new(0, 1), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Set real target
    app.world_mut().entity_mut(attacker).insert(AttackTarget {
        target: defender,
        aggressive: true,
    });

    run_execution_frame(&mut app);

    // Should NOT set Destination because in range
    let dest = app.world().get::<Destination>(attacker);
    assert!(dest.is_none(), "Should NOT move when target is within attack range");
}

/// HoldPosition unit auto-targets nearest enemy in range.
#[test]
fn test_hold_position_auto_targets_enemy() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Friendly unit with HoldPosition at (0,0)
    let holder = app.world_mut().spawn((
        Unit {
            faction: FactionId::Kayi,
            ..default()
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats {
            attack_range: 5.0,
            ..default()
        },
        Health::default(),
        MeleeWeapon {
            damage: 10.0,
            weapon_type: WeaponType::Sword,
            range: 1.0,
            armor_penetration: 0.0,
        },
        AttackTimer::new(1.0, TimerMode::Repeating),
        HoldPosition,
    )).id();

    // Enemy at (3,0) — within attack_range (5.0)
    let _enemy = app.world_mut().spawn((
        Unit {
            faction: FactionId::Byzantine,
            ..default()
        },
        WorldPosition::new(IVec2::new(3, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    run_execution_frame(&mut app);

    let target = app.world().get::<AttackTarget>(holder);
    assert!(target.is_some(), "HoldPosition unit should auto-acquire a target");
    assert!(target.unwrap().aggressive, "Auto-target should be aggressive");
}

/// Attack-move is interrupted when an enemy is within vision range.
#[test]
fn test_attack_move_interrupts_on_enemy_contact() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Unit at (0,0) with AttackTargetMarker (attack-moving)
    let unit = app.world_mut().spawn((
        Unit {
            faction: FactionId::Kayi,
            ..default()
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats {
            vision_range: 8.0,
            attack_range: 1.0,
            ..default()
        },
        Health::default(),
        Destination { target: GridPos(IVec2::new(20, 0)) },
        AttackTargetMarker,
    )).id();

    // Enemy at (5,0) — within vision_range (8.0)
    let _enemy = app.world_mut().spawn((
        Unit {
            faction: FactionId::Byzantine,
            ..default()
        },
        WorldPosition::new(IVec2::new(5, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    run_execution_frame(&mut app);

    // Should have removed marker, cleared Destination, and set AttackTarget
    assert!(
        app.world().get::<AttackTargetMarker>(unit).is_none(),
        "AttackTargetMarker should be removed on contact"
    );
    assert!(
        app.world().get::<Destination>(unit).is_none(),
        "Destination should be cleared on contact"
    );
    let target = app.world().get::<AttackTarget>(unit);
    assert!(target.is_some(), "Should acquire AttackTarget on enemy contact");
}

/// Target priority re-targets when current target is dead.
#[test]
fn test_target_priority_retargets_on_death() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(register_logic_plugins)
       .insert_resource(TimeUpdateStrategy::ManualDuration(
           Duration::from_secs_f32(1.0 / 60.0),
       ))
       .init_resource::<GameTime>()
       .init_resource::<ResourceStockpile>()
       .init_resource::<ProductionRates>()
       .init_resource::<TaxPolicy>()
       .init_resource::<TechTree>()
       .init_resource::<FactionRelations>();

    // Attacker at (0,0) with target (already set)
    let attacker = app.world_mut().spawn((
        Unit {
            faction: FactionId::Kayi,
            ..default()
        },
        WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        Stats {
            attack_range: 10.0,
            ..default()
        },
        Health::default(),
        AttackTarget {
            target: Entity::PLACEHOLDER, // will be updated
            aggressive: true,
        },
    )).id();

    // Enemy 1 (initially dead) at (8,0)
    let enemy1 = app.world_mut().spawn((
        Unit {
            faction: FactionId::Byzantine,
            ..default()
        },
        WorldPosition::new(IVec2::new(8, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
        Dead,
    )).id();

    // Enemy 2 (alive) at (5,0) — closer
    let _enemy2 = app.world_mut().spawn((
        Unit {
            faction: FactionId::Byzantine,
            ..default()
        },
        WorldPosition::new(IVec2::new(5, 0), CELL_SIZE),
        Stats::default(),
        Health::default(),
    )).id();

    // Set target to dead enemy1
    app.world_mut().entity_mut(attacker).insert(AttackTarget {
        target: enemy1,
        aggressive: true,
    });

    run_execution_frame(&mut app);

    // Should retarget to the alive enemy (enemy2 at (5,0))
    let target = app.world().get::<AttackTarget>(attacker);
    assert!(target.is_some(), "Should have retargeted after target died");
    assert_ne!(
        target.unwrap().target,
        enemy1,
        "Should NOT target dead enemy"
    );
}
