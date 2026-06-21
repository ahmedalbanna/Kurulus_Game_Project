use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                resource_production_system,
                resource_consumption_system,
                tax_collection_system,
            )
                .in_set(SimulationSet::ResourceProduction),
        );
    }
}

/// Produces resources based on buildings and regions
fn resource_production_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    rates: Res<ProductionRates>,
    _buildings: Query<&Building>,
) {
    let delta = time.time_scale as f32 * 0.1; // FixedUpdate tick
    stockpile.food += rates.food_per_second * delta;
    stockpile.gold += rates.gold_per_second * delta;
    stockpile.wood += rates.wood_per_second * delta;
    stockpile.stone += rates.stone_per_second * delta;
}

/// Consumes resources for unit upkeep, building maintenance
fn resource_consumption_system(
    _time: Res<GameTime>,
    _units: Query<&Unit>,
    _buildings: Query<&Building>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    // TODO: Implement resource consumption (upkeep, etc.)
}

/// Collects taxes from population periodically
fn tax_collection_system(
    _time: Res<GameTime>,
    _policy: Res<TaxPolicy>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    // TODO: Implement tax collection
}
