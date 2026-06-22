use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                resource_production_system.in_set(SimulationSet::ResourceProduction),
                resource_consumption_system.in_set(SimulationSet::ResourceConsumption),
                tax_collection_system.in_set(SimulationSet::ResourceProduction),
            ),
        );
    }
}

// ── Production ─────────────────────────────────────────────────────────────

/// Produces resources based on buildings and regions.
///
/// Each tick, the configured `ProductionRates` are scaled by `delta`
/// (game-seconds per tick) and added to the stockpile.  This represents
/// passive income from farms, pastures, mines, lumber camps, trade, etc.
fn resource_production_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    rates: Res<ProductionRates>,
    _buildings: Query<&Building>,
) {
    let delta = time.time_scale as f32 * 0.1; // FixedUpdate tick ≈ 0.1 game-sec
    stockpile.food += rates.food_per_second * delta;
    stockpile.gold += rates.gold_per_second * delta;
    stockpile.wood += rates.wood_per_second * delta;
    stockpile.stone += rates.stone_per_second * delta;
}

// ── Consumption ────────────────────────────────────────────────────────────

/// Consumes food, gold, and wood each tick based on the total population.
///
/// * **Food** — scaled by the active `RationingLevel` multiplier from
///   `PolicySettings`.  Tighter rationing reduces food consumption at the
///   cost of happiness (handled by the city builder's happiness system).
/// * **Gold** — flat per-capita upkeep (luxury goods, administration,
///   tool maintenance).  Not affected by rationing.
/// * **Wood** — flat per-capita fuel and tool consumption.  Not affected
///   by rationing.
///
/// Stone is **not** consumed here — it is only drawn for construction
/// via explicit build orders.
///
/// If `PolicySettings` is not registered (e.g. headless test without the
/// city builder plugin), the system gracefully falls back to a 1.0×
/// rationing multiplier (equivalent to `RationingLevel::Normal`).
fn resource_consumption_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    rates: Res<ResourceConsumption>,
    policy: Option<Res<PolicySettings>>,
) {
    let delta = time.time_scale as f32 * 0.1;
    let pop = stockpile.population as f32;
    if pop <= 0.0 {
        return;
    }

    // ── Food (affected by rationing policy) ─────────────────────────────
    let rationing_mult = policy
        .as_deref()
        .map(|p| p.rationing.food_consumption_mult())
        .unwrap_or(1.0);
    let food_consumed = pop * rates.food_per_second_per_capita * delta * rationing_mult;
    stockpile.food = (stockpile.food - food_consumed).max(0.0);

    // ── Gold (flat rate) ────────────────────────────────────────────────
    let gold_consumed = pop * rates.gold_per_second_per_capita * delta;
    stockpile.gold = (stockpile.gold - gold_consumed).max(0.0);

    // ── Wood (flat rate) ────────────────────────────────────────────────
    let wood_consumed = pop * rates.wood_per_second_per_capita * delta;
    stockpile.wood = (stockpile.wood - wood_consumed).max(0.0);
}

// ── Tax Collection (stub) ─────────────────────────────────────────────────

