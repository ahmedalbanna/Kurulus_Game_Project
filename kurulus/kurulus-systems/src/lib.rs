pub mod ai;
pub mod audio;
pub mod base_building;
pub mod beylik_politics;
pub mod city_builder;
pub mod combat;

use bevy_state::app::{AppExtStates, StatesPlugin};
use kurulus_core::prelude::*;
pub mod core_loop;
pub mod corule;
pub mod diplomacy;
pub mod economy;
pub mod hero;
pub mod imperial_admin;
pub mod narrative;
pub mod naval_warfare;
pub mod next_gen_heroes;
pub mod population;
pub mod progression;
pub mod romance_loyalty;
pub mod siege_warfare;
pub mod stealth;
pub mod succession_legacy;
pub mod ui;
pub mod unit_command;
pub mod world_map;

pub use ai::AIPlugin;
pub use audio::AudioLogicPlugin;
pub use base_building::BaseBuildingPlugin;
pub use beylik_politics::BeylikPoliticsPlugin;
pub use city_builder::CityBuilderPlugin;
pub use combat::CombatPlugin;
pub use core_loop::CoreLoopPlugin;
pub use corule::CoRulePlugin;
pub use diplomacy::DiplomacyPlugin;
pub use economy::EconomyPlugin;
pub use hero::HeroPlugin;
pub use imperial_admin::ImperialAdminPlugin;
pub use narrative::NarrativePlugin;
pub use naval_warfare::NavalWarfarePlugin;
pub use next_gen_heroes::NextGenHeroesPlugin;
pub use population::PopulationPlugin;
pub use progression::ProgressionPlugin;
pub use romance_loyalty::RomanceLoyaltyPlugin;
pub use siege_warfare::SiegeWarfarePlugin;
pub use stealth::StealthPlugin;
pub use succession_legacy::SuccessionLegacyPlugin;
pub use ui::UILogicPlugin;
pub use unit_command::UnitCommandPlugin;
pub use world_map::WorldMapPlugin;

