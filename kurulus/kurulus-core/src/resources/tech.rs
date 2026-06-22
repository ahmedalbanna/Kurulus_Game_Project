use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::components::{UnitArchetype, BuildingType, AbilityId};

/// The primary tech tree resource — holds all 45 nodes and the active ResearchProject.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct TechTree {
    pub techs: Vec<TechNode>,
    pub current_research: Option<ResearchProject>,
    /// Tracks which node IDs have already had their effects applied.
    pub applied_effects: Vec<String>,
}

impl Default for TechTree {
    fn default() -> Self {
        Self {
            techs: create_default_tech_tree(),
            current_research: None,
            applied_effects: Vec::new(),
        }
    }
}

// ── Tech Node ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechNode {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Sovereignty-point cost to begin research.
    pub cost: u32,
    /// Base research time in seconds (real time, modified by hero assignment).
    pub research_time: f32,
    pub prerequisites: Vec<String>,
    pub unlocked: bool,
    pub category: TechCategory,
    pub effects: Vec<TechEffect>,
    pub tier: u32,
}

// ── Research Project ────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct ResearchProject {
    pub tech_id: String,
    pub progress: f32,
    pub max_progress: f32,
    pub assigned_hero: Option<Entity>,
}

// ── Categories ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TechCategory {
    Military,
    Economy,
    Culture,
    Diplomacy,
    Building,
    Naval,
}

impl TechCategory {
    pub fn label(&self) -> &'static str {
        match self {
            TechCategory::Military => "Military",
            TechCategory::Economy => "Economy",
            TechCategory::Culture => "Culture",
            TechCategory::Diplomacy => "Diplomacy",
            TechCategory::Building => "Building",
            TechCategory::Naval => "Naval",
        }
    }
}

// ── Tech Effects ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TechEffect {
    UnitStatBonus(UnitArchetype, String, f32),
    BuildingCostReduction(BuildingType, f32),
    ResourceRateBonus(String, f32),
    UnlockAbility(AbilityId),
    UnlockBuilding(BuildingType),
    UnlockUnit(UnitArchetype),
    PassiveBonus(String, f32),
}

// ── Accumulated Modifiers (applied by tech_effect_application_system) ───────

/// Global resource that accumulates all active bonuses from unlocked techs.
/// Read by gameplay systems to apply stat bonuses, production rates, etc.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct TechModifiers {
    /// (UnitArchetype, stat_name, multiplier) — applied to Stats & Health
    pub unit_stat_bonuses: Vec<(UnitArchetype, String, f32)>,
    /// (resource_name, multiplier) — applied to ProductionRates
    pub resource_rate_bonuses: Vec<(String, f32)>,
    /// (BuildingType, reduction_multiplier) — cost reduction for construction
    pub building_cost_reductions: Vec<(BuildingType, f32)>,
    /// Buildings that can now be constructed
    pub unlocked_buildings: Vec<BuildingType>,
    /// Units that can now be recruited
    pub unlocked_units: Vec<UnitArchetype>,
    /// Abilities that are now available
    pub unlocked_abilities: Vec<AbilityId>,
    /// (bonus_name, value) — generic passive bonuses
    pub passive_bonuses: std::collections::HashMap<String, f32>,
}

// ── Validation Result ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ResearchStartResult {
    Success,
    AlreadyResearching,
    AlreadyUnlocked,
    MissingPrerequisites(Vec<String>),
    InsufficientSP(u32, u32), // (cost, available)
    TechNotFound,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn make_node(
    id: &str,
    name: &str,
    description: &str,
    cost: u32,
    research_time: f32,
    prerequisites: &[&str],
    category: TechCategory,
    tier: u32,
    effects: Vec<TechEffect>,
) -> TechNode {
    TechNode {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        cost,
        research_time,
        prerequisites: prerequisites.iter().map(|s| s.to_string()).collect(),
        unlocked: false,
        category,
        effects,
        tier,
    }
}

// ── The 45-Node, 15-Tier Tech Tree ──────────────────────────────────────────

