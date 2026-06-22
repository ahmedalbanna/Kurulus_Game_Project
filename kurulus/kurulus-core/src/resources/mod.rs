pub mod economy;
pub mod tech;
pub mod time;
pub mod config;
pub mod corule;
pub mod campaign;
pub mod ai_config;
pub mod terrain;

pub use economy::*;
pub use tech::*;
pub use time::*;
pub use config::*;
pub use corule::*;
pub use campaign::*;
pub use ai_config::*;
pub use terrain::*;

pub mod prelude {
    pub use crate::resources::economy::*;
    pub use crate::resources::tech::*;
    pub use crate::resources::time::*;
    pub use crate::resources::config::*;
    pub use crate::resources::corule::*;
    pub use crate::resources::campaign::*;
    pub use crate::resources::ai_config::*;
    pub use crate::resources::terrain::*;
    pub use crate::resources::config::{
        PhaseTransitionConfig, CombatActivityTracker, OrdersIssuedDuringPlanning,
    };
}
