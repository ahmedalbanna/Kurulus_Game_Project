use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum GamePhase {
    Planning,
    #[default]
    Execution,
    Harvest,
}
