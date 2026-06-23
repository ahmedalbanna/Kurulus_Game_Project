use crate::components::{AdjacencyBonusEffect, BuildingType, ServiceType, ZoneType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─── Cross-Zone Synergy ──────────────────────────────────────────────────────

/// A synergy bonus applied when two different zone types are orthogonally adjacent.
/// Synergies are symmetric (A↔B): the same bonus applies regardless of direction.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct CrossZoneSynergy {
    pub zone_a: ZoneType,
    pub zone_b: ZoneType,
    pub effect: AdjacencyBonusEffect,
}

/// All defined cross-zone synergies, runtime-loaded from config or hardcoded defaults.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Reflect)]
#[reflect(Resource, Serialize, Deserialize)]
pub struct CrossZoneSynergyTable {
    pub synergies: Vec<CrossZoneSynergy>,
}

impl Default for CrossZoneSynergyTable {
    fn default() -> Self {
        Self {
            synergies: vec![
                CrossZoneSynergy {
                    zone_a: ZoneType::Residential,
                    zone_b: ZoneType::Religious,
                    effect: AdjacencyBonusEffect {
                        happiness_bonus: 5.0,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Residential,
                    zone_b: ZoneType::Administrative,
                    effect: AdjacencyBonusEffect {
                        population_growth_mult: 1.10,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Military,
                    zone_b: ZoneType::Economic,
                    effect: AdjacencyBonusEffect {
                        recruit_speed_mult: 1.15,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Military,
                    zone_b: ZoneType::Administrative,
                    effect: AdjacencyBonusEffect {
                        recruit_speed_mult: 1.10,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Economic,
                    zone_b: ZoneType::Administrative,
                    effect: AdjacencyBonusEffect {
                        resource_output_mult: 1.10,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Protection,
                    zone_b: ZoneType::Residential,
                    effect: AdjacencyBonusEffect {
                        crime_reduction: 5.0,
                        ..Default::default()
                    },
                },
                CrossZoneSynergy {
                    zone_a: ZoneType::Protection,
                    zone_b: ZoneType::Military,
                    effect: AdjacencyBonusEffect {
                        defense_bonus_mult: 1.20,
                        ..Default::default()
                    },
                },
            ],
        }
    }
}

// ─── Imperial Administration / Edict Hooks ─────────────────────────────────────

/// Trait for edicts that modify zone bonuses.
/// Implemented by System 18 (Imperial Administration); the default placeholder
/// returns zero effect so the hook is wired today without breaking anything.
pub trait EdictZoneModifier: Send + Sync {
    fn compute_zone_bonus(&self, zone_type: ZoneType) -> AdjacencyBonusEffect;
}

/// Null implementation — returns no bonus. Used as placeholder until System 18.
pub struct NullEdictZoneModifier;

impl EdictZoneModifier for NullEdictZoneModifier {
    fn compute_zone_bonus(&self, _zone_type: ZoneType) -> AdjacencyBonusEffect {
        AdjacencyBonusEffect::default()
    }
}

// ─── City Level ──────────────────────────────────────────────────────────────

/// The developmental level of a settlement, from humble village to imperial city.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Reflect,
)]
pub enum CityLevel {
    Village,
    Town,
    City,
    Metropolis,
}

impl CityLevel {
    /// Minimum population required to reach this level.
    pub fn population_threshold(&self) -> u32 {
        match self {
            CityLevel::Village => 0,
            CityLevel::Town => 20,
            CityLevel::City => 100,
            CityLevel::Metropolis => 500,
        }
    }

    /// Maximum population cap imposed by this settlement level.
    pub fn max_population_cap(&self) -> u32 {
        match self {
            CityLevel::Village => 30,
            CityLevel::Town => 120,
            CityLevel::City => 600,
            CityLevel::Metropolis => 2000,
        }
    }

    /// Title awarded to the ruler at this city level.
    pub fn title(&self) -> &str {
        match self {
            CityLevel::Village => "Bey (Chieftain)",
            CityLevel::Town => "Subaşı (Town Governor)",
            CityLevel::City => "Sancak Beyi (District Governor)",
            CityLevel::Metropolis => "Beylerbeyi (Lord of Lords)",
        }
    }

    /// Human-readable name.
    pub fn display_name(&self) -> &str {
        match self {
            CityLevel::Village => "Village",
            CityLevel::Town => "Town",
            CityLevel::City => "City",
            CityLevel::Metropolis => "Metropolis",
        }
    }
}

impl Default for CityLevel {
    fn default() -> Self {
        CityLevel::Village
    }
}

// ─── City State Resource ────────────────────────────────────────────────────

/// The main city-state resource that tracks all high-level city metrics.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CityState {
    /// Name of the settlement (e.g., "Yenişehir").
    pub name: String,
    /// Current citizen count.
    pub population: u32,
    /// Soft cap from housing (Oba levels + Residential zones).
    pub max_population: u32,
    /// Current settlement tier.
    pub city_level: CityLevel,

    // ─── Quality-of-life metrics (0.0 – 100.0) ──────────────────────────
    /// Overall citizen happiness.
    pub happiness: f32,
    /// Crime rate (high = more negative events).
    pub crime: f32,
    /// Sanitation / disease resistance.
    pub sanitation: f32,
    /// Religious faith / cultural unity.
    pub faith: f32,

    // ─── Tracking ───────────────────────────────────────────────────────
    /// Sum of all residential zone levels (affects pop cap).
    pub total_residential_levels: u32,
    /// Number of active military personnel.
    pub total_military_pop: u32,
    /// Sum of building levels across all economic buildings (affects output).
    pub total_economic_power: u32,
    /// Active random events currently affecting the city.
    pub active_events: Vec<CityEvent>,
    /// Combined zone bonus from active Imperial Administration edicts (System 18).
    /// Updated each tick by `policy_effects_system`. Placeholder until System 18.
    pub edict_zone_bonuses: AdjacencyBonusEffect,
}

impl Default for CityState {
    fn default() -> Self {
        Self {
            name: "Yenişehir".to_string(),
            population: 5,
            max_population: 20,
            city_level: CityLevel::Village,
            happiness: 60.0,
            crime: 10.0,
            sanitation: 50.0,
            faith: 60.0,
            total_residential_levels: 0,
            total_military_pop: 0,
            total_economic_power: 0,
            active_events: Vec::new(),
            edict_zone_bonuses: AdjacencyBonusEffect::default(),
        }
    }
}

impl CityState {
    /// Returns a `CityState` initialized for a new game.
    pub fn new(name: &str, start_pop: u32, start_max_pop: u32) -> Self {
        Self {
            name: name.to_string(),
            population: start_pop,
            max_population: start_max_pop,
            ..Default::default()
        }
    }

    /// Whether the city can advance to the next level.
    pub fn can_level_up(&self) -> bool {
        let next_level = match self.city_level {
            CityLevel::Village => CityLevel::Town,
            CityLevel::Town => CityLevel::City,
            CityLevel::City => CityLevel::Metropolis,
            CityLevel::Metropolis => return false,
        };
        self.population >= next_level.population_threshold()
    }

    /// Advance to the next city level. Returns `(old_level, new_level)`, or `None` if already max.
    pub fn advance_level(&mut self) -> Option<(CityLevel, CityLevel)> {
        let next = match self.city_level {
            CityLevel::Village => CityLevel::Town,
            CityLevel::Town => CityLevel::City,
            CityLevel::City => CityLevel::Metropolis,
            CityLevel::Metropolis => return None,
        };
        let old = self.city_level;
        self.city_level = next;
        // Level up grants: happiness boost and population capacity increase
        self.happiness = (self.happiness + 15.0).min(100.0);
        self.max_population = self.max_population.max(next.max_population_cap());
        info!(
            "🏙️  {} advances from {} to {}!",
            self.name,
            old.display_name(),
            next.display_name()
        );
        Some((old, next))
    }
}

// ─── Citizen Needs Resource ─────────────────────────────────────────────────

/// Tracks how well citizen needs are being satisfied.
/// Each value ranges from 0.0 (critical shortage) to 1.0 (fully satisfied).
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CitizenNeeds {
    /// Food supply relative to consumption.
    pub food_satisfaction: f32,
    /// Available housing (population vs max_population ratio inverted).
    pub housing_satisfaction: f32,
    /// Access to mosques and religious services.
    pub faith_satisfaction: f32,
    /// Safety = low crime + watchtower/garrison presence.
    pub safety_satisfaction: f32,
}

impl Default for CitizenNeeds {
    fn default() -> Self {
        Self {
            food_satisfaction: 0.5,
            housing_satisfaction: 0.5,
            faith_satisfaction: 0.3,
            safety_satisfaction: 0.5,
        }
    }
}

impl CitizenNeeds {
    /// Overall average satisfaction across all need categories.
    pub fn average(&self) -> f32 {
        (self.food_satisfaction
            + self.housing_satisfaction
            + self.faith_satisfaction
            + self.safety_satisfaction)
            / 4.0
    }
}

// ─── Zone Definitions Resource ──────────────────────────────────────────────

/// Requirements that must be met before a zone type can be placed.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct ZoneUnlockRequirement {
    /// Minimum city level required.
    pub required_city_level: CityLevel,
    /// Minimum population required.
    pub required_population: u32,
    /// Sovereignty points (SP) cost to place one zone cell of this type.
    pub sp_cost: f32,
}

impl Default for ZoneUnlockRequirement {
    fn default() -> Self {
        Self {
            required_city_level: CityLevel::Village,
            required_population: 0,
            sp_cost: 0.0,
        }
    }
}

/// Configuration for a zone type that the player can designate on the map.
#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct ZoneDefinition {
    pub zone_type: ZoneType,
    /// Whether zone type is currently unlocked (runtime-computed).
    pub unlocked: bool,
    /// Base bonus multiplier applied to buildings within this zone.
    pub bonus_multiplier: f32,
    /// Requirements to unlock this zone type.
    pub unlock_requirement: ZoneUnlockRequirement,
    /// Gold cost per zone cell placed.
    pub placement_gold_cost: u32,
    /// Wood cost per zone cell placed.
    pub placement_wood_cost: u32,
    /// Soft-cap maximum size for districts of this type (beyond this = diminishing returns).
    pub max_district_size: u32,
    /// Gold cost to upgrade this zone from level N to N+1.
    pub upgrade_gold_cost: u32,
    /// Wood cost to upgrade this zone from level N to N+1.
    pub upgrade_wood_cost: u32,
    /// Max level this zone type can reach.
    pub max_level: u8,
    /// RGBA colour for overlay rendering.
    pub color: [f32; 4],
}

