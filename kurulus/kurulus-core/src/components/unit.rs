use crate::coord::GridPos;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub archetype: UnitArchetype,
    pub faction: FactionId,
    pub formation: FormationType,
    pub stance: CombatStance,
}

impl Default for Unit {
    fn default() -> Self {
        Self {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
    pub armor: f32,
    pub armor_type: ArmorType,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            armor: 5.0,
            armor_type: ArmorType::Light,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub move_speed: f32,
    pub attack_range: f32,
    pub attack_damage: f32,
    pub attack_speed: f32,
    pub vision_range: f32,
    pub morale: f32,
    pub morale_max: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            move_speed: 3.0,
            attack_range: 1.0,
            attack_damage: 10.0,
            attack_speed: 1.0,
            vision_range: 6.0,
            morale: 50.0,
            morale_max: 100.0,
        }
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Formation {
    pub leader: Entity,
    pub formation_type: FormationType,
    pub facing: Vec2,
    pub is_breaking: bool,
    pub break_timer: Option<f32>,
    pub unit_count: usize,
}

#[derive(Component, Clone, Debug, Default)]
pub struct AttackTimer(pub Timer);

impl AttackTimer {
    pub fn new(duration: f32, mode: TimerMode) -> Self {
        Self(Timer::from_seconds(duration, mode))
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Dead;

/// Marker: this entity is currently selected by the player.
/// Only player-owned entities can be selected.
#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Selected;

/// The entity this unit is currently trying to attack.
/// Set by the attack order processor or auto-engagement system.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AttackTarget {
    pub target: Entity,
    pub aggressive: bool,
}

/// Marker: unit should hold current position and auto-engage enemies in range.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct HoldPosition;

/// Computed slot position for a unit within its formation.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FormationSlot {
    pub offset: Vec2,
    pub facing: Vec2,
    pub rank: usize,
    pub file: usize,
}

/// Active stat modifiers granted by being in a formation.
/// Re-computed whenever the formation type or leader changes.
/// All multipliers default to 1.0 (neutral).
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormationBonus {
    /// Multiplied with base damage (e.g. 1.15 = +15%)
    pub damage_mult: f32,
    /// Multiplied with incoming damage (e.g. 0.90 = -10%)
    pub incoming_damage_mult: f32,
    /// Multiplied with base move speed (e.g. 1.25 = +25%)
    pub speed_mult: f32,
    /// Flat bonus to block/parry chance (percentage points, e.g. 5.0 = +5%)
    pub block_bonus: f32,
    /// Bonus damage vs cavalry (multiplier)
    pub anti_cavalry_mult: f32,
    /// Whether formation prevents flanking entirely
    pub prevents_flanking: bool,
}

impl Default for FormationBonus {
    fn default() -> Self {
        Self {
            damage_mult: 1.0,
            incoming_damage_mult: 1.0,
            speed_mult: 1.0,
            block_bonus: 0.0,
            anti_cavalry_mult: 1.0,
            prevents_flanking: false,
        }
    }
}

/// Marks that a unit in a broken formation and tracks remaining break time.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FormationBreak {
    /// Seconds remaining before the unit reforms.
    pub remaining: f32,
    /// Total break duration (for UI / effects reference).
    pub total_duration: f32,
    /// Whether this break was caused by panic (vs. charge/ambush).
    pub panic: bool,
}

// ─── FormationType methods ─────────────────────────────────────────────────

impl FormationType {
    const SLOT_SPACING: f32 = 1.0; // cells between adjacent units

