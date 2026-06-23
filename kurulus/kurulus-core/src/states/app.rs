use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    /// World Map / strategic overworld view
    WorldMap,
    Paused,
}