/// All zone definitions accessible to the player.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDefinitions {
    pub zones: Vec<ZoneDefinition>,
}

impl Default for ZoneDefinitions {
    fn default() -> Self {
        Self {
            zones: vec![
                ZoneDefinition {
                    zone_type: ZoneType::Residential,
                    unlocked: true,
                    bonus_multiplier: 1.25,
                    unlock_requirement: ZoneUnlockRequirement::default(),
                    placement_gold_cost: 5,
                    placement_wood_cost: 2,
                    max_district_size: 12,
                    upgrade_gold_cost: 10,
                    upgrade_wood_cost: 5,
                    max_level: 5,
                    color: [0.2, 0.8, 0.2, 1.0],
                },
                ZoneDefinition {
                    zone_type: ZoneType::Military,
                    unlocked: true,
                    bonus_multiplier: 1.30,
                    unlock_requirement: ZoneUnlockRequirement::default(),
                    placement_gold_cost: 8,
                    placement_wood_cost: 5,
                    max_district_size: 10,
                    upgrade_gold_cost: 12,
                    upgrade_wood_cost: 6,
                    max_level: 4,
                    color: [0.8, 0.2, 0.2, 1.0],
                },
                ZoneDefinition {
                    zone_type: ZoneType::Economic,
                    unlocked: false, // unlocked at Town level + 20 pop
                    bonus_multiplier: 1.25,
                    unlock_requirement: ZoneUnlockRequirement {
                        required_city_level: CityLevel::Town,
                        required_population: 20,
                        sp_cost: 5.0,
                        ..Default::default()
                    },
                    placement_gold_cost: 10,
                    placement_wood_cost: 5,
                    max_district_size: 14,
                    upgrade_gold_cost: 15,
                    upgrade_wood_cost: 8,
                    max_level: 5,
                    color: [0.8, 0.8, 0.2, 1.0],
                },
                ZoneDefinition {
                    zone_type: ZoneType::Religious,
                    unlocked: false, // unlocked at Town level + 30 pop
                    bonus_multiplier: 1.50,
                    unlock_requirement: ZoneUnlockRequirement {
                        required_city_level: CityLevel::Town,
                        required_population: 30,
                        sp_cost: 10.0,
                        ..Default::default()
                    },
                    placement_gold_cost: 15,
                    placement_wood_cost: 8,
                    max_district_size: 8,
                    upgrade_gold_cost: 20,
                    upgrade_wood_cost: 10,
                    max_level: 4,
                    color: [0.2, 0.4, 0.8, 1.0],
                },
                ZoneDefinition {
                    zone_type: ZoneType::Administrative,
                    unlocked: false,
                    bonus_multiplier: 1.40,
                    unlock_requirement: ZoneUnlockRequirement {
                        required_city_level: CityLevel::Town,
                        required_population: 20,
                        sp_cost: 15.0,
                        ..Default::default()
                    },
                    placement_gold_cost: 20,
                    placement_wood_cost: 10,
                    max_district_size: 10,
                    upgrade_gold_cost: 25,
                    upgrade_wood_cost: 12,
                    max_level: 4,
                    color: [0.6, 0.2, 0.8, 1.0],
                },
                ZoneDefinition {
                    zone_type: ZoneType::Protection,
                    unlocked: true,
                    bonus_multiplier: 1.0,
                    unlock_requirement: ZoneUnlockRequirement::default(),
                    placement_gold_cost: 5,
                    placement_wood_cost: 10,
                    max_district_size: 15,
                    upgrade_gold_cost: 8,
                    upgrade_wood_cost: 15,
                    max_level: 5,
                    color: [0.5, 0.5, 0.5, 1.0],
                },
            ],
        }
    }
}

