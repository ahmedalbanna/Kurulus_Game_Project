use bevy::prelude::*;
use bevy::math::IVec2;
use bevy::input::ButtonInput;
use kurulus_core::prelude::*;
use std::collections::HashMap;

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
            .add_event::<ZonePlaced>()
            .add_event::<ZoneUpgraded>()
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
                    // Phase 6d: Zone Assignment & Adjacency
                    city_zone_system
                        .in_set(SimulationSet::CityZones),
                    // Phase 6e: Building Upgrade Chains
                    upgrade_chain_system
                        .in_set(SimulationSet::ConstructionProgress),
                    // Phase 6f: City Progression (Village→Town→City)
                    city_progression_system
                        .in_set(SimulationSet::CityProgression),
                    // Phase 6g: Random City Events
                    city_event_processing_system
                        .in_set(SimulationSet::CityProgression),
                    city_random_event_system
                        .in_set(SimulationSet::CityProgression),
                    // Phase 6h: Policy Effects (consumption, draft)
                    policy_effects_system
                        .in_set(SimulationSet::CityHappiness),
                    // Sync PolicySettings.tax_rate → TaxPolicy.rate
                    tax_policy_sync_system
                        .in_set(SimulationSet::ResourceProduction),
                ),
            )
            // Keyboard-driven & input systems run in Update for responsive input
            .add_systems(
                Update,
                (
                    overlay_cycle_system,
                    policy_cycle_system,
                    policy_adjust_system,
                    zone_placement_toggle_system,
                    zone_placement_hover_system,
                    zone_placement_execute_system,
                ),
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
// 4. City Zone System
// ═══════════════════════════════════════════════════════════════════════════════

/// Assigns buildings to zones and calculates adjacency bonuses.
/// Buildings within the same zone type area receive production/happiness bonuses.
fn city_zone_system(
    mut city: ResMut<CityState>,
    buildings: Query<(Entity, &Building, &WorldPosition, Option<&Zone>)>,
    mut zone_defs: ResMut<ZoneDefinitions>,
) {
    // Track zone totals
    let mut total_res_levels = 0u32;
    let mut total_mil_levels = 0u32;
    let mut total_eco_power = 0u32;

    for (_entity, building, _pos, zone) in buildings.iter() {
        // ── Zone bonus application ───────────────────────────────────
            if let Some(z) = zone {
            if zone_defs
                .zones
                .iter()
                .any(|zd| zd.zone_type == z.zone_type)
            {
                // Sum building level * bonus_multiplier for resource output
                match z.zone_type {
                    ZoneType::Residential => {
                        total_res_levels += building.level as u32;
                    }
                    ZoneType::Military => {
                        total_mil_levels += building.level as u32;
                        city.total_military_pop = total_mil_levels * 5; // each level = 5 soldiers
                    }
                    ZoneType::Economic => {
                        total_eco_power += building.level as u32;
                        city.total_economic_power = total_eco_power;
                    }
                    ZoneType::Religious => {
                        // Faith bonus already handled in happiness system
                    }
                    ZoneType::Administrative => {
                        // Happiness bonus already handled via zone def
                    }
                }
            }
        } else {
            // Buildings not in a zone are still counted for baseline stats
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

    // Automatically update max_population based on residential zones
    let zone_pop_bonus = total_res_levels * 10; // each residential level = +10 pop cap
    let base_pop = match city.city_level {
        CityLevel::Village => 20,
        CityLevel::Town => 50,
        CityLevel::City => 150,
        CityLevel::Metropolis => 400,
    };
    city.max_population = base_pop + zone_pop_bonus;

    // Unlock Administrative zones at Town level
    if city.city_level >= CityLevel::Town {
        if let Some(admin) = zone_defs.zones.iter_mut().find(|z| z.zone_type == ZoneType::Administrative) {
            admin.unlocked = true;
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
/// * **Left click** — spawns a `Zone` + `WorldPosition` entity at the
///   hovered grid position (only if no zone already occupies that cell).
/// * **Right click** — despawns any zone entity at the hovered position.
///
/// Both fire a `ZonePlaced` event for downstream systems (service
/// coverage, adjacency bonuses, etc.).
fn zone_placement_execute_system(
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    placement: Res<ZonePlacementState>,
    mut commands: Commands,
    mut zone_placed_writer: EventWriter<ZonePlaced>,
    zones: Query<(Entity, &WorldPosition), With<Zone>>,
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
        // Don't place on an occupied cell
        let occupied = zones.iter().any(|(_, wp)| wp.grid.0 == grid_pos);
        if !occupied {
            let entity = commands
                .spawn((
                    Zone {
                        zone_type: placement.selected_type,
                        level: 1,
                    },
                    WorldPosition::new(grid_pos, CELL_SIZE),
                ))
                .id();
            zone_placed_writer.send(ZonePlaced {
                zone_type: placement.selected_type,
                position: GridPos(grid_pos),
                entity: Some(entity),
            });
            info!(
                "🏗️  Placed {:?} zone at ({}, {})",
                placement.selected_type, grid_pos.x, grid_pos.y,
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Helper to build a test app with the city builder plugin,
    /// time updates, and core resources.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<ProductionRates>()
            .init_resource::<TaxPolicy>()
            .init_resource::<FactionRelations>()
            .add_plugins(CityBuilderPlugin);
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
}
