use bevy::prelude::*;
use kurulus_core::prelude::*;

mod city_event_log;
use city_event_log::*;

mod zone_legend;
use zone_legend::*;

mod top_bar;
use top_bar::*;

mod selection_panel;
use selection_panel::*;

mod command_bar;
use command_bar::*;

mod hero_bar;
use hero_bar::*;

mod notifications;
use notifications::*;

mod objectives;
use objectives::*;

mod minimap;
use minimap::*;

mod hud_polish;
use hud_polish::*;

pub mod world_map;

// ─── Main Screen UI Modules ──────────────────────────────────────────────

mod hero_panel;
use hero_panel::*;

mod unit_stats;
use unit_stats::*;

mod action_bar;
use action_bar::*;

mod command_panel;
use command_panel::*;

mod settings_button;
use settings_button::*;

pub struct UILogicPlugin;

impl Plugin for UILogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityEventLog>()
            .init_resource::<ResourceBarState>()
            .init_resource::<NotificationStack>()
            .init_resource::<ObjectiveTracker>()
            .init_resource::<HeroBarState>()
            .init_resource::<SelectionState>()
            .init_resource::<MinimapState>()
            .init_resource::<HudSettings>()
            .init_resource::<MainScreenLayout>()
            .init_resource::<UnitStatsState>()
            .init_resource::<ActionBarState>()
            .add_systems(
                FixedUpdate,
                (
                    // Selection state is managed by `process_selection_system`
                    // in `unit_command/selection.rs` (runs in SimulationSet::InputProcessing).
                    // `selection_processing_system` is only used in tests.
                    notification_dispatch_system,
                    // City event log logic — runs after city events are generated
                    city_event_log_logic_system.in_set(SimulationSet::CityProgression),
                    // Top bar logic — reads stockpile + game time each tick
                    top_bar_logic_system,
                    // Hero bar — reads hero entities each tick
                    hero_bar_logic_system,
                    // Hero panel — reads hero entities each tick
                    hero_panel_logic_system,
                    // Unit stats — reads selected unit each tick
                    unit_stats_logic_system,
                    // Action bar — reads selection state each tick
                    action_bar_logic_system,
                    // Command panel logic (reserved for future use)
                    command_panel_logic_system,
                    // Notification stack — manages entry lifetimes
                    notification_logic_system,
                    // Minimap — reads terrain + unit positions each tick
                    minimap_logic_system,
                ),
            )
            // Render systems update the bevy_ui tree each frame.
            // Safe in headless mode (UiPlugin absent → entities invisible but harmless).
            .add_systems(
                Update,
                (
                    city_event_log_render_system,
                    zone_legend_render_system,
                    top_bar_render_system,
                    selection_panel_render_system,
                    command_bar_render_system,
                    hero_bar_render_system,
                    notification_render_system,
                    objectives_render_system,
                    minimap_render_system,
                    // Main screen panels
                    hero_panel_render_system,
                    unit_stats_render_system,
                    action_bar_render_system,
                    command_panel_render_system,
                    settings_button_render_system,
                    world_map::world_map_tooltip_system,
                    world_map::world_map_legend_system,
                    world_map::world_map_selection_panel_system,
                    // Polish systems — run after render systems so they
                    // apply settings to already-spawned entities.
                    hud_scaling_system,
                    hud_visibility_system,
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Selection Processing System
// ═══════════════════════════════════════════════════════════════════════════════

/// Processes unit/building selection events and maintains the SelectionState
/// resource. Handles single-click selection, box selection, and control groups.
pub fn selection_processing_system(
    mut select_events: EventReader<SelectEvent>,
    mut box_select_events: EventReader<BoxSelectEvent>,
    mut control_group_events: EventReader<ControlGroupEvent>,
    mut selection_state: ResMut<SelectionState>,
    mut selection_changed: EventWriter<SelectionChanged>,
) {
    let previous = selection_state.selected.clone();

    // Process single-click selection events
    for event in select_events.read() {
        match event.entity {
            Some(entity) => {
                if event.modifiers.shift {
                    // Shift-click: toggle this entity in/out of selection
                    if let Some(pos) = selection_state.selected.iter().position(|e| *e == entity) {
                        selection_state.selected.remove(pos);
                    } else {
                        selection_state.selected.push(entity);
                    }
                } else {
                    // Normal click: replace selection with this entity
                    selection_state.selected = vec![entity];
                }
            }
            None if !event.modifiers.shift => {
                // Click on empty ground without shift: deselect all
                selection_state.selected.clear();
            }
            None => {
                // Shift+click on empty ground: no change
            }
        }
    }

    // Process box-select events (drag to select multiple)
    for event in box_select_events.read() {
        if event.additive {
            // Add entities that aren't already selected
            for &entity in &event.entities {
                if !selection_state.selected.contains(&entity) {
                    selection_state.selected.push(entity);
                }
            }
        } else {
            // Replace selection with box contents
            selection_state.selected = event.entities.clone();
        }
    }

    // Process control group events
    for event in control_group_events.read() {
        match event.action {
            ControlGroupAction::Create => {
                // Clone first to avoid borrow conflict with control_groups
                let current = selection_state.selected.clone();
                selection_state.control_groups.insert(event.group, current);
            }
            ControlGroupAction::Recall => {
                if let Some(entities) = selection_state.control_groups.get(&event.group).cloned() {
                    selection_state.selected = entities;
                }
            }
            ControlGroupAction::Append => {
                if let Some(entities) = selection_state.control_groups.get(&event.group).cloned() {
                    for entity in entities {
                        if !selection_state.selected.contains(&entity) {
                            selection_state.selected.push(entity);
                        }
                    }
                }
            }
        }
    }

    // Emit SelectionChanged if the selection actually changed
    if selection_state.selected != previous {
        selection_changed.send(SelectionChanged {
            selected: selection_state.selected.clone(),
            previous,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Notification Dispatch System
// ═══════════════════════════════════════════════════════════════════════════════

/// Dispatches notifications based on game events.
/// Reads combat and mission events, emits Notification events for the
/// notification stack to pick up (Phase 3).
pub fn notification_dispatch_system(
    mut damage_events: EventReader<DamageDealt>,
    mut death_events: EventReader<UnitDied>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut notifications: EventWriter<Notification>,
) {
    // Combat damage notifications (only for significant hits)
    for event in damage_events.read() {
        if event.amount > 20.0 {
            notifications.send(Notification {
                message: format!("Unit took {:.0} damage!", event.amount),
                notification_type: NotificationType::Warning,
                duration: Some(5.0),
            });
        }
    }

    // Unit death notifications
    for _event in death_events.read() {
        notifications.send(Notification {
            message: "Unit lost!".to_string(),
            notification_type: NotificationType::Error,
            duration: Some(5.0),
        });
    }

    // Objective completion notifications
    for event in objective_events.read() {
        notifications.send(Notification {
            message: format!("Objective {:?} completed!", event.objective_id),
            notification_type: NotificationType::Success,
            duration: Some(4.0),
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    // ─── Helpers ─────────────────────────────────────────────────────────

    /// Build a minimal app with the selection processing system installed.
    fn selection_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<SelectionState>()
            .add_event::<SelectEvent>()
            .add_event::<BoxSelectEvent>()
            .add_event::<ControlGroupEvent>()
            .add_event::<SelectionChanged>()
            .add_systems(FixedUpdate, selection_processing_system);
        app
    }

    /// Build a minimal app with the notification dispatch system installed.
    fn notification_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<DamageDealt>()
            .add_event::<UnitDied>()
            .add_event::<ObjectiveCompleted>()
            .add_event::<Notification>()
            .add_systems(FixedUpdate, notification_dispatch_system);
        app
    }

    /// Run N frames (FixedUpdate + Update).
    fn run_frames(app: &mut App, n: u32) {
        for _ in 0..n {
            app.update();
        }
    }

    // ──────────────────────────────────────────────────────────────────────
    // Selection Processing Tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_select_single_entity() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();

        // Send a SelectEvent for e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });

        run_frames(&mut app, 2);

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.selected.len(), 1, "Should have one selected entity");
        assert_eq!(state.selected[0], e1, "Should select the clicked entity");
    }

    #[test]
    fn test_select_empty_ground_deselects_all() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();

        // First select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);
        assert_eq!(app.world().resource::<SelectionState>().selected.len(), 1);

        // Then click empty ground
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: None,
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        assert!(
            app.world().resource::<SelectionState>().selected.is_empty(),
            "Clicking empty ground should deselect all"
        );
    }

    #[test]
    fn test_shift_click_toggles_selection() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();

        // Select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        // Shift+click e2 to add
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e2),
                modifiers: SelectModifiers {
                    shift: true,
                    ..default()
                },
            });
        run_frames(&mut app, 2);

        assert_eq!(
            app.world().resource::<SelectionState>().selected.len(),
            2,
            "Shift+click should add to selection"
        );

        // Shift+click e1 again to remove (toggle off)
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers {
                    shift: true,
                    ..default()
                },
            });
        run_frames(&mut app, 2);

        let state = app.world().resource::<SelectionState>();
        assert_eq!(
            state.selected.len(),
            1,
            "Shift+click on already selected should toggle off"
        );
        assert_eq!(state.selected[0], e2, "Only e2 should remain selected");
    }

    #[test]
    fn test_shift_click_empty_ground_does_nothing() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();

        // Select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        // Shift+click empty ground — should NOT deselect
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: None,
                modifiers: SelectModifiers {
                    shift: true,
                    ..default()
                },
            });
        run_frames(&mut app, 2);

        assert_eq!(
            app.world().resource::<SelectionState>().selected.len(),
            1,
            "Shift+click on empty ground should not change selection"
        );
    }

    #[test]
    fn test_box_select_replaces_selection() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        let e3 = app.world_mut().spawn_empty().id();

        // First select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        // Box-select e2 and e3 (non-additive)
        app.world_mut()
            .resource_mut::<Events<BoxSelectEvent>>()
            .send(BoxSelectEvent {
                entities: vec![e2, e3],
                additive: false,
            });
        run_frames(&mut app, 2);

        let state = app.world().resource::<SelectionState>();
        assert_eq!(
            state.selected.len(),
            2,
            "Box select should replace selection"
        );
        assert!(state.selected.contains(&e2), "Box select should contain e2");
        assert!(state.selected.contains(&e3), "Box select should contain e3");
    }

    #[test]
    fn test_box_select_additive() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();

        // Select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        // Additive box-select e2
        app.world_mut()
            .resource_mut::<Events<BoxSelectEvent>>()
            .send(BoxSelectEvent {
                entities: vec![e2],
                additive: true,
            });
        run_frames(&mut app, 2);

        assert_eq!(
            app.world().resource::<SelectionState>().selected.len(),
            2,
            "Additive box select should add to existing selection"
        );
    }

    #[test]
    fn test_control_group_create_and_recall() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();

        // Select e1 and e2
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e2),
                modifiers: SelectModifiers {
                    shift: true,
                    ..default()
                },
            });
        run_frames(&mut app, 2);

        // Create control group 1
        app.world_mut()
            .resource_mut::<Events<ControlGroupEvent>>()
            .send(ControlGroupEvent {
                group: 1,
                action: ControlGroupAction::Create,
            });
        run_frames(&mut app, 2);

        // Verify group was stored
        assert!(
            app.world()
                .resource::<SelectionState>()
                .control_groups
                .contains_key(&1),
            "Control group 1 should exist"
        );
        assert_eq!(
            app.world()
                .resource::<SelectionState>()
                .control_groups
                .get(&1)
                .unwrap()
                .len(),
            2,
            "Control group 1 should have 2 entities"
        );

        // Deselect all
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: None,
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);
        assert!(
            app.world().resource::<SelectionState>().selected.is_empty(),
            "Selection should be empty after deselect"
        );

        // Recall control group 1
        app.world_mut()
            .resource_mut::<Events<ControlGroupEvent>>()
            .send(ControlGroupEvent {
                group: 1,
                action: ControlGroupAction::Recall,
            });
        run_frames(&mut app, 2);

        let state = app.world().resource::<SelectionState>();
        assert_eq!(state.selected.len(), 2, "Recall should restore 2 entities");
        assert!(state.selected.contains(&e1), "Recall should contain e1");
        assert!(state.selected.contains(&e2), "Recall should contain e2");
    }

    #[test]
    fn test_control_group_append() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();
        let e3 = app.world_mut().spawn_empty().id();

        // Store e1 and e2 in group 1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e2),
                modifiers: SelectModifiers {
                    shift: true,
                    ..default()
                },
            });
        run_frames(&mut app, 2);
        app.world_mut()
            .resource_mut::<Events<ControlGroupEvent>>()
            .send(ControlGroupEvent {
                group: 1,
                action: ControlGroupAction::Create,
            });
        run_frames(&mut app, 2);

        // Deselect, then select e3
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: None,
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e3),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);
        assert_eq!(
            app.world().resource::<SelectionState>().selected.len(),
            1,
            "Only e3 should be selected"
        );

        // Append group 1 to current selection
        app.world_mut()
            .resource_mut::<Events<ControlGroupEvent>>()
            .send(ControlGroupEvent {
                group: 1,
                action: ControlGroupAction::Append,
            });
        run_frames(&mut app, 2);

        let state = app.world().resource::<SelectionState>();
        assert_eq!(
            state.selected.len(),
            3,
            "Append should add group entities to selection"
        );
    }

    #[test]
    fn test_selection_changed_emitted() {
        let mut app = selection_test_app();
        let e1 = app.world_mut().spawn_empty().id();

        // Track SelectionChanged events
        let mut reader = app
            .world_mut()
            .resource_mut::<Events<SelectionChanged>>()
            .get_cursor();

        // Select e1
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: Some(e1),
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        let mut count = 0;
        for _ in reader.read(&app.world().resource::<Events<SelectionChanged>>()) {
            count += 1;
        }
        assert_eq!(count, 1, "SelectEvent should emit one SelectionChanged");
    }

    #[test]
    fn test_no_selection_changed_when_no_change() {
        let mut app = selection_test_app();

        // Click empty ground (starts empty, ends empty — no change)
        app.world_mut()
            .resource_mut::<Events<SelectEvent>>()
            .send(SelectEvent {
                entity: None,
                modifiers: SelectModifiers::default(),
            });
        run_frames(&mut app, 2);

        let mut count = 0;
        let mut reader = app
            .world_mut()
            .resource_mut::<Events<SelectionChanged>>()
            .get_cursor();
        for _ in reader.read(&app.world().resource::<Events<SelectionChanged>>()) {
            count += 1;
        }
        assert_eq!(
            count, 0,
            "Clicking empty ground when already empty should not emit SelectionChanged"
        );
    }

    // ──────────────────────────────────────────────────────────────────────
    // Notification Dispatch Tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn test_damage_over_threshold_emits_warning() {
        let mut app = notification_test_app();

        app.world_mut()
            .resource_mut::<Events<DamageDealt>>()
            .send(DamageDealt {
                amount: 30.0,
                ..default()
            });

        run_frames(&mut app, 2);

        let mut count = 0;
        let mut reader = app
            .world_mut()
            .resource_mut::<Events<Notification>>()
            .get_cursor();
        for ev in reader.read(&app.world().resource::<Events<Notification>>()) {
            assert_eq!(ev.notification_type, NotificationType::Warning);
            assert!(ev.message.contains("30"));
            assert!(ev.duration == Some(5.0));
            count += 1;
        }
        assert_eq!(count, 1, "Damage > 20 should emit one Warning notification");
    }

    #[test]
    fn test_small_damage_ignored() {
        let mut app = notification_test_app();

        app.world_mut()
            .resource_mut::<Events<DamageDealt>>()
            .send(DamageDealt {
                amount: 5.0, // below threshold
                ..default()
            });

        run_frames(&mut app, 2);

        let count = app.world().resource::<Events<Notification>>().len();
        assert_eq!(
            count, 0,
            "Small damage (< 20) should not emit a notification"
        );
    }

    #[test]
    fn test_unit_death_emits_error() {
        let mut app = notification_test_app();
        let e = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<Events<UnitDied>>()
            .send(UnitDied {
                entity: e,
                killer: e,
            });

        run_frames(&mut app, 2);

        let mut count = 0;
        let mut reader = app
            .world_mut()
            .resource_mut::<Events<Notification>>()
            .get_cursor();
        for ev in reader.read(&app.world().resource::<Events<Notification>>()) {
            assert_eq!(ev.notification_type, NotificationType::Error);
            assert!(ev.message.contains("Unit lost"));
            count += 1;
        }
        assert_eq!(count, 1, "UnitDied should emit one Error notification");
    }

    #[test]
    fn test_objective_completed_emits_success() {
        let mut app = notification_test_app();

        app.world_mut()
            .resource_mut::<Events<ObjectiveCompleted>>()
            .send(ObjectiveCompleted {
                objective_id: ObjectiveId::TutorialBuildOba,
                mission_id: MissionId::Mission01,
            });

        run_frames(&mut app, 2);

        let mut count = 0;
        let mut reader = app
            .world_mut()
            .resource_mut::<Events<Notification>>()
            .get_cursor();
        for ev in reader.read(&app.world().resource::<Events<Notification>>()) {
            assert_eq!(ev.notification_type, NotificationType::Success);
            assert!(
                ev.message.contains("TutorialBuildOba"),
                "Message should contain objective name: {}",
                ev.message
            );
            count += 1;
        }
        assert_eq!(
            count, 1,
            "ObjectiveCompleted should emit one Success notification"
        );
    }

    #[test]
    fn test_multiple_events_produce_multiple_notifications() {
        let mut app = notification_test_app();
        let e1 = app.world_mut().spawn_empty().id();
        let e2 = app.world_mut().spawn_empty().id();

        // Send multiple events in the same frame
        app.world_mut()
            .resource_mut::<Events<DamageDealt>>()
            .send(DamageDealt {
                amount: 35.0,
                ..default()
            });
        app.world_mut()
            .resource_mut::<Events<UnitDied>>()
            .send(UnitDied {
                entity: e1,
                killer: e2,
            });
        app.world_mut()
            .resource_mut::<Events<ObjectiveCompleted>>()
            .send(ObjectiveCompleted {
                objective_id: ObjectiveId::CaptureRegion,
                mission_id: MissionId::Mission02,
            });

        run_frames(&mut app, 2);

        let count = app.world().resource::<Events<Notification>>().len();
        assert_eq!(
            count, 3,
            "Three game events should produce three notifications"
        );
    }

    #[test]
    fn test_notification_content_format() {
        let mut app = notification_test_app();

        app.world_mut()
            .resource_mut::<Events<DamageDealt>>()
            .send(DamageDealt {
                amount: 50.0,
                ..default()
            });

        run_frames(&mut app, 2);

        let mut reader = app
            .world_mut()
            .resource_mut::<Events<Notification>>()
            .get_cursor();
        for ev in reader.read(&app.world().resource::<Events<Notification>>()) {
            assert!(
                ev.message.contains("50"),
                "Notification message should contain damage amount"
            );
            assert!(ev.duration.is_some(), "Notification should have a duration");
        }
    }
}