// ─── District Tracking ───────────────────────────────────────────────────────

/// A single contiguous district of same-type zone cells on the grid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct District {
    /// Unique identifier for this district.
    pub id: u32,
    /// The zone type of all cells in this district.
    pub zone_type: ZoneType,
    /// Grid positions of all cells in this district.
    pub cells: Vec<IVec2>,
    /// Entity IDs of the zone entities in this district.
    pub entities: Vec<Entity>,
    /// Sum of zone levels across all cells (for bonus scaling).
    pub total_level: u32,
    /// Number of cells in this district.
    pub size: u32,
}

impl District {
    /// Average level of zones in this district.
    pub fn avg_level(&self) -> f32 {
        if self.size == 0 {
            1.0
        } else {
            self.total_level as f32 / self.size as f32
        }
    }

    /// Computes the combined adjacency bonus multiplier for this district.
    ///
    /// Formula: `apply_soft_cap(base_multiplier × (1.0 + size_bonus + level_bonus))`
    /// - `size_bonus` = 10% per cell beyond the first (capped at 5 extra → +50%)
    /// - `level_bonus` = 15% per average level above 1 (capped at +45%)
    /// - Soft cap: beyond `max_district_size` (configurable), each additional cell
    ///   contributes only 30% of the normal size bonus.
    pub fn compute_bonus_multiplier(&self, base_multiplier: f32) -> f32 {
        let soft_cap = self.max_district_size().unwrap_or(15) as u32;
        let raw_size_bonus = ((self.size.saturating_sub(1)) as f32) * 0.10;
        let size_bonus = if self.size > soft_cap {
            let capped = (soft_cap.saturating_sub(1)) as f32 * 0.10;
            let over = (self.size - soft_cap) as f32 * 0.10 * 0.30; // 30% per extra cell
            capped + over
        } else {
            raw_size_bonus
        };
        let size_bonus = size_bonus.min(1.0); // hard ceiling: +100% max from size
        let level_bonus = ((self.avg_level() - 1.0).max(0.0)).min(3.0) * 0.15;
        base_multiplier * (1.0 + size_bonus + level_bonus)
    }

