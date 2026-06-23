use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct PopulationPlugin;

impl Plugin for PopulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PopulationRegistry>()
            .init_resource::<GrowthConfig>()
            .init_resource::<WorkerAllocation>()
            .init_resource::<WorkerSlotConfig>()
            .init_resource::<WorkerProductionModifier>()
            .add_event::<PopulationGrowth>()
            .add_event::<PopulationDecline>()
            .add_event::<WorkerAssigned>()
            .add_event::<WorkerUnassigned>()
            .add_event::<NeedsCritical>()
            // Phase 6a — Population
            .add_systems(
                FixedUpdate,
                (
                    population_sync_system
                        .in_set(SimulationSet::CityPopulation)
                        .before(population_growth_system),
                    population_growth_system.in_set(SimulationSet::CityPopulation),
                ),
            )
            // Phase 6b — Needs evaluation (before existing happiness system)
            .add_systems(
                FixedUpdate,
                needs_evaluation_system.in_set(SimulationSet::CityHappiness),
            )
            // Phase 6c — Worker allocation & output (within resource production)
            .add_systems(
                FixedUpdate,
                (
                    worker_allocation_system.in_set(SimulationSet::ResourceProduction),
                    worker_output_system
                        .in_set(SimulationSet::ResourceProduction)
                        .after(worker_allocation_system),
                    apply_worker_production_system
                        .in_set(SimulationSet::ResourceProduction)
                        .after(worker_output_system),
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WorkerProductionModifier (helper resource)
// ═══════════════════════════════════════════════════════════════════════════════

/// Accumulated production scaling from worker staffing, computed by
/// `worker_output_system` and applied by `apply_worker_production_system`.
#[derive(Resource, Default, Clone, Debug)]
pub struct WorkerProductionModifier {
    /// Additional food per tick (added to stockpile after base production).
    pub bonus_food: f32,
    /// Additional gold per tick.
    pub bonus_gold: f32,
    /// Additional wood per tick.
    pub bonus_wood: f32,
    /// Additional stone per tick.
    pub bonus_stone: f32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Population Sync System
// ═══════════════════════════════════════════════════════════════════════════════

/// Keeps `PopulationRegistry` in sync with `CityState`, `ResourceStockpile`,
/// and `WorkerAllocation`.
///
/// `CityState.population` is the **single source of truth** for total population.
/// This system propagates that value to all other tracking locations.
fn population_sync_system(
    city: Res<CityState>,
    allocation: Res<WorkerAllocation>,
    mut registry: ResMut<PopulationRegistry>,
) {
    registry.total = city.population;
    registry.capacity = city.max_population;
    registry.soldiers = city.total_military_pop;
    registry.workers = allocation.total_assigned;
    registry.recompute_idle();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Population Growth System
// ═══════════════════════════════════════════════════════════════════════════════

/// Drives population growth (or decline) based on citizen needs, housing capacity,
/// district bonuses, and configurable growth parameters.
///
/// Growth formula:
///
///   `satisfaction = needs.average()`
///   `growth_rate = config.base_growth_rate × satisfaction × happiness_factor`
///                  `× zone_mult × faith_weight × safety_weight`
///   `housing_dampener = 1.0 - smoothstep(housing_saturation, 1.0, pop_ratio)`
///   `final_growth = growth_rate × housing_dampener`
///
/// Starvation occurs when `food_satisfaction < config.starvation_threshold`.
fn population_growth_system(
    mut city: ResMut<CityState>,
    needs: Res<CitizenNeeds>,
    mut config: ResMut<GrowthConfig>,
    tracker: Res<DistrictTracker>,
    _game_time: Res<GameTime>,
    mut growth_writer: EventWriter<PopulationGrowth>,
    mut decline_writer: EventWriter<PopulationDecline>,
) {
    // ── Determine growth parameters ───────────────────────────────────
    let satisfaction = needs.average(); // 0.0 – 1.0
    let pop_ratio = if city.max_population > 0 {
        city.population as f32 / city.max_population as f32
    } else {
        1.0
    };

    // Happiness bonus (up to 1.5× when happiness > 80)
    let happiness_factor = if city.happiness > 80.0 {
        config.high_happiness_bonus
    } else {
        1.0
    };

    // Zone growth multiplier from Residential districts
    let zone_mult = tracker.aggregate_bonus.population_growth_mult.max(1.0);

    // Faith & safety weights
    let faith_factor = 1.0 + config.faith_growth_weight * needs.faith_satisfaction;
    let safety_factor = 1.0 + config.safety_growth_weight * needs.safety_satisfaction;

    // Housing dampener: growth slows as pop approaches max
    let housing_dampener = if pop_ratio > config.housing_saturation_threshold {
        let excess = (pop_ratio - config.housing_saturation_threshold)
            / (1.0 - config.housing_saturation_threshold);
        1.0 - excess.clamp(0.0, 1.0)
    } else {
        1.0
    };

    // ── Growth when food is adequate ──────────────────────────────────
    if city.population < city.max_population
        && needs.food_satisfaction > config.food_growth_threshold
    {
        let growth_rate = config.base_growth_rate
            * satisfaction
            * happiness_factor
            * zone_mult
            * faith_factor
            * safety_factor
            * housing_dampener;

        config.accumulator += growth_rate;

        if config.accumulator >= 1.0 {
            let gained = config.accumulator as u32;
            city.population = (city.population + gained).min(city.max_population);
            config.accumulator -= gained as f32;

            growth_writer.send(PopulationGrowth {
                amount: gained,
                total: city.population,
            });
        }
    }
    // ── Starvation when food is critically low ────────────────────────
    else if needs.food_satisfaction < config.starvation_threshold {
        let decline_rate = config.starvation_death_rate * (1.0 - needs.food_satisfaction);

        config.accumulator -= decline_rate;

        if config.accumulator <= -1.0 {
            let lost = (-config.accumulator) as u32;
            city.population = city.population.saturating_sub(lost);
            config.accumulator += lost as f32;

            decline_writer.send(PopulationDecline {
                amount: lost,
                total: city.population,
                reason: DeclineReason::Starvation,
            });
        }
    }

    // ── Clamp at max_population ──────────────────────────────────────
    city.population = city.population.min(city.max_population);

    // ── Sync ResourceStockpile population ───────────────────────────
    // (CityState.population is the canonical value; ResourceStockpile
    //  is updated here for convenience.  The economy system reads it.)
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Needs Evaluation System
// ═══════════════════════════════════════════════════════════════════════════════

/// Evaluates citizen needs each tick and updates `CitizenNeeds`.
///
/// This centralises all four need calculations (food, housing, faith, safety)
/// in one place.  The existing `city_population_system` (now replaced by
/// `population_growth_system`) and `city_happiness_system` read the results.
///
/// **Faith satisfaction** now includes a bonus from workers assigned to Mosques.
/// **Safety satisfaction** now includes a bonus from workers assigned to
/// Watchtowers (and other defence buildings).
fn needs_evaluation_system(
    mut needs: ResMut<CitizenNeeds>,
    city: Res<CityState>,
    stockpile: Res<ResourceStockpile>,
    coverage: Res<ServiceCoverage>,
    tracker: Res<DistrictTracker>,
    buildings: Query<(&Building, &WorkerSlot)>,
    mut critical_writer: EventWriter<NeedsCritical>,
) {
    // ── Food satisfaction ────────────────────────────────────────────
    let daily_consumption = (city.population as f32 * 2.0).max(1.0);
    needs.food_satisfaction = (stockpile.food / daily_consumption).clamp(0.0, 1.0);

    // ── Housing satisfaction ─────────────────────────────────────────
    needs.housing_satisfaction = if city.max_population > 0 {
        let ratio = city.population as f32 / city.max_population as f32;
        (1.0 - ratio).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ── Faith satisfaction ───────────────────────────────────────────
    // Base from religious service coverage × district faith bonus
    let faith_base = coverage.faith * tracker.aggregate_bonus.faith_generation_mult;

    // Bonus from Mosque workers: +20% per fully-staffed Mosque
    let faith_worker_bonus: f32 = buildings
        .iter()
        .filter(|(b, _)| b.building_type == BuildingType::Mosque)
        .map(|(_, slot)| {
            if slot.is_full() {
                0.20
            } else {
                slot.staffing_fraction() * 0.10
            }
        })
        .sum::<f32>()
        .min(0.5); // cap at +50%

    needs.faith_satisfaction = (faith_base + faith_worker_bonus).clamp(0.0, 1.0);

    // ── Safety satisfaction ─────────────────────────────────────────
    let defender_ratio = if city.population > 0 {
        (city.total_military_pop as f32 / city.population as f32).min(1.0)
    } else {
        0.0
    };

    // Crime reduction from Protection zones + Watchtower workers
    let crime_factor = (1.0 - city.crime / 100.0).clamp(0.0, 1.0);
    let protection_bonus = tracker.aggregate_bonus.crime_reduction / 100.0;

    // Watchtower worker bonus
    let watchtower_safety: f32 = buildings
        .iter()
        .filter(|(b, _)| b.building_type == BuildingType::Watchtower)
        .map(|(_, slot)| slot.staffing_fraction() * 0.15)
        .sum::<f32>()
        .min(0.4); // cap at +40%

    needs.safety_satisfaction =
        (crime_factor * 0.4 + defender_ratio * 0.3 + protection_bonus * 0.1 + watchtower_safety)
            .clamp(0.0, 1.0);

    // ── Emit critical-need warnings ──────────────────────────────────
    if needs.food_satisfaction < 0.2 {
        critical_writer.send(NeedsCritical {
            need_type: CriticalNeedType::Food,
            severity: needs.food_satisfaction,
        });
    }
    if needs.housing_satisfaction < 0.2 {
        critical_writer.send(NeedsCritical {
            need_type: CriticalNeedType::Housing,
            severity: needs.housing_satisfaction,
        });
    }
    if needs.faith_satisfaction < 0.2 {
        critical_writer.send(NeedsCritical {
            need_type: CriticalNeedType::Faith,
            severity: needs.faith_satisfaction,
        });
    }
    if needs.safety_satisfaction < 0.2 {
        critical_writer.send(NeedsCritical {
            need_type: CriticalNeedType::Safety,
            severity: needs.safety_satisfaction,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Worker Allocation System
// ═══════════════════════════════════════════════════════════════════════════════

/// Assigns idle workers to buildings that have open `WorkerSlot`s.
///
/// **Priority**: Buildings with higher `worker_efficiency` are staffed first
/// (better return on labour).  Within the same efficiency tier, Gathering
/// buildings are preferred over Military, then Civic, then Merchant.
///
/// Worker assignment only runs every `allocation_interval_ticks` to reduce
/// churn (controlled by `GrowthConfig.allocation_interval_ticks`).
fn worker_allocation_system(
    mut buildings: Query<(Entity, &Building, &mut WorkerSlot)>,
    mut registry: ResMut<PopulationRegistry>,
    mut allocation: ResMut<WorkerAllocation>,
    mut config: ResMut<GrowthConfig>,
    mut assigned_writer: EventWriter<WorkerAssigned>,
    _unassigned_writer: EventWriter<WorkerUnassigned>,
) {
    // ── Throttle: only allocate every N ticks ─────────────────────────
    config.allocation_cooldown += 1;
    if config.allocation_cooldown < config.allocation_interval_ticks {
        return;
    }
    config.allocation_cooldown = 0;

    // ── Recompute idle workers ───────────────────────────────────────
    registry.recompute_idle();
    let mut idle = registry.idle as i32;

    // ── Collect buildings with open slots, sorted by priority ────────
    struct OpenSlot {
        entity: Entity,
        building_type: BuildingType,
        role: WorkerRole,
        available: i32,
        efficiency: f32,
    }

    let mut open_slots: Vec<OpenSlot> = buildings
        .iter_mut()
        .filter_map(|(entity, building, slot)| {
            let available = slot.available_slots();
            if available <= 0 {
                return None;
            }
            Some(OpenSlot {
                entity,
                building_type: building.building_type,
                role: slot.required_role,
                available,
                efficiency: slot.worker_efficiency,
            })
        })
        .collect();

    // Sort by efficiency descending (best workers first)
    open_slots.sort_by(|a, b| {
        b.efficiency
            .partial_cmp(&a.efficiency)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // ── Assign workers ───────────────────────────────────────────────
    let mut total_assigned: u32 = 0;
    let mut assignments: Vec<WorkerAssignment> = Vec::new();

    for slot_info in &open_slots {
        if idle <= 0 {
            break;
        }

        let assign_count = slot_info.available.min(idle) as u32;
        if assign_count == 0 {
            continue;
        }

        idle -= assign_count as i32;
        total_assigned += assign_count;

        // Update building's WorkerSlot
        if let Ok((_entity, _building, mut slot)) = buildings.get_mut(slot_info.entity) {
            slot.assigned_workers += assign_count;
        }

        assignments.push(WorkerAssignment {
            building_entity: slot_info.entity,
            building_type: slot_info.building_type,
            worker_count: assign_count,
            role: slot_info.role,
        });

        assigned_writer.send(WorkerAssigned {
            building: slot_info.entity,
            building_type: slot_info.building_type,
            worker_count: assign_count,
            role: slot_info.role,
        });
    }

    // ── Store allocation results ─────────────────────────────────────
    allocation.assignments = assignments;
    allocation.total_assigned = total_assigned;

    // ── Update registry idle count ───────────────────────────────────
    registry.workers = total_assigned;
    registry.recompute_idle();
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Worker Output System
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the production bonus from worker staffing and stores the result
/// in `WorkerProductionModifier` for `apply_worker_production_system` to use.
///
/// For each building type, the base production per worker is defined implicitly
/// by the `WorkerSlotConfig` default values.  The effective contribution is:
///
///   `bonus = base_production × slot.production_multiplier() × delta`
///
/// The production multiplier incorporates staffing fraction, worker efficiency,
/// and the full-staff bonus.
fn worker_output_system(
    buildings: Query<(&Building, &WorkerSlot)>,
    _slot_config: Res<WorkerSlotConfig>,
    game_time: Res<GameTime>,
    mut modifier: ResMut<WorkerProductionModifier>,
) {
    let delta = game_time.time_scale as f32 * 0.1;

    // Reset modifiers
    modifier.bonus_food = 0.0;
    modifier.bonus_gold = 0.0;
    modifier.bonus_wood = 0.0;
    modifier.bonus_stone = 0.0;

    for (building, slot) in buildings.iter() {
        let multiplier = slot.production_multiplier();
        if multiplier <= 0.0 {
            continue;
        }

        // Base per-worker production rates (per tick)
        // These represent the additional output a worker contributes.
        match building.building_type {
            BuildingType::Pasture => {
                modifier.bonus_food += 0.4 * multiplier * delta;
            }
            BuildingType::MiningCamp => {
                modifier.bonus_gold += 0.3 * multiplier * delta;
            }
            BuildingType::LumberCamp => {
                modifier.bonus_wood += 0.35 * multiplier * delta;
            }
            BuildingType::Market => {
                modifier.bonus_gold += 0.25 * multiplier * delta;
            }
            BuildingType::Warehouse => {
                modifier.bonus_wood += 0.15 * multiplier * delta;
                modifier.bonus_food += 0.10 * multiplier * delta;
            }
            BuildingType::Barracks
            | BuildingType::ArcheryRange
            | BuildingType::Stable
            | BuildingType::SiegeWorkshop => {
                // Military buildings contribute to gold (weapon sales / training fees)
                modifier.bonus_gold += 0.15 * multiplier * delta;
            }
            BuildingType::CouncilChamber | BuildingType::GuildHall => {
                modifier.bonus_gold += 0.20 * multiplier * delta;
            }
            BuildingType::Mosque => {
                // Mosques do not produce resources directly,
                // but they contribute faith (handled by needs_evaluation_system).
            }
            _ => {}
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Apply Worker Production System
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies the worker production bonus computed by `worker_output_system`
/// to the resource stockpile.
fn apply_worker_production_system(
    mut stockpile: ResMut<ResourceStockpile>,
    modifier: Res<WorkerProductionModifier>,
) {
    stockpile.food += modifier.bonus_food;
    stockpile.gold += modifier.bonus_gold;
    stockpile.wood += modifier.bonus_wood;
    stockpile.stone += modifier.bonus_stone;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<CitizenNeeds>()
            .init_resource::<GrowthConfig>()
            .init_resource::<PopulationRegistry>()
            .init_resource::<WorkerAllocation>()
            .init_resource::<WorkerSlotConfig>()
            .init_resource::<WorkerProductionModifier>()
            .init_resource::<DistrictTracker>()
            .add_event::<PopulationGrowth>()
            .add_event::<PopulationDecline>()
            .add_event::<WorkerAssigned>()
            .add_event::<WorkerUnassigned>()
            .add_event::<NeedsCritical>();
        app
    }

    fn run_ticks(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── Population Sync System ───────────────────────────────────────────────

    #[test]
    fn test_population_sync_matches_city_state() {
        let mut app = test_app();
        app.insert_resource(CityState::new("TestCity", 15, 50));
        app.add_systems(FixedUpdate, population_sync_system);

        run_ticks(&mut app, 5);

        let registry = app.world().resource::<PopulationRegistry>();
        assert_eq!(registry.total, 15);
        assert_eq!(registry.capacity, 50);
    }

    #[test]
    fn test_population_sync_reflects_military_pop() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 20,
            max_population: 50,
            total_military_pop: 5,
            ..Default::default()
        });
        app.add_systems(FixedUpdate, population_sync_system);

        run_ticks(&mut app, 5);

        let registry = app.world().resource::<PopulationRegistry>();
        assert_eq!(registry.soldiers, 5);
    }

    #[test]
    fn test_population_idle_is_total_minus_workers_minus_soldiers() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 30,
            max_population: 80,
            total_military_pop: 8,
            ..Default::default()
        });
        app.insert_resource(WorkerAllocation {
            total_assigned: 12,
            ..Default::default()
        });
        app.add_systems(FixedUpdate, population_sync_system);

        run_ticks(&mut app, 5);

        let registry = app.world().resource::<PopulationRegistry>();
        // idle = 30 - 12 - 8 = 10
        assert_eq!(
            registry.idle, 10,
            "idle = total({}) - workers({}) - soldiers({})",
            registry.total, registry.workers, registry.soldiers
        );
    }

    // ── Population Growth System ─────────────────────────────────────────────

    #[test]
    fn test_population_grows_when_needs_are_met() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 10,
            max_population: 50,
            happiness: 70.0,
            ..Default::default()
        });
        app.insert_resource(CitizenNeeds {
            food_satisfaction: 0.8,
            housing_satisfaction: 0.7,
            faith_satisfaction: 0.6,
            safety_satisfaction: 0.7,
        });
        app.add_systems(FixedUpdate, population_growth_system);

        run_ticks(&mut app, 60); // ~1 sec of game time

        let city = app.world().resource::<CityState>();
        assert!(
            city.population > 10,
            "Population should grow when needs are met, got: {}",
            city.population
        );
    }

    #[test]
    fn test_starvation_reduces_population() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 10,
            max_population: 50,
            ..Default::default()
        });
        app.insert_resource(CitizenNeeds {
            food_satisfaction: 0.05,
            housing_satisfaction: 0.5,
            faith_satisfaction: 0.5,
            safety_satisfaction: 0.5,
        });
        app.add_systems(FixedUpdate, population_growth_system);

        run_ticks(&mut app, 120); // ~2 sec of game time

        let city = app.world().resource::<CityState>();
        assert!(
            city.population < 10,
            "Population should decline during starvation, got: {}",
            city.population
        );
    }

    #[test]
    fn test_population_capped_at_max_population() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 49,
            max_population: 50,
            happiness: 90.0,
            ..Default::default()
        });
        app.insert_resource(CitizenNeeds {
            food_satisfaction: 1.0,
            housing_satisfaction: 0.9,
            faith_satisfaction: 0.9,
            safety_satisfaction: 0.9,
        });
        app.add_systems(FixedUpdate, population_growth_system);

        run_ticks(&mut app, 60);

        let city = app.world().resource::<CityState>();
        assert!(
            city.population <= 50,
            "Population should not exceed max_population, got: {}",
            city.population
        );
    }

    // ── Needs Evaluation System ──────────────────────────────────────────────

    #[test]
    fn test_needs_evaluate_food_from_stockpile() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 10,
            ..Default::default()
        });
        app.insert_resource(ResourceStockpile {
            food: 100.0,
            ..Default::default()
        });
        app.insert_resource(ServiceCoverage::default());
        app.add_systems(FixedUpdate, needs_evaluation_system);

        run_ticks(&mut app, 5);

        let needs = app.world().resource::<CitizenNeeds>();
        // daily_consumption = 10 * 2 = 20, food = 100, so satisfaction = 100/20 = 1.0 (clamped)
        assert!(
            (needs.food_satisfaction - 1.0).abs() < 0.01,
            "Food satisfaction should be ~1.0 with plenty of food, got: {}",
            needs.food_satisfaction
        );
    }

    #[test]
    fn test_needs_housing_from_pop_ratio() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 10,
            max_population: 20,
            ..Default::default()
        });
        app.insert_resource(ResourceStockpile {
            food: 100.0,
            ..Default::default()
        });
        app.insert_resource(ServiceCoverage::default());
        app.add_systems(FixedUpdate, needs_evaluation_system);

        run_ticks(&mut app, 5);

        let needs = app.world().resource::<CitizenNeeds>();
        // ratio = 10/20 = 0.5, housing = 1.0 - 0.5 = 0.5
        assert!(
            (needs.housing_satisfaction - 0.5).abs() < 0.01,
            "Housing satisfaction should be ~0.5 at 50% capacity, got: {}",
            needs.housing_satisfaction
        );
    }

    // ── Worker Allocation System ─────────────────────────────────────────────

    #[test]
    fn test_workers_assigned_to_open_slots() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 20,
            max_population: 50,
            ..Default::default()
        });
        app.init_resource::<ResourceStockpile>();
        app.init_resource::<WorkerProductionModifier>();

        // Spawn a Pasture with open worker slots
        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Pasture,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 3,
                    name: "Test Pasture".into(),
                },
                WorkerSlot {
                    max_workers: 3,
                    assigned_workers: 0,
                    required_role: WorkerRole::Gatherer,
                    worker_efficiency: 0.9,
                    full_staff_bonus: 1.30,
                },
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            (
                population_sync_system.before(worker_allocation_system),
                worker_allocation_system,
            ),
        );

        run_ticks(&mut app, 10);

        let slot = app.world().get::<WorkerSlot>(building).unwrap();
        assert!(
            slot.assigned_workers > 0,
            "Workers should be assigned to Pasture, got: {}",
            slot.assigned_workers
        );

        let registry = app.world().resource::<PopulationRegistry>();
        assert!(
            registry.workers > 0,
            "Registry should track workers, got: {}",
            registry.workers
        );
    }

    #[test]
    fn test_no_over_assignment_beyond_max() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 5,
            max_population: 50,
            ..Default::default()
        });
        app.init_resource::<ResourceStockpile>();
        app.init_resource::<WorkerProductionModifier>();

        // Spawn building with max_workers = 2
        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::LumberCamp,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 3,
                    name: "Test LumberCamp".into(),
                },
                WorkerSlot {
                    max_workers: 2,
                    assigned_workers: 0,
                    required_role: WorkerRole::Gatherer,
                    worker_efficiency: 0.95,
                    full_staff_bonus: 1.20,
                },
            ))
            .id();

        app.add_systems(
            FixedUpdate,
            (
                population_sync_system.before(worker_allocation_system),
                worker_allocation_system,
            ),
        );

        run_ticks(&mut app, 20);

        let slot = app.world().get::<WorkerSlot>(building).unwrap();
        assert!(
            slot.assigned_workers <= 2,
            "Should not over-assign (max=2), got: {}",
            slot.assigned_workers
        );
    }

    // ── Worker Output System ─────────────────────────────────────────────────

    #[test]
    fn test_worker_output_adds_bonus_production() {
        let mut app = test_app();
        app.insert_resource(CityState {
            population: 20,
            max_population: 50,
            ..Default::default()
        });
        app.init_resource::<ResourceStockpile>();
        app.init_resource::<WorkerProductionModifier>();
        app.insert_resource(CityState {
            population: 20,
            ..Default::default()
        });

        // Spawn a Pasture with a fully-assigned worker
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Pasture,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Pasture".into(),
            },
            WorkerSlot {
                max_workers: 3,
                assigned_workers: 3, // fully staffed
                required_role: WorkerRole::Gatherer,
                worker_efficiency: 0.9,
                full_staff_bonus: 1.30,
            },
        ));

        app.add_systems(
            FixedUpdate,
            (
                worker_output_system,
                apply_worker_production_system.after(worker_output_system),
            ),
        );

        let food_before = app.world().resource::<ResourceStockpile>().food;

        run_ticks(&mut app, 10);

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food > food_before,
            "Worker output should add food: before={}, after={}",
            food_before,
            stockpile.food
        );
    }
}