fn create_default_tech_tree() -> Vec<TechNode> {
    vec![
        // ═══════════════════════════════════════════════════════════════════
        //  TIER 1 — 6 nodes (one per category)
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t1", "Bronze Weapons",
            "Smith bronze-tipped arrows and blades. +10% LightInfantry damage.",
            10, 30.0, &[], TechCategory::Military, 1,
            vec![TechEffect::UnitStatBonus(UnitArchetype::LightInfantry, "attack_damage".into(), 0.10)],
        ),
        make_node(
            "eco_t1", "Basic Farming",
            "Improve crop yields with simple techniques. +25% food production.",
            10, 30.0, &[], TechCategory::Economy, 1,
            vec![TechEffect::ResourceRateBonus("food".into(), 0.25)],
        ),
        make_node(
            "cul_t1", "Oral Tradition",
            "Preserve tales of our ancestors. +10% XP gain rate.",
            10, 30.0, &[], TechCategory::Culture, 1,
            vec![TechEffect::PassiveBonus("xp_rate".into(), 0.10)],
        ),
        make_node(
            "dip_t1", "Envoys",
            "Train messengers to carry words between tribes. +30% diplomatic range.",
            10, 30.0, &[], TechCategory::Diplomacy, 1,
            vec![TechEffect::PassiveBonus("diplomatic_range".into(), 0.30)],
        ),
        make_node(
            "bld_t1", "Wooden Palisades",
            "Erect sharpened logs around our settlement. Unlocks Palisade walls.",
            15, 30.0, &[], TechCategory::Building, 1,
            vec![
                TechEffect::UnlockBuilding(BuildingType::Palisade),
                TechEffect::BuildingCostReduction(BuildingType::Palisade, 0.20),
            ],
        ),
        make_node(
            "nav_t1", "Fishing Boats",
            "Build simple rafts for river fishing. +15% food production.",
            15, 30.0, &[], TechCategory::Naval, 1,
            vec![TechEffect::ResourceRateBonus("food".into(), 0.15)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 2 — 6 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t2", "Leather Armor",
            "Harden leather hides for battle. +15% LightInfantry armor.",
            25, 45.0, &["mil_t1"], TechCategory::Military, 2,
            vec![TechEffect::UnitStatBonus(UnitArchetype::LightInfantry, "armor".into(), 0.15)],
        ),
        make_node(
            "eco_t2", "Hand Tools",
            "Craft better axes and picks. +20% wood production.",
            25, 45.0, &["eco_t1"], TechCategory::Economy, 2,
            vec![TechEffect::ResourceRateBonus("wood".into(), 0.20)],
        ),
        make_node(
            "cul_t2", "Stone Masonry",
            "Shape rough stone into walls and monuments. -20% StoneWall & Monument cost.",
            25, 45.0, &["cul_t1"], TechCategory::Culture, 2,
            vec![
                TechEffect::BuildingCostReduction(BuildingType::StoneWall, 0.20),
                TechEffect::BuildingCostReduction(BuildingType::Monument, 0.20),
            ],
        ),
        make_node(
            "dip_t2", "Tribal Alliances",
            "Forge bonds with neighboring clans. +15% alliance stability.",
            25, 45.0, &["dip_t1"], TechCategory::Diplomacy, 2,
            vec![TechEffect::PassiveBonus("alliance_stability".into(), 0.15)],
        ),
        make_node(
            "bld_t2", "Stone Walls",
            "Quarry granite for proper defenses. Unlocks StoneWall construction.",
            30, 45.0, &["bld_t1"], TechCategory::Building, 2,
            vec![
                TechEffect::UnlockBuilding(BuildingType::StoneWall),
                TechEffect::BuildingCostReduction(BuildingType::StoneWall, 0.15),
            ],
        ),
        make_node(
            "nav_t2", "River Transports",
            "Build sturdy river barges. Unlocks Scout unit for river exploration.",
            30, 45.0, &["nav_t1"], TechCategory::Naval, 2,
            vec![TechEffect::UnlockUnit(UnitArchetype::Scout)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 3 — 8 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t3a", "Iron Weapons",
            "Forge iron swords and spearheads. +15% HeavyInfantry damage.",
            50, 60.0, &["mil_t2"], TechCategory::Military, 3,
            vec![TechEffect::UnitStatBonus(UnitArchetype::HeavyInfantry, "attack_damage".into(), 0.15)],
        ),
        make_node(
            "mil_t3b", "Shield Wall",
            "Train soldiers to lock shields. +20% HeavyInfantry armor.",
            50, 60.0, &["mil_t2"], TechCategory::Military, 3,
            vec![TechEffect::UnitStatBonus(UnitArchetype::HeavyInfantry, "armor".into(), 0.20)],
        ),
        make_node(
            "eco_t3a", "Crop Rotation",
            "Rotate fields to enrich soil. +50% food production.",
            50, 60.0, &["eco_t2"], TechCategory::Economy, 3,
            vec![TechEffect::ResourceRateBonus("food".into(), 0.50)],
        ),
        make_node(
            "eco_t3b", "Deep Mining",
            "Sink shafts for precious ores. +25% gold and stone production.",
            50, 60.0, &["eco_t2"], TechCategory::Economy, 3,
            vec![
                TechEffect::ResourceRateBonus("gold".into(), 0.25),
                TechEffect::ResourceRateBonus("stone".into(), 0.25),
            ],
        ),
        make_node(
            "cul_t3", "Epic Poetry",
            "Compose songs of our heroes. +20% morale bonus.",
            50, 60.0, &["cul_t2"], TechCategory::Culture, 3,
            vec![TechEffect::PassiveBonus("morale_bonus".into(), 0.20)],
        ),
        make_node(
            "dip_t3a", "Marriage Ties",
            "Seal alliances with wedding bonds. +25% alliance stability.",
            50, 60.0, &["dip_t2"], TechCategory::Diplomacy, 3,
            vec![TechEffect::PassiveBonus("alliance_stability".into(), 0.25)],
        ),
        make_node(
            "dip_t3b", "Trade Agreements",
            "Establish regular trade caravans. +20% gold production.",
            50, 60.0, &["dip_t2"], TechCategory::Diplomacy, 3,
            vec![TechEffect::ResourceRateBonus("gold".into(), 0.20)],
        ),
        make_node(
            "bld_t3", "Watchtowers",
            "Raise vantage points along the border. Unlocks Watchtower; +20% vision range.",
            60, 60.0, &["bld_t2"], TechCategory::Building, 3,
            vec![
                TechEffect::UnlockBuilding(BuildingType::Watchtower),
                TechEffect::PassiveBonus("vision_range".into(), 0.20),
            ],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 4 — 8 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t4a", "Steel Weapons",
            "Master the art of steel. +20% Archer damage.",
            80, 90.0, &["mil_t3a"], TechCategory::Military, 4,
            vec![TechEffect::UnitStatBonus(UnitArchetype::Archer, "attack_damage".into(), 0.20)],
        ),
        make_node(
            "mil_t4b", "Cavalry Tactics",
            "Teach horsemen to charge and withdraw. +20% Light & Heavy Cavalry damage.",
            80, 90.0, &["mil_t3b"], TechCategory::Military, 4,
            vec![
                TechEffect::UnitStatBonus(UnitArchetype::LightCavalry, "attack_damage".into(), 0.20),
                TechEffect::UnitStatBonus(UnitArchetype::HeavyCavalry, "attack_damage".into(), 0.20),
            ],
        ),
        make_node(
            "eco_t4a", "Irrigation",
            "Channel water to dry fields. +75% food production.",
            80, 90.0, &["eco_t3a"], TechCategory::Economy, 4,
            vec![TechEffect::ResourceRateBonus("food".into(), 0.75)],
        ),
        make_node(
            "eco_t4b", "Smelting",
            "Refine ore with advanced furnaces. +50% gold production.",
            80, 90.0, &["eco_t3b"], TechCategory::Economy, 4,
            vec![TechEffect::ResourceRateBonus("gold".into(), 0.50)],
        ),
        make_node(
            "cul_t4a", "Mosque Building",
            "Construct houses of worship. Unlocks Mosque construction.",
            80, 90.0, &["cul_t3"], TechCategory::Culture, 4,
            vec![TechEffect::UnlockBuilding(BuildingType::Mosque)],
        ),
        make_node(
            "cul_t4b", "Calligraphy",
            "Refine the art of writing. +15% research speed.",
            80, 90.0, &["cul_t3"], TechCategory::Culture, 4,
            vec![TechEffect::PassiveBonus("research_speed".into(), 0.15)],
        ),
        make_node(
            "dip_t4a", "Tributary System",
            "Demand tribute from weaker clans. +50% tribute income.",
            80, 90.0, &["dip_t3a", "dip_t3b"], TechCategory::Diplomacy, 4,
            vec![TechEffect::PassiveBonus("tribute_income".into(), 0.50)],
        ),
        make_node(
            "bld_t4", "Barracks Upgrades",
            "Expand and improve training grounds. -25% Barracks cost.",
            100, 90.0, &["bld_t3"], TechCategory::Building, 4,
            vec![TechEffect::BuildingCostReduction(BuildingType::Barracks, 0.25)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 5 — 8 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t5a", "Siege Engineering",
            "Design rams and catapults. Unlocks SiegeCrew unit; +25% siege damage.",
            120, 120.0, &["mil_t4a", "mil_t4b"], TechCategory::Military, 5,
            vec![
                TechEffect::UnlockUnit(UnitArchetype::SiegeCrew),
                TechEffect::UnitStatBonus(UnitArchetype::SiegeCrew, "attack_damage".into(), 0.25),
            ],
        ),
        make_node(
            "mil_t5b", "Veteran Training",
            "Seasoned warriors teach new recruits. +20% all units HP.",
            120, 120.0, &["mil_t4a", "mil_t4b"], TechCategory::Military, 5,
            vec![TechEffect::PassiveBonus("veteran_bonus".into(), 0.20)],
        ),
        make_node(
            "eco_t5a", "Trade Routes",
            "Open permanent caravan roads. +50% gold production; +30% trade income.",
            120, 120.0, &["eco_t4a", "eco_t4b"], TechCategory::Economy, 5,
            vec![
                TechEffect::ResourceRateBonus("gold".into(), 0.50),
                TechEffect::PassiveBonus("trade_income".into(), 0.30),
            ],
        ),
        make_node(
            "eco_t5b", "Marketplaces",
            "Build central squares for commerce. -30% Market cost; +30% gold production.",
            120, 120.0, &["eco_t4a", "eco_t4b"], TechCategory::Economy, 5,
            vec![
                TechEffect::BuildingCostReduction(BuildingType::Market, 0.30),
                TechEffect::ResourceRateBonus("gold".into(), 0.30),
            ],
        ),
        make_node(
            "cul_t5a", "Madrassas",
            "Establish centers of learning. +30% research speed.",
            120, 120.0, &["cul_t4a", "cul_t4b"], TechCategory::Culture, 5,
            vec![TechEffect::PassiveBonus("research_speed".into(), 0.30)],
        ),
        make_node(
            "cul_t5b", "Architecture",
            "Design grand structures. -30% CouncilChamber & GuildHall cost.",
            120, 120.0, &["cul_t4a", "cul_t4b"], TechCategory::Culture, 5,
            vec![
                TechEffect::BuildingCostReduction(BuildingType::CouncilChamber, 0.30),
                TechEffect::BuildingCostReduction(BuildingType::GuildHall, 0.30),
            ],
        ),
        make_node(
            "dip_t5", "Border Treaties",
            "Define clear boundaries with neighbors. +50% diplomatic range.",
            120, 120.0, &["dip_t4a"], TechCategory::Diplomacy, 5,
            vec![TechEffect::PassiveBonus("diplomatic_range".into(), 0.50)],
        ),
        make_node(
            "bld_t5", "Granaries",
            "Store surplus grain for hard times. +30% max population; +30% food production.",
            100, 90.0, &["bld_t4"], TechCategory::Building, 5,
            vec![
                TechEffect::ResourceRateBonus("food".into(), 0.30),
                TechEffect::PassiveBonus("max_population".into(), 0.30),
            ],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 6 — 4 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "mil_t6", "Elite Guard",
            "Form the Sultan's personal bodyguard. +25% all units damage.",
            150, 150.0, &["mil_t5a", "mil_t5b"], TechCategory::Military, 6,
            vec![TechEffect::PassiveBonus("elite_bonus".into(), 0.25)],
        ),
        make_node(
            "eco_t6a", "Merchant Guilds",
            "Organize merchants into powerful guilds. +100% gold production; +50% guild income.",
            150, 150.0, &["eco_t5a", "eco_t5b"], TechCategory::Economy, 6,
            vec![
                TechEffect::ResourceRateBonus("gold".into(), 1.0),
                TechEffect::PassiveBonus("guild_income".into(), 0.50),
            ],
        ),
        make_node(
            "cul_t6", "Legal Code",
            "Write down our laws and customs. +25% tax efficiency.",
            150, 150.0, &["cul_t5a", "cul_t5b"], TechCategory::Culture, 6,
            vec![TechEffect::PassiveBonus("tax_efficiency".into(), 0.25)],
        ),
        make_node(
            "dip_t6", "Vassal States",
            "Accept sworn fealty from lesser lords. +100% tribute income.",
            150, 150.0, &["dip_t5"], TechCategory::Diplomacy, 6,
            vec![TechEffect::PassiveBonus("tribute_income".into(), 1.0)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 7 — 3 nodes
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "eco_t7", "Royal Mint",
            "Strike silver coins bearing the Kayi stamp. +100% passive gold income.",
            180, 180.0, &["eco_t6a"], TechCategory::Economy, 7,
            vec![TechEffect::PassiveBonus("mint_income".into(), 1.0)],
        ),
        make_node(
            "cul_t7", "Cultural Renaissance",
            "A flowering of art and learning. +30% XP rate; +25% research speed.",
            180, 180.0, &["cul_t6"], TechCategory::Culture, 7,
            vec![
                TechEffect::PassiveBonus("xp_rate".into(), 0.30),
                TechEffect::PassiveBonus("research_speed".into(), 0.25),
            ],
        ),
        make_node(
            "dip_t7", "Diplomatic Corps",
            "Station ambassadors at foreign courts. +70% diplomatic range.",
            180, 180.0, &["dip_t6"], TechCategory::Diplomacy, 7,
            vec![TechEffect::PassiveBonus("diplomatic_range".into(), 0.70)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 8 — 1 node
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "bld_t8", "Fortifications",
            "Reinforce all defenses. -30% StoneWall cost; +25% building HP.",
            200, 120.0, &["bld_t5"], TechCategory::Building, 8,
            vec![
                TechEffect::BuildingCostReduction(BuildingType::StoneWall, 0.30),
                TechEffect::PassiveBonus("building_hp".into(), 0.25),
            ],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 9 — 1 node
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "dip_t9", "Sovereignty Claim",
            "Proclaim the right to rule these lands. +100% SP from all sources.",
            250, 240.0, &["dip_t7"], TechCategory::Diplomacy, 9,
            vec![TechEffect::PassiveBonus("sovereignty_rate".into(), 1.0)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 10 — 1 node
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "bld_t10", "Citadel",
            "Fortify the heart of the realm. +50% building HP; +30% defense bonus.",
            300, 180.0, &["bld_t8"], TechCategory::Building, 10,
            vec![
                TechEffect::PassiveBonus("building_hp".into(), 0.50),
                TechEffect::PassiveBonus("defense_bonus".into(), 0.30),
            ],
        ),
        make_node(
            "dip_t10", "Imperial Bureaucracy",
            "Organize governance with scribes and officials. +50% tax efficiency.",
            300, 240.0, &["dip_t9"], TechCategory::Diplomacy, 10,
            vec![TechEffect::PassiveBonus("tax_efficiency".into(), 0.50)],
        ),

        // ═══════════════════════════════════════════════════════════════════
        //  TIER 11+ — naval capstone
        // ═══════════════════════════════════════════════════════════════════
        make_node(
            "nav_t11", "War Galleys",
            "Commission swift warships for river patrol. +50% naval power.",
            200, 120.0, &["nav_t2"], TechCategory::Naval, 11,
            vec![TechEffect::PassiveBonus("naval_power".into(), 0.50)],
        ),
    ]
}

// ── Validation Logic ────────────────────────────────────────────────────────

impl TechTree {
    /// Validate whether a tech can be researched.
    /// Checks: existence, already unlocked, already researching, SP cost, prerequisites.
    pub fn can_research(&self, tech_id: &str, available_sp: u32) -> ResearchStartResult {
        let node = match self.techs.iter().find(|t| t.id == tech_id) {
            Some(n) => n,
            None => return ResearchStartResult::TechNotFound,
        };

        if node.unlocked {
            return ResearchStartResult::AlreadyUnlocked;
        }

        if self.current_research.is_some() {
            return ResearchStartResult::AlreadyResearching;
        }

        if node.cost > available_sp {
            return ResearchStartResult::InsufficientSP(node.cost, available_sp);
        }

        let missing: Vec<String> = node
            .prerequisites
            .iter()
            .filter(|prereq| {
                !self.techs.iter().any(|t| t.id == **prereq && t.unlocked)
            })
            .cloned()
            .collect();

        if !missing.is_empty() {
            return ResearchStartResult::MissingPrerequisites(missing);
        }

        ResearchStartResult::Success
    }

    /// Count how many techs are unlocked in a given category.
    pub fn unlocked_in_category(&self, category: TechCategory) -> usize {
        self.techs
            .iter()
            .filter(|t| t.category == category && t.unlocked)
            .count()
    }

    /// Total SP spent across all unlocked techs.
    pub fn total_sp_spent(&self) -> u32 {
        self.techs
            .iter()
            .filter(|t| t.unlocked)
            .map(|t| t.cost)
            .sum()
    }
}