    /// Returns the configured maximum district size for this zone type from
    /// `ZoneDefinitions`, or a default of 15 if definitions are not available.
    fn max_district_size(&self) -> Option<u32> {
        // Accessed via resource lookup at runtime; this is a placeholder
        // that returns None so the caller uses the default soft cap of 15.
        None
    }

    /// Soft-cap diminishing returns adjustment.
    /// Beyond `soft_cap` (default 15), each additional cell contributes
    /// only 30% of the normal size bonus. This returns the effective
    /// multiplier after applying the soft cap.
    pub fn apply_soft_cap(&self, raw_multiplier: f32) -> f32 {
        let soft_cap = 15u32;
        if self.size <= soft_cap {
            raw_multiplier
        } else {
            let over = (self.size - soft_cap) as f32;
            let penalty = over * 0.007; // -0.7% per extra cell beyond soft cap
            (raw_multiplier - penalty).max(1.0)
        }
    }

    /// Administrative overhead for oversized districts.
    /// Districts beyond the hard cap (15 cells) generate crime per extra cell.
    pub fn administrative_overhead(&self) -> f32 {
        const HARD_CAP_SIZE: u32 = 15;
        if self.size > HARD_CAP_SIZE {
            (self.size - HARD_CAP_SIZE) as f32 * 0.5
        } else {
            0.0
        }
    }

    /// Computes the numeric adjacency bonus effect for this district.
    /// Combines the zone type's base effect with the district's computed multiplier.
    pub fn compute_bonus_effect(
        &self,
        base_multiplier: f32,
    ) -> crate::components::city::AdjacencyBonusEffect {
        use crate::components::city::ZoneType as ZT;
        let mult = self.compute_bonus_multiplier(base_multiplier);
        let extra = mult - 1.0; // bonus above 1.0
        match self.zone_type {
            ZT::Residential => crate::components::city::AdjacencyBonusEffect {
                population_growth_mult: 1.0 + extra,
                ..Default::default()
            },
            ZT::Military => crate::components::city::AdjacencyBonusEffect {
                recruit_speed_mult: 1.0 + extra,
                ..Default::default()
            },
            ZT::Economic => crate::components::city::AdjacencyBonusEffect {
                resource_output_mult: 1.0 + extra,
                ..Default::default()
            },
            ZT::Religious => crate::components::city::AdjacencyBonusEffect {
                faith_generation_mult: 1.0 + extra,
                ..Default::default()
            },
            ZT::Administrative => crate::components::city::AdjacencyBonusEffect {
                happiness_bonus: extra * 20.0, // flat bonus: up to ~10 points
                ..Default::default()
            },
            ZT::Protection => crate::components::city::AdjacencyBonusEffect {
                crime_reduction: extra * 5.0, // flat crime reduction: up to ~2.5 points
                defense_bonus_mult: 1.0 + extra, // defensive HP/upkeep multiplier
                ..Default::default()
            },
        }
    }
}

/// Tracks all computed districts on the city grid.
/// Updated each tick by `district_detection_system`.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct DistrictTracker {
    /// All active districts, grouped by zone type.
    pub districts: Vec<District>,
    /// Next unique district ID to assign.
    pub next_id: u32,
    /// Aggregated bonus effects from all districts, applied to city metrics each tick.
    pub aggregate_bonus: crate::components::city::AdjacencyBonusEffect,
    /// Computed cross-zone synergy effects from adjacent different-type districts.
    /// Populated by `cross_zone_synergy_system`.
    pub cross_zone_bonus: crate::components::city::AdjacencyBonusEffect,
}

