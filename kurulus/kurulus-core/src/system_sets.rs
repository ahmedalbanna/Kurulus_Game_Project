use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    // Phase 1: Input -> Commands
    InputProcessing,
    CommandQueueing,

    // Phase 2: AI Decision Making
    AIDecision,
    AICommandIssuing,

    // Phase 3: Movement & Positioning
    Pathfinding,
    FormationMovement,
    FreeFormMovement,
    GridQuantization,

    // Phase 4: Detection & Stealth
    VisionCalculation,
    StealthDetection,
    AmbushTrigger,

    // Phase 5: Combat Resolution (expanded)
    // 5a: Pre-resolution (charge tracking, flanking calc)
    ChargeUpdate,
    FlankingCalculation,
    // 5b: Resolution (attack timers, melee, ranged, siege)
    AttackTimers,
    MeleeResolution,
    RangedResolution,
    SiegeResolution,
    // 5c: Application (damage, DoT, morale checks)
    DamageApplication,
    DotApplication,
    MoraleCheck,
    MoraleUpdate,
    PanicProcessing,
    // 5d: Death
    DeathProcessing,
    // 5e: Feedback (hit feedback, damage numbers)
    HitFeedback,
    // 5f: Formation combat bonus application
    FormationCombatEffects,

    // Phase 6: Economy & Progression
    ResourceProduction,
    ResourceConsumption,
    ConstructionProgress,
    CityPopulation,
    CityHappiness,
    CityServices,
    CityZones,
    CityProgression,
    ResearchProgress,
    XPGain,

    // Phase 7: World & Mission
    MissionObjectiveCheck,
    FactionRelationUpdate,
    WorldEventProcessing,

    // Phase 8: Co-Rule & Delegation
    DelegationProcessing,
    TrustUpdate,
    SynergyCalculation,

    // Phase 9: Cleanup & Sync
    ComponentCleanup,
    PositionSync,
}
