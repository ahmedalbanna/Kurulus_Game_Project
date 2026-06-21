use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct ImperialAdminPlugin;

impl Plugin for ImperialAdminPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                tax_distribution_system,
                province_management_system,
                imperial_decree_system,
            ),
        );
    }
}

/// Distributes tax income across provinces and the treasury
fn tax_distribution_system(
    _time: Res<GameTime>,
    _policy: Res<TaxPolicy>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    // TODO: Implement tax distribution
}

/// Manages province-level administration and resource allocation
fn province_management_system(_regions: Query<&Region>) {
    // TODO: Implement province management
}

/// Processes imperial decrees and their effects
fn imperial_decree_system(_commands: Commands, _events: EventReader<Notification>) {
    // TODO: Implement imperial decrees
}