impl Default for DistrictTracker {
    fn default() -> Self {
        Self {
            districts: Vec::new(),
            next_id: 1,
            aggregate_bonus: crate::components::city::AdjacencyBonusEffect::default(),
            cross_zone_bonus: crate::components::city::AdjacencyBonusEffect::default(),
        }
    }
}

impl DistrictTracker {
    /// Returns all districts of a given zone type.
    pub fn districts_of_type(&self, zone_type: ZoneType) -> Vec<&District> {
        self.districts
            .iter()
            .filter(|d| d.zone_type == zone_type)
            .collect()
    }

    /// Total district count for a given zone type.
    pub fn count_of_type(&self, zone_type: ZoneType) -> usize {
        self.districts
            .iter()
            .filter(|d| d.zone_type == zone_type)
            .count()
    }

    /// Total zone cells across all districts of a given type.
    pub fn total_cells_of_type(&self, zone_type: ZoneType) -> u32 {
        self.districts
            .iter()
            .filter(|d| d.zone_type == zone_type)
            .map(|d| d.size)
            .sum()
    }
}

// ─── City Events ────────────────────────────────────────────────────────────

/// A currently active event affecting the city (positive or negative).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityEvent {
    pub event_type: CityEventType,
    /// How many more (simulated) seconds this event lasts.
    pub duration_remaining: f32,
    /// How strong the effect is (0.0 – 1.0).
    pub magnitude: f32,
}

/// Types of events that can occur in a city.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CityEventType {
    // Positive
    Festival,
    TradeBoom,
    ScholarArrival,
    // Negative
    Plague,
    Riot,
    Fire,
    Drought,
    BanditRaids,
}

impl CityEventType {
    /// Description for the event log / UI.
    pub fn description(&self) -> &str {
        match self {
            CityEventType::Festival => "A grand festival lifts spirits!",
            CityEventType::TradeBoom => "Trade caravans bring wealth!",
            CityEventType::ScholarArrival => "A learned scholar arrives!",
            CityEventType::Plague => "Disease spreads through the city!",
            CityEventType::Riot => "Citizens protest in the streets!",
            CityEventType::Fire => "A fire breaks out!",
            CityEventType::Drought => "Crops wither in the drought!",
            CityEventType::BanditRaids => "Bandits raid the outskirts!",
        }
    }

    /// Whether this is a positive event.
    pub fn is_positive(&self) -> bool {
        matches!(
            self,
            CityEventType::Festival | CityEventType::TradeBoom | CityEventType::ScholarArrival
        )
    }
}

// ─── Upgrade Chain Registry ─────────────────────────────────────────────────

/// Registers all building upgrade chains so systems can look up tier names
/// and requirements at runtime.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Default)]
pub struct UpgradeChainRegistry {
    /// Map from BuildingType to its tier names.
    pub chains: Vec<(BuildingType, Vec<String>)>,
}

impl UpgradeChainRegistry {
    /// Pre-populate with all defined upgrade chains.
    pub fn new() -> Self {
        let mut chains = Vec::new();
        let types = [
            BuildingType::Oba,
            BuildingType::Pasture,
            BuildingType::MiningCamp,
            BuildingType::LumberCamp,
            BuildingType::Barracks,
            BuildingType::ArcheryRange,
            BuildingType::Stable,
            BuildingType::SiegeWorkshop,
            BuildingType::Watchtower,
            BuildingType::Market,
            BuildingType::Warehouse,
            BuildingType::Well,
            BuildingType::Mosque,
            BuildingType::CouncilChamber,
            BuildingType::GuildHall,
            BuildingType::Monument,
        ];
        for bt in types {
            if let Some(chain) = crate::components::UpgradeChain::for_building_type(bt) {
                chains.push((bt, chain.tier_names));
            }
        }
        Self { chains }
    }

    /// Get the tier name for a building at a given level.
    pub fn tier_name(&self, bt: BuildingType, level: u8) -> Option<&str> {
        self.chains
            .iter()
            .find(|(b, _)| *b == bt)
            .and_then(|(_, names)| {
                if level == 0 || level as usize > names.len() {
                    None
                } else {
                    Some(names[(level - 1) as usize].as_str())
                }
            })
    }
}

// ─── Service Coverage ───────────────────────────────────────────────────────

/// Per-service-type coverage snapshot, computed each frame.
#[derive(Resource, Clone, Debug, Serialize, Deserialize, Default)]
pub struct ServiceCoverage {
    /// What fraction of buildings/citizens are covered by each service (0.0 – 1.0).
    pub water: f32,
    pub trade: f32,
    pub faith: f32,
    pub prestige: f32,
}

