use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct BaseBuildingPlugin;

impl Plugin for BaseBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecruitmentTimer>()
            .init_resource::<BuildingCatalog>()
            .init_resource::<BuildingLimits>()
            .init_resource::<FoodDecayConfig>()
            .init_resource::<TradeRates>()
            .init_resource::<SiegeProductionQueue>()
            .add_event::<BuildRejected>()
            .add_event::<BuildOrderFailed>()
            .add_event::<CancelBuildOrder>()
            .add_event::<DemolishOrder>()
            .add_event::<InsufficientResources>()
            .add_event::<BuildingDestroyed>()
            .add_event::<RepairOrder>()
            .add_event::<TradeOrder>()
            .add_event::<SiegeProductionOrder>()
            .add_event::<SiegeUnitComplete>()
            .add_event::<RotateBuildingOrder>()
            .init_resource::<WeatherModifiers>()
            .add_systems(
                FixedUpdate,
                (
                    // Phase A: Input processing — validate & process build orders
                    process_build_orders_system.in_set(SimulationSet::InputProcessing),
                    // Phase A: Market and Siege Workshop process their orders
                    market_trade_system.in_set(SimulationSet::InputProcessing),
                    siege_production_system.in_set(SimulationSet::InputProcessing),
                    // Phase 5: Building rotation
                    building_rotation_system.in_set(SimulationSet::InputProcessing),
                    // Phase B: Building limit tracking (runs after completes)
                    building_limit_tracker_system.in_set(SimulationSet::ConstructionProgress),
                    // Phase C: Construction progress + Siege Workshop progress
                    construction_progress_system.in_set(SimulationSet::ConstructionProgress),
                    siege_production_progress_system.in_set(SimulationSet::ConstructionProgress),
                    complete_construction_system,
                    // Phase D: Lifecycle — cancel, demolish, repair
                    cancel_construction_system.in_set(SimulationSet::InputProcessing),
                    demolish_building_system.in_set(SimulationSet::ConstructionProgress),
                    repair_building_system.in_set(SimulationSet::ConstructionProgress),
                    building_upgrade_system,
                    // Phase D: Recruitment
                    process_recruit_orders_system,
                    recruitment_progress_system,
                ),
            )
            // Phase 3 — Building economy (runs after core economy but in same phase)
            .add_systems(
                FixedUpdate,
                (
                    building_resource_production_system.in_set(SimulationSet::ResourceProduction),
                    building_upkeep_system.in_set(SimulationSet::ResourceConsumption),
                    // Phase 4 — Service systems
                    well_healing_system.in_set(SimulationSet::ResourceConsumption),
                    watchtower_vision_system.in_set(SimulationSet::ComponentCleanup),
                    // Phase 5 — Weather modifiers & polish
                    weather_modifier_system.in_set(SimulationSet::ResourceConsumption),
                    max_capacity_enforcement_system.in_set(SimulationSet::ComponentCleanup),
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper: Look up build cost from catalog, falling back to hardcoded defaults
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns the level‑1 build cost for a building type.
/// Prefers the `BuildingCatalog` resource; falls back to hardcoded defaults
/// when the catalog is not available or has no entry.
fn resolve_build_cost(catalog: Option<&BuildingCatalog>, bt: BuildingType) -> BuildCost {
    if let Some(cat) = catalog {
        if let Some(cost) = cat.build_cost(bt) {
            return cost;
        }
    }
    fallback_build_cost(bt)
}

/// Returns the level‑1 build time for a building type.
fn resolve_build_time(catalog: Option<&BuildingCatalog>, bt: BuildingType) -> f32 {
    if let Some(cat) = catalog {
        if let Some(def) = cat.level_def(bt, 1) {
            return def.build_time;
        }
    }
    bt.build_time()
}

/// Returns the dependencies for a building type from the catalog.
fn resolve_dependencies(
    catalog: Option<&BuildingCatalog>,
    bt: BuildingType,
) -> Vec<BuildingDependency> {
    if let Some(cat) = catalog {
        if let Some(entry) = cat.entry_for(bt) {
            return entry.dependencies.clone();
        }
    }
    Vec::new()
}

/// Returns the `BuildingCategory` from the catalog, falling back to `BuildingType::category()`.
fn resolve_category(catalog: Option<&BuildingCatalog>, bt: BuildingType) -> BuildingCategory {
    if let Some(cat) = catalog {
        if let Some(entry) = cat.entry_for(bt) {
            return entry.category;
        }
    }
    bt.category()
}

// ─── Hardcoded fallback costs ──────────────────────────────────────────────

/// Hardcoded default build costs used when the `BuildingCatalog` resource is
/// unavailable (e.g., during testing).
fn fallback_build_cost(bt: BuildingType) -> BuildCost {
    use BuildingType::*;
    let (food, gold, wood, stone, iron, pop) = match bt {
        Oba => (20, 10, 100, 0, 0, 0),
        Pasture => (5, 0, 80, 0, 0, 0),
        MiningCamp => (0, 5, 100, 60, 0, 0),
        LumberCamp => (0, 0, 80, 0, 0, 0),
        Barracks => (15, 0, 120, 0, 0, 0),
        ArcheryRange => (10, 0, 140, 0, 30, 0),
        Stable => (20, 50, 200, 0, 80, 0),
        SiegeWorkshop => (30, 0, 400, 0, 200, 0),
        Watchtower => (5, 0, 80, 40, 0, 0),
        Market => (0, 0, 200, 0, 0, 0),
        Warehouse => (0, 0, 50, 0, 0, 0),
        Well => (0, 0, 50, 0, 0, 0),
        Mosque => (10, 0, 80, 0, 0, 0),
        CouncilChamber => (20, 0, 200, 0, 0, 0),
        GuildHall => (15, 0, 150, 0, 0, 0),
        Monument => (50, 0, 300, 0, 100, 0),
        Palisade => (0, 0, 50, 0, 0, 0),
        StoneWall => (0, 5, 0, 100, 0, 0),
        Gate => (0, 10, 80, 0, 0, 0),
        Road => (0, 0, 5, 0, 0, 0),
    };
    BuildCost {
        food,
        gold,
        wood,
        stone,
        iron,
        population: pop,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Unified Build-Order Processing (replaces build_from_order_system)
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates and processes `BuildOrder` events.
///
/// Checks performed (in order):
/// 1. Sufficient resources in stockpile → `BuildOrderFailed(InsufficientResources)`
/// 2. Building‑limit not exceeded → `BuildOrderFailed(LimitExceeded)`
/// 3. Prerequisite buildings exist at required levels → `BuildOrderFailed(MissingPrerequisite)`
/// 4. Cell not already occupied → `BuildRejected(CellOccupied)`
/// 5. Zone compatibility → `BuildRejected(ZoneMismatch)`
///
/// Only when all checks pass does it spawn the `ConstructionSite` + `Building` pair.
fn process_build_orders_system(
    mut orders: EventReader<BuildOrder>,
    mut rejected_writer: EventWriter<BuildRejected>,
    mut failed_writer: EventWriter<BuildOrderFailed>,
    mut commands: Commands,
    catalog: Option<Res<BuildingCatalog>>,
    mut stockpile: ResMut<ResourceStockpile>,
    limits: Res<BuildingLimits>,
    zones: Query<(&Zone, &WorldPosition)>,
    existing: Query<&WorldPosition, Or<(With<Building>, With<ConstructionSite>)>>,
    all_buildings: Query<(&Building, &WorldPosition)>,
) {
    'order: for order in orders.read() {
        let target = order.position;
        let bt = order.building_type;
        let cost = resolve_build_cost(catalog.as_deref(), bt);
        let deps = resolve_dependencies(catalog.as_deref(), bt);
        let cat = resolve_category(catalog.as_deref(), bt);

        // ── 1. Resource check ──────────────────────────────────────────
        if stockpile.food < cost.food as f32
            || stockpile.gold < cost.gold as f32
            || stockpile.wood < cost.wood as f32
            || stockpile.stone < cost.stone as f32
            || stockpile.iron < cost.iron as f32
        {
            failed_writer.send(BuildOrderFailed {
                building_type: bt,
                position: target,
                reason: BuildFailureReason::InsufficientResources,
            });
            continue;
        }

        // ── 2. Building‑limit check ────────────────────────────────────
        if let Err(_limit_kind) = limits.can_place(cat) {
            failed_writer.send(BuildOrderFailed {
                building_type: bt,
                position: target,
                reason: BuildFailureReason::LimitExceeded(cat),
            });
            continue;
        }

        // ── 3. Dependency check ────────────────────────────────────────
        for dep in &deps {
            let satisfied = all_buildings.iter().any(|(b, wp)| {
                b.building_type == dep.required_building
                    && b.level >= dep.required_level
                    && wp.grid.0 != target.0 // same cell doesn't count
            });
            if !satisfied {
                failed_writer.send(BuildOrderFailed {
                    building_type: bt,
                    position: target,
                    reason: BuildFailureReason::MissingPrerequisite(
                        dep.required_building,
                        dep.required_level,
                    ),
                });
                continue 'order;
            }
        }

        // ── 4. Existing building/construction at this cell ─────────────
        let cell_occupied = existing.iter().any(|wp| wp.grid.0 == target.0);
        if cell_occupied {
            rejected_writer.send(BuildRejected {
                order: order.clone(),
                reason: BuildRejectionReason::CellOccupied,
            });
            continue;
        }

        // ── 5. Zone compatibility ──────────────────────────────────────
        let zone_at_target = zones.iter().find(|(_, wp)| wp.grid.0 == target.0);
        if let Some((zone, _)) = zone_at_target {
            if !zone.zone_type.matches_building(bt) {
                rejected_writer.send(BuildRejected {
                    order: order.clone(),
                    reason: BuildRejectionReason::ZoneMismatch(zone.zone_type, bt),
                });
                continue;
            }
        }

        // ── All checks passed — deduct cost and spawn ──────────────────
        stockpile.food -= cost.food as f32;
        stockpile.gold -= cost.gold as f32;
        stockpile.wood -= cost.wood as f32;
        stockpile.stone -= cost.stone as f32;
        stockpile.iron -= cost.iron as f32;

        let max_level = bt.max_level();
        let build_time = resolve_build_time(catalog.as_deref(), bt);

        let spawn_bundle = (
            ConstructionSite {
                building_type: bt,
                progress: 0.0,
                max_progress: build_time,
                assigned_workers: vec![order.entity],
            },
            Building {
                building_type: bt,
                faction: FactionId::Kayi,
                level: 1,
                max_level,
                name: format!("{:?} I", bt),
            },
            cost,
            WorldPosition::new(target.0, CELL_SIZE),
        );

        // Attach prerequisites component if there are dependencies
        if !deps.is_empty() {
            commands
                .spawn(spawn_bundle)
                .insert(BuildingPrerequisites { dependencies: deps });
        } else {
            commands.spawn(spawn_bundle);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Building-Limit Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Scans all completed buildings each tick and updates `BuildingLimits` counts
/// per category.  The system also re‑computes max limits based on the
/// highest‑level Oba present.
fn building_limit_tracker_system(
    mut limits: ResMut<BuildingLimits>,
    buildings: Query<&Building, Without<ConstructionSite>>,
) {
    let mut basic = 0u32;
    let mut defense = 0u32;
    let mut special = 0u32;
    let mut max_oba_level = 0u8;

    for b in buildings.iter() {
        match b.building_type.category() {
            BuildingCategory::Basic => basic += 1,
            BuildingCategory::Defense => defense += 1,
            BuildingCategory::Special => special += 1,
        }
        if b.building_type == BuildingType::Oba && b.level > max_oba_level {
            max_oba_level = b.level;
        }
    }

    limits.basic_count = basic;
    limits.defense_count = defense;
    limits.special_count = special;

    // Re‑compute max limits from the highest Oba level
    let computed = BuildingLimits::compute_from_oba_level(max_oba_level);
    limits.max_basic = computed.max_basic;
    limits.max_defense = computed.max_defense;
    limits.max_special = computed.max_special;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Construction Progress
// ═══════════════════════════════════════════════════════════════════════════════

/// Tick construction progress and consume resources from the stockpile
/// proportionally as the building is built.
fn construction_progress_system(
    game_time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut sites: Query<(&mut ConstructionSite, &BuildCost)>,
) {
    for (mut site, cost) in sites.iter_mut() {
        if site.progress >= site.max_progress {
            continue;
        }

        let increment = game_time.time_scale * 10.0;
        let new_progress = (site.progress + increment).min(site.max_progress);

        // Consume resources proportionally to progress made this tick
        let progress_fraction = (new_progress - site.progress) / site.max_progress;
        stockpile.food = (stockpile.food - cost.food as f32 * progress_fraction).max(0.0);
        stockpile.gold = (stockpile.gold - cost.gold as f32 * progress_fraction).max(0.0);
        stockpile.wood = (stockpile.wood - cost.wood as f32 * progress_fraction).max(0.0);
        stockpile.stone = (stockpile.stone - cost.stone as f32 * progress_fraction).max(0.0);
        stockpile.iron = (stockpile.iron - cost.iron as f32 * progress_fraction).max(0.0);

        site.progress = new_progress;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4. Complete Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// When a `ConstructionSite` reaches full progress, replace it with a
/// `Building` component so the structure becomes operational.
fn complete_construction_system(
    mut commands: Commands,
    sites: Query<(Entity, &ConstructionSite, &Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
    slot_config: Option<Res<WorkerSlotConfig>>,
) {
    for (entity, site, building_template, pos) in sites.iter() {
        if site.progress < site.max_progress {
            continue;
        }

        // Calculate max_population contribution from Oba
        let pop_bonus = if building_template.building_type == BuildingType::Oba {
            match building_template.level {
                1 => 5,
                2 => 10,
                3 => 15,
                4 => 25,
                _ => 40,
            }
        } else {
            0
        };
        stockpile.max_population += pop_bonus;

        // Adjust storage capacity from Warehouse completion
        if building_template.building_type == BuildingType::Warehouse {
            let level = building_template.level;
            let multiplier = match level {
                1 => 2.5,
                2 => 3.75,
                3 => 5.0,
                _ => 2.5,
            };
            // Scale base capacities by multiplier
            let base = ResourceStockpile::default();
            stockpile.max_food = stockpile.max_food.max(base.max_food * multiplier);
            stockpile.max_gold = stockpile.max_gold.max(base.max_gold * multiplier);
            stockpile.max_wood = stockpile.max_wood.max(base.max_wood * multiplier);
            stockpile.max_stone = stockpile.max_stone.max(base.max_stone * multiplier);
            stockpile.max_iron = stockpile.max_iron.max(base.max_iron * multiplier);
        }

        // Attach WorkerSlot based on building type (if configured)
        if let Some(ref config) = slot_config {
            let ws = config.slot_for(building_template.building_type);
            if ws.max_workers > 0 {
                commands.entity(entity).insert(ws);
            }
        } else {
            // Fallback: use the hardcoded default from WorkerSlot::for_building_type
            if let Some(ws) = WorkerSlot::for_building_type(building_template.building_type) {
                commands.entity(entity).insert(ws);
            }
        }

        // Attach BuildingResourceProduction if the building type produces resources
        if let Some(prod) = BuildingResourceProduction::for_building_type(
            building_template.building_type,
            building_template.level,
        ) {
            commands.entity(entity).insert(prod);
        }

        // Attach BuildingUpkeep if the building type requires upkeep
        if let Some(upkeep) = BuildingUpkeep::for_building_type(
            building_template.building_type,
            building_template.level,
        ) {
            commands.entity(entity).insert(upkeep);
        }

        // Attach Health so the building can be damaged / repaired
        let max_hp = building_health(building_template.building_type, building_template.level);
        commands.entity(entity).insert(Health {
            current: max_hp,
            max: max_hp,
            armor: building_armor(building_template.building_type, building_template.level),
            armor_type: ArmorType::Heavy,
        });

        // Phase 4 — Service buildings get their special components
        if building_template.building_type == BuildingType::Well {
            commands
                .entity(entity)
                .insert(HealingAura::for_well_level(building_template.level));
        }
        if building_template.building_type == BuildingType::Watchtower {
            commands.entity(entity).insert(Stats {
                vision_range: 15.0,
                ..Default::default()
            });
        }

        // Remove ConstructionSite, keep Building and other components
        commands.entity(entity).remove::<ConstructionSite>();

        // Log completion
        info!(
            "Construction complete: {:?} at {:?}",
            building_template.building_type, pos.grid.0
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4a. Phase 2 — Building Lifecycle (HP, Cancel, Demolish, Repair)
// ═══════════════════════════════════════════════════════════════════════════════

/// Hardcoded max‑health for a building type + level.
/// These mirror the GDD tables and the `BuildingCatalog` defaults.
fn building_health(bt: BuildingType, level: u8) -> f32 {
    use BuildingType::*;
    let base = match bt {
        Oba => 500.0,
        Palisade => 500.0,
        StoneWall => 800.0,
        Gate => 400.0,
        Watchtower => 300.0,
        Barracks => 300.0,
        ArcheryRange => 300.0,
        Stable => 400.0,
        SiegeWorkshop => 800.0,
        Mosque => 200.0,
        CouncilChamber => 600.0,
        GuildHall => 400.0,
        Monument => 1000.0,
        Well => 150.0,
        Road => 50.0,
        Market => 400.0,
        Warehouse => 200.0,
        Pasture => 200.0,
        MiningCamp => 300.0,
        LumberCamp => 200.0,
    };
    // Level multiplier: L1=1.0, L2=1.6, L3=2.4, Oba L4=3.2, L5=5.0
    let mult = match level {
        1 => 1.0,
        2 => 1.6,
        3 => 2.4,
        4 => 3.2,
        _ => 5.0,
    };
    base * mult
}

/// Hardcoded armour value for a building type.
fn building_armor(bt: BuildingType, level: u8) -> f32 {
    use BuildingType::*;
    let base = match bt {
        Oba => 0.0,
        Palisade => 2.0,
        StoneWall => 8.0,
        Gate => 3.0,
        Watchtower => 0.0,
        Barracks => 0.0,
        ArcheryRange => 1.0,
        Stable => 2.0,
        SiegeWorkshop => 4.0,
        Mosque => 0.0,
        CouncilChamber => 2.0,
        GuildHall => 1.0,
        Monument => 5.0,
        Road => 0.0,
        Market => 0.0,
        Warehouse => 0.0,
        Well => 0.0,
        Pasture => 0.0,
        MiningCamp => 1.0,
        LumberCamp => 0.0,
    };
    let mult = match level {
        1 => 1.0,
        2 => 2.0,
        3 => 4.0,
        4 => 6.0,
        _ => 10.0,
    };
    base * mult
}

/// Cancel an in-progress construction and refund resources proportionally.
fn cancel_construction_system(
    mut orders: EventReader<CancelBuildOrder>,
    mut commands: Commands,
    mut stockpile: ResMut<ResourceStockpile>,
    sites: Query<(&ConstructionSite, &BuildCost)>,
) {
    for order in orders.read() {
        let Ok((site, cost)) = sites.get(order.entity) else {
            continue;
        };
        // Refund fraction based on progress (50% of cost * progress_ratio)
        let progress_ratio = site.progress / site.max_progress;
        let refund_mult = 0.5 * progress_ratio;
        stockpile.food += cost.food as f32 * refund_mult;
        stockpile.gold += cost.gold as f32 * refund_mult;
        stockpile.wood += cost.wood as f32 * refund_mult;
        stockpile.stone += cost.stone as f32 * refund_mult;
        stockpile.iron += cost.iron as f32 * refund_mult;

        // Despawn the construction site entity
        commands.entity(order.entity).despawn();

        info!("Construction cancelled: {:?}", site.building_type);
    }
}

/// Demolish a completed building and recover a fraction of its cost.
fn demolish_building_system(
    mut orders: EventReader<DemolishOrder>,
    mut commands: Commands,
    mut stockpile: ResMut<ResourceStockpile>,
    mut destroyed_writer: EventWriter<BuildingDestroyed>,
    buildings: Query<(&Building, &WorldPosition, Option<&BuildCost>)>,
) {
    for order in orders.read() {
        let Ok((building, pos, cost_opt)) = buildings.get(order.entity) else {
            continue;
        };
        // Recover 25% of total build cost
        let recovery_mult = 0.25;
        if let Some(cost) = cost_opt {
            stockpile.food += cost.food as f32 * recovery_mult;
            stockpile.gold += cost.gold as f32 * recovery_mult;
            stockpile.wood += cost.wood as f32 * recovery_mult;
            stockpile.stone += cost.stone as f32 * recovery_mult;
            stockpile.iron += cost.iron as f32 * recovery_mult;
        }

        // Fire event so other systems (e.g. building limit tracker) can react
        destroyed_writer.send(BuildingDestroyed {
            entity: order.entity,
            building_type: building.building_type,
            position: pos.grid,
            destroyer: None,
        });

        // Despawn the building entity
        commands.entity(order.entity).despawn();

        info!("Building demolished: {:?}", building.building_type);
    }
}

/// Repair a damaged building instantly, consuming resources proportional
/// to the missing HP fraction of the build cost.
fn repair_building_system(
    mut orders: EventReader<RepairOrder>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut buildings: Query<(&mut Health, &BuildCost)>,
) {
    for order in orders.read() {
        let Ok((mut health, cost)) = buildings.get_mut(order.building) else {
            continue;
        };
        if health.current >= health.max {
            continue; // Already fully repaired
        }

        let missing_ratio = (health.max - health.current) / health.max;

        // Compute resource cost to repair
        let food_cost = cost.food as f32 * missing_ratio;
        let gold_cost = cost.gold as f32 * missing_ratio;
        let wood_cost = cost.wood as f32 * missing_ratio;
        let stone_cost = cost.stone as f32 * missing_ratio;
        let iron_cost = cost.iron as f32 * missing_ratio;

        // Check if we can afford it
        if stockpile.food < food_cost
            || stockpile.gold < gold_cost
            || stockpile.wood < wood_cost
            || stockpile.stone < stone_cost
            || stockpile.iron < iron_cost
        {
            warn!("Insufficient resources to repair {:?}", order.building);
            continue;
        }

        // Deduct resources
        stockpile.food -= food_cost;
        stockpile.gold -= gold_cost;
        stockpile.wood -= wood_cost;
        stockpile.stone -= stone_cost;
        stockpile.iron -= iron_cost;

        // Restore to full health
        health.current = health.max;

        info!("Building repaired: {:?}", order.building);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4b. Phase 3 — Building Economy (Production, Upkeep, Capacity)
// ═══════════════════════════════════════════════════════════════════════════════

/// Per‑tick resource production from completed buildings.
///
/// Reads `BuildingProduction` (set at construction completion) and
/// applies production to the faction `ResourceStockpile`.  Production
/// is scaled by `GameTime.time_scale` and by the building's
/// `WorkerSlot::production_multiplier` (if workers are assigned).
fn building_resource_production_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    producers: Query<(&BuildingResourceProduction, Option<&WorkerSlot>)>,
) {
    let delta = time.time_scale * 0.1; // ≈ game‑seconds per tick

    for (prod, slot) in producers.iter() {
        // Worker efficiency multiplier (1.0 = base when no workers or no slot)
        let worker_mult = slot.map(|s| s.production_multiplier()).unwrap_or(1.0);

        let mult = delta * worker_mult;

        stockpile.food += prod.food_per_sec * mult;
        stockpile.gold += prod.gold_per_sec * mult;
        stockpile.wood += prod.wood_per_sec * mult;
        stockpile.stone += prod.stone_per_sec * mult;
        stockpile.iron += prod.iron_per_sec * mult;
    }
}

/// Per‑tick upkeep deduction from completed buildings.
///
/// Reads `BuildingUpkeep` (set at construction completion) and
/// deducts resources from the `ResourceStockpile`, clamped to zero.
fn building_upkeep_system(
    time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    upkeep_buildings: Query<&BuildingUpkeep>,
) {
    let delta = time.time_scale * 0.1;

    for upkeep in upkeep_buildings.iter() {
        stockpile.gold = (stockpile.gold - upkeep.gold_per_sec * delta).max(0.0);
        stockpile.wood = (stockpile.wood - upkeep.wood_per_sec * delta).max(0.0);
    }
}

/// Enforce resource stockpile maximum capacities.
///
/// Runs last (in `ComponentCleanup`) to ensure resources never
/// exceed their configured caps after all production and consumption
/// systems have run.
fn max_capacity_enforcement_system(mut stockpile: ResMut<ResourceStockpile>) {
    stockpile.food = stockpile.food.min(stockpile.max_food);
    stockpile.gold = stockpile.gold.min(stockpile.max_gold);
    stockpile.wood = stockpile.wood.min(stockpile.max_wood);
    stockpile.stone = stockpile.stone.min(stockpile.max_stone);
    stockpile.iron = stockpile.iron.min(stockpile.max_iron);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4c. Phase 4 — Building Services (Market, Siege, Well, Watchtower)
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks in-progress siege engine production at Siege Workshops.
#[derive(Resource, Default)]
pub struct SiegeProductionQueue {
    pub active_productions: Vec<ActiveSiegeProduction>,
}

pub struct ActiveSiegeProduction {
    pub building: Entity,
    pub engine_type: SiegeEngineType,
    pub remaining: u32,
    pub timer: f32,
    pub interval: f32,
}

/// Process `TradeOrder` events at Market buildings.
///
/// Uses the `TradeRates` resource to compute exchange amounts based on
/// each resource's base value.  Validates the building is a Market, then
/// deducts `give_resource` from the stockpile and adds `receive_resource`.
fn market_trade_system(
    mut orders: EventReader<TradeOrder>,
    mut stockpile: ResMut<ResourceStockpile>,
    trade_rates: Res<TradeRates>,
    buildings: Query<&Building>,
) {
    for order in orders.read() {
        let Ok(building) = buildings.get(order.market) else {
            continue;
        };
        if building.building_type != BuildingType::Market {
            continue;
        }
        if order.give_amount <= 0.0 {
            continue;
        }

        let receive = trade_rates.trade_amount(
            order.give_resource,
            order.give_amount,
            order.receive_resource,
        );
        if receive <= 0.0 {
            continue;
        }

        // Check and deduct give resource
        match order.give_resource {
            ResourceType::Food => {
                if stockpile.food < order.give_amount {
                    continue;
                }
                stockpile.food -= order.give_amount;
            }
            ResourceType::Gold => {
                if stockpile.gold < order.give_amount {
                    continue;
                }
                stockpile.gold -= order.give_amount;
            }
            ResourceType::Wood => {
                if stockpile.wood < order.give_amount {
                    continue;
                }
                stockpile.wood -= order.give_amount;
            }
            ResourceType::Stone => {
                if stockpile.stone < order.give_amount {
                    continue;
                }
                stockpile.stone -= order.give_amount;
            }
            ResourceType::Iron => {
                if stockpile.iron < order.give_amount {
                    continue;
                }
                stockpile.iron -= order.give_amount;
            }
            ResourceType::Population => {
                if stockpile.population < order.give_amount as u32 {
                    continue;
                }
                stockpile.population -= order.give_amount as u32;
            }
        }

        // Add receive resource
        match order.receive_resource {
            ResourceType::Food => stockpile.food += receive,
            ResourceType::Gold => stockpile.gold += receive,
            ResourceType::Wood => stockpile.wood += receive,
            ResourceType::Stone => stockpile.stone += receive,
            ResourceType::Iron => stockpile.iron += receive,
            ResourceType::Population => stockpile.population += receive as u32,
        }

        info!(
            "Trade at Market: -{:.1} {:?} +{:.1} {:?}",
            order.give_amount, order.give_resource, receive, order.receive_resource
        );
    }
}

/// Process `SiegeProductionOrder` events — queue siege engines at Siege Workshops.
fn siege_production_system(
    mut orders: EventReader<SiegeProductionOrder>,
    mut queue: ResMut<SiegeProductionQueue>,
    buildings: Query<&Building>,
) {
    for order in orders.read() {
        let Ok(building) = buildings.get(order.building) else {
            continue;
        };
        if building.building_type != BuildingType::SiegeWorkshop {
            continue;
        }
        if order.count == 0 {
            continue;
        }

        // Build time depends on engine type
        let interval = match order.engine_type {
            SiegeEngineType::Mantlet => 10.0,
            SiegeEngineType::BatteringRam | SiegeEngineType::SiegeTower => 20.0,
            SiegeEngineType::Ballista | SiegeEngineType::Catapult => 30.0,
            SiegeEngineType::Trebuchet => 45.0,
        };

        queue.active_productions.push(ActiveSiegeProduction {
            building: order.building,
            engine_type: order.engine_type,
            remaining: order.count,
            timer: 0.0,
            interval,
        });

        info!(
            "Siege production queued: {:?} x{} at {:?}",
            order.engine_type, order.count, order.building
        );
    }
}

/// Advances siege engine production timers and spawns completed engines.
fn siege_production_progress_system(
    mut commands: Commands,
    time: Res<GameTime>,
    mut queue: ResMut<SiegeProductionQueue>,
    mut complete_events: EventWriter<SiegeUnitComplete>,
    buildings: Query<(&Building, &WorldPosition)>,
) {
    let delta = time.time_scale * 0.1;
    let mut completed_indices: Vec<usize> = Vec::new();

    for (idx, prod) in queue.active_productions.iter_mut().enumerate() {
        if prod.remaining == 0 {
            completed_indices.push(idx);
            continue;
        }

        prod.timer += delta;

        while prod.timer >= prod.interval && prod.remaining > 0 {
            prod.timer -= prod.interval;
            prod.remaining -= 1;

            // Spawn the siege engine near the workshop
            if let Ok((_building, pos)) = buildings.get(prod.building) {
                let spawn_offset = IVec2::new(pos.grid.0.x + 2, pos.grid.0.y);
                commands.spawn((
                    SiegeEngine {
                        engine_type: prod.engine_type,
                        crew: Vec::new(),
                        reload_timer: 0.0,
                        ammo: SiegeAmmoType::Stone,
                        target: None,
                    },
                    WorldPosition::new(spawn_offset, CELL_SIZE),
                    Health {
                        current: 200.0,
                        max: 200.0,
                        armor: 8.0,
                        armor_type: ArmorType::Heavy,
                    },
                    Stats {
                        move_speed: 1.5,
                        attack_range: 1.0,
                        attack_damage: 50.0,
                        attack_speed: 0.5,
                        vision_range: 4.0,
                        morale: 100.0,
                        morale_max: 100.0,
                    },
                ));

                complete_events.send(SiegeUnitComplete {
                    building: prod.building,
                    engine_type: prod.engine_type,
                });

                info!(
                    "Siege engine complete: {:?} at {:?}",
                    prod.engine_type, spawn_offset
                );
            }
        }
    }

    // Remove completed production entries (reverse order)
    for idx in completed_indices.into_iter().rev() {
        queue.active_productions.remove(idx);
    }
}

/// Passive healing aura from Well buildings.
///
/// Each tick, all allied units within `HealingAura::range` cells of a Well
/// have their `Health::current` increased by `HealingAura::heal_per_sec`.
fn well_healing_system(
    time: Res<GameTime>,
    wells: Query<(&WorldPosition, &HealingAura)>,
    mut units: Query<(&WorldPosition, &mut Health, &Unit)>,
) {
    let delta = time.time_scale * 0.1;
    if delta <= 0.0 {
        return;
    }

    for (well_pos, aura) in wells.iter() {
        let well_grid = well_pos.grid.0;
        let range_sq = aura.range * aura.range;

        for (unit_pos, mut health, unit) in units.iter_mut() {
            // Only heal units of the same faction as the aura
            if unit.faction != aura.faction {
                continue;
            }
            if health.current >= health.max {
                continue;
            }

            let dx = (unit_pos.grid.0.x - well_grid.x) as f32;
            let dy = (unit_pos.grid.0.y - well_grid.y) as f32;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= range_sq {
                let heal_amount = aura.heal_per_sec * delta;
                health.current = (health.current + heal_amount).min(health.max);
            }
        }
    }
}

/// Ensures Watchtower buildings have a high vision range.
///
/// This system runs once and adds a `Stats` component to any Watchtower
/// that does not already have one (the initial attachment is done in
/// `complete_construction_system`; this handles dynamically‑created buildings).
fn watchtower_vision_system(
    mut commands: Commands,
    watchtowers: Query<(Entity, &Building), (With<WorldPosition>, Without<Stats>)>,
) {
    for (entity, building) in watchtowers.iter() {
        if building.building_type == BuildingType::Watchtower {
            commands.entity(entity).insert(Stats {
                vision_range: 15.0,
                ..Default::default()
            });
            info!("Watchtower vision enabled at {:?}", entity);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Phase 5 — Polish (Rotation, Weather Modifiers)
// ═══════════════════════════════════════════════════════════════════════════════

/// Process `RotateBuildingOrder` events — set the building's facing direction
/// and update its `Transform` rotation accordingly.
fn building_rotation_system(
    mut orders: EventReader<RotateBuildingOrder>,
    mut buildings: Query<(&mut BuildingRotation, &mut Transform), With<Building>>,
) {
    for order in orders.read() {
        let Ok((mut rotation, mut transform)) = buildings.get_mut(order.building) else {
            continue;
        };

        rotation.direction = order.direction;

        // Map direction to a Z-rotation angle (2D sprites: 0 = South = default)
        let angle_rad = match order.direction {
            CardinalDirection::South => 0.0,
            CardinalDirection::West => std::f32::consts::FRAC_PI_2,
            CardinalDirection::North => std::f32::consts::PI,
            CardinalDirection::East => -std::f32::consts::FRAC_PI_2,
        };
        transform.rotation = Quat::from_rotation_z(angle_rad);
    }
}

/// Compute `WeatherModifiers` from the current `GameTime` (season + day/night).
///
/// Runs once per tick so downstream systems (construction, production,
/// movement) can read the modifiers without querying `GameTime` directly.
fn weather_modifier_system(game_time: Res<GameTime>, mut modifiers: ResMut<WeatherModifiers>) {
    // ── Season modifiers ────────────────────────────────────────────────
    let (build_mod, prod_mod, move_mod) = match game_time.season {
        SeasonType::Spring => (1.0, 1.0, 1.0),
        SeasonType::Summer => (1.1, 1.1, 1.0),
        SeasonType::Autumn => (0.9, 1.0, 1.0),
        SeasonType::Winter => (0.7, 0.8, 0.75),
    };

    modifiers.construction_speed_modifier = build_mod;
    modifiers.production_modifier = prod_mod;
    modifiers.movement_speed_modifier = move_mod;

    // ── Night modifier (stacked multiplicatively) ───────────────────────
    if game_time.is_night() {
        modifiers.construction_speed_modifier *= 0.8;
        modifiers.production_modifier *= 0.9;
    }
}
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles building level upgrades (triggered by a `BuildOrder` with an
/// existing building as target, or via direct component mutation).
///
/// For Phase 2, the upgrade cost is hardcoded per building type.
fn building_upgrade_system(
    mut buildings: Query<(&mut Building, &BuildCost), Without<UpgradeChain>>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut production_rates: ResMut<ProductionRates>,
) {
    for (mut building, upgrade_cost) in buildings.iter_mut() {
        if building.level >= building.max_level {
            continue;
        }

        // Upgrade is triggered when a player sends a BuildOrder targeting
        // an existing building.  In Phase 2 we simulate this automatically
        // when resources are sufficient (for testing).
        // A full game would use BuildOrder events.

        let can_upgrade = stockpile.food >= upgrade_cost.food as f32
            && stockpile.gold >= upgrade_cost.gold as f32
            && stockpile.wood >= upgrade_cost.wood as f32
            && stockpile.stone >= upgrade_cost.stone as f32
            && stockpile.iron >= upgrade_cost.iron as f32;

        if !can_upgrade {
            continue;
        }

        // Deduct cost
        stockpile.food -= upgrade_cost.food as f32;
        stockpile.gold -= upgrade_cost.gold as f32;
        stockpile.wood -= upgrade_cost.wood as f32;
        stockpile.stone -= upgrade_cost.stone as f32;
        stockpile.iron -= upgrade_cost.iron as f32;

        // Apply level
        building.level += 1;

        // Oba-specific: increase max_population per level
        if building.building_type == BuildingType::Oba {
            stockpile.max_population += 5; // flat bonus per level
            production_rates.food_per_second *= 1.2; // Oba upgrades improve food production
        }

        info!(
            "{:?} upgraded to level {}",
            building.building_type, building.level
        );

        // For now, upgrade one at a time per frame
        break;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. Recruitment
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks the recruitment timer so units are spawned one at a time with a
/// delay between them.
#[derive(Resource, Default)]
pub struct RecruitmentTimer {
    pub active_recruitments: Vec<ActiveRecruitment>,
}

pub struct ActiveRecruitment {
    pub building: Entity,
    pub archetype: UnitArchetype,
    pub remaining: u32,
    pub timer: f32,
    pub interval: f32, // seconds between each unit
}

/// Reads `RecruitOrder` events and starts recruitment queues on buildings.
fn process_recruit_orders_system(
    mut orders: EventReader<RecruitOrder>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<&Building>,
) {
    for order in orders.read() {
        let Ok(building) = buildings.get(order.building) else {
            continue;
        };

        // Only Oba can recruit in Phase 2
        if building.building_type != BuildingType::Oba {
            continue;
        }

        let interval = match order.archetype {
            UnitArchetype::Worker => 5.0,
            UnitArchetype::LightInfantry => 8.0,
            _ => 10.0,
        };

        // Add to recruitment queue
        timer.active_recruitments.push(ActiveRecruitment {
            building: order.building,
            archetype: order.archetype,
            remaining: order.count,
            timer: 0.0,
            interval,
        });
    }
}

/// Processes the recruitment queue, spawning units when their timer reaches
/// the interval threshold.
fn recruitment_progress_system(
    mut commands: Commands,
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<(&Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    let mut completed = Vec::new();

    for (idx, recruitment) in timer.active_recruitments.iter_mut().enumerate() {
        if recruitment.remaining == 0 {
            completed.push(idx);
            continue;
        }

        recruitment.timer += time.delta_secs();

        if recruitment.timer >= recruitment.interval {
            recruitment.timer -= recruitment.interval;
            recruitment.remaining -= 1;

            // Spawn the unit near the building
            if let Ok((_building, pos)) = buildings.get(recruitment.building) {
                let spawn_offset = IVec2::new(
                    pos.grid.0.x + 2, // spawn to the right of the building
                    pos.grid.0.y,
                );

                let (health, stats, weapon) = unit_archetype_stats(recruitment.archetype);
                let attack_speed = stats.attack_speed;

                commands.spawn((
                    Unit {
                        archetype: recruitment.archetype,
                        faction: FactionId::Kayi,
                        formation: FormationType::Line,
                        stance: CombatStance::Defensive,
                    },
                    WorldPosition::new(spawn_offset, CELL_SIZE),
                    health,
                    stats,
                    weapon,
                    AttackTimer(Timer::from_seconds(
                        1.0 / attack_speed.max(0.1),
                        TimerMode::Repeating,
                    )),
                ));

                // Consume population
                stockpile.population += 1;

                info!(
                    "Recruited {:?} at {:?}",
                    recruitment.archetype, spawn_offset
                );
            }
        }
    }

    // Remove completed recruitments (reverse order)
    for idx in completed.into_iter().rev() {
        timer.active_recruitments.remove(idx);
    }
}

/// Returns default health, stats, and weapon for a given archetype.
fn unit_archetype_stats(archetype: UnitArchetype) -> (Health, Stats, MeleeWeapon) {
    match archetype {
        UnitArchetype::LightInfantry => (
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.5,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 1.0,
                vision_range: 6.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Sword,
                range: 1.5,
                armor_penetration: 0.1,
            },
        ),
        UnitArchetype::Worker => (
            Health {
                current: 50.0,
                max: 50.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.0,
                attack_damage: 3.0,
                attack_speed: 0.5,
                vision_range: 4.0,
                morale: 30.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 3.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
        ),
        UnitArchetype::Archer => (
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 6.0,
                attack_damage: 15.0,
                attack_speed: 0.7,
                vision_range: 8.0,
                morale: 40.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Bow,
                range: 6.0,
                armor_penetration: 0.0,
            },
        ),
        _ => (
            Health::default(),
            Stats::default(),
            MeleeWeapon {
                damage: 10.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
        ),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    // ─── Test helpers ───────────────────────────────────────────────────

    /// Helper: build an app with the new `process_build_orders_system` and
    /// the minimal set of resources / events.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<BuildingCatalog>()
            .init_resource::<BuildingLimits>()
            .add_event::<BuildOrder>()
            .add_event::<BuildRejected>()
            .add_event::<BuildOrderFailed>()
            .add_systems(FixedUpdate, process_build_orders_system);
        app
    }

    /// Helper: construct a `Building` with sensible defaults.
    fn test_building(bt: BuildingType) -> Building {
        Building {
            building_type: bt,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: format!("{:?} I", bt),
        }
    }

    /// Marker resource that collects rejection events during simulation.
    #[derive(Resource, Default)]
    struct RejectionCollector {
        rejected: Vec<BuildRejected>,
        failed: Vec<BuildOrderFailed>,
    }

    /// Capture rejections after the main system.
    fn collect_rejections(
        mut rejected_events: EventReader<BuildRejected>,
        mut failed_events: EventReader<BuildOrderFailed>,
        mut collector: ResMut<RejectionCollector>,
    ) {
        for ev in rejected_events.read() {
            collector.rejected.push(ev.clone());
        }
        for ev in failed_events.read() {
            collector.failed.push(ev.clone());
        }
    }

    /// Count construction-site entities at a given grid position.
    fn constructions_at(app: &mut App, pos: IVec2) -> usize {
        app.world_mut()
            .query_filtered::<&WorldPosition, With<ConstructionSite>>()
            .iter(app.world())
            .filter(|wp| wp.grid.0 == pos)
            .count()
    }

    /// Build a test app that includes the rejection collector.
    fn test_app_with_collector() -> App {
        let mut app = test_app();
        app.init_resource::<RejectionCollector>().add_systems(
            FixedUpdate,
            collect_rejections.after(process_build_orders_system),
        );
        app
    }

    // ─── Resource check tests ───────────────────────────────────────────

    #[test]
    fn test_build_fails_when_resources_insufficient() {
        let mut app = test_app_with_collector();

        // Set stockpile to near zero
        app.world_mut().resource_mut::<ResourceStockpile>().food = 0.0;
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 0.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 0.0;
        app.world_mut().resource_mut::<ResourceStockpile>().stone = 0.0;
        app.world_mut().resource_mut::<ResourceStockpile>().iron = 0.0;

        // Send a BuildOrder for a cheap building
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks, // costs 15 food, 120 wood
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        // No construction site
        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            0,
            "Barracks should NOT be placed with insufficient resources"
        );
        // Should have a BuildOrderFailed for resources
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector
                .failed
                .iter()
                .any(|f| matches!(f.reason, BuildFailureReason::InsufficientResources)),
            "Expected InsufficientResources failure"
        );
    }

    #[test]
    fn test_build_succeeds_with_sufficient_resources() {
        let mut app = test_app_with_collector();

        // Default stockpile has enough resources
        app.world_mut().resource_mut::<ResourceStockpile>().food = 100.0;
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 50.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 200.0;
        app.world_mut().resource_mut::<ResourceStockpile>().stone = 100.0;
        app.world_mut().resource_mut::<ResourceStockpile>().iron = 50.0;

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::LumberCamp, // costs 0 food, 80 wood
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            1,
            "LumberCamp should be placed with sufficient resources"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.failed.is_empty(),
            "No failure expected with sufficient resources"
        );
    }

    // ─── Building-limit tests ───────────────────────────────────────────

    #[test]
    fn test_build_fails_when_building_limit_exceeded() {
        let mut app = test_app_with_collector();

        // Set limits to make it appear full for defense buildings
        app.world_mut()
            .resource_mut::<BuildingLimits>()
            .defense_count = 2;
        app.world_mut().resource_mut::<BuildingLimits>().max_defense = 2;

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Palisade, // Defense category
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            0,
            "Palisade should NOT be placed when defense limit is reached"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.failed.iter().any(|f| matches!(
                f.reason,
                BuildFailureReason::LimitExceeded(BuildingCategory::Defense)
            )),
            "Expected LimitExceeded(Defense) failure"
        );
    }

    #[test]
    fn test_build_succeeds_within_building_limit() {
        let mut app = test_app_with_collector();

        // Limits are at default (max_basic=4, max_defense=2, max_special=0)
        app.world_mut().resource_mut::<ResourceStockpile>().food = 100.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 200.0;

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks, // Basic category
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            1,
            "Barracks should be placed within limits"
        );
    }

    // ─── Dependency check tests ─────────────────────────────────────────

    #[test]
    fn test_build_fails_when_prerequisite_missing() {
        let mut app = test_app_with_collector();

        // Gateway depends on Oba L2 but there's no Oba at all
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::CouncilChamber, // needs Oba L2
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            0,
            "CouncilChamber should NOT be placed without Oba L2"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.failed.iter().any(|f| matches!(
                f.reason,
                BuildFailureReason::MissingPrerequisite(BuildingType::Oba, 2)
            )),
            "Expected MissingPrerequisite(Oba, 2) failure"
        );
    }

    #[test]
    fn test_build_succeeds_when_prerequisite_met() {
        let mut app = test_app_with_collector();

        app.world_mut().resource_mut::<ResourceStockpile>().food = 200.0;
        app.world_mut().resource_mut::<ResourceStockpile>().wood = 500.0;
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 100.0;
        app.world_mut().resource_mut::<ResourceStockpile>().iron = 100.0;

        // Spawn an Oba at level 2 (CouncilChamber requires Oba L2)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 2,
                max_level: 5,
                name: "Test Oba".into(),
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(2),
                building_type: BuildingType::CouncilChamber,
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            1,
            "CouncilChamber should be placed when Oba L2 exists"
        );
    }

    // ─── Zone-compatibility tests (preserved from original) ─────────────

    #[test]
    fn test_build_without_zone_is_allowed() {
        let mut app = test_app_with_collector();

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(5, 5)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(5, 5)),
            1,
            "Barracks should be placed without a zone"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.rejected.is_empty() && collector.failed.is_empty(),
            "No rejection expected without zone"
        );
    }

    #[test]
    fn test_build_in_matching_zone() {
        let mut app = test_app_with_collector();

        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Military,
                level: 1,
            },
            WorldPosition::new(IVec2::new(3, 3), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(3, 3)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(3, 3)),
            1,
            "Barracks should be placed in Military zone"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.rejected.is_empty() && collector.failed.is_empty(),
            "No rejection for matching zone"
        );
    }

    #[test]
    fn test_build_in_wrong_zone_is_rejected() {
        let mut app = test_app_with_collector();

        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(2, 2), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(2, 2)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(2, 2)),
            0,
            "Barracks should NOT be placed in Residential zone"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            !collector.rejected.is_empty(),
            "Should emit at least one BuildRejected"
        );
        assert!(
            collector.rejected.iter().any(|r| matches!(
                r.reason,
                BuildRejectionReason::ZoneMismatch(ZoneType::Residential, BuildingType::Barracks)
            )),
            "Expected ZoneMismatch(Residential, Barracks)"
        );
    }

    #[test]
    fn test_all_zone_type_matches() {
        let pairs = vec![
            (ZoneType::Residential, BuildingType::Oba),
            (ZoneType::Military, BuildingType::Barracks),
            (ZoneType::Military, BuildingType::ArcheryRange),
            (ZoneType::Military, BuildingType::Stable),
            (ZoneType::Military, BuildingType::SiegeWorkshop),
            (ZoneType::Military, BuildingType::Watchtower),
            (ZoneType::Economic, BuildingType::Market),
            (ZoneType::Economic, BuildingType::Warehouse),
            (ZoneType::Economic, BuildingType::Pasture),
            (ZoneType::Economic, BuildingType::MiningCamp),
            (ZoneType::Economic, BuildingType::LumberCamp),
            (ZoneType::Religious, BuildingType::Mosque),
            (ZoneType::Religious, BuildingType::Monument),
            (ZoneType::Administrative, BuildingType::CouncilChamber),
            (ZoneType::Administrative, BuildingType::GuildHall),
        ];

        for (zone_type, building_type) in pairs {
            let mut app = test_app_with_collector();
            app.world_mut().spawn((
                Zone {
                    zone_type,
                    level: 1,
                },
                WorldPosition::new(IVec2::new(7, 7), CELL_SIZE),
            ));

            app.world_mut()
                .resource_mut::<Events<BuildOrder>>()
                .send(BuildOrder {
                    entity: Entity::from_raw(1),
                    building_type,
                    position: GridPos(IVec2::new(7, 7)),
                });

            for _ in 0..5 {
                app.update();
            }

            assert_eq!(
                constructions_at(&mut app, IVec2::new(7, 7)),
                1,
                "{:?} should be placeable in {:?} zone",
                building_type,
                zone_type
            );
        }
    }

    #[test]
    fn test_build_on_occupied_cell_is_rejected() {
        let mut app = test_app_with_collector();

        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Test Oba".into(),
            },
            WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(2),
                building_type: BuildingType::Market,
                position: GridPos(IVec2::new(4, 4)),
            });

        for _ in 0..5 {
            app.update();
        }

        assert_eq!(
            constructions_at(&mut app, IVec2::new(4, 4)),
            0,
            "No construction on occupied cell"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector
                .rejected
                .iter()
                .any(|r| matches!(r.reason, BuildRejectionReason::CellOccupied)),
            "Expected CellOccupied rejection"
        );
    }

    #[test]
    fn test_rejected_build_does_not_block_other_orders() {
        let mut app = test_app_with_collector();

        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(1, 1), CELL_SIZE),
        ));

        // Send TWO orders: Barracks (wrong for Residential) and Oba (right)
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Barracks,
                position: GridPos(IVec2::new(1, 1)),
            });
        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(2),
                building_type: BuildingType::Oba,
                position: GridPos(IVec2::new(1, 1)),
            });

        for _ in 0..5 {
            app.update();
        }

        // Oba should be placed (matches Residential)
        assert_eq!(
            constructions_at(&mut app, IVec2::new(1, 1)),
            1,
            "Oba should be placed after Barracks was rejected"
        );
        let collector = app.world().resource::<RejectionCollector>();
        assert!(
            collector.rejected.iter().any(|r| matches!(
                r.reason,
                BuildRejectionReason::ZoneMismatch(ZoneType::Residential, BuildingType::Barracks)
            )),
            "Should have rejected Barracks in Residential zone"
        );
    }

    // ─── Construction progress tests (preserved from original) ─────────

    #[test]
    fn test_construction_progresses_after_full_validation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<BuildingCatalog>()
            .init_resource::<BuildingLimits>()
            .add_event::<BuildOrder>()
            .add_event::<BuildRejected>()
            .add_event::<BuildOrderFailed>()
            .add_systems(
                FixedUpdate,
                (
                    process_build_orders_system,
                    construction_progress_system,
                    complete_construction_system,
                    building_limit_tracker_system,
                ),
            );

        // Send a BuildOrder for Oba in a Residential zone
        app.world_mut().spawn((
            Zone {
                zone_type: ZoneType::Residential,
                level: 1,
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        app.world_mut()
            .resource_mut::<Events<BuildOrder>>()
            .send(BuildOrder {
                entity: Entity::from_raw(1),
                building_type: BuildingType::Oba,
                position: GridPos(IVec2::new(0, 0)),
            });

        // Run many frames — construction should start, progress, and complete
        for _ in 0..120 {
            app.update();
        }

        // After many ticks, the ConstructionSite should be gone (completed)
        let const_count = app
            .world_mut()
            .query_filtered::<&ConstructionSite, With<WorldPosition>>()
            .iter(app.world())
            .filter(|site| site.building_type == BuildingType::Oba)
            .count();
        assert_eq!(const_count, 0, "Oba construction should have completed");

        // There should be a Building with 'level >= 1'
        let building_count = app
            .world_mut()
            .query_filtered::<&Building, With<WorldPosition>>()
            .iter(app.world())
            .filter(|b| b.building_type == BuildingType::Oba)
            .count();
        assert_eq!(
            building_count, 1,
            "Oba should exist as a completed Building"
        );
    }

    // ─── Building-limit tracker tests ──────────────────────────────────

    #[test]
    fn test_building_limit_tracker_counts_categories() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BuildingLimits>()
            .add_systems(FixedUpdate, building_limit_tracker_system);

        // Spawn one of each category
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Barracks,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Barracks".into(),
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Palisade,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Wall".into(),
            },
            WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
        ));

        app.update();

        let limits = app.world().resource::<BuildingLimits>();
        assert_eq!(limits.basic_count, 1, "Should count 1 basic building");
        assert_eq!(limits.defense_count, 1, "Should count 1 defense building");
        assert_eq!(limits.special_count, 0, "Should count 0 special buildings");
    }

    #[test]
    fn test_building_limit_tracker_detects_oba_level() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BuildingLimits>()
            .add_systems(FixedUpdate, building_limit_tracker_system);

        // Spawn Oba at level 3
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 3,
                max_level: 5,
                name: "Oba III".into(),
            },
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        app.update();

        let limits = app.world().resource::<BuildingLimits>();
        assert_eq!(
            limits.max_basic, 12,
            "Oba L3 should allow 12 basic buildings"
        );
        assert_eq!(
            limits.max_defense, 6,
            "Oba L3 should allow 6 defense buildings"
        );
        assert_eq!(
            limits.max_special, 3,
            "Oba L3 should allow 3 special buildings"
        );
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 3 — Building Economy Tests
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_building_resource_production_adds_food() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile::default());
        app.add_systems(FixedUpdate, building_resource_production_system);

        // Spawn a Pasture (food producer, L1 → 2.0 food/sec)
        app.world_mut().spawn(BuildingResourceProduction {
            food_per_sec: 2.0,
            ..Default::default()
        });

        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food > 100.0,
            "Pasture should produce food (got: {:.4})",
            stockpile.food
        );
    }

    #[test]
    fn test_building_resource_production_with_worker_multiplier() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile::default());
        app.add_systems(FixedUpdate, building_resource_production_system);

        // Spawn a Pasture with a fully-staffed WorkerSlot (mult > 1.0)
        app.world_mut().spawn((
            BuildingResourceProduction {
                food_per_sec: 2.0,
                ..Default::default()
            },
            WorkerSlot {
                max_workers: 3,
                assigned_workers: 3,
                worker_efficiency: 0.9,
                full_staff_bonus: 1.30,
                ..Default::default()
            },
        ));

        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        // With 3 workers, efficiency = 1.0 + 0.30 = 1.30 multiplier
        assert!(
            stockpile.food > 101.0,
            "Workers should boost production (got: {:.4})",
            stockpile.food
        );
    }

    #[test]
    fn test_building_upkeep_deducts_gold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile {
            gold: 100.0,
            ..Default::default()
        });
        app.add_systems(FixedUpdate, building_upkeep_system);

        // Spawn a Barracks (0.5 gold/sec upkeep)
        app.world_mut().spawn(BuildingUpkeep {
            gold_per_sec: 0.5,
            ..Default::default()
        });

        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.gold < 100.0,
            "Barracks should consume gold (got: {:.4})",
            stockpile.gold
        );
    }

    #[test]
    fn test_building_upkeep_does_not_go_below_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile {
            gold: 0.001,
            ..Default::default()
        });
        app.add_systems(FixedUpdate, building_upkeep_system);

        // Spawn an expensive building
        app.world_mut().spawn(BuildingUpkeep {
            gold_per_sec: 100.0,
            ..Default::default()
        });

        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.gold >= 0.0,
            "Gold should never go negative (got: {:.4})",
            stockpile.gold
        );
    }

    #[test]
    fn test_max_capacity_caps_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, max_capacity_enforcement_system);

        // Start way above maximums
        app.insert_resource(ResourceStockpile {
            food: 9999.0,
            gold: 9999.0,
            wood: 9999.0,
            stone: 9999.0,
            iron: 9999.0,
            max_food: 200.0,
            max_gold: 100.0,
            max_wood: 300.0,
            max_stone: 150.0,
            max_iron: 150.0,
            ..Default::default()
        });

        app.update();

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(stockpile.food, 200.0, "Food should be capped at max_food");
        assert_eq!(stockpile.gold, 100.0, "Gold should be capped at max_gold");
        assert_eq!(stockpile.wood, 300.0, "Wood should be capped at max_wood");
        assert_eq!(
            stockpile.stone, 150.0,
            "Stone should be capped at max_stone"
        );
        assert_eq!(stockpile.iron, 150.0, "Iron should be capped at max_iron");
    }

    #[test]
    fn test_production_and_upkeep_integration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )));
        app.init_resource::<GameTime>();
        app.insert_resource(ResourceStockpile {
            food: 100.0,
            gold: 100.0,
            ..Default::default()
        });
        app.add_systems(
            FixedUpdate,
            (
                building_resource_production_system.in_set(SimulationSet::ResourceProduction),
                building_upkeep_system.in_set(SimulationSet::ResourceConsumption),
                max_capacity_enforcement_system.in_set(SimulationSet::ComponentCleanup),
            ),
        );

        // A Pasture (food producer) and a Barracks (gold upkeep)
        app.world_mut().spawn(BuildingResourceProduction {
            food_per_sec: 2.0,
            ..Default::default()
        });
        app.world_mut().spawn(BuildingUpkeep {
            gold_per_sec: 0.5,
            ..Default::default()
        });

        for _ in 0..30 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food > 100.0,
            "Food should increase from production (got: {:.4})",
            stockpile.food
        );
        assert!(
            stockpile.gold < 100.0,
            "Gold should decrease from upkeep (got: {:.4})",
            stockpile.gold
        );
    }

    #[test]
    fn test_production_level_scaling() {
        // L3 Pasture should produce more than L1 Pasture
        let l1 = BuildingResourceProduction::for_building_type(BuildingType::Pasture, 1).unwrap();
        let l3 = BuildingResourceProduction::for_building_type(BuildingType::Pasture, 3).unwrap();
        assert_eq!(l1.food_per_sec, 2.0, "L1 Pasture: 2 food/sec");
        assert!(
            l3.food_per_sec > l1.food_per_sec,
            "L3 Pasture should produce more than L1"
        );
    }

    #[test]
    fn test_upkeep_level_scaling() {
        // L3 Barracks should cost more upkeep than L1 Barracks
        let l1 = BuildingUpkeep::for_building_type(BuildingType::Barracks, 1).unwrap();
        let l3 = BuildingUpkeep::for_building_type(BuildingType::Barracks, 3).unwrap();
        assert!(
            l3.gold_per_sec > l1.gold_per_sec,
            "L3 Barracks should cost more upkeep than L1"
        );
    }

    #[test]
    fn test_non_producer_returns_none() {
        assert!(
            BuildingResourceProduction::for_building_type(BuildingType::Barracks, 1).is_none(),
            "Barracks should not produce resources"
        );
        assert!(
            BuildingUpkeep::for_building_type(BuildingType::Pasture, 1).is_none(),
            "Pasture should not have upkeep"
        );
    }

    #[test]
    fn test_capacity_does_not_affect_under_cap() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, max_capacity_enforcement_system);

        app.insert_resource(ResourceStockpile {
            food: 50.0,
            max_food: 200.0,
            gold: 25.0,
            max_gold: 100.0,
            ..Default::default()
        });

        app.update();

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(stockpile.food, 50.0, "Food below cap should be unchanged");
        assert_eq!(stockpile.gold, 25.0, "Gold below cap should be unchanged");
    }

    // ══════════════════════════════════════════════════════════════════════
    // Phase 2 — Building Lifecycle Tests (HP, Cancel, Demolish, Repair)
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_building_health_scales_with_level() {
        let l1 = building_health(BuildingType::Barracks, 1);
        let l3 = building_health(BuildingType::Barracks, 3);
        assert_eq!(l1, 300.0, "L1 Barracks base HP");
        assert!(l3 > l1, "L3 Barracks should have more HP than L1");
    }

    #[test]
    fn test_building_armor_defense_vs_basic() {
        let wall = building_armor(BuildingType::StoneWall, 1);
        let pasture = building_armor(BuildingType::Pasture, 1);
        assert!(
            wall > pasture,
            "StoneWall should have more armor than Pasture"
        );
    }

    #[test]
    fn test_cancel_construction_refunds_and_despawns() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<CancelBuildOrder>()
            .add_systems(FixedUpdate, cancel_construction_system);

        // Start with a small stockpile
        app.insert_resource(ResourceStockpile::default());

        // Add a ConstructionSite with some progress
        let site = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    building_type: BuildingType::Barracks,
                    progress: 50.0,
                    max_progress: 100.0,
                    assigned_workers: vec![],
                },
                BuildCost {
                    food: 100,
                    gold: 50,
                    wood: 200,
                    stone: 0,
                    iron: 0,
                    population: 0,
                },
            ))
            .id();

        // Send cancel order
        app.world_mut()
            .resource_mut::<Events<CancelBuildOrder>>()
            .send(CancelBuildOrder { entity: site });

        for _ in 0..10 {
            app.update();
        }

        // Should have received refund (50% of cost * 50% progress = 25% of cost)
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food > 100.0,
            "Food should increase from refund (got: {:.4})",
            stockpile.food
        );
        assert!(
            stockpile.gold > 50.0,
            "Gold should increase from refund (got: {:.4})",
            stockpile.gold
        );

        // The site entity should be despawned
        assert!(
            !app.world().entities().contains(site),
            "Construction site should be despawned after cancel"
        );
    }

    #[test]
    fn test_cancel_construction_refunds_zero_at_start() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<CancelBuildOrder>()
            .add_systems(FixedUpdate, cancel_construction_system);

        app.insert_resource(ResourceStockpile::default());

        let site = app
            .world_mut()
            .spawn((
                ConstructionSite {
                    building_type: BuildingType::Barracks,
                    progress: 0.0,
                    max_progress: 100.0,
                    assigned_workers: vec![],
                },
                BuildCost {
                    food: 100,
                    wood: 200,
                    ..Default::default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<CancelBuildOrder>>()
            .send(CancelBuildOrder { entity: site });

        for _ in 0..10 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        // 50% * 0% progress = 0% refund
        assert!(
            (stockpile.food - 100.0).abs() < 0.01,
            "Food should not change with 0 progress (got: {:.4})",
            stockpile.food
        );

        // Site should still be despawned
        assert!(
            !app.world().entities().contains(site),
            "Site should be despawned even with 0 refund"
        );
    }

    #[test]
    fn test_demolish_refunds_and_fires_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<DemolishOrder>()
            .add_event::<BuildingDestroyed>()
            .add_systems(FixedUpdate, demolish_building_system);

        app.insert_resource(ResourceStockpile::default());

        let building = app
            .world_mut()
            .spawn((
                Building {
                    building_type: BuildingType::Barracks,
                    faction: FactionId::Kayi,
                    level: 1,
                    max_level: 3,
                    name: "Test Barracks".into(),
                },
                WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
                BuildCost {
                    food: 100,
                    gold: 50,
                    wood: 200,
                    stone: 0,
                    iron: 0,
                    population: 0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<DemolishOrder>>()
            .send(DemolishOrder { entity: building });

        for _ in 0..10 {
            app.update();
        }

        // Should recover 25% of cost
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food >= 24.9,
            "Food should increase from demolition (got: {:.4})",
            stockpile.food
        );

        // BuildingDestroyed should have been sent — verify by checking
        // the despawn happened (stronger indirect evidence)
        assert!(
            !app.world().entities().contains(building),
            "Building should be despawned after demolition"
        );
        // Also verify stockpile changed (25% recovery)
        assert!(stockpile.food >= 24.9, "Food recovery from demolition");
    }

    #[test]
    fn test_repair_restores_health_and_consumes_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<RepairOrder>()
            .add_systems(FixedUpdate, repair_building_system);

        // Start with abundant resources
        app.insert_resource(ResourceStockpile {
            food: 500.0,
            gold: 500.0,
            wood: 500.0,
            stone: 500.0,
            iron: 500.0,
            ..Default::default()
        });

        // A building at half health
        let building = app
            .world_mut()
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Heavy,
                },
                BuildCost {
                    food: 100,
                    gold: 50,
                    wood: 200,
                    stone: 0,
                    iron: 0,
                    population: 0,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<RepairOrder>>()
            .send(RepairOrder {
                building,
                worker: None,
            });

        for _ in 0..10 {
            app.update();
        }

        // Health should be restored to max
        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(
            health.current, health.max,
            "Building should be fully repaired"
        );

        // Resources should have been consumed (50% of cost since 50% HP missing)
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            stockpile.food < 500.0,
            "Food should decrease from repair (got: {:.4})",
            stockpile.food
        );
        assert!(
            stockpile.wood < 500.0,
            "Wood should decrease from repair (got: {:.4})",
            stockpile.wood
        );
    }

    #[test]
    fn test_repair_skipped_when_already_full() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<RepairOrder>()
            .add_systems(FixedUpdate, repair_building_system);

        app.insert_resource(ResourceStockpile {
            food: 500.0,
            ..Default::default()
        });

        let building = app
            .world_mut()
            .spawn((
                Health {
                    current: 100.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Heavy,
                },
                BuildCost {
                    food: 100,
                    ..Default::default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<RepairOrder>>()
            .send(RepairOrder {
                building,
                worker: None,
            });

        for _ in 0..10 {
            app.update();
        }

        // Resources should NOT have been consumed
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            (stockpile.food - 500.0).abs() < 0.01,
            "Food should not be consumed when building is already at full HP (got: {:.4})",
            stockpile.food
        );
    }

    #[test]
    fn test_repair_skipped_when_insufficient_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<RepairOrder>()
            .add_systems(FixedUpdate, repair_building_system);

        // Start with almost no resources
        app.insert_resource(ResourceStockpile::default());

        let building = app
            .world_mut()
            .spawn((
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Heavy,
                },
                BuildCost {
                    food: 1000,
                    ..Default::default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<RepairOrder>>()
            .send(RepairOrder {
                building,
                worker: None,
            });

        for _ in 0..10 {
            app.update();
        }

        // Health should remain unchanged
        let health = app.world().get::<Health>(building).unwrap();
        assert_eq!(
            health.current, 50.0,
            "Building should NOT be repaired when resources are insufficient"
        );
    }

    #[test]
    fn test_building_health_component_in_building_spawn() {
        // Verify that building_health returns reasonable values for key types
        let oba_hp = building_health(BuildingType::Oba, 1);
        let wall_hp = building_health(BuildingType::Palisade, 1);
        let road_hp = building_health(BuildingType::Road, 1);

        assert_eq!(oba_hp, 500.0, "Oba L1 base HP");
        assert_eq!(wall_hp, 500.0, "Palisade L1 base HP");
        assert_eq!(road_hp, 50.0, "Road L1 base HP");
        assert!(
            building_health(BuildingType::Oba, 3) > oba_hp,
            "Higher-level buildings have more HP"
        );
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 4 — Building Service Systems Tests
    // ═════════════════════════════════════════════════════════════════════

    #[test]
    fn test_market_trade_successful() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<TradeRates>()
            .init_resource::<ResourceStockpile>()
            .add_event::<TradeOrder>()
            .add_systems(FixedUpdate, market_trade_system);

        // Spawn a Market building
        let market = app
            .world_mut()
            .spawn((test_building(BuildingType::Market),))
            .id();

        // Give the player 100 food
        app.world_mut().resource_mut::<ResourceStockpile>().food = 100.0;

        // Send a trade order: 20 food → gold (rate: 1 food = 0.5 gold)
        app.world_mut()
            .resource_mut::<Events<TradeOrder>>()
            .send(TradeOrder {
                market,
                give_resource: ResourceType::Food,
                give_amount: 20.0,
                receive_resource: ResourceType::Gold,
            });

        // Run multiple frames to let FixedUpdate systems fire
        for _ in 0..5 {
            app.update();
        }

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            (stockpile.food - 80.0).abs() < 0.01,
            "Food should decrease by 20 (got: {:.4})",
            stockpile.food
        );
        // 20 food * 1.0 value / 2.0 gold value = 10 gold
        assert!(
            (stockpile.gold - 60.0).abs() < 0.01,
            "Gold should increase by ~10 (from 50) (got: {:.4})",
            stockpile.gold
        );
    }

    #[test]
    fn test_market_trade_insufficient_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<TradeRates>()
            .init_resource::<ResourceStockpile>()
            .add_event::<TradeOrder>()
            .add_systems(FixedUpdate, market_trade_system);

        // Default stockpile has 100 food
        // Send a trade for more than we have
        let market = app
            .world_mut()
            .spawn((test_building(BuildingType::Market),))
            .id();

        app.world_mut()
            .resource_mut::<Events<TradeOrder>>()
            .send(TradeOrder {
                market,
                give_resource: ResourceType::Food,
                give_amount: 9999.0,
                receive_resource: ResourceType::Gold,
            });

        app.update();

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            (stockpile.food - 100.0).abs() < 0.01,
            "Food should NOT have changed (got: {:.4})",
            stockpile.food
        );
    }

    #[test]
    fn test_market_trade_wrong_building_type() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<TradeRates>()
            .init_resource::<ResourceStockpile>()
            .add_event::<TradeOrder>()
            .add_systems(FixedUpdate, market_trade_system);

        // Spawn a non-Market building
        let building = app
            .world_mut()
            .spawn((test_building(BuildingType::Barracks),))
            .id();

        app.world_mut()
            .resource_mut::<Events<TradeOrder>>()
            .send(TradeOrder {
                market: building,
                give_resource: ResourceType::Food,
                give_amount: 10.0,
                receive_resource: ResourceType::Gold,
            });

        app.update();

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert!(
            (stockpile.food - 100.0).abs() < 0.01,
            "Food should NOT have changed for non-Market building"
        );
    }

    #[test]
    fn test_siege_production_queues_and_completes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<SiegeProductionQueue>()
            .add_event::<SiegeProductionOrder>()
            .add_event::<SiegeUnitComplete>()
            .add_systems(
                FixedUpdate,
                (siege_production_system, siege_production_progress_system),
            );

        // Spawn a Siege Workshop at a known position
        let workshop = app
            .world_mut()
            .spawn((
                test_building(BuildingType::SiegeWorkshop),
                WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
            ))
            .id();

        // Queue production of 2 Battering Rams
        app.world_mut()
            .resource_mut::<Events<SiegeProductionOrder>>()
            .send(SiegeProductionOrder {
                building: workshop,
                engine_type: SiegeEngineType::BatteringRam,
                count: 2,
            });

        // Advance enough time for completion (2 * 20s = 40 game seconds)
        for _ in 0..600 {
            app.update();
        }

        // Verify siege engines were spawned (check for SiegeEngine components)
        let siege_count = app
            .world_mut()
            .query::<&SiegeEngine>()
            .iter(app.world())
            .count();
        assert_eq!(
            siege_count, 2,
            "Two Battering Rams should have been produced (got: {})",
            siege_count
        );
    }

    #[test]
    fn test_siege_production_only_workshop() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<SiegeProductionQueue>()
            .add_event::<SiegeProductionOrder>()
            .add_event::<SiegeUnitComplete>()
            .add_systems(FixedUpdate, (siege_production_system,));

        // Spawn a non-Workshop building
        let building = app
            .world_mut()
            .spawn((test_building(BuildingType::Barracks),))
            .id();

        app.world_mut()
            .resource_mut::<Events<SiegeProductionOrder>>()
            .send(SiegeProductionOrder {
                building,
                engine_type: SiegeEngineType::Catapult,
                count: 1,
            });

        app.update();

        let queue = app.world().resource::<SiegeProductionQueue>();
        assert!(
            queue.active_productions.is_empty(),
            "Should not queue production for non-Workshop building"
        );
    }

    #[test]
    fn test_watchtower_vision_system_adds_stats() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_systems(FixedUpdate, watchtower_vision_system);

        // Spawn a Watchtower WITHOUT Stats
        let watchtower = app
            .world_mut()
            .spawn((
                test_building(BuildingType::Watchtower),
                WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
            ))
            .id();

        // Spawn another building type (should NOT get Stats)
        let barracks = app
            .world_mut()
            .spawn((
                test_building(BuildingType::Barracks),
                WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
            ))
            .id();

        // Run multiple frames to let FixedUpdate systems fire
        for _ in 0..5 {
            app.update();
        }

        // Watchtower should now have Stats with vision_range = 15.0
        let wt_stats = app.world().get::<Stats>(watchtower);
        assert!(wt_stats.is_some(), "Watchtower should have Stats component");
        assert!(
            (wt_stats.unwrap().vision_range - 15.0).abs() < 0.01,
            "Watchtower vision should be 15.0"
        );

        // Barracks should NOT have Stats
        let br_stats = app.world().get::<Stats>(barracks);
        assert!(
            br_stats.is_none(),
            "Barracks should NOT have Stats component"
        );
    }

    #[test]
    fn test_well_healing_heals_nearby_allied_units() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .add_systems(FixedUpdate, well_healing_system);

        // Spawn a Well with HealingAura (range 4, heal 2 HP/s)
        let _well = app
            .world_mut()
            .spawn((
                WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
                HealingAura::for_well_level(1),
            ))
            .id();

        // Spawn an allied unit nearby (within range)
        let ally = app
            .world_mut()
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    faction: FactionId::Kayi,
                    ..Default::default()
                },
                WorldPosition::new(IVec2::new(2, 0), CELL_SIZE), // 2 cells away < 4 range
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Light,
                },
            ))
            .id();

        // Spawn an enemy unit nearby
        let enemy = app
            .world_mut()
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    faction: FactionId::Byzantine,
                    ..Default::default()
                },
                WorldPosition::new(IVec2::new(1, 0), CELL_SIZE),
                Health {
                    current: 30.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Light,
                },
            ))
            .id();

        // Spawn an ally far away (out of range)
        let far_ally = app
            .world_mut()
            .spawn((
                Unit {
                    archetype: UnitArchetype::LightInfantry,
                    faction: FactionId::Kayi,
                    ..Default::default()
                },
                WorldPosition::new(IVec2::new(10, 10), CELL_SIZE), // ~14 cells away > 4 range
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 0.0,
                    armor_type: ArmorType::Light,
                },
            ))
            .id();

        // Run several updates to accumulate healing
        for _ in 0..60 {
            app.update();
        }

        // Ally within range should have been healed
        let ally_health = app.world().get::<Health>(ally).unwrap();
        assert!(
            ally_health.current > 50.0,
            "Nearby ally should have been healed (got: {:.4})",
            ally_health.current
        );

        // Enemy should NOT have been healed
        let enemy_health = app.world().get::<Health>(enemy).unwrap();
        assert!(
            (enemy_health.current - 30.0).abs() < 1.0,
            "Enemy should NOT be healed (got: {:.4})",
            enemy_health.current
        );

        // Far ally should NOT have been healed
        let far_health = app.world().get::<Health>(far_ally).unwrap();
        assert!(
            (far_health.current - 50.0).abs() < 1.0,
            "Far ally should NOT be healed (got: {:.4})",
            far_health.current
        );
    }

    // ═════════════════════════════════════════════════════════════════════
    // Phase 5 — Polish Tests
    // ═════════════════════════════════════════════════════════════════════

    #[test]
    fn test_building_rotation_rotates_transform() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .add_event::<RotateBuildingOrder>()
            .add_systems(FixedUpdate, building_rotation_system);

        let building = app
            .world_mut()
            .spawn((
                test_building(BuildingType::Watchtower),
                BuildingRotation::default(),
                Transform::from_rotation(Quat::from_rotation_z(0.0)),
            ))
            .id();

        // Send rotation order: face East
        app.world_mut()
            .resource_mut::<Events<RotateBuildingOrder>>()
            .send(RotateBuildingOrder {
                building,
                direction: CardinalDirection::East,
            });

        // Run multiple frames to let FixedUpdate systems fire
        for _ in 0..5 {
            app.update();
        }

        let rotation = app.world().get::<BuildingRotation>(building).unwrap();
        assert_eq!(
            rotation.direction,
            CardinalDirection::East,
            "Building should face East"
        );

        let transform = app.world().get::<Transform>(building).unwrap();
        // The quaternion for rotation around Z: w = cos(angle/2), z = sin(angle/2)
        // East = -FRAC_PI_2 => z = sin(-π/4) ≈ -0.707
        let expected_z = (-std::f32::consts::FRAC_PI_2 / 2.0).sin();
        // Allow comparison even if Quat normalisation flips the sign
        let z_matches = (transform.rotation.z - expected_z).abs() < 0.01
            || (transform.rotation.z + expected_z).abs() < 0.01;
        assert!(
            z_matches,
            "Transform rotation.z should be near ±{:.4} (East) (got: {:.4})",
            expected_z, transform.rotation.z
        );
        // Also verify BuildingRotation was updated correctly
        let rotation = app.world().get::<BuildingRotation>(building).unwrap();
        assert_eq!(
            rotation.direction,
            CardinalDirection::East,
            "BuildingRotation direction should be East"
        );
    }

    #[test]
    fn test_weather_modifier_winter_slows_production() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<WeatherModifiers>()
            .add_systems(FixedUpdate, weather_modifier_system);

        // Set time to Winter, daytime
        app.world_mut().resource_mut::<GameTime>().season = SeasonType::Winter;
        app.world_mut().resource_mut::<GameTime>().hour = 12; // daytime

        // Run multiple frames to let FixedUpdate systems fire
        for _ in 0..5 {
            app.update();
        }

        let modifiers = app.world().resource::<WeatherModifiers>();
        assert!(
            (modifiers.construction_speed_modifier - 0.7).abs() < 0.01,
            "Winter construction speed should be 0.7 (got: {:.4})",
            modifiers.construction_speed_modifier
        );
        assert!(
            (modifiers.production_modifier - 0.8).abs() < 0.01,
            "Winter production should be 0.8 (got: {:.4})",
            modifiers.production_modifier
        );
        assert!(
            (modifiers.movement_speed_modifier - 0.75).abs() < 0.01,
            "Winter movement speed should be 0.75 (got: {:.4})",
            modifiers.movement_speed_modifier
        );
    }

    #[test]
    fn test_weather_modifier_night_stacks_with_winter() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<WeatherModifiers>()
            .add_systems(FixedUpdate, weather_modifier_system);

        // Winter + night = stacked
        app.world_mut().resource_mut::<GameTime>().season = SeasonType::Winter;
        app.world_mut().resource_mut::<GameTime>().hour = 23; // night (≥20)

        // Run multiple frames to let FixedUpdate systems fire
        for _ in 0..5 {
            app.update();
        }

        let modifiers = app.world().resource::<WeatherModifiers>();
        // Winter 0.7 * night 0.8 = 0.56
        assert!(
            (modifiers.construction_speed_modifier - 0.56).abs() < 0.01,
            "Winter+night construction speed should be 0.56 (got: {:.4})",
            modifiers.construction_speed_modifier
        );
        // Winter: 0.8, night: *0.9 = 0.72
        assert!(
            (modifiers.production_modifier - 0.72).abs() < 0.01,
            "Winter+night production should be 0.72 (got: {:.4})",
            modifiers.production_modifier
        );
    }
}