    /// Compute the slot offsets (relative to formation centre/leader) for
    /// `count` units arranged in this formation type.
    /// `facing` is the formation's facing direction (unit vector).
    /// Returns a Vec of (offset, rank, file) tuples, one per slot.
    pub fn compute_slots(&self, count: usize, facing: Vec2) -> Vec<(Vec2, usize, usize)> {
        let perp = Vec2::new(-facing.y, facing.x); // perpendicular (left)
        let spacing = Self::SLOT_SPACING;
        match self {
            // ── Line: units arrayed side-by-side perpendicular to facing ──
            Self::Line => {
                let mut slots = Vec::with_capacity(count);
                let half = (count.saturating_sub(1)) as f32 / 2.0;
                for i in 0..count {
                    let offset = perp * ((i as f32) - half) * spacing;
                    slots.push((offset, 0, i));
                }
                slots
            }
            // ── Column: units stacked behind the front unit ──────────────
            Self::Column => {
                let mut slots = Vec::with_capacity(count);
                for i in 0..count {
                    let offset = -facing * (i as f32) * spacing;
                    slots.push((offset, i, 0));
                }
                slots
            }
            // ── Square: dense grid, roughly square ───────────────────────
            Self::Square => {
                let cols = (count as f32).sqrt().ceil() as usize;
                let rows = (count + cols - 1) / cols;
                let mut slots = Vec::with_capacity(count);
                let half_cols = (cols.saturating_sub(1)) as f32 / 2.0;
                for r in 0..rows {
                    for c in 0..cols {
                        if slots.len() >= count {
                            break;
                        }
                        let offset = -facing * (r as f32) * spacing
                            + perp * ((c as f32) - half_cols) * spacing;
                        slots.push((offset, r, c));
                    }
                }
                slots
            }
            // ── Wedge: arrow/triangle widening with each rank ────────────
            Self::Wedge => {
                let mut slots = Vec::with_capacity(count);
                let mut rank = 0;
                let mut idx = 0;
                while idx < count {
                    let width = rank + 1; // rank 0 → 1 unit, rank 1 → 2, etc.
                    if idx + width > count {
                        // Last rank might be partial — centre it
                        let remaining = count - idx;
                        let half = (remaining.saturating_sub(1)) as f32 / 2.0;
                        for i in 0..remaining {
                            let offset = -facing * (rank as f32) * spacing
                                + perp * ((i as f32) - half) * spacing;
                            slots.push((offset, rank, i));
                        }
                    } else {
                        let half = (width.saturating_sub(1)) as f32 / 2.0;
                        for i in 0..width {
                            let offset = -facing * (rank as f32) * spacing
                                + perp * ((i as f32) - half) * spacing;
                            slots.push((offset, rank, i));
                        }
                    }
                    idx += width;
                    rank += 1;
                }
                slots
            }
            // ── Crescent: concave arc facing out ─────────────────────────
            Self::Crescent => {
                let mut slots = Vec::with_capacity(count);
                // Two prongs: left and right, meeting at the centre
                let half = count / 2;
                // Left prong (going out left then forward)
                for i in 0..half {
                    let t = (i as f32) / (half.max(1) as f32) * 0.8; // 0..0.8 rad; // 0..~46°
                    let offset = perp * (-t * 3.0 * spacing) - facing * (t * 2.0 * spacing);
                    slots.push((offset, i, 0));
                }
                // Right prong (mirror)
                for i in 0..(count - half) {
                    let t = (i as f32) / ((count - half).max(1) as f32) * 0.8;
                    let offset = perp * (t * 3.0 * spacing) - facing * (t * 2.0 * spacing);
                    slots.push((offset, i, 1));
                }
                slots
            }
            // ── Scatter: random offsets in a 2–3 cell radius ─────────────
            Self::Scatter => {
                // Deterministic pseudo-random positions based on slot index
                let mut slots = Vec::with_capacity(count);
                for i in 0..count {
                    let angle = (i as f32) * 2.4; // golden-angle-like spacing
                    let dist = 2.0 + (i as f32 % 3.0); // 2, 3, or 4
                    let offset = Vec2::new(angle.cos(), angle.sin()) * dist;
                    slots.push((offset, 0, i));
                }
                slots
            }
            // ── Ambush: diamond expanding outward ───────────────────────
            Self::Ambush => {
                let mut slots = Vec::with_capacity(count);
                let mut rank = 0;
                let mut idx = 0;
                while idx < count {
                    let width: usize = if rank == 0 { 1 } else { 2 + (rank - 1) * 2 };
                    let half = (width.saturating_sub(1)) as f32 / 2.0;
                    let remaining = count.saturating_sub(idx);
                    for i in 0..width.min(remaining) {
                        let offset = -facing * (rank as f32) * spacing
                            + perp * ((i as f32) - half) * spacing;
                        slots.push((offset, rank, i));
                    }
                    idx += width;
                    rank += 1;
                }
                slots
            }
        }
    }

