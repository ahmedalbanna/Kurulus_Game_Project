pub mod city;
pub mod economy;
pub mod tech;
pub mod time;
pub mod config;
pub mod corule;
pub mod campaign;
pub mod ai_config;
pub mod terrain;
pub mod world_map;
pub mod population;
pub mod phase;
pub mod selection;

pub use city::*;
pub use economy::*;
pub use tech::*;
pub use time::*;
pub use config::*;
pub use corule::*;
pub use campaign::*;
pub use ai_config::*;
pub use terrain::*;
pub use world_map::*;
pub use phase::*;
pub use selection::*;

pub mod prelude {
    pub use crate::resources::city::*;
    pub use crate::resources::economy::*;
    pub use crate::resources::tech::*;
    pub use crate::resources::time::*;
    pub use crate::resources::config::*;
    pub use crate::resources::corule::*;
    pub use crate::resources::campaign::*;
    pub use crate::resources::ai_config::*;
    pub use crate::resources::terrain::*;
    pub use crate::resources::world_map::*;
    pub use crate::resources::population::*;
    pub use crate::resources::phase::*;
    pub use crate::resources::selection::*;
    pub use crate::resources::config::{
        PhaseTransitionConfig, CombatActivityTracker, OrdersIssuedDuringPlanning,
    };
}
