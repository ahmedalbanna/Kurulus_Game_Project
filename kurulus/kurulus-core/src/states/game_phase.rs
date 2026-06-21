use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum GamePhase {
    #[default]
    Planning,
    Execution,
    Harvest,
}