/// Register all common events used across systems.
/// Must be called before plugins are added (or is idempotent if called after).
fn register_common_events(app: &mut bevy::prelude::App) {
    app.add_event::<DamageDealt>()
        .add_event::<UnitDied>()
        .add_event::<AttackTriggered>()
        .add_event::<MoraleBroken>()
        .add_event::<MoraleCheckTriggered>()
        .add_event::<PanicTriggered>()
        .add_event::<AmbushTriggered>()
        .add_event::<StealthRevealed>()
        .add_event::<MoraleEvent>()
        .add_event::<RangedFriendlyFire>()
        .add_event::<ChargeImpact>()
        .add_event::<RangedAttackResolved>()
        .add_event::<SiegeWallDamaged>()
        .add_event::<MoveOrder>()
        .add_event::<AttackOrder>()
        .add_event::<BuildOrder>()
        .add_event::<RecruitOrder>()
        .add_event::<BuildRejected>()
        .add_event::<ResearchOrder>()
        .add_event::<AbilityUsed>()
        .add_event::<ObjectiveCompleted>()
        .add_event::<MissionEnded>()
        .add_event::<MissionStarted>()
        .add_event::<DialogueTriggered>()
        .add_event::<ObjectiveProgress>()
        .add_event::<FactionRelationChanged>()
        .add_event::<RegionCaptured>()
        .add_event::<WarDeclared>()
        .add_event::<SelectionChanged>()
        .add_event::<SelectEvent>()
        .add_event::<BoxSelectEvent>()
        .add_event::<ControlGroupEvent>()
        .add_event::<Notification>()
        .add_event::<TreatySigned>()
        .add_event::<SeasonChanged>()
        .add_event::<DayNightCycle>()
        .add_event::<PanelOpened>()
        .add_event::<PanelClosed>()
        .add_event::<CampaignAdvanced>()
        .add_event::<PlaceAmbushOrder>()
        .add_event::<WorldMapTravelStarted>()
        .add_event::<WorldMapTravelCompleted>()
        .add_event::<TileRevealed>()
        .add_event::<TileExplored>()
        .add_event::<PoiDiscovered>()
        .add_event::<PoiVisited>()
        .add_event::<EncounterTriggered>()
        .add_event::<ScoutOrdered>()
        .add_event::<RecallScout>()
        .add_event::<ScoutEncounterResolved>()
        // Stealth events
        .add_event::<UnitSuspicionRaised>()
        .add_event::<UnitConfirmed>()
        .add_event::<AlarmRaised>()
        .add_event::<AmbushPhaseChanged>()
        .add_event::<StagedAmbushWave>()
        .add_event::<SilentKill>()
        .add_event::<LureEngaged>()
        .add_event::<LureDisengaged>()
        // Core loop events
        .add_event::<GamePhaseChanged>()
        .add_event::<TacticalPhaseChanged>()
        .add_event::<HarvestStarted>()
        .add_event::<HarvestCompleted>()
        .add_event::<HarvestXpAwarded>()
        .add_event::<HeroMomentTriggered>()
        .add_event::<HeroMomentCompleted>()
        .add_event::<MissionSetupComplete>()
        .add_event::<ComplicationStarted>()
        .add_event::<ClimaxTriggered>()
        .add_event::<ResolutionStarted>()
        .add_event::<GameSaved>()
        .add_event::<GameLoaded>()
        // Phase B command events
        .add_event::<StopCommand>()
        .add_event::<HoldCommand>()
        .add_event::<RetreatCommand>()
        .add_event::<ScatterCommand>()
        .add_event::<FormationCommand>()
        .add_event::<StanceChangeCommand>()
        .add_event::<AttackMoveOrder>()
        .add_event::<FormationBreakEvent>()
        // Building & construction events (Phase C)
        .add_event::<BuildOrderFailed>()
        .add_event::<CancelBuildOrder>()
        .add_event::<DemolishOrder>()
        .add_event::<RepairOrder>()
        .add_event::<InsufficientResources>()
        .add_event::<BuildingDestroyed>()
        .add_event::<TradeOrder>();
}

