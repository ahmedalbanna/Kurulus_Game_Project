use bevy::prelude::*;
use crate::coord::GridPos;
use crate::components::{BuildingType, ZoneType, UnitArchetype, AbilityId, FormationType, CombatStance};

#[derive(Event, Clone, Debug)]
pub struct MoveOrder {
    pub entity: Entity,
    pub target: GridPos,
    pub formation: Option<Entity>,
}

#[derive(Event, Clone, Debug)]
pub struct AttackOrder {
    pub entity: Entity,
    pub target: Entity,
    pub aggressive: bool,
}

#[derive(Event, Clone, Debug)]
pub struct BuildOrder {
    pub entity: Entity,
    pub building_type: BuildingType,
    pub position: GridPos,
}

/// Fired when a `BuildOrder` is rejected because the target grid cell
/// has a zone that doesn't allow the requested building type.
#[derive(Event, Clone, Debug)]
pub struct BuildRejected {
    pub order: BuildOrder,
    pub reason: BuildRejectionReason,
}

/// Why a build order was rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildRejectionReason {
    /// The zone at the target position does not match the building type.
    /// `.0` = the zone type found, `.1` = the building type requested.
    ZoneMismatch(ZoneType, BuildingType),
    /// The target grid cell is already occupied by another building or construction.
    CellOccupied,
}

#[derive(Event, Clone, Debug)]
pub struct RecruitOrder {
    pub building: Entity,
    pub archetype: UnitArchetype,
    pub count: u32,
}

#[derive(Event, Clone, Debug)]
pub struct ResearchOrder {
    pub entity: Entity,
    pub tech_id: String,
}

#[derive(Event, Clone, Debug)]
pub struct AbilityUsed {
    pub caster: Entity,
    pub ability_id: AbilityId,
    pub target: AbilityTarget,
}

#[derive(Event, Clone, Debug)]
pub enum AbilityTarget {
    Entity(Entity),
    Position(GridPos),
    Direction(Vec2),
    None,
}

/// Order to place an ambush zone at the given position.
/// Consumes a resource (gold or special resource).
#[derive(Event, Clone, Debug)]
pub struct PlaceAmbushOrder {
    pub entity: Entity,
    pub position: GridPos,
}

/// Direction to cycle the active hero selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeroSwitchDirection {
    Next,
    Previous,
}

/// Fired when the player wants to switch the actively controlled hero
/// (e.g. by pressing Tab or clicking a hero portrait).
#[derive(Event, Clone, Debug)]
pub struct HeroSwitchEvent {
    /// `Some(Next/Previous)` to cycle, `None` to switch to the specified hero entity.
    pub direction: Option<HeroSwitchDirection>,
    /// If `direction` is `None`, switch directly to this hero entity.
    pub target: Option<Entity>,
}

// ─── Phase B: Core Command Events ──────────────────────────────────────────

/// Stop: clear movement, cancel current action, hold position.
#[derive(Event, Clone, Debug)]
pub struct StopCommand {
    pub entity: Entity,
}

/// Hold: stay in place, auto-engage enemies within attack range.
#[derive(Event, Clone, Debug)]
pub struct HoldCommand {
    pub entity: Entity,
}

/// Retreat: flee toward a safe direction or toward the nearest owned structure.
#[derive(Event, Clone, Debug)]
pub struct RetreatCommand {
    pub entity: Entity,
    /// Optional retreat direction. If `None`, the unit picks a direction away
    /// from the nearest visible enemy.
    pub direction: Option<Vec2>,
}

/// Scatter: move a short random distance from the current position.
/// Used to break formations under ranged fire or to avoid AoE.
#[derive(Event, Clone, Debug)]
pub struct ScatterCommand {
    pub entity: Entity,
}

/// Assign a formation type to the entity.
/// The formation positioning system (Phase D) computes slot offsets later.
#[derive(Event, Clone, Debug)]
pub struct FormationCommand {
    pub entity: Entity,
    pub formation: FormationType,
}

/// Change the combat stance of the entity.
#[derive(Event, Clone, Debug)]
pub struct StanceChangeCommand {
    pub entity: Entity,
    pub stance: CombatStance,
}

/// Attack-Move: move to a grid position. If an enemy is encountered en route,
/// auto-engage (switch to attack mode) without completing the full path.
#[derive(Event, Clone, Debug)]
pub struct AttackMoveOrder {
    pub entity: Entity,
    pub target: GridPos,
}
