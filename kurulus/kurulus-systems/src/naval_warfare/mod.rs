use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct NavalWarfarePlugin;

impl Plugin for NavalWarfarePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                naval_combat_system,
                harbor_management_system,
                blockade_system,
            ),
        );
    }
}

/// Resolves naval combat between ships
fn naval_combat_system(
    _commands: Commands,
    _ships: Query<&Ship>,
    _damage_events: EventWriter<DamageDealt>,
) {
    // TODO: Implement naval combat
}

/// Manages harbor docking, repairs, and shipbuilding
fn harbor_management_system(
    _commands: Commands,
    _harbors: Query<&mut Harbor>,
    _stockpile: Res<ResourceStockpile>,
) {
    // TODO: Implement harbor management
}

/// Processes naval blockades and their effects on trade
fn blockade_system(
    _time: Res<GameTime>,
    _blockades: Query<&BlockadeZone>,
    _trade_routes: Query<&mut TradeRoute>,
) {
    // TODO: Implement blockade effects
}