/// Collects taxes from population periodically.
///
/// TODO: Full implementation — read `PolicySettings.tax_rate` (or the
/// legacy `TaxPolicy.rate`), multiply by economic output, and add gold
/// to the stockpile.  For now this is a placeholder.
fn tax_collection_system(
    _time: Res<GameTime>,
    _policy: Res<TaxPolicy>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    // TODO: Implement tax collection
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Helper: build a minimal test app, run one frame so FixedUpdate fires,
    /// and return the stockpile.
    fn run_one_tick(stockpile: ResourceStockpile) -> ResourceStockpile {
        run_with_rationing(stockpile, RationingLevel::Normal)
    }

    fn run_with_rationing(
        stockpile: ResourceStockpile,
        rationing: RationingLevel,
    ) -> ResourceStockpile {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ));
        app.init_resource::<GameTime>();
        app.insert_resource(stockpile);
        app.init_resource::<ResourceConsumption>();
        app.insert_resource(PolicySettings {
            tax_rate: 0.1,
            rationing,
            conscription: ConscriptionPolicy::Volunteer,
        });
        app.add_systems(FixedUpdate, resource_consumption_system);

        // Run several frames to ensure FixedUpdate fires at least once.
        // With fixed timestep ≈ 1/64 and manual delta of 1/60, the first
        // frame might not always trigger FixedUpdate due to accumulator rounding.
        for _ in 0..5 {
            app.update();
        }
        app.world().resource::<ResourceStockpile>().clone()
    }

    // ── Food Consumption ────────────────────────────────────────────────

    #[test]
    fn test_food_consumed_with_population() {
        let after = run_one_tick(ResourceStockpile {
            food: 100.0,
            population: 10,
            ..Default::default()
        });
        assert!(
            after.food < 100.0,
            "Food should decrease: {:.4}",
            after.food
        );
        // Each FixedUpdate tick consumes pop × rate × delta
        //                      = 10 × 0.005 × 0.1 = 0.005 per tick
        // With 5 frames we get ≥1 tick, so food < 100
        assert!(
            after.food > 99.9,
            "Consumption should be small (expected >99.9): {:.6}",
            after.food
        );
    }

    #[test]
    fn test_no_consumption_with_zero_population() {
        let after = run_one_tick(ResourceStockpile {
            food: 100.0,
            population: 0,
            ..Default::default()
        });
        assert_eq!(after.food, 100.0, "No consumption with zero population");
    }

    #[test]
    fn test_consumption_stops_at_zero() {
        let after = run_one_tick(ResourceStockpile {
            food: 0.001,
            population: 1000,
            ..Default::default()
        });
        assert!(
            after.food >= 0.0,
            "Food should not go negative: {:.4}",
            after.food
        );
    }

    #[test]
    fn test_rationing_multiplier_reduces_food_consumption() {
        let normal = run_with_rationing(
            ResourceStockpile {
                food: 100.0,
                population: 100,
                ..Default::default()
            },
            RationingLevel::Normal,
        );
        let emergency = run_with_rationing(
            ResourceStockpile {
                food: 100.0,
                population: 100,
                ..Default::default()
            },
            RationingLevel::Emergency,
        );
        assert!(
            emergency.food > normal.food,
            "Emergency rationing should consume less food: normal={:.4}, emergency={:.4}",
            normal.food,
            emergency.food,
        );
        // At Emergency (0.3×) vs Normal (1.0×), emergency leftover is strictly larger.
        let diff = emergency.food - normal.food;
        assert!(
            diff > 0.0,
            "Expected emergency > normal leftover, got diff={:.6}",
            diff,
        );
    }

    #[test]
    fn test_none_rationing_increases_consumption() {
        let normal = run_with_rationing(
            ResourceStockpile {
                food: 100.0,
                population: 100,
                ..Default::default()
            },
            RationingLevel::Normal,
        );
        let none = run_with_rationing(
            ResourceStockpile {
                food: 100.0,
                population: 100,
                ..Default::default()
            },
            RationingLevel::None,
        );
        assert!(
            none.food < normal.food,
            "\"None\" rationing should consume more food: normal={:.4}, none={:.4}",
            normal.food,
            none.food,
        );
    }

    // ── Gold Consumption ────────────────────────────────────────────────

    #[test]
    fn test_gold_consumed_with_population() {
        let after = run_one_tick(ResourceStockpile {
            gold: 100.0,
            population: 10,
            ..Default::default()
        });
        assert!(
            after.gold < 100.0,
            "Gold should decrease: {:.4}",
            after.gold
        );
        assert!(
            after.gold > 99.9,
            "Consumption should be small (expected >99.9): {:.6}",
            after.gold
        );
    }

    #[test]
    fn test_gold_not_affected_by_rationing() {
        let normal = run_with_rationing(
            ResourceStockpile {
                gold: 100.0,
                population: 50,
                ..Default::default()
            },
            RationingLevel::Normal,
        );
        let emergency = run_with_rationing(
            ResourceStockpile {
                gold: 100.0,
                population: 50,
                ..Default::default()
            },
            RationingLevel::Emergency,
        );
        assert_eq!(
            normal.gold, emergency.gold,
            "Rationing should NOT affect gold consumption"
        );
    }

    // ── Wood Consumption ────────────────────────────────────────────────

    #[test]
    fn test_wood_consumed_with_population() {
        let after = run_one_tick(ResourceStockpile {
            wood: 100.0,
            population: 10,
            ..Default::default()
        });
        assert!(
            after.wood < 100.0,
            "Wood should decrease: {:.4}",
            after.wood
        );
        assert!(
            after.wood > 99.9,
            "Consumption should be small (expected >99.9): {:.6}",
            after.wood
        );
    }

    // ── Stone Is NOT Consumed ───────────────────────────────────────────

    #[test]
    fn test_stone_not_consumed_daily() {
        let after = run_one_tick(ResourceStockpile {
            stone: 100.0,
            population: 100,
            ..Default::default()
        });
        assert_eq!(
            after.stone, 100.0,
            "Stone should NOT be consumed by the daily upkeep system"
        );
    }

    // ── PolicySettings Absent → Falls Back Gracefully ───────────────────

    #[test]
    fn test_consumption_without_policy_settings() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile {
            food: 100.0,
            population: 10,
            ..Default::default()
        });
        app.init_resource::<ResourceConsumption>();
        // Do NOT insert PolicySettings
        app.add_systems(FixedUpdate, resource_consumption_system);

        for _ in 0..5 {
            app.update();
        }
        let stockpile = app.world().resource::<ResourceStockpile>().clone();

        // Should still consume, falling back to 1.0× rationing
        assert!(
            stockpile.food < 100.0,
            "Food should decrease even without PolicySettings: {:.4}",
            stockpile.food
        );
    }

    // ── Production & Consumption Integration ────────────────────────────

    #[test]
    fn test_production_and_consumption_together() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile {
            food: 100.0,
            gold: 50.0,
            wood: 50.0,
            population: 10,
            ..Default::default()
        });
        app.init_resource::<ProductionRates>();
        app.init_resource::<ResourceConsumption>();
        app.insert_resource(PolicySettings::default());
        app.add_systems(
            FixedUpdate,
            (
                resource_production_system.in_set(SimulationSet::ResourceProduction),
                resource_consumption_system.in_set(SimulationSet::ResourceConsumption),
            ),
        );

        // Run several ticks
        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        // Production should outpace consumption at this low population (10)
        assert!(
            stockpile.food > 100.0,
            "Food should grow (production > consumption): {:.4}",
            stockpile.food
        );
        assert!(
            stockpile.gold > 50.0,
            "Gold should grow: {:.4}",
            stockpile.gold
        );
        assert!(
            stockpile.wood > 50.0,
            "Wood should grow: {:.4}",
            stockpile.wood
        );
    }
}
