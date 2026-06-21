use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, States)]
pub enum MissionState {
    #[default]
    Loading,
    Active,
    Complete,
    Failed,
}