impl ServiceCoverage {
    /// Overall average service coverage.
    pub fn average(&self) -> f32 {
        (self.water + self.trade + self.faith + self.prestige) / 4.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Policy Settings (Tax, Rationing, Conscription)
// ═══════════════════════════════════════════════════════════════════════════════

/// How strictly food is rationed to the population.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RationingLevel {
    /// No rationing — full consumption.  +happiness, highest food drain.
    None,
    /// Standard consumption.  Neutral.
    Normal,
    /// Reduced portions.  −happiness, lower food drain.
    Reduced,
    /// Bare minimum to survive.  −−happiness, lowest food drain.
    Emergency,
}

impl RationingLevel {
    /// Food-consumption multiplier applied each tick.
    pub fn food_consumption_mult(&self) -> f32 {
        match self {
            RationingLevel::None => 1.5,
            RationingLevel::Normal => 1.0,
            RationingLevel::Reduced => 0.6,
            RationingLevel::Emergency => 0.3,
        }
    }

    /// Happiness impact (−0 is good, negative is bad).
    pub fn happiness_modifier(&self) -> f32 {
        match self {
            RationingLevel::None => 5.0,
            RationingLevel::Normal => 0.0,
            RationingLevel::Reduced => -8.0,
            RationingLevel::Emergency => -20.0,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            RationingLevel::None => "None (Full Bellies)",
            RationingLevel::Normal => "Normal",
            RationingLevel::Reduced => "Reduced",
            RationingLevel::Emergency => "Emergency",
        }
    }

    /// Return the next (more restrictive) level, wrapping to None.
    pub fn next(self) -> Self {
        match self {
            RationingLevel::None => RationingLevel::Normal,
            RationingLevel::Normal => RationingLevel::Reduced,
            RationingLevel::Reduced => RationingLevel::Emergency,
            RationingLevel::Emergency => RationingLevel::None,
        }
    }

    /// Return the previous (less restrictive) level, wrapping to Emergency.
    pub fn prev(self) -> Self {
        match self {
            RationingLevel::None => RationingLevel::Emergency,
            RationingLevel::Normal => RationingLevel::None,
            RationingLevel::Reduced => RationingLevel::Normal,
            RationingLevel::Emergency => RationingLevel::Reduced,
        }
    }
}

/// How aggressively citizens are conscripted into military service.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConscriptionPolicy {
    /// Only volunteers.  Happy, but military fills slowly.
    Volunteer,
    /// Light forced draft.  Mild unhappiness.
    LightDraft,
    /// Heavy forced draft.  Significant unhappiness.
    HeavyDraft,
    /// Every able-bodied adult is called up.  Severe unhappiness.
    TotalWar,
}

impl ConscriptionPolicy {
    /// Fraction of the adult population automatically assigned as military.
    pub fn forced_recruit_fraction(&self) -> f32 {
        match self {
            ConscriptionPolicy::Volunteer => 0.0,
            ConscriptionPolicy::LightDraft => 0.15,
            ConscriptionPolicy::HeavyDraft => 0.35,
            ConscriptionPolicy::TotalWar => 0.60,
        }
    }

    /// Happiness modifier for this conscription level.
    pub fn happiness_modifier(&self) -> f32 {
        match self {
            ConscriptionPolicy::Volunteer => 2.0,
            ConscriptionPolicy::LightDraft => -5.0,
            ConscriptionPolicy::HeavyDraft => -15.0,
            ConscriptionPolicy::TotalWar => -30.0,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            ConscriptionPolicy::Volunteer => "Volunteer Only",
            ConscriptionPolicy::LightDraft => "Light Draft",
            ConscriptionPolicy::HeavyDraft => "Heavy Draft",
            ConscriptionPolicy::TotalWar => "Total War",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ConscriptionPolicy::Volunteer => ConscriptionPolicy::LightDraft,
            ConscriptionPolicy::LightDraft => ConscriptionPolicy::HeavyDraft,
            ConscriptionPolicy::HeavyDraft => ConscriptionPolicy::TotalWar,
            ConscriptionPolicy::TotalWar => ConscriptionPolicy::Volunteer,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ConscriptionPolicy::Volunteer => ConscriptionPolicy::TotalWar,
            ConscriptionPolicy::LightDraft => ConscriptionPolicy::Volunteer,
            ConscriptionPolicy::HeavyDraft => ConscriptionPolicy::LightDraft,
            ConscriptionPolicy::TotalWar => ConscriptionPolicy::HeavyDraft,
        }
    }
}

/// All player-adjustable city policies.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PolicySettings {
    /// Tax rate 0.0 – 0.5 (fraction of gold income collected).
    pub tax_rate: f32,
    /// Current food rationing level.
    pub rationing: RationingLevel,
    /// Current conscription policy.
    pub conscription: ConscriptionPolicy,
}

impl Default for PolicySettings {
    fn default() -> Self {
        Self {
            tax_rate: 0.10,
            rationing: RationingLevel::Normal,
            conscription: ConscriptionPolicy::Volunteer,
        }
    }
}