/// Register all logic plugins (no render) — used by headless test harness.
/// Also registers all common events, resources, and states so tests don't need manual setup.
pub fn register_logic_plugins(app: &mut bevy::prelude::App) {
    use kurulus_core::states::AppState;
    use kurulus_core::states::GamePhase;
    use kurulus_core::states::MissionState;

    // StatesPlugin is required for init_state to work
    app.add_plugins(StatesPlugin)
        // States
        .init_state::<AppState>()
        .init_state::<MissionState>()
        .init_state::<GamePhase>()
        // Core resources
        .init_resource::<GameTime>()
        .init_resource::<ResourceStockpile>()
        .init_resource::<ProductionRates>()
        .init_resource::<ResourceConsumption>()
        .init_resource::<TaxPolicy>()
        .init_resource::<TechTree>()
        .init_resource::<FactionRelations>()
        .init_resource::<TickTimer>()
        .init_resource::<CoCommandState>()
        .init_resource::<TrustMatrix>()
        .init_resource::<CampaignState>()
        .init_resource::<ResourcePersistence>()
        .init_resource::<TerrainGrid>()
        .init_resource::<WorldMap>()
        .init_resource::<BuildingCatalog>()
        .init_resource::<BuildingLimits>()
        // Core loop resources
        .init_resource::<TacticalPhaseTracker>()
        .init_resource::<HarvestState>()
        .init_resource::<LootTable>()
        .init_resource::<DifficultyConfig>()
        .init_resource::<HeroMomentsForMission>()
        // Combat resources
        .init_resource::<CombatConfig>()
        .init_resource::<ArmorResistanceTable>()
        .init_resource::<DamageEffectivenessTable>()
        .init_resource::<crate::combat::DeathQueue>()
        .init_resource::<DamageTracker>()
        // Stealth resources
        .init_resource::<AmbushConfig>()
        // Register types for BRP/MCP reflection debugging
        .register_type::<Stealth>()
        .register_type::<StealthType>()
        .register_type::<Detection>()
        .register_type::<AmbushZone>()
        .register_type::<NoiseLevel>()
        .register_type::<UnitVisibility>()
        .register_type::<AmbushMarker>()
        .register_type::<StealthZone>()
        .register_type::<StealthZoneType>()
        .register_type::<VisionCone>()
        .register_type::<ObserverState>()
        .register_type::<AlertLevel>()
        .register_type::<DetectionChain>()
        .register_type::<Shock>()
        .register_type::<AmbushState>()
        .register_type::<AmbushPhase>()
        .register_type::<TriggerType>()
        .register_type::<LureTarget>()
        .register_type::<AmbushConfig>()
        .register_type::<TerrainType>()
        .register_type::<TerrainTile>()
        .register_type::<TerrainGrid>()
        .register_type::<GameTime>()
        .register_type::<GridPos>()
        .register_type::<WorldPosition>()
        .register_type::<WorldPos2D>()
        .register_type::<WorldPos3D>()
        .register_type::<CampaignState>()
        .register_type::<ResourceStockpile>()
        .register_type::<ActiveHero>()
        .register_type::<WorldMap>()
        .register_type::<WorldTile>()
        .register_type::<FogState>()
        .register_type::<WorldTerrain>()
        .register_type::<RegionDef>()
        .register_type::<RegionResources>()
        .register_type::<WorldMapTravel>()
        .register_type::<RegionId>()
        .register_type::<PointOfInterest>()
        .register_type::<PoiId>()
        .register_type::<PoiType>()
        .register_type::<Encounter>()
        .register_type::<EncounterType>()
        .register_type::<EncounterDifficulty>()
        .register_type::<EncounterChoice>()
        .register_type::<EncounterOutcome>()
        .register_type::<EncounterTemplate>()
        .register_type::<WorldMapViewState>()
        .register_type::<kurulus_core::resources::world_map::WorldMapHover>()
        .register_type::<kurulus_core::resources::world_map::WorldMapSelection>()
        .register_type::<ScoutParty>()
        .register_type::<ScoutStatus>()
        .register_type::<CrossZoneSynergy>()
        .register_type::<CrossZoneSynergyTable>()
        .register_type::<CrossZoneSynergyEffect>()
        .register_type::<ZoneDefinition>()
        .register_type::<ZoneUnlockRequirement>()
        .register_type::<PopulationRegistry>()
        .register_type::<WorkerRole>()
        // Core loop types
        .register_type::<TacticalPhaseTracker>()
        .register_type::<DifficultyLevel>()
        .register_type::<DifficultyConfig>()
        .register_type::<HarvestState>()
        .register_type::<LootTable>()
        .register_type::<TacticalPhase>()
        // Selection types
        .register_type::<Selected>();
    // NOTE: HeroMomentsForMission registration skipped because HeroMomentConfig
    // contains non-Reflect types (HeroId, AbilityId). Reflect those types if
    // BRP introspection is needed for hero moments.

    register_common_events(app);

    // Group 1: Core Systems (1-8)
    app.add_plugins((
        CoreLoopPlugin,
        WorldMapPlugin,
        AIPlugin,
        EconomyPlugin,
        UnitCommandPlugin,
        CombatPlugin,
        StealthPlugin,
        HeroPlugin,
    ));

    // Group 2: Feature Systems (9-16)
    app.add_plugins((
        BaseBuildingPlugin,
        CityBuilderPlugin,
        PopulationPlugin,
        ProgressionPlugin,
        NarrativePlugin,
        DiplomacyPlugin,
        BeylikPoliticsPlugin,
        NextGenHeroesPlugin,
        RomanceLoyaltyPlugin,
        SuccessionLegacyPlugin,
    ));

    // Group 3: Advanced Systems (17-22)
    app.add_plugins((
        SiegeWarfarePlugin,
        ImperialAdminPlugin,
        NavalWarfarePlugin,
        CoRulePlugin,
        AudioLogicPlugin,
        UILogicPlugin,
    ));
}
