pub mod mission_editor;

use bevy::prelude::*;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(mission_editor::MissionEditorPlugin);
    }
}