/// Which policy the player is currently adjusting via keyboard.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct PolicyFocus {
    pub active: PolicyMode,
}

impl Default for PolicyFocus {
    fn default() -> Self {
        Self {
            active: PolicyMode::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PolicyMode {
    /// No policy panel open.
    None,
    /// Adjusting tax rate (Up/Down).
    TaxRate,
    /// Adjusting rationing level (Left/Right).
    Rationing,
    /// Adjusting conscription level (Left/Right).
    Conscription,
}

impl PolicyMode {
    pub fn label(&self) -> &str {
        match self {
            PolicyMode::None => "—",
            PolicyMode::TaxRate => "Tax Rate",
            PolicyMode::Rationing => "Rationing",
            PolicyMode::Conscription => "Conscription",
        }
    }

    /// Returns the next focus mode in the cycle.
    pub fn next(self) -> Self {
        match self {
            PolicyMode::None => PolicyMode::TaxRate,
            PolicyMode::TaxRate => PolicyMode::Rationing,
            PolicyMode::Rationing => PolicyMode::Conscription,
            PolicyMode::Conscription => PolicyMode::None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// City View Overlay
// ═══════════════════════════════════════════════════════════════════════════════

/// Visual overlay modes for the city view, toggled by the player.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct OverlaySettings {
    pub mode: OverlayMode,
    /// Which service type to show when mode == ServiceCoverage.
    pub active_service: ServiceType,
}

impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            mode: OverlayMode::None,
            active_service: ServiceType::Water,
        }
    }
}

/// The active city overlay mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OverlayMode {
    /// No overlay — normal rendering.
    None,
    /// Color-code buildings by their zone district type.
    Zones,
    /// Heatmap of service coverage for a specific service type.
    ServiceCoverage,
    /// Color-code buildings by the city's happiness level.
    Happiness,
}

impl OverlayMode {
    pub fn label(&self) -> &str {
        match self {
            OverlayMode::None => "None",
            OverlayMode::Zones => "Zone View",
            OverlayMode::ServiceCoverage => "Service Coverage",
            OverlayMode::Happiness => "Happiness Map",
        }
    }

