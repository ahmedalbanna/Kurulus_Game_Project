use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::corule::DelegationOrder;

#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct CoCommandState {
    pub active: bool,
    pub primary_hero: Option<Entity>,
    pub secondary_hero: Option<Entity>,
    pub trust_matrix: TrustMatrix,
    pub delegation_history: Vec<DelegationLog>,
}

impl Default for CoCommandState {
    fn default() -> Self {
        Self {
            active: false,
            primary_hero: None,
            secondary_hero: None,
            trust_matrix: TrustMatrix::default(),
            delegation_history: Vec::new(),
        }
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct TrustMatrix {
    pub trust_values: Vec<(Entity, Entity, f32)>,
}

impl Default for TrustMatrix {
    fn default() -> Self {
        Self {
            trust_values: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct DelegationLog {
    pub order: DelegationOrder,
    pub timestamp: f32,
    pub completed: bool,
    pub success: bool,
}
