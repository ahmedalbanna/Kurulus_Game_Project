pub mod ai_config;
pub mod ambush_config;
pub mod campaign;
pub mod city;
pub mod config;
pub mod corule;
pub mod economy;
pub mod phase;
pub mod population;
pub mod selection;
pub mod tech;
pub mod terrain;
pub mod time;
pub mod world_map;

pub use ai_config::*;
pub use ambush_config::*;
pub use campaign::*;
pub use city::*;
pub use config::*;
pub use corule::*;
pub use economy::*;
pub use phase::*;
pub use selection::*;
pub use tech::*;
pub use terrain::*;
pub use time::*;
pub use world_map::*;

pub mod prelude {
    pub use crate::resources::ai_config::*;
    pub use crate::resources::ambush_config::*;
    pub use crate::resources::campaign::*;
    pub use crate::resources::city::*;
    pub use crate::resources::config::*;
    pub use crate::resources::config::{
        CombatActivityTracker, OrdersIssuedDuringPlanning, PhaseTransitionConfig,
    };
    pub use crate::resources::corule::*;
    pub use crate::resources::economy::*;
    pub use crate::resources::phase::*;
    pub use crate::resources::population::*;
    pub use crate::resources::selection::*;
    pub use crate::resources::tech::*;
    pub use crate::resources::terrain::*;
    pub use crate::resources::time::*;
    pub use crate::resources::world_map::*;
}
