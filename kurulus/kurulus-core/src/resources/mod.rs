pub mod economy;
pub mod tech;
pub mod time;
pub mod config;
pub mod corule;

pub use economy::*;
pub use tech::*;
pub use time::*;
pub use config::*;
pub use corule::*;

pub mod prelude {
    pub use crate::resources::economy::*;
    pub use crate::resources::tech::*;
    pub use crate::resources::time::*;
    pub use crate::resources::config::*;
    pub use crate::resources::corule::*;
    pub use crate::resources::config::{
        PhaseTransitionConfig, CombatActivityTracker, OrdersIssuedDuringPlanning,
    };
}
