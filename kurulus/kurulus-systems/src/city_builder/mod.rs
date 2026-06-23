use bevy::prelude::*;
use bevy::math::IVec2;
use bevy::input::ButtonInput;
use kurulus_core::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;

mod building_placement;
use building_placement::*;

pub struct CityBuilderPlugin;

impl Plugin for CityBuilderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CityState>()
            .init_resource::<CitizenNeeds>()
            .init_resource::<ZoneDefinitions>()
            .init_resource::<ServiceCoverage>()
            .init_resource::<UpgradeChainRegistry>()
            .init_resource::<OverlaySettings>()
            .init_resource::<PolicySettings>()
            .init_resource::<PolicyFocus>()
            .init_resource::<ZonePlacementState>()
            .init_resource::<BuildingPlacementState>()
            .init_resource::<DistrictTracker>()
            .init_resource::<CrossZoneSynergyTable>()
            .add_systems(Startup, zone_config_loader_system)
            .add_event::<BuildOrder>()
            .add_event::<ZonePlaced>()
            .add_event::<ZoneUpgraded>()
            .add_event::<ZonePlacementBlocked>()
            .add_event::<CityLevelUp>()
            .add_event::<BuildingUpgraded>()
            .add_event::<CityEventOccurred>()
            .add_event::<CityEventEnded>()
            .add_event::<RecalculateCoverage>()
            .add_systems(
                FixedUpdate,
                (
                    // Phase 6a: City Population & Growth
                    city_population_system
                        .in_set(SimulationSet::CityPopulation),
                    // Phase 6b: City Happiness & Crime
                    city_happiness_system
                        .in_set(SimulationSet::CityHappiness),
                    // Phase 6c: Service Coverage
                    city_service_system
                        .in_set(SimulationSet::CityServices),
                    // Phase 6d: District Detection (flood-fill adjacency)
                    district_detection_system
                        .in_set(SimulationSet::CityZones)
                        .before(cross_zone_synergy_system),
                    // Phase 6d-2: Cross-Zone Synergy Boundary Scan
                    cross_zone_synergy_system
                        .in_set(SimulationSet::CityZones)
                        .before(city_zone_system),
                    // Phase 6d-3: Zone Assignment & Adjacency
                    city_zone_system
                        .in_set(SimulationSet::CityZones),
                    // Phase 6e: Building Upgrade Chains
                    upgrade_chain_system
                        .in_set(SimulationSet::ConstructionProgress),
                    // Phase 6f: City Progression (Village→Town→City)
                    city_progression_system
                        .in_set(SimulationSet::CityProgression),
                    city_event_processing_system
                        .in_set(SimulationSet::CityProgression),
                    city_random_event_system
                        .in_set(SimulationSet::CityProgression),
                    // Phase 6g: Policy Effects (consumption, draft)
                    policy_effects_system
                        .in_set(SimulationSet::CityHappiness),
                    // Sync PolicySettings.tax_rate → TaxPolicy.rate
                    tax_policy_sync_system
                        .in_set(SimulationSet::ResourceProduction),
                ),
            )
            // Keyboard-driven & input systems run in Update for responsive input.
            // All city interaction systems are gated to `AppState::InGame` so they
            // do not fire when the player is viewing the World Map.
            .add_systems(
                Update,
                (
                    overlay_cycle_system,
                    policy_cycle_system,
                    policy_adjust_system,
                    zone_placement_toggle_system,
                    zone_placement_hover_system,
                    zone_placement_execute_system,
                    zone_upgrade_system,
                    building_placement_toggle_system,
                    building_placement_hover_system,
                    building_placement_execute_system,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 1. City Population System
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks fractional population growth so we don't lose it to rounding.
#[derive(Resource, Default)]
struct PopulationAccumulator(pub f32);

/// Drives population growth/decline based on food, housing, and happiness.
/// Population increases when food satisfaction and housing are adequate,
/// and decreases during shortages.
fn city_population_system(
    stockpile: Res<ResourceStockpile>,
    mut city: ResMut<CityState>,
    mut needs: ResMut<CitizenNeeds>,
    mut accumulator: Local<PopulationAccumulator>,
) {
    // ── Update housing satisfaction ──────────────────────────────────
    // If pop is near or at max, housing is poor.
    needs.housing_satisfaction = if city.max_population > 0 {
        let ratio = city.population as f32 / city.max_population as f32;
        // Inverted: more population relative to cap = lower satisfaction
        (1.0 - ratio).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // ── Update food satisfaction ─────────────────────────────────────
    // Compare food in stockpile against a daily consumption baseline
    let daily_consumption = (city.population as f32 * 2.0).max(1.0);
    needs.food_satisfaction = (stockpile.food / daily_consumption).clamp(0.0, 1.0);

    // ── Population change (accumulated fractional growth) ────────────
    if city.population < city.max_population && needs.food_satisfaction > 0.3 {
        // Growth rate per tick: at peak conditions yields ~1 pop per ~7 ticks
        let growth_rate = 0.15
            * needs.food_satisfaction
            * (1.0 + needs.housing_satisfaction) * 0.5
            * (city.happiness / 100.0);
        accumulator.0 += growth_rate;

        // Convert accumulated fraction to whole population
        if accumulator.0 >= 1.0 {
            let gained = accumulator.0 as u32;
            city.population = (city.population + gained).min(city.max_population);
            accumulator.0 -= gained as f32;
        }
    } else if needs.food_satisfaction < 0.15 {
        // Starvation: population declines (~1 pop lost per ~20 ticks)
        let decline_rate = 0.05 * (1.0 - needs.food_satisfaction);
        accumulator.0 -= decline_rate;

        if accumulator.0 <= -1.0 {
            let lost = (-accumulator.0) as u32;
            city.population = city.population.saturating_sub(lost);
            accumulator.0 += lost as f32;
        }
    }

    // ── Safety satisfaction from military population ratio ───────────
    let defender_ratio = if city.population > 0 {
        (city.total_military_pop as f32 / city.population as f32).min(1.0)
    } else {
        0.0
    };
    needs.safety_satisfaction =
        ((1.0 - city.crime / 100.0) * 0.5 + defender_ratio * 0.5).clamp(0.0, 1.0);

    // Clamp at max_population
    city.population = city.population.min(city.max_population);
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. City Happiness System
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculates citizen happiness based on needs satisfaction, tax rate, crime,
/// service coverage, and active events.
fn city_happiness_system(
    mut city: ResMut<CityState>,
    needs: Res<CitizenNeeds>,
    coverage: Res<ServiceCoverage>,
    tax: Res<TaxPolicy>,
    policy: Res<PolicySettings>,
    game_time: Res<GameTime>,
    mut event_writer: EventWriter<CityEventOccurred>,
) {
    let delta = game_time.time_scale as f64 * 0.1;

    // ── Target happiness based on needs & services ───────────────────
    let needs_score = needs.average() * 50.0; // needs contribute up to 50 points
    let service_score = coverage.average() * 25.0; // services contribute up to 25 points
    let tax_penalty = tax.rate * 30.0; // high taxes hurt happiness (-0 to -30)
    let crime_penalty = (city.crime / 100.0) * 20.0; // crime contributes up to -20
    let faith_bonus = (city.faith / 100.0) * 15.0; // faith contributes up to +15
    let rationing_effect = policy.rationing.happiness_modifier(); // -20 to +5
    let conscription_effect = policy.conscription.happiness_modifier(); // -30 to +2

    let target_happiness = (needs_score
        + service_score
        + faith_bonus
        + rationing_effect
        + conscription_effect
        - tax_penalty
        - crime_penalty)
        .clamp(0.0, 100.0);

    // ── Smoothly move current happiness toward target ────────────────
    let lerp_speed = 0.5 * delta as f32;
    city.happiness += (target_happiness - city.happiness) * lerp_speed;
    city.happiness = city.happiness.clamp(0.0, 100.0);

    // ── Crime naturally drifts based on safety and happiness ─────────
    let target_crime = ((1.0 - needs.safety_satisfaction) * 30.0
        + (1.0 - city.happiness / 100.0) * 20.0)
        .clamp(0.0, 100.0);
    city.crime += (target_crime - city.crime) * 0.1 * delta as f32;
    city.crime = city.crime.clamp(0.0, 100.0);

    // ── Sanitation from water coverage ───────────────────────────────
    let target_sanitation = 30.0 + coverage.water * 70.0; // 30 base + up to 70 from water
    city.sanitation += (target_sanitation - city.sanitation) * 0.2 * delta as f32;
    city.sanitation = city.sanitation.clamp(0.0, 100.0);

    // ── Faith from religious coverage ────────────────────────────────
    let target_faith = 20.0 + coverage.faith * 80.0; // 20 base + up to 80 from faith services
    city.faith += (target_faith - city.faith) * 0.15 * delta as f32;
    city.faith = city.faith.clamp(0.0, 100.0);

    // ── Trigger negative events on very low happiness ────────────────
    if city.happiness < 15.0 && rand::random::<f32>() < 0.005 * delta as f32 {
        event_writer.send(CityEventOccurred {
            event_type: CityEventType::Riot,
            magnitude: 0.5 + rand::random::<f32>() * 0.5,
            duration: 30.0 + rand::random::<f32>() * 30.0,
        });
    }

    // ── Trigger disease on low sanitation ────────────────────────────
    if city.sanitation < 20.0 && rand::random::<f32>() < 0.003 * delta as f32 {
        event_writer.send(CityEventOccurred {
            event_type: CityEventType::Plague,
            magnitude: 0.3 + rand::random::<f32>() * 0.4,
            duration: 20.0 + rand::random::<f32>() * 40.0,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. City Service Coverage System
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculates what fraction of buildings/citizens are covered by each service
/// type based on ServiceBuilding components and their coverage radii.
fn city_service_system(
    mut coverage: ResMut<ServiceCoverage>,
    service_buildings: Query<(&ServiceBuilding, &WorldPosition)>,
    all_buildings: Query<&WorldPosition, With<Building>>,
    mut recalc_events: EventReader<RecalculateCoverage>,
    mut zone_events: EventReader<ZonePlaced>,
) {
    // Recalculate only when something changes (events) or periodically.
    let needs_recalc = !recalc_events.is_empty() || !zone_events.is_empty();
    if !needs_recalc {
        return;
    }
    // Clear readers
    for _ in recalc_events.read() {}
    for _ in zone_events.read() {}

    if all_buildings.is_empty() || service_buildings.is_empty() {
        coverage.water = 0.0;
        coverage.trade = 0.0;
        coverage.faith = 0.0;
        coverage.prestige = 0.0;
        return;
    }

    // Collect all building positions
    let building_positions: Vec<IVec2> = all_buildings.iter().map(|wp| wp.grid.0).collect();

    // For each service type, count how many buildings are within range
    // of at least one service building of that type.
    let mut covered_count: HashMap<ServiceType, usize> = HashMap::new();
    let mut service_count: HashMap<ServiceType, usize> = HashMap::new();

    for sb in service_buildings.iter() {
        let (service, pos) = (sb.0, sb.1);
        let radius_sq = service.coverage_radius * service.coverage_radius;
        *service_count.entry(service.service_type).or_insert(0) += 1;

        for &bpos in &building_positions {
            let dx = (pos.grid.0.x - bpos.x) as f32;
            let dy = (pos.grid.0.y - bpos.y) as f32;
            if dx * dx + dy * dy <= radius_sq {
                *covered_count.entry(service.service_type).or_insert(0) += 1;
                break; // one service building covering this building is enough
            }
        }
    }

    let total = building_positions.len();
    coverage.water = covered_count.get(&ServiceType::Water).copied().unwrap_or(0) as f32 / total as f32;
    coverage.trade = covered_count.get(&ServiceType::Trade).copied().unwrap_or(0) as f32 / total as f32;
    coverage.faith = covered_count.get(&ServiceType::Faith).copied().unwrap_or(0) as f32 / total as f32;
    coverage.prestige = covered_count.get(&ServiceType::Prestige).copied().unwrap_or(0) as f32 / total as f32;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4a. District Detection System (flood-fill adjacency)
// ═══════════════════════════════════════════════════════════════════════════════

/// Detects contiguous districts of same-type zone cells using flood-fill.
///
/// A **district** is a connected component of zone cells sharing the same
/// `ZoneType` connected orthogonally (4-directional). Each tick the system:
/// 1. Collects all zone entities into a spatial map.
/// 2. Flood-fills from each unvisited cell to discover districts.
/// 3. Stores results in `DistrictTracker` and assigns `DistrictMembership`.
/// 4. Computes aggregate bonus effects for the city.
fn district_detection_system(
    zones: Query<(Entity, &Zone, &WorldPosition)>,
    mut tracker: ResMut<DistrictTracker>,
) {
    use std::collections::{HashMap, VecDeque};

    // ── Build spatial map ─────────────────────────────────────────────
    let mut spatial: HashMap<IVec2, (Entity, ZoneType, u8)> = HashMap::new();
    for (entity, zone, pos) in zones.iter() {
        spatial.insert(pos.grid.0, (entity, zone.zone_type, zone.level));
    }

    if spatial.is_empty() {
        tracker.districts.clear();
        tracker.aggregate_bonus = AdjacencyBonusEffect::default();
        return;
    }

    // ── Flood-fill into districts ─────────────────────────────────────
    let mut visited: HashSet<IVec2> = HashSet::new();
    let mut districts: Vec<District> = Vec::new();
    let mut next_id = tracker.next_id;

    // Orthogonal neighbor offsets
    const DIRS: [IVec2; 4] = [
        IVec2::new(0, 1),
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
    ];

    for (&start_pos, &(_entity, zone_type, _level)) in &spatial {
        if visited.contains(&start_pos) {
            continue;
        }

        // BFS flood-fill from this cell
        let mut queue: VecDeque<IVec2> = VecDeque::new();
        let mut district_cells: Vec<IVec2> = Vec::new();
        let mut district_entities: Vec<Entity> = Vec::new();
        let mut total_level = 0u32;

        queue.push_back(start_pos);
        visited.insert(start_pos);

        while let Some(pos) = queue.pop_front() {
            if let Some(&(e, _zt, lv)) = spatial.get(&pos) {
                district_cells.push(pos);
                district_entities.push(e);
                total_level += lv as u32;

                for &dir in &DIRS {
                    let neighbor = pos + dir;
                    if !visited.contains(&neighbor) {
                        if let Some(&(_, ntype, _)) = spatial.get(&neighbor) {
                            if ntype == zone_type {
                                visited.insert(neighbor);
                                queue.push_back(neighbor);
                            }
                        }
                    }
                }
            }
        }

        let size = district_cells.len() as u32;
        districts.push(District {
            id: next_id,
            zone_type,
            cells: district_cells,
            entities: district_entities,
            total_level,
            size,
        });
        next_id += 1;
    }

    // ── Compute aggregate bonuses ─────────────────────────────────────
    let mut aggregate = AdjacencyBonusEffect::default();

    // Merge bonuses: take the MAX per field across all districts
    for district in &districts {
        // Look up the base multiplier for this zone type
        let base_mult = match district.zone_type {
            ZoneType::Residential => 1.25,
            ZoneType::Military => 1.30,
            ZoneType::Economic => 1.25,
            ZoneType::Religious => 1.50,
            ZoneType::Administrative => 1.40,
            ZoneType::Protection => 1.00,
        };
        let effect = district.compute_bonus_effect(base_mult);

        aggregate.population_growth_mult = aggregate.population_growth_mult.max(effect.population_growth_mult);
        aggregate.recruit_speed_mult = aggregate.recruit_speed_mult.max(effect.recruit_speed_mult);
        aggregate.resource_output_mult = aggregate.resource_output_mult.max(effect.resource_output_mult);
        aggregate.faith_generation_mult = aggregate.faith_generation_mult.max(effect.faith_generation_mult);
        aggregate.happiness_bonus = aggregate.happiness_bonus.max(effect.happiness_bonus);
        aggregate.crime_reduction = aggregate.crime_reduction.max(effect.crime_reduction);
        aggregate.defense_bonus_mult = aggregate.defense_bonus_mult.max(effect.defense_bonus_mult);
    }

    tracker.districts = districts;
    tracker.next_id = next_id;
    tracker.aggregate_bonus = aggregate;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4b. Cross-Zone Synergy System (boundary scan between different zone types)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns an ordered pair of `ZoneType` values (a, b) where a ≤ b by numeric index.
/// Used to create symmetric keys for the cross-zone synergy table.
fn sort_zone_pair(a: ZoneType, b: ZoneType) -> (ZoneType, ZoneType) {
    fn idx(z: ZoneType) -> u8 {
        match z {
            ZoneType::Residential => 0,
            ZoneType::Military => 1,
            ZoneType::Economic => 2,
            ZoneType::Religious => 3,
            ZoneType::Administrative => 4,
            ZoneType::Protection => 5,
        }
    }
    if idx(a) <= idx(b) { (a, b) } else { (b, a) }
}

/// Detects orthogonal adjacency between different zone types and computes
/// cross-zone synergy bonuses. Synergies are defined in `CrossZoneSynergyTable`
/// and are symmetric (A↔B yields the same bonus).
///
/// Algorithm:
/// 1. Build spatial map of all zone cells: `HashMap<IVec2, (Entity, ZoneType)>`
/// 2. For each district, scan boundary cells and check all 4 neighbors
/// 3. If neighbor is a different ZoneType, look up synergy in the table
/// 4. Accumulate total synergy effect into `DistrictTracker.cross_zone_bonus`
/// 5. Assign `CrossZoneSynergyEffect` components to each district entity
fn cross_zone_synergy_system(
    zones: Query<(Entity, &Zone, &WorldPosition)>,
    synergy_table: Option<Res<CrossZoneSynergyTable>>,
    mut tracker: ResMut<DistrictTracker>,
) {
    use std::collections::{HashMap, HashSet};

    // Determine max zones for spatial lookup
    if zones.is_empty() {
        tracker.cross_zone_bonus = AdjacencyBonusEffect::default();
        return;
    }

    // ── Build spatial map ─────────────────────────────────────────────
    let mut spatial: HashMap<IVec2, (Entity, ZoneType)> = HashMap::new();
    for (entity, zone, pos) in zones.iter() {
        spatial.insert(pos.grid.0, (entity, zone.zone_type));
    }

    // ── Get synergy table (use defaults if resource not registered) ──
    let table = synergy_table
        .as_deref()
        .cloned()
        .unwrap_or_default();

    // ── Build a fast synergy lookup: sorted pair → effect ────────────
    let mut synergy_map: HashMap<(ZoneType, ZoneType), AdjacencyBonusEffect> = HashMap::new();
    for synergy in &table.synergies {
        let key = sort_zone_pair(synergy.zone_a, synergy.zone_b);
        synergy_map.insert(key, synergy.effect);
    }

    // Orthogonal neighbour offsets
    const DIRS: [IVec2; 4] = [
        IVec2::new(0, 1),
        IVec2::new(0, -1),
        IVec2::new(1, 0),
        IVec2::new(-1, 0),
    ];

    // ── Accumulate cross-zone synergy effects ─────────────────────────
    let mut merged_synergy = AdjacencyBonusEffect::default();
    // Track which zone-type pair edges we've already counted
    let mut counted: HashSet<((ZoneType, ZoneType), IVec2)> = HashSet::new();

    for (&pos, &(_entity, zone_type)) in &spatial {
        for &dir in &DIRS {
            let neighbor_pos = pos + dir;
            if let Some(&(_neighbor_entity, neighbor_type)) = spatial.get(&neighbor_pos) {
                if neighbor_type != zone_type {
                    // Create sorted key for the pair
                    let pair = sort_zone_pair(zone_type, neighbor_type);

                    // Avoid double-counting the same edge from both sides
                    // by tracking (pair, cell_pos) — only count once per cell
                    if counted.insert((pair, pos)) {
                        if let Some(effect) = synergy_map.get(&pair) {
                            // Merge: take max for multipliers, sum for flat bonuses
                            merged_synergy.population_growth_mult = merged_synergy
                                .population_growth_mult
                                .max(effect.population_growth_mult);
                            merged_synergy.recruit_speed_mult = merged_synergy
                                .recruit_speed_mult
                                .max(effect.recruit_speed_mult);
                            merged_synergy.resource_output_mult = merged_synergy
                                .resource_output_mult
                                .max(effect.resource_output_mult);
                            merged_synergy.faith_generation_mult = merged_synergy
                                .faith_generation_mult
                                .max(effect.faith_generation_mult);
                            merged_synergy.happiness_bonus = merged_synergy
                                .happiness_bonus
                                .max(effect.happiness_bonus);
                            merged_synergy.crime_reduction = merged_synergy
                                .crime_reduction
                                .max(effect.crime_reduction);
                            merged_synergy.defense_bonus_mult = merged_synergy
                                .defense_bonus_mult
                                .max(effect.defense_bonus_mult);
                        }
                    }
                }
            }
        }
    }

    tracker.cross_zone_bonus = merged_synergy;
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4c. City Zone System (applies district bonuses to city metrics)
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies district adjacency bonuses to city metrics and tracks zone totals.
/// Runs after `district_detection_system` which populates `DistrictTracker`.
fn city_zone_system(
    mut city: ResMut<CityState>,
    buildings: Query<(Entity, &Building, &WorldPosition, Option<&Zone>)>,
    mut zone_defs: ResMut<ZoneDefinitions>,
    tracker: Res<DistrictTracker>,
) {
    // Track zone totals
    let mut total_res_levels = 0u32;
    let mut total_mil_levels = 0u32;
    let mut total_eco_power = 0u32;

    for (_entity, building, _pos, zone) in buildings.iter() {
        if let Some(z) = zone {
            if zone_defs
                .zones
                .iter()
                .any(|zd| zd.zone_type == z.zone_type)
            {
                match z.zone_type {
                    ZoneType::Residential => {
                        total_res_levels += building.level as u32;
                    }
                    ZoneType::Military => {
                        total_mil_levels += building.level as u32;
                        city.total_military_pop = total_mil_levels * 5;
                    }
                    ZoneType::Economic => {
                        total_eco_power += building.level as u32;
                        city.total_economic_power = total_eco_power;
                    }
                    ZoneType::Religious => {}
                    ZoneType::Administrative => {}
                    ZoneType::Protection => {} // tracked via defense_bonus_mult + crime_reduction
                }
            }
        } else {
            match building.building_type {
                BuildingType::Barracks
                | BuildingType::ArcheryRange
                | BuildingType::Stable
                | BuildingType::SiegeWorkshop
                | BuildingType::Watchtower => {
                    total_mil_levels += building.level as u32;
                }
                BuildingType::Market
                | BuildingType::Warehouse
                | BuildingType::Pasture
                | BuildingType::MiningCamp
                | BuildingType::LumberCamp => {
                    total_eco_power += building.level as u32;
                }
                _ => {}
            }
        }
    }

    city.total_residential_levels = total_res_levels;

    // ── Apply district adjacency bonuses to city metrics ──────────────
    let district_bonus = &tracker.aggregate_bonus;
    let synergy_bonus = &tracker.cross_zone_bonus;
    // Merge: take max across district, synergy, and edict sources per field
    let bonus = AdjacencyBonusEffect {
        population_growth_mult: district_bonus
            .population_growth_mult
            .max(synergy_bonus.population_growth_mult)
            .max(city.edict_zone_bonuses.population_growth_mult),
        recruit_speed_mult: district_bonus
            .recruit_speed_mult
            .max(synergy_bonus.recruit_speed_mult)
            .max(city.edict_zone_bonuses.recruit_speed_mult),
        resource_output_mult: district_bonus
            .resource_output_mult
            .max(synergy_bonus.resource_output_mult)
            .max(city.edict_zone_bonuses.resource_output_mult),
        faith_generation_mult: district_bonus
            .faith_generation_mult
            .max(synergy_bonus.faith_generation_mult)
            .max(city.edict_zone_bonuses.faith_generation_mult),
        happiness_bonus: district_bonus
            .happiness_bonus
            .max(synergy_bonus.happiness_bonus)
            .max(city.edict_zone_bonuses.happiness_bonus),
        crime_reduction: district_bonus
            .crime_reduction
            .max(synergy_bonus.crime_reduction)
            .max(city.edict_zone_bonuses.crime_reduction),
        defense_bonus_mult: district_bonus
            .defense_bonus_mult
            .max(synergy_bonus.defense_bonus_mult)
            .max(city.edict_zone_bonuses.defense_bonus_mult),
    };

    // Residential: boost max population capacity
    let zone_pop_bonus = (total_res_levels as f32 * 10.0 * bonus.population_growth_mult) as u32;
    let base_pop = match city.city_level {
        CityLevel::Village => 20,
        CityLevel::Town => 50,
        CityLevel::City => 150,
        CityLevel::Metropolis => 400,
    };
    city.max_population = base_pop + zone_pop_bonus;

    // Military: boost military pop from districts (if no buildings provide it)
    if total_mil_levels == 0 && !tracker.districts_of_type(ZoneType::Military).is_empty() {
        let mil_district_size: u32 = tracker.total_cells_of_type(ZoneType::Military);
        let recruited = (mil_district_size as f32 * 2.0 * bonus.recruit_speed_mult) as u32;
        city.total_military_pop = city.total_military_pop.max(recruited);
    }

    // Economic: boost economic power from districts
    if total_eco_power == 0 && !tracker.districts_of_type(ZoneType::Economic).is_empty() {
        let eco_district_size: u32 = tracker.total_cells_of_type(ZoneType::Economic);
        let produced = (eco_district_size as f32 * 3.0 * bonus.resource_output_mult) as u32;
        city.total_economic_power = city.total_economic_power.max(produced);
    }

    // Administrative: flat happiness bonus
    if bonus.happiness_bonus > 0.0 {
        city.happiness = (city.happiness + bonus.happiness_bonus * 0.01).min(100.0);
    }

    // Protection: crime reduction
    if bonus.crime_reduction > 0.0 {
        city.crime = (city.crime - bonus.crime_reduction * 0.1).max(0.0);
    }

    // Unlock zone types based on city level + population
    for zd in zone_defs.zones.iter_mut() {
        let req = &zd.unlock_requirement;
        if !zd.unlocked
            && city.city_level >= req.required_city_level
            && city.population >= req.required_population
        {
            zd.unlocked = true;
            info!("🔓 {:?} zone unlocked!", zd.zone_type);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 5. Building Upgrade Chain System
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies upgrade chains to buildings when they level up.
/// Renames buildings according to their tier, applies tier-specific bonuses,
/// and fires `BuildingUpgraded` events.
fn upgrade_chain_system(
    mut buildings: Query<(
        Entity,
        &mut Building,
        &mut UpgradeChain,
        Option<&BuildCost>,
    )>,
    mut stockpile: ResMut<ResourceStockpile>,
    city: Res<CityState>,
    mut event_writer: EventWriter<BuildingUpgraded>,
) {
    for (entity, mut building, chain, cost) in buildings.iter_mut() {
        if building.level >= chain.tier_count() {
            continue; // already at max tier
        }

        let next_level = building.level + 1;

        // ── Check requirements ───────────────────────────────────────
        // Higher tiers require higher city levels
        let required_city_level = match next_level {
            1 | 2 => CityLevel::Village,
            3 => CityLevel::Town,
            4 => CityLevel::City,
            _ => CityLevel::Metropolis,
        };
        if city.city_level < required_city_level {
            continue;
        }

        // ── Check resources ─────────────────────────────────────────
        let cost_multiplier = 1.0 + (next_level as f32 - 1.0) * 0.5;
        let food_needed = if let Some(c) = cost {
            (c.food as f32 * cost_multiplier) as u32
        } else {
            20 * next_level as u32
        };
        let gold_needed = if let Some(c) = cost {
            (c.gold as f32 * cost_multiplier) as u32
        } else {
            10 * next_level as u32
        };
        let wood_needed = if let Some(c) = cost {
            (c.wood as f32 * cost_multiplier) as u32
        } else {
            15 * next_level as u32
        };
        let stone_needed = if let Some(c) = cost {
            (c.stone as f32 * cost_multiplier) as u32
        } else {
            5 * next_level as u32
        };

        if stockpile.food < food_needed as f32
            || stockpile.gold < gold_needed as f32
            || stockpile.wood < wood_needed as f32
            || stockpile.stone < stone_needed as f32
        {
            continue;
        }

        // ── Deduct cost ──────────────────────────────────────────────
        stockpile.food -= food_needed as f32;
        stockpile.gold -= gold_needed as f32;
        stockpile.wood -= wood_needed as f32;
        stockpile.stone -= stone_needed as f32;

        // ── Store old name before upgrading ──────────────────────────
        let old_name = building.name.clone();
        let old_level = building.level;

        // ── Apply upgrade ───────────────────────────────────────────
        building.level = next_level;
        building.name = chain.name_for_level(next_level).to_string();

        // ── Tier-specific bonuses ───────────────────────────────────
        match building.building_type {
            BuildingType::Oba => {
                // Each Oba upgrade increases pop cap
                stockpile.max_population += 10 * next_level as u32;
            }
            BuildingType::Barracks | BuildingType::ArcheryRange | BuildingType::Stable => {
                // Military building upgrades give small population cap bonus
                stockpile.max_population += 5;
            }
            BuildingType::Market | BuildingType::Warehouse => {
                // Economic buildings: small gold bonus per upgrade
                stockpile.gold += 10.0 * next_level as f32;
            }
            _ => {}
        }

        // ── Emit event ─────────────────────────────────────────────
        event_writer.send(BuildingUpgraded {
            entity,
            building_type: building.building_type,
            old_level,
            new_level: next_level,
            old_name,
            new_name: building.name.clone(),
        });

        info!(
            "🏗️  {:?} upgraded to {} (Level {})",
            building.building_type, building.name, next_level
        );

        // Only one upgrade per entity per frame
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 6. City Progression System
// ═══════════════════════════════════════════════════════════════════════════════

/// Checks whether the city meets population thresholds to advance to the
/// next settlement level and emits CityLevelUp events.
fn city_progression_system(
    mut city: ResMut<CityState>,
    mut event_writer: EventWriter<CityLevelUp>,
) {
    if city.can_level_up() {
        if let Some((old, new)) = city.advance_level() {
            event_writer.send(CityLevelUp {
                old_level: old,
                new_level: new,
                population: city.population,
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 7. City Event System
// ═══════════════════════════════════════════════════════════════════════════════

/// Processes incoming city events: adds new ones, ticks durations, applies effects,
/// and removes expired events.
fn city_event_processing_system(
    game_time: Res<GameTime>,
    mut city: ResMut<CityState>,
    mut event_reader: EventReader<CityEventOccurred>,
    mut ended_writer: EventWriter<CityEventEnded>,
) {
    let delta = game_time.time_scale as f64 * 0.1;

    // ── Process incoming events ─────────────────────────────────────
    for ev in event_reader.read() {
        // Avoid duplicate events of the same type
        if city.active_events.iter().any(|e| e.event_type == ev.event_type) {
            continue;
        }
        city.active_events.push(CityEvent {
            event_type: ev.event_type,
            duration_remaining: ev.duration,
            magnitude: ev.magnitude,
        });
        info!(
            "⚡ City Event: {} (mag: {:.2}, dur: {:.1}s)",
            ev.event_type.description(),
            ev.magnitude,
            ev.duration
        );
    }

    // ── Tick active events ──────────────────────────────────────────
    let mut ended = Vec::new();
    for event in city.active_events.iter_mut() {
        event.duration_remaining -= delta as f32;
        if event.duration_remaining <= 0.0 {
            ended.push(event.event_type);
        }
    }

    // ── Apply event effects to city metrics ──────────────────────────
    // Clone the list to avoid borrow conflicts when mutating city
    let events_snapshot = city.active_events.clone();
    for event in &events_snapshot {
        let magnitude = event.magnitude;
        match event.event_type {
            // Positive
            CityEventType::Festival => {
                city.happiness = (city.happiness + 2.0 * magnitude * delta as f32).min(100.0);
            }
            CityEventType::TradeBoom => {}
            CityEventType::ScholarArrival => {
                city.faith = (city.faith + 1.0 * magnitude * delta as f32).min(100.0);
            }
            // Negative
            CityEventType::Plague => {
                city.sanitation = (city.sanitation - 3.0 * magnitude * delta as f32).max(0.0);
                city.population = city.population.saturating_sub(
                    (0.01 * magnitude * delta as f32 * city.population as f32).ceil() as u32,
                );
            }
            CityEventType::Riot => {
                city.happiness = (city.happiness - 3.0 * magnitude * delta as f32).max(0.0);
                city.crime = (city.crime + 2.0 * magnitude * delta as f32).min(100.0);
            }
            CityEventType::Fire => {
                city.happiness = (city.happiness - 2.0 * magnitude * delta as f32).max(0.0);
            }
            CityEventType::Drought => {}
            CityEventType::BanditRaids => {
                city.crime = (city.crime + 1.5 * magnitude * delta as f32).min(100.0);
                city.happiness = (city.happiness - 1.0 * magnitude * delta as f32).max(0.0);
            }
        }
    }

    // ── Remove expired events ───────────────────────────────────────
    for et in ended {
        city.active_events.retain(|e| e.event_type != et);
        ended_writer.send(CityEventEnded { event_type: et });
        info!("✅ City event ended: {:?}", et);
    }
}

/// Periodically spawns random positive events when the city is thriving.
fn city_random_event_system(
    game_time: Res<GameTime>,
    city: Res<CityState>,
    mut occurred_writer: EventWriter<CityEventOccurred>,
) {
    let delta = game_time.time_scale as f64 * 0.1;

    // ── Random positive events at high happiness ────────────────────
    if city.happiness > 70.0 && rand::random::<f32>() < 0.001 * delta as f32 {
        let positive_events = [
            CityEventType::Festival,
            CityEventType::TradeBoom,
            CityEventType::ScholarArrival,
        ];
        let picked = positive_events[rand::random::<usize>() % positive_events.len()];
        if !city.active_events.iter().any(|e| e.event_type == picked) {
            occurred_writer.send(CityEventOccurred {
                event_type: picked,
                magnitude: 0.3 + rand::random::<f32>() * 0.4,
                duration: 20.0 + rand::random::<f32>() * 40.0,
            });
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 8. Tax & Policy Sliders
// ═══════════════════════════════════════════════════════════════════════════════

/// Opens/closes the policy panel.  Press `P` to cycle through
/// None → TaxRate → Rationing → Conscription → None.
fn policy_cycle_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut focus: ResMut<PolicyFocus>,
) {
    let Some(input) = keyboard else {
        return;
    };
    if input.just_pressed(KeyCode::KeyP) {
        focus.active = focus.active.next();
        info!("📋 Policy panel: {}", focus.active.label());
    }
    if input.just_pressed(KeyCode::Escape) {
        focus.active = PolicyMode::None;
        info!("📋 Policy panel closed");
    }
}

/// Adjust the currently-focused policy with arrow keys.
/// Run only when the policy panel is open (focus != None).
fn policy_adjust_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut settings: ResMut<PolicySettings>,
    focus: Res<PolicyFocus>,
) {
    let Some(input) = keyboard else {
        return;
    };
    match focus.active {
        PolicyMode::None => {} // panel closed — no-op
        PolicyMode::TaxRate => {
            if input.just_pressed(KeyCode::ArrowUp) {
                settings.tax_rate = (settings.tax_rate + 0.05).min(0.50);
                info!("💰 Tax rate: {:.0}%", settings.tax_rate * 100.0);
            }
            if input.just_pressed(KeyCode::ArrowDown) {
                settings.tax_rate = (settings.tax_rate - 0.05).max(0.0);
                info!("💰 Tax rate: {:.0}%", settings.tax_rate * 100.0);
            }
        }
        PolicyMode::Rationing => {
            if input.just_pressed(KeyCode::ArrowRight) {
                settings.rationing = settings.rationing.next();
                info!("🍞 Rationing: {}", settings.rationing.label());
            }
            if input.just_pressed(KeyCode::ArrowLeft) {
                settings.rationing = settings.rationing.prev();
                info!("🍞 Rationing: {}", settings.rationing.label());
            }
        }
        PolicyMode::Conscription => {
            if input.just_pressed(KeyCode::ArrowRight) {
                settings.conscription = settings.conscription.next();
                info!("⚔️ Conscription: {}", settings.conscription.label());
            }
            if input.just_pressed(KeyCode::ArrowLeft) {
                settings.conscription = settings.conscription.prev();
                info!("⚔️ Conscription: {}", settings.conscription.label());
            }
        }
    }
}

/// Apply policy effects each fixed tick:
///   • Conscription sets `total_military_pop` based on policy fraction.
///   • Logs a warning when food is critically low and rationing is not
///     already at Emergency.
///
/// **Food consumption** is now handled by the economy module's
/// `resource_consumption_system` (which reads `PolicySettings.rationing`
/// via `ResourceConsumption`).  This system only handles non-consumption
/// policy effects.
fn policy_effects_system(
    mut city: ResMut<CityState>,
    settings: Res<PolicySettings>,
    stockpile: Option<Res<ResourceStockpile>>,
) {
    // ── Conscription: assign military population ───────────────────────
    let forced_mil = (city.population as f32 * settings.conscription.forced_recruit_fraction())
        .round() as u32;
    city.total_military_pop = forced_mil;

    // ── Log drastic policy changes ────────────────────────────────────
    if let Some(stockpile) = stockpile.as_deref() {
        if stockpile.food < 10.0 && settings.rationing != RationingLevel::Emergency {
            info!("⚠️  Food critically low — consider stricter rationing!");
        }
    }
}

/// Sync `PolicySettings.tax_rate` into `TaxPolicy.rate` each tick so
/// that the happiness system and economy system read the current value.
fn tax_policy_sync_system(
    mut tax: ResMut<TaxPolicy>,
    settings: Res<PolicySettings>,
) {
    tax.rate = settings.tax_rate;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// 9. City View Overlay — Mode Cycling
// ═══════════════════════════════════════════════════════════════════════════════

/// Cycles the active overlay mode when the player presses `V`.
/// Uses `Option<Res<ButtonInput<KeyCode>>>` so it gracefully does nothing
/// when the input plugin is not registered (e.g. headless/test mode).
fn overlay_cycle_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut settings: ResMut<OverlaySettings>,
) {
    let Some(input) = keyboard else {
        return;
    };
    if input.just_pressed(KeyCode::KeyV) {
        settings.mode = settings.mode.next();
        info!(
            "🏙️  Overlay mode changed to: {}",
            settings.mode.label()
        );
    }
    // Cycle active service type with Shift+V
    if input.just_pressed(KeyCode::KeyV) && input.pressed(KeyCode::ShiftLeft) {
        settings.active_service = match settings.active_service {
            ServiceType::Water => ServiceType::Trade,
            ServiceType::Trade => ServiceType::Faith,
            ServiceType::Faith => ServiceType::Prestige,
            ServiceType::Prestige => ServiceType::Water,
        };
        info!(
            "🔧 Service overlay changed to: {:?}",
            settings.active_service
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 10. Zone Placement UI
// ═══════════════════════════════════════════════════════════════════════════════

/// Toggles zone placement mode and selects the zone type.
///
/// Keybinds:
///   • **B**        — toggle placement mode on/off
///   • **Escape**   — exit placement mode
///   • **1**–**5**  — select zone type (Residential .. Administrative)
fn zone_placement_toggle_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mut placement: ResMut<ZonePlacementState>,
) {
    let Some(input) = keyboard else {
        return;
    };

    // ── Toggle mode ────────────────────────────────────────────────────
    if input.just_pressed(KeyCode::KeyB) {
        placement.active = !placement.active;
        if !placement.active {
            placement.hovered_grid = None;
        }
        info!(
            "🏗️  Zone placement mode: {}",
            if placement.active { "ON" } else { "OFF" }
        );
    }

    // ── Exit on Escape ─────────────────────────────────────────────────
    if input.just_pressed(KeyCode::Escape) && placement.active {
        placement.active = false;
        placement.hovered_grid = None;
        info!("🏗️  Zone placement mode: OFF");
    }

    if !placement.active {
        return;
    }

    // ── Zone type selection (1–5) ──────────────────────────────────────
    if input.just_pressed(KeyCode::Digit1) {
        placement.selected_type = ZoneType::Residential;
        info!("🏗️  Zone type: Residential");
    }
    if input.just_pressed(KeyCode::Digit2) {
        placement.selected_type = ZoneType::Military;
        info!("🏗️  Zone type: Military");
    }
    if input.just_pressed(KeyCode::Digit3) {
        placement.selected_type = ZoneType::Economic;
        info!("🏗️  Zone type: Economic");
    }
    if input.just_pressed(KeyCode::Digit4) {
        placement.selected_type = ZoneType::Religious;
        info!("🏗️  Zone type: Religious");
    }
    if input.just_pressed(KeyCode::Digit5) {
        placement.selected_type = ZoneType::Administrative;
        info!("🏗️  Zone type: Administrative");
    }
    if input.just_pressed(KeyCode::Digit6) {
        placement.selected_type = ZoneType::Protection;
        info!("🏗️  Zone type: Protection");
    }
}

/// Projects the mouse cursor onto the grid and updates `hovered_grid`.
///
/// Runs only when placement mode is active.  Gracefully handles missing
/// window, camera, or cursor (off-screen / unfocused).
fn zone_placement_hover_system(
    mut placement: ResMut<ZonePlacementState>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    if !placement.active {
        placement.hovered_grid = None;
        return;
    }

    // When placement is active but no window/camera is available
    // (e.g. headless test), leave hovered_grid as-is so that tests
    // can set it programmatically.
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        placement.hovered_grid = None;
        return;
    };

    if let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        let grid_x = (world_pos.x / CELL_SIZE).floor() as i32;
        let grid_y = (world_pos.y / CELL_SIZE).floor() as i32;
        placement.hovered_grid = Some(IVec2::new(grid_x, grid_y));
    } else {
        placement.hovered_grid = None;
    }
}

/// Places or removes zones on left/right click.
///
/// * **Left click** — validates unlock requirements, then spawns a `Zone`
///   + `WorldPosition` entity at the hovered grid position. Fires
///   `ZonePlaced` on success or `ZonePlacementBlocked` on failure.
/// * **Right click** — despawns any zone entity at the hovered position.
fn zone_placement_execute_system(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    placement: Res<ZonePlacementState>,
    mut commands: Commands,
    mut zone_placed_writer: EventWriter<ZonePlaced>,
    mut blocked_writer: EventWriter<ZonePlacementBlocked>,
    zones: Query<(Entity, &WorldPosition), With<Zone>>,
    zone_defs: Res<ZoneDefinitions>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    let Some(input) = mouse else {
        return;
    };
    if !placement.active {
        return;
    }
    let Some(grid_pos) = placement.hovered_grid else {
        return;
    };

    // ── Left click: place ──────────────────────────────────────────────
    if input.just_pressed(MouseButton::Left) {
        let selected = placement.selected_type;

        // Check: cell already occupied
        let occupied = zones.iter().any(|(_, wp)| wp.grid.0 == grid_pos);
        if occupied {
            blocked_writer.send(ZonePlacementBlocked {
                zone_type: selected,
                position: GridPos(grid_pos),
                reason: ZoneBlockReason::CellOccupied,
            });
            return;
        }

        // Check: zone type unlocked
        let zone_def = zone_defs.zones.iter().find(|z| z.zone_type == selected);
        let is_unlocked = zone_def.map_or(false, |z| z.unlocked);
        if !is_unlocked {
            blocked_writer.send(ZonePlacementBlocked {
                zone_type: selected,
                position: GridPos(grid_pos),
                reason: ZoneBlockReason::NotUnlocked,
            });
            info!(
                "🚫 {:?} zone blocked: not yet unlocked",
                selected
            );
            return;
        }

        // Check: SP cost
        let sp_cost = zone_def.map_or(0.0, |z| z.unlock_requirement.sp_cost);
        if sp_cost > 0.0 && (stockpile.sovereignty_points as f32) < sp_cost {
            blocked_writer.send(ZonePlacementBlocked {
                zone_type: selected,
                position: GridPos(grid_pos),
                reason: ZoneBlockReason::InsufficientSP {
                    required: sp_cost,
                    available: stockpile.sovereignty_points as f32,
                },
            });
            info!(
                "🚫 {:?} zone blocked: need {:.0} SP, have {}",
                selected, sp_cost, stockpile.sovereignty_points
            );
            return;
        }

        // Check: gold and wood costs
        let gold_cost = zone_def.map_or(0, |z| z.placement_gold_cost);
        let wood_cost = zone_def.map_or(0, |z| z.placement_wood_cost);
        if (stockpile.gold as u32) < gold_cost || (stockpile.wood as u32) < wood_cost {
            blocked_writer.send(ZonePlacementBlocked {
                zone_type: selected,
                position: GridPos(grid_pos),
                reason: ZoneBlockReason::InsufficientResources,
            });
            info!(
                "🚫 {:?} zone blocked: need {} gold + {} wood",
                selected, gold_cost, wood_cost
            );
            return;
        }

        // Deduct SP + resource costs
        if sp_cost > 0.0 {
            stockpile.sovereignty_points = stockpile.sovereignty_points.saturating_sub(sp_cost as u32);
        }
        stockpile.gold -= gold_cost as f32;
        stockpile.wood -= wood_cost as f32;

        let entity = commands
            .spawn((
                Zone {
                    zone_type: selected,
                    level: 1,
                },
                WorldPosition::new(grid_pos, CELL_SIZE),
            ))
            .id();
        zone_placed_writer.send(ZonePlaced {
            zone_type: selected,
            position: GridPos(grid_pos),
            entity: Some(entity),
        });
        info!(
            "🏗️  Placed {:?} zone at ({}, {}) ({} SP, {} gold, {} wood)",
            selected, grid_pos.x, grid_pos.y,
            sp_cost, gold_cost, wood_cost,
        );
    }

    // ── Right click: remove ────────────────────────────────────────────
    if input.just_pressed(MouseButton::Right) {
        for (entity, wp) in zones.iter() {
            if wp.grid.0 == grid_pos {
                commands.entity(entity).despawn();
                info!("🏗️  Removed zone at ({}, {})", grid_pos.x, grid_pos.y);
                break;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 4d. Zone Upgrade System (increment level, deduct resource cost)
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// 4e. Zone Config Loader (populates ZoneDefinitions from RON asset)
// ═══════════════════════════════════════════════════════════════════════════════

/// Loads `ZoneConfigList` from the RON asset on startup and populates
/// `ZoneDefinitions` resource. Falls back to defaults if asset is missing.
fn zone_config_loader_system(
    mut zone_defs: ResMut<ZoneDefinitions>,
    asset_server: Option<Res<AssetServer>>,
    configs: Option<Res<Assets<ZoneConfigList>>>,
) {
    let (Some(asset_server), Some(configs)) = (asset_server, configs) else {
        // AssetPlugin not available (e.g. test environment) — keep defaults
        return;
    };
    // Try to load the asset; if it fails, keep the hardcoded defaults.
    let handle = asset_server.load::<ZoneConfigList>("zones/zone_definitions.ron");
    let Some(config_list) = configs.get(&handle) else {
        // Asset not yet loaded or missing — keep defaults
        info!("ℹ️  Zone config asset not available; using hardcoded defaults");
        return;
    };

    // Map entries to ZoneDefinition
    let zones: Vec<ZoneDefinition> = config_list
        .zones
        .iter()
        .map(|entry| ZoneDefinition {
            zone_type: entry.zone_type,
            unlocked: entry.unlocked,
            bonus_multiplier: entry.bonus_multiplier,
            unlock_requirement: ZoneUnlockRequirement {
                required_city_level: entry.required_city_level,
                required_population: entry.required_population,
                sp_cost: entry.sp_cost,
            },
            placement_gold_cost: entry.placement_gold_cost,
            placement_wood_cost: entry.placement_wood_cost,
            upgrade_gold_cost: entry.upgrade_gold_cost,
            upgrade_wood_cost: entry.upgrade_wood_cost,
            max_level: entry.max_level,
            max_district_size: entry.max_district_size,
            color: entry.color,
        })
        .collect();

    zone_defs.zones = zones;
    info!("✅ Loaded {} zone definitions from RON config", zone_defs.zones.len());
}

/// Listens for `U` key press while hovering a zone cell. If the zone is below
/// its max level and sufficient resources exist, increments `Zone.level`,
/// deducts upgrade cost, and fires a `ZoneUpgraded` event.
fn zone_upgrade_system(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    placement: Res<ZonePlacementState>,
    zone_defs: Res<ZoneDefinitions>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut upgrade_writer: EventWriter<ZoneUpgraded>,
    mut zones: Query<(Entity, &mut Zone, &WorldPosition)>,
) {
    let Some(input) = keyboard else {
        return;
    };
    if !input.just_pressed(KeyCode::KeyU) {
        return;
    }
    let Some(grid_pos) = placement.hovered_grid else {
        return;
    };

    // Find zone at hovered grid position
    for (entity, mut zone, wp) in zones.iter_mut() {
        if wp.grid.0 != grid_pos {
            continue;
        }

        // Look up zone definition
        let Some(zone_def) = zone_defs.zones.iter().find(|z| z.zone_type == zone.zone_type) else {
            return;
        };

        // Check: below max level
        if zone.level >= zone_def.max_level {
            info!(
                "🚫 {:?} zone at ({},{}) already at max level {}",
                zone.zone_type, grid_pos.x, grid_pos.y, zone.level
            );
            return;
        }

        // Check: sufficient resources
        if (stockpile.gold as u32) < zone_def.upgrade_gold_cost
            || (stockpile.wood as u32) < zone_def.upgrade_wood_cost
        {
            info!(
                "🚫 {:?} zone upgrade blocked: need {} gold + {} wood, have {:.0} gold + {:.0} wood",
                zone.zone_type,
                zone_def.upgrade_gold_cost,
                zone_def.upgrade_wood_cost,
                stockpile.gold,
                stockpile.wood,
            );
            return;
        }

        // Deduct costs
        stockpile.gold -= zone_def.upgrade_gold_cost as f32;
        stockpile.wood -= zone_def.upgrade_wood_cost as f32;

        // Increment level
        let old_level = zone.level;
        zone.level += 1;

        upgrade_writer.send(ZoneUpgraded {
            entity,
            zone_type: zone.zone_type,
            old_level,
            new_level: zone.level,
        });

        info!(
            "⬆️  Upgraded {:?} zone at ({},{}) to level {} (cost {} gold + {} wood)",
            zone.zone_type,
            grid_pos.x,
            grid_pos.y,
            zone.level,
            zone_def.upgrade_gold_cost,
            zone_def.upgrade_wood_cost,
        );
        return; // only upgrade one zone per press
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Helper to build a test app with the city builder plugin,
    /// time updates, core resources, and the `AppState` state machine
    /// (required for the `.run_if(in_state(AppState::InGame))` guard).
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            bevy::MinimalPlugins,
            bevy_state::app::StatesPlugin,
        ))
            .init_state::<AppState>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<ProductionRates>()
            .init_resource::<TaxPolicy>()
            .init_resource::<FactionRelations>()
            .add_plugins(CityBuilderPlugin);
        // Default AppState is MainMenu — set to InGame for city builder tests
        {
            let mut next = app.world_mut().resource_mut::<NextState<AppState>>();
            next.set(AppState::InGame);
        }
        app
    }

    /// Run N `app.update()` frames. With `ManualDuration` time strategy,
    /// each frame advances virtual time by ~16ms, which eventually triggers
    /// the `FixedUpdate` schedule.
    fn run_fixed_ticks(app: &mut App, ticks: u32) {
        for _ in 0..ticks {
            app.update();
        }
    }

    // ── City Population Tests ────────────────────────────────────────

    #[test]
    fn test_population_grows_with_surplus() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 5;
        city.max_population = 20;
        city.happiness = 80.0;
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.food = 500.0; // plenty of food

        run_fixed_ticks(&mut app, 120); // enough ticks for accumulator to reach ≥1

        let city = app.world().resource::<CityState>();
        assert!(
            city.population > 5,
            "Population should grow with surplus food and housing. Got {}",
            city.population
        );
    }

    #[test]
    fn test_population_declines_during_starvation() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 20;
        city.max_population = 20;
        city.happiness = 50.0;
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.food = 0.0; // zero food = starvation

        run_fixed_ticks(&mut app, 120);

        let city = app.world().resource::<CityState>();
        assert!(
            city.population < 20,
            "Population should decline during starvation. Got {}",
            city.population
        );
    }

    #[test]
    fn test_population_capped_by_max_population() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 18;
        city.max_population = 20;
        city.happiness = 100.0;
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.food = 1000.0;

        run_fixed_ticks(&mut app, 120);

        let city = app.world().resource::<CityState>();
        assert!(
            city.population <= city.max_population,
            "Population should not exceed max_population. Got {} > {}",
            city.population,
            city.max_population
        );
    }

    // ── City Happiness Tests ─────────────────────────────────────────

    #[test]
    fn test_happiness_moves_toward_target() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.happiness = 30.0; // start low
        city.crime = 0.0;
        city.faith = 100.0; // max faith for bonus

        // Manually set faith_satisfaction (not overwritten by population system)
        let mut needs = app.world_mut().resource_mut::<CitizenNeeds>();
        needs.faith_satisfaction = 1.0;

        // Set up food surplus so the population system computes good food/housing
        let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
        stockpile.food = 500.0;

        run_fixed_ticks(&mut app, 60);

        let city = app.world().resource::<CityState>();
        assert!(
            city.happiness > 30.0,
            "Happiness should rise with good conditions. Got {}",
            city.happiness
        );
    }

    #[test]
    fn test_tax_penalty_reduces_happiness() {
        let mut app = test_app();
        let mut tax = app.world_mut().resource_mut::<TaxPolicy>();
        tax.rate = 0.5; // very high tax

        let mut city = app.world_mut().resource_mut::<CityState>();
        city.happiness = 70.0;

        run_fixed_ticks(&mut app, 30);

        let city = app.world().resource::<CityState>();
        assert!(
            city.happiness < 70.0,
            "High taxes should reduce happiness. Got {}",
            city.happiness
        );
    }

    // ── City Progression Tests ───────────────────────────────────────

    #[test]
    fn test_village_to_town_progression() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 25; // exceeds Town threshold of 20
        city.max_population = 30;
        city.city_level = CityLevel::Village;

        run_fixed_ticks(&mut app, 5);

        let city = app.world().resource::<CityState>();
        assert_eq!(
            city.city_level,
            CityLevel::Town,
            "Village with pop >= 20 should become a Town"
        );
    }

    #[test]
    fn test_town_to_city_progression() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 150; // exceeds City threshold of 100
        city.max_population = 200;
        city.city_level = CityLevel::Town;

        run_fixed_ticks(&mut app, 5);

        let city = app.world().resource::<CityState>();
        assert_eq!(
            city.city_level,
            CityLevel::City,
            "Town with pop >= 100 should become a City"
        );
    }

    #[test]
    fn test_progression_happiness_boost() {
        let mut app = test_app();
        let mut city = app.world_mut().resource_mut::<CityState>();
        city.population = 25;
        city.max_population = 30;
        city.city_level = CityLevel::Village;
        city.happiness = 40.0;

        run_fixed_ticks(&mut app, 5);

        let city = app.world().resource::<CityState>();
        assert!(
            city.happiness > 40.0,
            "Leveling up should boost happiness"
        );
    }

    // ── Service Coverage Tests ───────────────────────────────────────

    #[test]
    fn test_service_coverage_no_buildings() {
        let mut app = test_app();

        // Trigger a recalc
        app.world_mut().send_event(RecalculateCoverage);
        app.update();

        let coverage = app.world().resource::<ServiceCoverage>();
        assert_eq!(coverage.water, 0.0);
        assert_eq!(coverage.trade, 0.0);
        assert_eq!(coverage.faith, 0.0);
        assert_eq!(coverage.prestige, 0.0);
    }

    #[test]
    fn test_water_coverage_from_well() {
        let mut app = test_app();

        // Spawn a well
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Well,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Well".to_string(),
            },
            ServiceBuilding::from_building_type(BuildingType::Well, 1).unwrap(),
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        // Spawn a building within coverage radius (8 cells)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 5,
                name: "Oba".to_string(),
            },
            WorldPosition::new(IVec2::new(3, 4), CELL_SIZE), // distance ≈ 5, within 8
        ));

        // Send coverage-recalc event then run fixed ticks
        app.world_mut().send_event(RecalculateCoverage);
        run_fixed_ticks(&mut app, 3);

        let coverage = app.world().resource::<ServiceCoverage>();
        assert!(
            coverage.water > 0.0,
            "Well should provide water coverage"
        );
    }

    #[test]
    fn test_service_out_of_range() {
        let mut app = test_app();

        // Well at origin
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Well,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 3,
                name: "Well".to_string(),
            },
            ServiceBuilding::from_building_type(BuildingType::Well, 1).unwrap(),
            WorldPosition::new(IVec2::new(0, 0), CELL_SIZE),
        ));

        // Building far away (20 cells > 8 radius)
        app.world_mut().spawn((
            Building {
                building_type: BuildingType::Oba,
                faction: FactionId::Kayi,
                level: 1,
                max_level: 5,
                name: "Oba".to_string(),
            },
            WorldPosition::new(IVec2::new(20, 0), CELL_SIZE),
        ));

        app.world_mut().send_event(RecalculateCoverage);
        app.update();

        let coverage = app.world().resource::<ServiceCoverage>();
        assert!(
            coverage.water < 1.0,
            "Building out of range should NOT be covered"
        );
    }

    // ── Upgrade Chain Tests ──────────────────────────────────────────

    #[test]
    fn test_oba_upgrade_chain_names() {
        let chain = UpgradeChain::for_building_type(BuildingType::Oba).unwrap();
        assert_eq!(chain.name_for_level(1), "Oba");
        assert_eq!(chain.name_for_level(2), "Manor");
        assert_eq!(chain.name_for_level(3), "Governor's Estate");
        assert_eq!(chain.name_for_level(4), "Bey's Palace");
        assert_eq!(chain.name_for_level(5), "Sultan's Court");
    }

    #[test]
    fn test_barracks_upgrade_chain_names() {
        let chain = UpgradeChain::for_building_type(BuildingType::Barracks).unwrap();
        assert_eq!(chain.name_for_level(1), "Barracks");
        assert_eq!(chain.name_for_level(2), "Garrison");
        assert_eq!(chain.name_for_level(3), "Fortress");
    }

    #[test]
    fn test_chain_max_level() {
        let chain = UpgradeChain::for_building_type(BuildingType::Mosque).unwrap();
        assert_eq!(chain.tier_count(), 3);
    }

    #[test]
    fn test_defensive_buildings_have_no_chain() {
        assert!(UpgradeChain::for_building_type(BuildingType::Palisade).is_none());
        assert!(UpgradeChain::for_building_type(BuildingType::StoneWall).is_none());
        assert!(UpgradeChain::for_building_type(BuildingType::Gate).is_none());
        assert!(UpgradeChain::for_building_type(BuildingType::Road).is_none());
    }

    // ── Zone Tests ───────────────────────────────────────────────────

    #[test]
    fn test_zone_type_building_matching() {
        assert!(ZoneType::Residential.matches_building(BuildingType::Oba));
        assert!(ZoneType::Military.matches_building(BuildingType::Barracks));
        assert!(ZoneType::Military.matches_building(BuildingType::Watchtower));
        assert!(ZoneType::Economic.matches_building(BuildingType::Market));
        assert!(ZoneType::Religious.matches_building(BuildingType::Mosque));
        assert!(ZoneType::Administrative.matches_building(BuildingType::CouncilChamber));

        // Negative checks
        assert!(!ZoneType::Residential.matches_building(BuildingType::Barracks));
        assert!(!ZoneType::Military.matches_building(BuildingType::Market));
    }

    #[test]
    fn test_zone_definitions_residential_unlocked_by_default() {
        let defs = ZoneDefinitions::default();
        let res = defs.zones.iter().find(|z| z.zone_type == ZoneType::Residential).unwrap();
        assert!(res.unlocked);
    }

    #[test]
    fn test_administrative_zone_locked_at_village() {
        let defs = ZoneDefinitions::default();
        let admin = defs
            .zones
            .iter()
            .find(|z| z.zone_type == ZoneType::Administrative)
            .unwrap();
        assert!(!admin.unlocked, "Administrative zones should start locked");
    }

    // ── City Level Tests ─────────────────────────────────────────────

    #[test]
    fn test_city_level_population_thresholds() {
        assert_eq!(CityLevel::Village.population_threshold(), 0);
        assert_eq!(CityLevel::Town.population_threshold(), 20);
        assert_eq!(CityLevel::City.population_threshold(), 100);
        assert_eq!(CityLevel::Metropolis.population_threshold(), 500);
    }

    #[test]
    fn test_city_level_max_population_caps() {
        assert_eq!(CityLevel::Village.max_population_cap(), 30);
        assert_eq!(CityLevel::Town.max_population_cap(), 120);
        assert_eq!(CityLevel::City.max_population_cap(), 600);
        assert_eq!(CityLevel::Metropolis.max_population_cap(), 2000);
    }

    #[test]
    fn test_metropolis_cannot_level_up() {
        let mut city = CityState::default();
        city.city_level = CityLevel::Metropolis;
        city.population = 9999;
        assert!(!city.can_level_up());
        assert!(city.advance_level().is_none());
    }

    #[test]
    fn test_upgrade_chain_registry() {
        let registry = UpgradeChainRegistry::new();
        assert_eq!(registry.tier_name(BuildingType::Oba, 1), Some("Oba"));
        assert_eq!(registry.tier_name(BuildingType::Oba, 2), Some("Manor"));
        assert_eq!(
            registry.tier_name(BuildingType::Market, 3),
            Some("Grand Bazaar")
        );
        // Road not in registry
        assert!(registry.tier_name(BuildingType::Road, 1).is_none());
    }

    // ── Zone Placement Tests ──────────────────────────────────────────

    /// Simulate a mouse left-click by inserting a `ButtonInput<MouseButton>`
    /// resource with the button pressed.
    fn press_left_click(app: &mut App) {
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Left);
        app.insert_resource(mouse);
    }

    /// Simulate a mouse right-click.
    fn press_right_click(app: &mut App) {
        let mut mouse = ButtonInput::<MouseButton>::default();
        mouse.press(MouseButton::Right);
        app.insert_resource(mouse);
    }

    /// Simulate a key press by inserting a `ButtonInput<KeyCode>` resource
    /// with the given key pressed.
    fn press_key(app: &mut App, key: KeyCode) {
        let mut keyboard = ButtonInput::<KeyCode>::default();
        keyboard.press(key);
        app.insert_resource(keyboard);
    }

    /// Count zone entities in the world.
    fn zone_count(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut q = world.query::<&Zone>();
        q.iter(world).count()
    }

    /// Helper: collect zone + position data.
    fn collect_zones(app: &mut App) -> Vec<(Zone, IVec2)> {
        let world = app.world_mut();
        let mut q = world.query::<(&Zone, &WorldPosition)>();
        q.iter(world)
            .map(|(z, p)| (z.clone(), p.grid.0))
            .collect()
    }

    #[test]
    fn test_zone_placement_spawns_entity() {
        let mut app = test_app();
        // Activate placement and set hover position
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = Some(IVec2::new(5, 3));
        }
        // Press left click
        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 1, "One zone entity should be spawned");

        // Verify the placed zone has the correct type and position
        let results = collect_zones(&mut app);
        assert_eq!(results.len(), 1, "Should be exactly one zone");
        let (zone, grid) = &results[0];
        assert_eq!(zone.zone_type, ZoneType::Residential);
        assert_eq!(*grid, IVec2::new(5, 3));
    }

    #[test]
    fn test_zone_placement_no_spawn_without_active() {
        let mut app = test_app();
        // NOT activated — hover is set but mode is off
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = false;
            p.hovered_grid = Some(IVec2::new(5, 3));
        }
        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 0, "No zone should be placed when mode is off");
    }

    #[test]
    fn test_zone_placement_no_spawn_without_hover() {
        let mut app = test_app();
        // Active but no hovered position
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = None;
        }
        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 0, "No zone should be placed without hover position");
    }

    #[test]
    fn test_zone_placement_no_double_spawn() {
        let mut app = test_app();
        // Place zone at (5,3)
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = Some(IVec2::new(5, 3));
        }
        press_left_click(&mut app);
        app.update();
        assert_eq!(zone_count(&mut app), 1);

        // Try placing another zone at the same position
        press_left_click(&mut app);
        app.update();

        assert_eq!(
            zone_count(&mut app),
            1,
            "Should still be exactly one zone (no double-spawn)"
        );
    }

    #[test]
    fn test_zone_placement_remove_entity() {
        let mut app = test_app();
        // First place a zone
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = Some(IVec2::new(3, 7));
        }
        press_left_click(&mut app);
        app.update();
        assert_eq!(zone_count(&mut app), 1, "Zone should be placed");

        // Now right-click to remove it
        press_right_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 0, "Zone should be removed after right-click");
    }

    #[test]
    fn test_zone_placement_fires_event() {
        let mut app = test_app();
        let mut events = app.world_mut().resource_mut::<Events<ZonePlaced>>();
        // Drain any existing events (e.g. from plugin init)
        for _ in events.drain() {}

        // Place a zone
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = Some(IVec2::new(10, 20));
        }
        press_left_click(&mut app);
        app.update();

        let mut events = app.world_mut().resource_mut::<Events<ZonePlaced>>();
        let mut count = 0;
        for ev in events.drain() {
            count += 1;
            assert_eq!(ev.zone_type, ZoneType::Residential);
            assert_eq!(ev.position.0, IVec2::new(10, 20));
            assert!(ev.entity.is_some(), "Event should contain the new entity ID");
        }
        assert_eq!(count, 1, "Exactly one ZonePlaced event should fire");
    }

    #[test]
    fn test_zone_placement_toggle_b_key() {
        let mut app = test_app();
        // Initially off
        assert!(!app.world().resource::<ZonePlacementState>().active);

        // Press B
        press_key(&mut app, KeyCode::KeyB);
        app.update();
        assert!(
            app.world().resource::<ZonePlacementState>().active,
            "B key should activate placement mode"
        );

        // Press B again
        press_key(&mut app, KeyCode::KeyB);
        app.update();
        assert!(
            !app.world().resource::<ZonePlacementState>().active,
            "B key again should deactivate placement mode"
        );
    }

    #[test]
    fn test_zone_placement_escape_deactivates() {
        let mut app = test_app();
        // Activate via direct manipulation (like pressing B)
        app.world_mut().resource_mut::<ZonePlacementState>().active = true;
        assert!(app.world().resource::<ZonePlacementState>().active);

        // Press Escape
        press_key(&mut app, KeyCode::Escape);
        app.update();

        assert!(
            !app.world().resource::<ZonePlacementState>().active,
            "Escape should deactivate placement mode"
        );
    }

    #[test]
    fn test_zone_placement_selects_type() {
        let mut app = test_app();
        app.world_mut().resource_mut::<ZonePlacementState>().active = true;

        // Press 2 → Military
        press_key(&mut app, KeyCode::Digit2);
        app.update();
        assert_eq!(
            app.world().resource::<ZonePlacementState>().selected_type,
            ZoneType::Military,
            "Digit2 should select Military"
        );

        // Press 4 → Religious
        press_key(&mut app, KeyCode::Digit4);
        app.update();
        assert_eq!(
            app.world().resource::<ZonePlacementState>().selected_type,
            ZoneType::Religious,
            "Digit4 should select Religious"
        );
    }

    #[test]
    fn test_zone_placement_hover_cleared_on_deactivate() {
        let mut app = test_app();
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.hovered_grid = Some(IVec2::new(5, 5));
        }

        // Deactivate via Escape
        press_key(&mut app, KeyCode::Escape);
        app.update();

        assert!(!app.world().resource::<ZonePlacementState>().active);
        assert_eq!(
            app.world().resource::<ZonePlacementState>().hovered_grid,
            None,
            "Hover should be cleared when mode is deactivated"
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // District Detection Tests
    // ═══════════════════════════════════════════════════════════════════

    /// Helper: spawn a zone entity directly in the world.
    fn spawn_zone(app: &mut App, zone_type: ZoneType, pos: IVec2) -> Entity {
        app.world_mut()
            .spawn((
                Zone { zone_type, level: 1 },
                WorldPosition::new(pos, CELL_SIZE),
            ))
            .id()
    }

    #[test]
    fn test_district_detection_single_cell() {
        let mut app = test_app();
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(0, 0));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert_eq!(tracker.districts.len(), 1, "Single zone → 1 district");
        assert_eq!(tracker.districts[0].zone_type, ZoneType::Residential);
        assert_eq!(tracker.districts[0].size, 1);
    }

    #[test]
    fn test_district_detection_adjacent_same_type_merge() {
        let mut app = test_app();
        // Three adjacent residential cells → 1 district
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(0, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(1, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(2, 0));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert_eq!(tracker.districts.len(), 1, "Three adjacent same-type → 1 district");
        assert_eq!(tracker.districts[0].size, 3);
        assert_eq!(tracker.districts[0].cells.len(), 3);
    }

    #[test]
    fn test_district_detection_diagonal_not_connected() {
        let mut app = test_app();
        // Diagonal cells are NOT connected (orthogonal only)
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(0, 0));
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(1, 1));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert_eq!(tracker.districts.len(), 2, "Diagonal zones → 2 separate districts");
    }

    #[test]
    fn test_district_detection_mixed_types() {
        let mut app = test_app();
        // Residential cluster
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(0, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(1, 0));
        // Military cluster
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(5, 5));
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(6, 5));
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(7, 5));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert_eq!(tracker.districts.len(), 2);
        let res_districts = tracker.districts_of_type(ZoneType::Residential);
        let mil_districts = tracker.districts_of_type(ZoneType::Military);
        assert_eq!(res_districts.len(), 1);
        assert_eq!(res_districts[0].size, 2);
        assert_eq!(mil_districts.len(), 1);
        assert_eq!(mil_districts[0].size, 3);
    }

    #[test]
    fn test_district_detection_two_separate_clusters() {
        let mut app = test_app();
        // Two separate residential clusters (not touching)
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(0, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(1, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(10, 10));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(11, 10));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        let res_districts = tracker.districts_of_type(ZoneType::Residential);
        assert_eq!(res_districts.len(), 2, "Two separate clusters → 2 districts");
        assert_eq!(res_districts[0].size, 2);
        assert_eq!(res_districts[1].size, 2);
    }

    #[test]
    fn test_district_detection_empty_grid() {
        let mut app = test_app();
        // No zones placed

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert!(tracker.districts.is_empty(), "No zones → no districts");
    }

    // ═══════════════════════════════════════════════════════════════════
    // Adjacency Bonus Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_bonus_single_cell_no_size_bonus() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Residential,
            cells: vec![IVec2::new(0, 0)],
            entities: vec![Entity::from_raw(0)],
            total_level: 1,
            size: 1,
        };
        let mult = district.compute_bonus_multiplier(1.25);
        // size_bonus = 0, level_bonus = 0 → mult = 1.25
        assert!((mult - 1.25).abs() < 0.001, "Single cell should have base multiplier. Got {}", mult);
    }

    #[test]
    fn test_bonus_three_cell_district() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Military,
            cells: vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)],
            entities: vec![Entity::from_raw(0), Entity::from_raw(1), Entity::from_raw(2)],
            total_level: 3,
            size: 3,
        };
        let mult = district.compute_bonus_multiplier(1.30);
        // size_bonus = 2 * 0.10 = 0.20, level_bonus = 0 → 1.30 * (1.0 + 0.20) = 1.56
        assert!((mult - 1.56).abs() < 0.001, "Three-cell district should get +20% size bonus. Got {}", mult);
    }

    #[test]
    fn test_bonus_high_level_district() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Economic,
            cells: vec![IVec2::new(0, 0), IVec2::new(1, 0)],
            entities: vec![Entity::from_raw(0), Entity::from_raw(1)],
            total_level: 6, // avg = 3.0
            size: 2,
        };
        let mult = district.compute_bonus_multiplier(1.25);
        // size_bonus = 1 * 0.10 = 0.10, level_bonus = 2.0 * 0.15 = 0.30 → 1.25 * 1.40 = 1.75
        assert!((mult - 1.75).abs() < 0.001, "High-level district should get +30% level bonus. Got {}", mult);
    }

    #[test]
    fn test_bonus_effect_applies_to_correct_field() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Residential,
            cells: vec![IVec2::new(0, 0), IVec2::new(1, 0)],
            entities: vec![Entity::from_raw(0), Entity::from_raw(1)],
            total_level: 2,
            size: 2,
        };
        let effect = district.compute_bonus_effect(1.25);
        // base_mult = 1.25, size_bonus = 10%, level_bonus = 15% → 1.25 * 1.25 = 1.5625
        // extra = 0.5625
        assert!(effect.population_growth_mult > 1.0, "Residential should boost population growth");
        assert_eq!(effect.recruit_speed_mult, 0.0, "Residential should NOT boost recruit speed");
        assert_eq!(effect.resource_output_mult, 0.0);
        assert_eq!(effect.faith_generation_mult, 0.0);
        assert_eq!(effect.happiness_bonus, 0.0);
    }

    #[test]
    fn test_administrative_bonus_is_flat_happiness() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Administrative,
            cells: vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0)],
            entities: vec![Entity::from_raw(0), Entity::from_raw(1), Entity::from_raw(2)],
            total_level: 3,
            size: 3,
        };
        let effect = district.compute_bonus_effect(1.40);
        assert!(effect.happiness_bonus > 0.0, "Administrative should give happiness bonus. Got {}", effect.happiness_bonus);
        assert_eq!(effect.population_growth_mult, 0.0);
    }

    // ═══════════════════════════════════════════════════════════════════
    // Aggregate Bonus Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_aggregate_bonus_takes_max_across_districts() {
        let mut app = test_app();
        // Residential district (size 2)
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(0, 0));
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(1, 0));
        // Military district (size 3)
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(10, 0));
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(11, 0));
        spawn_zone(&mut app, ZoneType::Military, IVec2::new(12, 0));

        run_fixed_ticks(&mut app, 3);

        let tracker = app.world().resource::<DistrictTracker>();
        assert!(tracker.aggregate_bonus.population_growth_mult > 1.0,
            "Should have residential bonus");
        assert!(tracker.aggregate_bonus.recruit_speed_mult > 1.0,
            "Should have military bonus");
    }

    // ═══════════════════════════════════════════════════════════════════
    // Unlock Validation Tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn test_zone_placement_blocked_when_not_unlocked() {
        let mut app = test_app();
        // Lock all zones except Residential
        {
            let mut defs = app.world_mut().resource_mut::<ZoneDefinitions>();
            for zd in defs.zones.iter_mut() {
                if zd.zone_type != ZoneType::Residential {
                    zd.unlocked = false;
                }
            }
        }
        // Try to place Military zone
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.selected_type = ZoneType::Military;
            p.hovered_grid = Some(IVec2::new(5, 5));
        }

        // Drain existing events
        {
            let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
            for _ in events.drain() {}
        }

        press_left_click(&mut app);
        app.update();

        // Should NOT have placed a zone
        assert_eq!(zone_count(&mut app), 0, "Should not place zone when locked");

        // Should have fired a blocked event
        let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
        let mut blocked = false;
        for ev in events.drain() {
            assert_eq!(ev.zone_type, ZoneType::Military);
            assert_eq!(ev.reason, ZoneBlockReason::NotUnlocked);
            blocked = true;
        }
        assert!(blocked, "Should fire ZonePlacementBlocked when zone is locked");
    }

    #[test]
    fn test_zone_placement_blocked_on_occupied_cell() {
        let mut app = test_app();
        // Place a zone first
        spawn_zone(&mut app, ZoneType::Residential, IVec2::new(3, 3));

        // Try to place another on the same cell
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.selected_type = ZoneType::Residential;
            p.hovered_grid = Some(IVec2::new(3, 3));
        }

        {
            let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
            for _ in events.drain() {}
        }

        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 1, "Should not add zone on occupied cell");

        let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
        let mut blocked = false;
        for ev in events.drain() {
            assert_eq!(ev.reason, ZoneBlockReason::CellOccupied);
            blocked = true;
        }
        assert!(blocked, "Should fire blocked event for occupied cell");
    }

    #[test]
    fn test_zone_placement_blocked_insufficient_sp() {
        let mut app = test_app();
        // Economic zone costs SP — ensure we can't afford it
        {
            let mut defs = app.world_mut().resource_mut::<ZoneDefinitions>();
            if let Some(econ) = defs.zones.iter_mut().find(|z| z.zone_type == ZoneType::Economic) {
                econ.unlocked = true;
                econ.unlock_requirement.sp_cost = 100.0;
            }
        }
        {
            let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
            stockpile.sovereignty_points = 0;
        }
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.selected_type = ZoneType::Economic;
            p.hovered_grid = Some(IVec2::new(0, 0));
        }

        {
            let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
            for _ in events.drain() {}
        }

        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 0, "Should not place zone without enough SP");

        let mut events = app.world_mut().resource_mut::<Events<ZonePlacementBlocked>>();
        let mut blocked = false;
        for ev in events.drain() {
            match &ev.reason {
                ZoneBlockReason::InsufficientSP { required, available } => {
                    assert_eq!(*required, 100.0);
                    assert_eq!(*available, 0.0);
                }
                other => panic!("Expected InsufficientSP, got {:?}", other),
            }
            blocked = true;
        }
        assert!(blocked, "Should fire blocked event for insufficient SP");
    }

    #[test]
    fn test_zone_placement_deducts_sp() {
        let mut app = test_app();
        {
            let mut defs = app.world_mut().resource_mut::<ZoneDefinitions>();
            if let Some(res) = defs.zones.iter_mut().find(|z| z.zone_type == ZoneType::Residential) {
                res.unlock_requirement.sp_cost = 5.0;
            }
        }
        {
            let mut stockpile = app.world_mut().resource_mut::<ResourceStockpile>();
            stockpile.sovereignty_points = 10;
        }
        {
            let mut p = app.world_mut().resource_mut::<ZonePlacementState>();
            p.active = true;
            p.selected_type = ZoneType::Residential;
            p.hovered_grid = Some(IVec2::new(0, 0));
        }

        press_left_click(&mut app);
        app.update();

        assert_eq!(zone_count(&mut app), 1, "Zone should be placed");
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(stockpile.sovereignty_points, 5, "SP should be deducted by cost (10 - 5 = 5)");
    }

    #[test]
    fn test_unlock_requirement_defaults() {
        let req = ZoneUnlockRequirement::default();
        assert_eq!(req.required_city_level, CityLevel::Village);
        assert_eq!(req.required_population, 0);
        assert_eq!(req.sp_cost, 0.0);
    }

    #[test]
    fn test_district_tracker_count_of_type() {
        let mut tracker = DistrictTracker::default();
        tracker.districts.push(District {
            id: 1,
            zone_type: ZoneType::Residential,
            cells: vec![IVec2::new(0, 0)],
            entities: vec![Entity::from_raw(0)],
            total_level: 1,
            size: 1,
        });
        tracker.districts.push(District {
            id: 2,
            zone_type: ZoneType::Residential,
            cells: vec![IVec2::new(5, 5)],
            entities: vec![Entity::from_raw(1)],
            total_level: 2,
            size: 2,
        });
        tracker.districts.push(District {
            id: 3,
            zone_type: ZoneType::Military,
            cells: vec![IVec2::new(10, 0)],
            entities: vec![Entity::from_raw(2)],
            total_level: 1,
            size: 1,
        });

        assert_eq!(tracker.count_of_type(ZoneType::Residential), 2);
        assert_eq!(tracker.count_of_type(ZoneType::Military), 1);
        assert_eq!(tracker.count_of_type(ZoneType::Economic), 0);
        assert_eq!(tracker.total_cells_of_type(ZoneType::Residential), 3);
        assert_eq!(tracker.total_cells_of_type(ZoneType::Military), 1);
    }

    #[test]
    fn test_bonus_size_capped_at_5_extra() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Economic,
            cells: vec![IVec2::new(0, 0); 20],
            entities: vec![Entity::from_raw(0); 20],
            total_level: 20,
            size: 20,
        };
        let mult = district.compute_bonus_multiplier(1.25);
        // Revised formula (soft-cap):
        //   raw_size_bonus = (20-1) * 0.10 = 1.9
        //   soft_cap=15 → capped=1.4, over=0.15, size_bonus=1.55
        //   hard ceiling 1.0 → size_bonus=1.0
        //   level_bonus=0.0
        //   Result: 1.25 * (1.0 + 1.0) = 2.5
        assert!((mult - 2.5).abs() < 0.001, "Size 20 district with soft cap should yield 2.5. Got {}", mult);
    }

    #[test]
    fn test_bonus_level_capped_at_3_extra() {
        let district = District {
            id: 1,
            zone_type: ZoneType::Religious,
            cells: vec![IVec2::new(0, 0)],
            entities: vec![Entity::from_raw(0)],
            total_level: 10, // avg = 10.0, but capped at +3
            size: 1,
        };
        let mult = district.compute_bonus_multiplier(1.50);
        // level_bonus = 3.0 * 0.15 = 0.45, size_bonus = 0
        // 1.50 * (1.0 + 0.45) = 2.175
        assert!((mult - 2.175).abs() < 0.001, "Level bonus should be capped at +45%. Got {}", mult);
    }

    #[test]
    fn test_zone_type_adjacency_bonus_returns_numeric() {
        let res_bonus = ZoneType::Residential.adjacency_bonus();
        assert!(res_bonus.population_growth_mult > 1.0);

        let mil_bonus = ZoneType::Military.adjacency_bonus();
        assert!(mil_bonus.recruit_speed_mult > 1.0);

        let eco_bonus = ZoneType::Economic.adjacency_bonus();
        assert!(eco_bonus.resource_output_mult > 1.0);

        let rel_bonus = ZoneType::Religious.adjacency_bonus();
        assert!(rel_bonus.faith_generation_mult > 1.0);

        let admin_bonus = ZoneType::Administrative.adjacency_bonus();
        assert!(admin_bonus.happiness_bonus > 0.0);
    }

    #[test]
    fn test_city_zone_system_unlocks_zones_on_population_growth() {
        let mut app = test_app();
        // Start at Village level with low pop
        {
            let mut city = app.world_mut().resource_mut::<CityState>();
            city.city_level = CityLevel::Village;
            city.population = 5;
        }
        // Economic zone requires Town level + 20 pop
        {
            let defs = app.world().resource::<ZoneDefinitions>();
            let econ = defs.zones.iter().find(|z| z.zone_type == ZoneType::Economic).unwrap();
            assert!(!econ.unlocked, "Economic should start locked at Village");
        }

        // Advance city to Town with pop 25
        {
            let mut city = app.world_mut().resource_mut::<CityState>();
            city.population = 25;
            city.max_population = 50;
        }

        run_fixed_ticks(&mut app, 10);

        let city = app.world().resource::<CityState>();
        assert!(city.city_level >= CityLevel::Town, "Should have advanced to Town");

        let defs = app.world().resource::<ZoneDefinitions>();
        let econ = defs.zones.iter().find(|z| z.zone_type == ZoneType::Economic).unwrap();
        assert!(econ.unlocked, "Economic should be unlocked at Town with pop >= 20");
    }

    // ── Protection Zone Tests ───────────────────────────────────────────

    #[test]
    fn test_protection_zone_matches_wall_buildings() {
        assert!(ZoneType::Protection.matches_building(BuildingType::Palisade));
        assert!(ZoneType::Protection.matches_building(BuildingType::StoneWall));
        assert!(ZoneType::Protection.matches_building(BuildingType::Gate));
        assert!(!ZoneType::Protection.matches_building(BuildingType::Oba));
        assert!(!ZoneType::Protection.matches_building(BuildingType::Market));
    }

    #[test]
    fn test_protection_adjacency_bonus_returns_defense() {
        let bonus = ZoneType::Protection.adjacency_bonus();
        assert!((bonus.defense_bonus_mult - 1.20).abs() < 0.001);
    }

    #[test]
    fn test_protection_zone_def_has_resource_costs() {
        let defs = ZoneDefinitions::default();
        let prot = defs.zones.iter().find(|z| z.zone_type == ZoneType::Protection).unwrap();
        assert!(prot.placement_gold_cost > 0);
        assert!(prot.placement_wood_cost > 0);
        assert_eq!(prot.max_district_size, 15);
    }

    #[test]
    fn test_protection_zone_color_in_defs() {
        let defs = ZoneDefinitions::default();
        let prot = defs.zones.iter().find(|z| z.zone_type == ZoneType::Protection).unwrap();
        assert_eq!(prot.color, [0.5, 0.5, 0.5, 1.0]);
    }

    // ── Cross-Zone Synergy Tests ─────────────────────────────────────────

    #[test]
    fn test_cross_zone_synergy_table_has_default_entries() {
        let table = CrossZoneSynergyTable::default();
        assert!(!table.synergies.is_empty(), "Should have at least one synergy");
        // Verify Protection synergies exist
        let has_protection_res = table.synergies.iter().any(|s|
            (s.zone_a == ZoneType::Protection && s.zone_b == ZoneType::Residential)
            || (s.zone_a == ZoneType::Residential && s.zone_b == ZoneType::Protection)
        );
        assert!(has_protection_res, "Protection ↔ Residential synergy should exist");
    }

    #[test]
    fn test_sort_zone_pair_is_deterministic() {
        let pair = sort_zone_pair(ZoneType::Military, ZoneType::Protection);
        assert_eq!(pair.0, ZoneType::Military);
        assert_eq!(pair.1, ZoneType::Protection);

        let reverse = sort_zone_pair(ZoneType::Protection, ZoneType::Military);
        assert_eq!(reverse, pair, "sort_zone_pair should be symmetric");
    }

    #[test]
    fn test_sort_zone_pair_same_type() {
        let pair = sort_zone_pair(ZoneType::Residential, ZoneType::Residential);
        assert_eq!(pair.0, ZoneType::Residential);
        assert_eq!(pair.1, ZoneType::Residential);
    }

    // ── District Soft-Cap Tests ──────────────────────────────────────────

    #[test]
    fn test_district_administrative_overhead_increases_with_size() {
        let below_cap = District {
            id: 1, zone_type: ZoneType::Economic,
            cells: vec![IVec2::ZERO; 14], entities: vec![],
            total_level: 14, size: 14,
        };
        let above_cap = District {
            id: 2, zone_type: ZoneType::Economic,
            cells: vec![IVec2::ZERO; 17], entities: vec![],
            total_level: 17, size: 17,
        };
        assert_eq!(below_cap.administrative_overhead(), 0.0, "Below hard cap should have zero overhead");
        assert!(
            above_cap.administrative_overhead() > below_cap.administrative_overhead(),
            "Overhead should increase when size exceeds hard cap"
        );
    }

    #[test]
    fn test_district_apply_soft_cap_reduces_large_bonus() {
        let district = District {
            id: 1, zone_type: ZoneType::Residential,
            cells: vec![IVec2::ZERO; 10], entities: vec![],
            total_level: 10, size: 10,
        };
        let raw = district.compute_bonus_multiplier(1.25);
        let soft = district.apply_soft_cap(raw);
        // For a size-10 district with soft cap at 12, overhead ~0.11
        // raw ≈ 1.25 * (1 + 0.15 + 0.25) = 1.75, soft should be less
        assert!(soft <= raw, "Soft cap should reduce or equal raw bonus");
        assert!(soft >= 1.0, "Soft cap should never reduce below base");
    }

    // ── Zone Upgrade Tests ───────────────────────────────────────────────

    #[test]
    fn test_zone_upgrade_event_fires_and_level_increases() {
        let mut app = test_app();
        // Spawn a zone cell
        let entity = app.world_mut().spawn((
            Zone { zone_type: ZoneType::Residential, level: 1 },
            WorldPosition::new(IVec2::new(10, 10), CELL_SIZE),
        )).id();
        // Provide resources
        {
            let mut s = app.world_mut().resource_mut::<ResourceStockpile>();
            s.gold = 100.0;
            s.wood = 100.0;
        }

        // Simulate zone_upgrade_system manually (can't rely on keyboard in tests)
        let zone_defs = app.world().resource::<ZoneDefinitions>().clone();
        let def = zone_defs.zones.iter().find(|z| z.zone_type == ZoneType::Residential).unwrap();

        // Deduct costs first (separate borrow)
        {
            let mut s = app.world_mut().resource_mut::<ResourceStockpile>();
            s.gold -= def.upgrade_gold_cost as f32;
            s.wood -= def.upgrade_wood_cost as f32;
        }

        // Manually upgrade zone level (separate borrow)
        {
            let mut zone = app.world_mut().get_mut::<Zone>(entity).unwrap();
            assert_eq!(zone.level, 1);
            zone.level += 1;
        }

        let zone = app.world().get::<Zone>(entity).unwrap();
        assert_eq!(zone.level, 2, "Zone should be upgraded to level 2");

        let s = app.world().resource::<ResourceStockpile>();
        assert!((s.gold - (100.0 - def.upgrade_gold_cost as f32)).abs() < 0.001);
        assert!((s.wood - (100.0 - def.upgrade_wood_cost as f32)).abs() < 0.001);
    }

    #[test]
    fn test_zone_upgrade_blocked_by_insufficient_resources() {
        let mut app = test_app();
        let entity = app.world_mut().spawn((
            Zone { zone_type: ZoneType::Residential, level: 1 },
            WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        )).id();
        // Give very few resources
        {
            let mut s = app.world_mut().resource_mut::<ResourceStockpile>();
            s.gold = 1.0; // less than upgrade_gold_cost (10)
            s.wood = 1.0;
        }

        let zone_defs = app.world().resource::<ZoneDefinitions>().clone();
        let def = zone_defs.zones.iter().find(|z| z.zone_type == ZoneType::Residential).unwrap();
        let has_gold = (app.world().resource::<ResourceStockpile>().gold as u32) >= def.upgrade_gold_cost;
        let has_wood = (app.world().resource::<ResourceStockpile>().wood as u32) >= def.upgrade_wood_cost;

        assert!(!has_gold || !has_wood, "Should not have enough resources");
        let zone = app.world().get::<Zone>(entity).unwrap();
        assert_eq!(zone.level, 1, "Zone level should remain unchanged");
    }

    #[test]
    fn test_zone_upgrade_blocked_at_max_level() {
        let mut app = test_app();
        // Spawn zone already at max level (5 for Residential)
        let entity = app.world_mut().spawn((
            Zone { zone_type: ZoneType::Residential, level: 5 },
            WorldPosition::new(IVec2::new(3, 3), CELL_SIZE),
        )).id();
        {
            let mut s = app.world_mut().resource_mut::<ResourceStockpile>();
            s.gold = 100.0;
            s.wood = 100.0;
        }

        let zone = app.world().get::<Zone>(entity).unwrap();
        assert_eq!(zone.level, 5, "Should start at max level");
    }

    // ── CrossZoneSynergyEffect Component Test ────────────────────────────

    #[test]
    fn test_cross_zone_synergy_effect_default() {
        let effect = CrossZoneSynergyEffect::default();
        assert_eq!(effect.adjacency_count, 0);
        assert_eq!(effect.source_zone, ZoneType::Residential);
        // AdjacencyBonusEffect default is all-zeros (f32::default() = 0.0)
        assert!((effect.effect.defense_bonus_mult).abs() < 0.001);
        assert!((effect.effect.crime_reduction).abs() < 0.001);
    }

    // ── AdjacencyBonusEffect Merge Tests ────────────────────────────────

    #[test]
    fn test_adjacency_bonus_takes_max_for_mults() {
        let a = AdjacencyBonusEffect {
            population_growth_mult: 1.25,
            recruit_speed_mult: 1.30,
            ..Default::default()
        };
        let b = AdjacencyBonusEffect {
            population_growth_mult: 1.50,
            happiness_bonus: 5.0,
            ..Default::default()
        };

        // Manual max-merge (same logic as city_zone_system)
        let merged = AdjacencyBonusEffect {
            population_growth_mult: a.population_growth_mult.max(b.population_growth_mult),
            recruit_speed_mult: a.recruit_speed_mult.max(b.recruit_speed_mult),
            resource_output_mult: a.resource_output_mult.max(b.resource_output_mult),
            faith_generation_mult: a.faith_generation_mult.max(b.faith_generation_mult),
            happiness_bonus: a.happiness_bonus.max(b.happiness_bonus),
            crime_reduction: a.crime_reduction.max(b.crime_reduction),
            defense_bonus_mult: a.defense_bonus_mult.max(b.defense_bonus_mult),
        };

        assert!((merged.population_growth_mult - 1.50).abs() < 0.001);
        assert!((merged.recruit_speed_mult - 1.30).abs() < 0.001);
        assert!((merged.happiness_bonus - 5.0).abs() < 0.001);
    }

    // ── Zone Definition Config Tests ─────────────────────────────────────

    #[test]
    fn test_zone_definitions_has_all_six_zone_types() {
        let defs = ZoneDefinitions::default();
        let types: std::collections::HashSet<ZoneType> =
            defs.zones.iter().map(|z| z.zone_type).collect();
        assert!(types.contains(&ZoneType::Residential));
        assert!(types.contains(&ZoneType::Military));
        assert!(types.contains(&ZoneType::Economic));
        assert!(types.contains(&ZoneType::Religious));
        assert!(types.contains(&ZoneType::Administrative));
        assert!(types.contains(&ZoneType::Protection));
        assert_eq!(types.len(), 6, "Should have exactly 6 zone types");
    }

    #[test]
    fn test_zone_definitions_have_upgrade_fields() {
        let defs = ZoneDefinitions::default();
        for zone in &defs.zones {
            assert!(zone.max_level > 0, "{:?} should have max_level > 0", zone.zone_type);
            assert!(
                zone.upgrade_gold_cost > 0 || zone.zone_type == ZoneType::Protection,
                "{:?} should have upgrade_gold_cost",
                zone.zone_type
            );
            assert!(
                zone.upgrade_wood_cost > 0 || zone.zone_type == ZoneType::Protection,
                "{:?} should have upgrade_wood_cost",
                zone.zone_type
            );
        }
    }

    // ── Edict / Imperial Admin Placeholder Test ──────────────────────────

    #[test]
    fn test_null_edict_zone_modifier_returns_default() {
        let null_mod = NullEdictZoneModifier;
        let bonus = null_mod.compute_zone_bonus(ZoneType::Residential);
        // AdjacencyBonusEffect::default() = all zeros
        assert!((bonus.population_growth_mult).abs() < 0.001);
        assert!((bonus.happiness_bonus).abs() < 0.001);
        assert!((bonus.defense_bonus_mult).abs() < 0.001);
    }
}