    /// Stat multipliers/bonuses granted by this formation type.
    pub fn bonuses(&self) -> FormationBonus {
        match self {
            Self::Line => FormationBonus {
                damage_mult: 1.15,
                incoming_damage_mult: 0.90,
                speed_mult: 0.80,
                block_bonus: 5.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: false,
            },
            Self::Column => FormationBonus {
                damage_mult: 1.0,
                incoming_damage_mult: 1.0, // +25% from sides applied in melee system
                speed_mult: 1.25,
                block_bonus: 0.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: false,
            },
            Self::Square => FormationBonus {
                damage_mult: 0.85,
                incoming_damage_mult: 1.0,
                speed_mult: 0.70,
                block_bonus: 10.0,
                anti_cavalry_mult: 1.40,
                prevents_flanking: true,
            },
            Self::Wedge => FormationBonus {
                damage_mult: 1.20,
                incoming_damage_mult: 1.0,
                speed_mult: 1.0,
                block_bonus: 0.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: false,
            },
            Self::Scatter => FormationBonus {
                damage_mult: 0.80,
                incoming_damage_mult: 1.0, // -30% from ranged applied in ranged system
                speed_mult: 1.0,
                block_bonus: 0.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: true,
            },
            Self::Crescent => FormationBonus {
                damage_mult: 1.0,
                incoming_damage_mult: 1.0,
                speed_mult: 1.0,
                block_bonus: 0.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: true,
            },
            Self::Ambush => FormationBonus {
                damage_mult: 2.0, // double damage on first strike
                incoming_damage_mult: 1.0,
                speed_mult: 1.0,
                block_bonus: 0.0,
                anti_cavalry_mult: 1.0,
                prevents_flanking: false,
            },
        }
    }
}

/// For patrol command — unit cycles between waypoints.
/// Uses Vec<GridPos> from day one to support future multi-waypoint expansion.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PatrolRoute {
    pub waypoints: Vec<GridPos>,
    pub current_index: usize,
    pub forward: bool,
}

/// Queued orders from shift-click command chaining (max 10).
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct CommandQueue {
    pub orders: VecDeque<QueuedOrder>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QueuedOrder {
    Move(GridPos),
    Attack(Entity),
    AttackMove(GridPos),
    Patrol(Vec<GridPos>),
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitArchetype {
    LightInfantry,
    HeavyInfantry,
    Archer,
    LightCavalry,
    HeavyCavalry,
    Scout,
    /// Bait rider — fast, fragile unit used for luring enemies into ambushes.
    /// GDD: 6 m/s speed, 100 HP, 8 damage, doesn't stop under fire.
    BaitRider,
    SiegeCrew,
    Worker,
}

#[derive(Component, Reflect, Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[reflect(Component)]
pub enum FactionId {
    Kayi,
    Dodurga,
    Candar,
    Germiyan,
    Karaman,
    Byzantine,
    Mongol,
    Neutral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Column,
    Square,
    Wedge,
    Scatter,
    Crescent,
    Ambush,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatStance {
    Aggressive,
    Defensive,
    Hold,
    Passive,
    Retreat,
    Flee,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArmorType {
    Light,         // Leather/light infantry
    Medium,        // Partial plate/medium infantry
    Heavy,         // Full plate/heavy infantry
    Fortification, // Stone fortifications
    Cloth,         // Archers, support units (NEW)
    Wood,          // Wooden structures (NEW)
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct FormationFollower;

/// A calculated path from the unit's position to its destination.
/// The unit moves toward `waypoints[current_index]` each tick.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Path {
    pub waypoints: Vec<GridPos>,
    pub current_index: usize,
}

impl Path {
    pub fn new(waypoints: Vec<GridPos>) -> Self {
        Self {
            waypoints,
            current_index: 0,
        }
    }

    /// Returns the next waypoint the unit should move toward, if any.
    pub fn next_waypoint(&self) -> Option<GridPos> {
        self.waypoints.get(self.current_index).copied()
    }

    /// Advance to the next waypoint.  Returns `true` if the path is complete.
    pub fn advance(&mut self) -> bool {
        if self.current_index + 1 < self.waypoints.len() {
            self.current_index += 1;
            false
        } else {
            true
        }
    }
}

/// Marks that a unit wants to move to a specific grid cell.
/// The pathfinding system reads this and writes a `Path` component.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Destination {
    pub target: GridPos,
}

/// Marker for attack-move mode: move to destination but auto-engage enemies
/// encountered along the way. Removed when the unit engages or reaches the
/// destination.
#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct AttackTargetMarker;
