use bevy::prelude::*;
use kurulus_core::prelude::*;

// ─── Selection System ──────────────────────────────────────────────────────

/// Processes `SelectEvent` (single-click) and `BoxSelectEvent` (drag-select)
/// from the render layer, maintaining `SelectionState` and the `Selected`
/// component on entities.
///
/// # Logic
/// - **Single click on entity**: replaces selection (unless Shift held).
/// - **Single click on empty ground**: deselects all (unless Shift held—does nothing).
/// - **Shift+click on entity**: toggles entity in/out of selection.
/// - **Box select**: selects all entities in the rectangle. If `additive`,
///   adds to current selection; otherwise replaces.
/// - **Ctrl+A** is handled by a separate UI keybinding that emits a
///   `SelectEvent` targeting all player-owned units.
pub fn process_selection_system(
    mut commands: Commands,
    mut select_events: EventReader<SelectEvent>,
    mut box_events: EventReader<BoxSelectEvent>,
    mut selection: ResMut<SelectionState>,
    unit_query: Query<Entity, (With<Unit>, Without<Dead>)>,
) {
    for event in select_events.read() {
        if event.modifiers.shift {
            // Shift-click: toggle entity in/out of selection
            if let Some(entity) = event.entity {
                if selection.selected.contains(&entity) {
                    // Remove from selection
                    selection.selected.retain(|e| *e != entity);
                    commands.entity(entity).remove::<Selected>();
                } else if unit_query.get(entity).is_ok() {
                    // Add to selection
                    selection.selected.push(entity);
                    commands.entity(entity).insert(Selected);
                }
            }
            // Shift+click on empty ground: no-op
        } else {
            // Click without modifiers: replace selection
            clear_selection(&mut commands, &mut selection);

            if let Some(entity) = event.entity {
                if unit_query.get(entity).is_ok() {
                    selection.selected.push(entity);
                    commands.entity(entity).insert(Selected);
                }
            }
        }
    }

    for event in box_events.read() {
        if !event.additive {
            // Not additive: clear previous selection first
            clear_selection(&mut commands, &mut selection);
        }

        for entity in &event.entities {
            if unit_query.get(*entity).is_ok() && !selection.selected.contains(entity) {
                selection.selected.push(*entity);
                commands.entity(*entity).insert(Selected);
            }
        }
    }
}

// ─── Control Group System ──────────────────────────────────────────────────

/// Processes `ControlGroupEvent` to create, recall, or append control groups.
///
/// # Logic
/// - **Create** (Ctrl+number): stores current selection into the group slot.
/// - **Recall** (number press): replaces selection with the group's entities.
/// - **Append** (Shift+number): adds the group's entities to current selection.
pub fn process_control_groups_system(
    mut commands: Commands,
    mut cg_events: EventReader<ControlGroupEvent>,
    mut selection: ResMut<SelectionState>,
) {
    for event in cg_events.read() {
        let group = event.group.max(1).min(9);

        match event.action {
            ControlGroupAction::Create => {
                let group_entities = selection.selected.clone();
                selection.control_groups.insert(group, group_entities);
            }
            ControlGroupAction::Recall => {
                let group_entities = selection
                    .control_groups
                    .get(&group)
                    .cloned()
                    .unwrap_or_default();

                // Clear current selection
                clear_selection(&mut commands, &mut selection);

                // Load group entities
                for entity in &group_entities {
                    selection.selected.push(*entity);
                    commands.entity(*entity).insert(Selected);
                }
            }
            ControlGroupAction::Append => {
                let group_entities = selection
                    .control_groups
                    .get(&group)
                    .cloned()
                    .unwrap_or_default();

                // Add group entities to current selection (no duplicates)
                for entity in &group_entities {
                    if !selection.selected.contains(entity) {
                        selection.selected.push(*entity);
                        commands.entity(*entity).insert(Selected);
                    }
                }
            }
        }
    }
}

// ─── Dead Entity Cleanup ───────────────────────────────────────────────────

/// Removes dead entities from the selection state and removes their
/// `Selected` component. Runs in `ComponentCleanup` so it fires the frame
/// after `Dead` is added.
pub fn cleanup_dead_selection_system(
    mut commands: Commands,
    mut selection: ResMut<SelectionState>,
    dead_query: Query<Entity, Added<Dead>>,
) {
    for entity in dead_query.iter() {
        if selection.selected.contains(&entity) {
            selection.selected.retain(|e| *e != entity);
            commands.entity(entity).remove::<Selected>();
        }
        // Clean up control group entries as well
        for group in selection.control_groups.values_mut() {
            group.retain(|e| *e != entity);
        }
    }
}

// ─── Helpers ───────────────────────────────────────────────────────────────

/// Remove `Selected` from all currently selected entities and clear the list.
fn clear_selection(commands: &mut Commands, selection: &mut SelectionState) {
    for entity in selection.selected.drain(..) {
        commands.entity(entity).remove::<Selected>();
    }
}