    /// Returns the next mode in the cycle (for keyboard toggle).
    pub fn next(self) -> Self {
        match self {
            OverlayMode::None => OverlayMode::Zones,
            OverlayMode::Zones => OverlayMode::ServiceCoverage,
            OverlayMode::ServiceCoverage => OverlayMode::Happiness,
            OverlayMode::Happiness => OverlayMode::None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Zone Placement (player interaction state)
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks the player's zone placement interaction state.
///
/// Activated by pressing **B**, deactivated by **Escape** or **B** again.
/// While active, the player can:
///   • Hover over the grid to preview zone placement.
///   • Left-click to place a zone of the selected type.
///   • Right-click to remove a zone at the cursor position.
///   • Press 1–6 to switch zone types (Residential .. Protection).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct ZonePlacementState {
    /// Whether zone placement mode is currently active.
    pub active: bool,
    /// Which zone type is selected for placement.
    pub selected_type: ZoneType,
    /// Grid position currently under the cursor, if any.
    pub hovered_grid: Option<IVec2>,
}

impl Default for ZonePlacementState {
    fn default() -> Self {
        Self {
            active: false,
            selected_type: ZoneType::Residential,
            hovered_grid: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Building Placement (player interaction state)
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks the player's building placement interaction state.
///
/// Activated by pressing **U**, deactivated by **Escape** or **U** again.
/// While active, the player can:
///   • Hover over the grid to preview building placement.
///   • Left-click to send a [`BuildOrder`](crate::events::command::BuildOrder).
///   • Press **Tab** / **Shift+Tab** to cycle through building types.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct BuildingPlacementState {
    /// Whether building placement mode is currently active.
    pub active: bool,
    /// Which building type is selected for placement.
    pub selected_type: crate::components::building::BuildingType,
    /// Grid position currently under the cursor, if any.
    pub hovered_grid: Option<IVec2>,
}

/// Ordered list of building types for keyboard cycling.
pub const BUILDING_TYPE_CYCLE: &[crate::components::building::BuildingType] = &[
    crate::components::building::BuildingType::Oba,
    crate::components::building::BuildingType::Barracks,
    crate::components::building::BuildingType::ArcheryRange,
    crate::components::building::BuildingType::Stable,
    crate::components::building::BuildingType::SiegeWorkshop,
    crate::components::building::BuildingType::Watchtower,
    crate::components::building::BuildingType::Market,
    crate::components::building::BuildingType::Warehouse,
    crate::components::building::BuildingType::Pasture,
    crate::components::building::BuildingType::MiningCamp,
    crate::components::building::BuildingType::LumberCamp,
    crate::components::building::BuildingType::Mosque,
    crate::components::building::BuildingType::Monument,
    crate::components::building::BuildingType::CouncilChamber,
    crate::components::building::BuildingType::GuildHall,
    crate::components::building::BuildingType::Road,
    crate::components::building::BuildingType::Well,
    crate::components::building::BuildingType::Palisade,
    crate::components::building::BuildingType::StoneWall,
    crate::components::building::BuildingType::Gate,
];

impl Default for BuildingPlacementState {
    fn default() -> Self {
        Self {
            active: false,
            selected_type: crate::components::building::BuildingType::Oba,
            hovered_grid: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// City Event Log (for HUD display)
// ═══════════════════════════════════════════════════════════════════════════════

/// A single entry in the city event log, displayed as a HUD line item.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityEventLogEntry {
    /// The type of event that occurred.
    pub event_type: CityEventType,
    /// Human-readable message (e.g., "A grand festival lifts spirits!").
    pub message: String,
    /// Seconds remaining before this entry fades from the log.
    pub time_remaining: f32,
}

impl CityEventLogEntry {
    pub fn new(event_type: CityEventType) -> Self {
        Self {
            message: event_type.description().to_string(),
            time_remaining: entry_lifetime_default(),
            event_type,
        }
    }
}

/// Default lifetime (seconds) for a log entry before it expires.
pub const fn entry_lifetime_default() -> f32 {
    12.0
}

/// Maximum number of log entries visible at once.
pub const fn max_visible_entries() -> usize {
    8
}

/// Tracks recent city events for the HUD event log display.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct CityEventLog {
    /// Active entries in the log, newest first.
    pub entries: Vec<CityEventLogEntry>,
}

impl Default for CityEventLog {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl CityEventLog {
    /// Add a new entry at the front of the log.
    pub fn push(&mut self, entry: CityEventLogEntry) {
        self.entries.insert(0, entry);
        // Cap at max visible + some buffer for fading.
        self.entries.truncate(max_visible_entries() + 4);
    }

    /// Tick down all entry lifetimes and remove expired ones.
    /// Returns the number of entries removed.
    pub fn tick(&mut self, delta: f32) -> usize {
        let before = self.entries.len();
        self.entries.retain_mut(|e| {
            e.time_remaining -= delta;
            e.time_remaining > 0.0
        });
        before - self.entries.len()
    }
}

/// Marker component placed on the root UI node of the city event log HUD.
/// Allows systems to find and update the UI tree efficiently.
#[derive(Component, Clone, Debug, Default)]
pub struct CityEventLogRoot;

/// Marker component placed on the root UI node of the overlay legend HUD.
/// Allows systems to find and update the legend UI tree efficiently.
#[derive(Component, Clone, Debug, Default)]
pub struct ZoneLegendRoot;

// ═══════════════════════════════════════════════════════════════════════════════
// World Map UI Marker Components
// ═══════════════════════════════════════════════════════════════════════════════

/// Marker component on the world map region- or POI-tooltip root node.
#[derive(Component, Clone, Debug, Default)]
pub struct WorldMapTooltipRoot;

/// Marker component on the world map selection info panel root node.
#[derive(Component, Clone, Debug, Default)]
pub struct WorldMapSelectionRoot;

/// Marker component on the world map legend overlay root node.
#[derive(Component, Clone, Debug, Default)]
pub struct WorldMapLegendRoot;

// ═══════════════════════════════════════════════════════════════════════════════
// Overlay colours
// ═══════════════════════════════════════════════════════════════════════════════

/// Colors used for each overlay mode.
pub mod overlay_colors {
    use bevy::prelude::Color;

    /// Zone district colours
    pub const ZONE_RESIDENTIAL: Color = Color::srgb(0.2, 0.8, 0.2);
    pub const ZONE_MILITARY: Color = Color::srgb(0.8, 0.2, 0.2);
    pub const ZONE_ECONOMIC: Color = Color::srgb(0.8, 0.8, 0.2);
    pub const ZONE_RELIGIOUS: Color = Color::srgb(0.2, 0.4, 0.8);
    pub const ZONE_ADMIN: Color = Color::srgb(0.6, 0.2, 0.8);
    pub const ZONE_PROTECTION: Color = Color::srgb(0.5, 0.5, 0.5);
    pub const ZONE_UNASSIGNED: Color = Color::srgb(0.4, 0.4, 0.4);

    /// Service-coverage heatmap colours
    pub const COVERAGE_FULL: Color = Color::srgb(0.2, 0.8, 0.2);
    pub const COVERAGE_PARTIAL: Color = Color::srgb(0.8, 0.8, 0.2);
    pub const COVERAGE_NONE: Color = Color::srgb(0.6, 0.2, 0.2);

    /// Happiness-map colours (green → yellow → orange → red)
    pub const HAPPY: Color = Color::srgb(0.2, 0.8, 0.2);
    pub const NEUTRAL: Color = Color::srgb(0.8, 0.8, 0.2);
    pub const UNHAPPY: Color = Color::srgb(0.8, 0.4, 0.2);
    pub const MISERABLE: Color = Color::srgb(0.6, 0.2, 0.2);
}
