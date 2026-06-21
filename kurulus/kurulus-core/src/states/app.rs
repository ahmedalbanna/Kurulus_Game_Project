use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Paused,
}
