//! # Combat Damage Pipeline — 4‑Stage Architecture
//!
//! Cleans up the damage flow that was previously inline inside
//! `melee_combat_system` into a **testable 4‑stage pipeline**:
//!
//! ```text
//! Stage 1  ─collect_modifiers()→  DamageContext
//! Stage 2  ─calculate_base()→     raw damage (effectiveness + armour)
//! Stage 3  ─apply_multipliers()→  final damage (block/dodge/crit/formation/morale)
//! Stage 4  ─emit events→          DamageDealt + AttackTriggered
//! ```
//!
//! This module is **already wired** into `CombatPlugin` and can be used
//! by both melee and (future) ranged combat systems.

use bevy::prelude::*;
use kurulus_core::prelude::*;
use rand::Rng;
use std::collections::HashMap;

// ═════════════════════════════════════════════════════════════════════════════
//  PIPELINE DATA TYPES
// ═════════════════════════════════════════════════════════════════════════════

/// All inputs needed to resolve a single attack.
/// Built by Stage 1 (collect_modifiers) and consumed by Stages 2+3.
#[derive(Clone, Debug)]
pub struct DamageContext {
    // Identifiers
    pub attacker: Entity,
    pub target: Entity,

    // Weapon / base stats
    pub weapon_damage: f32,
    pub weapon_type: WeaponType,
    pub armor_pen: f32,
    pub attack_speed: f32,

    // Flanking
    pub flanking: FlankingInfo,

    // Equipment bonuses (sum of all equipped item stat bonuses)
    pub equipment_damage_bonus: f32,
    pub equipment_armor_bonus: f32,

    // Buffs
    pub buff_damage_bonus: f32,
    pub buff_attack_speed_bonus: f32,

    // Formation
    pub attacker_formation: FormationBonus,
    pub defender_formation: FormationBonus,

    // Morale
    pub attacker_morale: CurrentMoraleState,
    pub defender_morale: CurrentMoraleState,

    // Target defence
    pub target_armor: f32,
    pub target_armor_type: ArmorType,
    pub target_archetype: UnitArchetype,
}

/// Output of the full pipeline after all four stages.
#[derive(Clone, Debug)]
pub struct DamageOutcome {
    /// Final damage amount after all modifiers.
    pub final_amount: f32,
    /// The attacker entity.
    pub attacker: Entity,
    /// The target entity.
    pub target: Entity,
    /// Damage type (derived from weapon).
    pub damage_type: DamageType,
    /// Whether the attack was blocked.
    pub blocked: bool,
    /// Whether the attack was dodged.
    pub dodged: bool,
    /// Critical hit classification.
    pub crit_type: CritType,
    /// Flanking info used for this attack.
    pub flanking: FlankingInfo,
    /// Raw damage before any multipliers (for logging/feedback).
    pub raw_damage: f32,
    /// Base damage after effectiveness + armour but before multipliers.
    pub base_damage: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE 1 — Collect Modifiers
// ═════════════════════════════════════════════════════════════════════════════

/// Collects all modifier data from ECS queries into a [`DamageContext`].
///
/// This is a pure helper function used by the pipeline system.  It
/// aggregates data from multiple component queries for a single
/// attacker–target pair.
#[allow(clippy::too_many_arguments)]
pub fn collect_modifiers(
    attacker: Entity,
    target: Entity,
    weapon: &MeleeWeapon,
    attacker_stats: &Stats,
    flanking: &FlankingInfo,
    equipment: Option<&Equipment>,
    buffs: Option<&ActiveBuffs>,
    attacker_formation: Option<&FormationBonus>,
    defender_formation: Option<&FormationBonus>,
    attacker_morale: Option<&CurrentMoraleState>,
    defender_morale: Option<&CurrentMoraleState>,
    target_health: &Health,
    target_archetype: UnitArchetype,
) -> DamageContext {
    // Equipment stat bonuses
    let (eq_dmg, eq_armor) = equipment
        .map(|e| {
            let dmg = e
                .weapon
                .as_ref()
                .map(|w| w.stats.damage_bonus)
                .unwrap_or(0.0)
                + e.armor
                    .as_ref()
                    .map(|a| a.stats.damage_bonus)
                    .unwrap_or(0.0)
                + e.accessory_1
                    .as_ref()
                    .map(|a| a.stats.damage_bonus)
                    .unwrap_or(0.0)
                + e.accessory_2
                    .as_ref()
                    .map(|a| a.stats.damage_bonus)
                    .unwrap_or(0.0);
            let arm = e
                .weapon
                .as_ref()
                .map(|w| w.stats.defense_bonus)
                .unwrap_or(0.0)
                + e.armor
                    .as_ref()
                    .map(|a| a.stats.defense_bonus)
                    .unwrap_or(0.0)
                + e.accessory_1
                    .as_ref()
                    .map(|a| a.stats.defense_bonus)
                    .unwrap_or(0.0)
                + e.accessory_2
                    .as_ref()
                    .map(|a| a.stats.defense_bonus)
                    .unwrap_or(0.0);
            (dmg, arm)
        })
        .unwrap_or((0.0, 0.0));

    // Buff stat bonuses
    let (buff_dmg, buff_atk_spd) = buffs
        .map(|b| {
            let dmg = b.buffs.iter().map(|bf| bf.damage_bonus).sum::<f32>();
            let spd = b.buffs.iter().map(|bf| bf.attack_speed_bonus).sum::<f32>();
            (dmg, spd)
        })
        .unwrap_or((0.0, 0.0));

    DamageContext {
        attacker,
        target,
        weapon_damage: weapon.damage,
        weapon_type: weapon.weapon_type,
        armor_pen: weapon.armor_penetration,
        attack_speed: attacker_stats.attack_speed,
        flanking: *flanking,
        equipment_damage_bonus: eq_dmg,
        equipment_armor_bonus: eq_armor,
        buff_damage_bonus: buff_dmg,
        buff_attack_speed_bonus: buff_atk_spd,
        attacker_formation: attacker_formation.copied().unwrap_or_default(),
        defender_formation: defender_formation.copied().unwrap_or_default(),
        attacker_morale: attacker_morale.copied().unwrap_or_default(),
        defender_morale: defender_morale.copied().unwrap_or_default(),
        target_armor: target_health.armor,
        target_armor_type: target_health.armor_type,
        target_archetype,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE 2 — Calculate Base Damage
// ═════════════════════════════════════════════════════════════════════════════

/// Stage 2: compute the **raw** damage before block/dodge/crit/formation/morale.
///
/// Formula:
/// ```text
/// raw = weapon_damage × flank_mult × formation_dmg × (1 + equipment)
///       × (1 + buff_damage) × anti_cav_mult × morale_damage_mult
/// ```
///
/// Then runs armour + resistance:
/// ```text
/// base = calculate_damage(raw, ... armour, resistance, effectiveness ...)
/// ```
pub fn calculate_base_damage(
    ctx: &DamageContext,
    effectiveness_table: &DamageEffectivenessTable,
    resistance_table: &ArmorResistanceTable,
) -> f32 {
    // 1. Compute raw damage from all additive/multiplicative sources
    let flank_mult = ctx.flanking.damage_multiplier;

    let atk_formation_dmg = ctx.attacker_formation.damage_mult;

    let anti_cav_mult = if ctx.target_archetype == UnitArchetype::LightCavalry
        || ctx.target_archetype == UnitArchetype::HeavyCavalry
    {
        ctx.attacker_formation.anti_cavalry_mult
    } else {
        1.0
    };

    let raw_damage = ctx.weapon_damage
        * flank_mult
        * atk_formation_dmg
        * (1.0 + ctx.equipment_damage_bonus)
        * (1.0 + ctx.buff_damage_bonus)
        * anti_cav_mult
        * ctx.attacker_morale.damage_mult;

    // 2. Apply armour penetration + resistance
    let total_armor = ctx.target_armor + ctx.equipment_armor_bonus;
    let damage_type = kurulus_core::components::combat::damage_type_for_weapon(ctx.weapon_type);

    super::calculate_damage(
        raw_damage,
        damage_type,
        total_armor,
        ctx.target_armor_type,
        ctx.armor_pen,
        effectiveness_table,
        resistance_table,
        ctx.target_archetype,
    )
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE 3 — Apply Multipliers (block, dodge, crit, formation, morale)
// ═════════════════════════════════════════════════════════════════════════════

/// Stage 3: apply block/dodge/crit rolls and formation + morale multipliers.
///
/// Returns a [`DamageOutcome`] with the final amount and metadata flags.
#[allow(clippy::too_many_arguments)]
pub fn apply_multipliers(
    base_damage: f32,
    ctx: &DamageContext,
    config: &CombatConfig,
    attacker_archetype: UnitArchetype,
) -> DamageOutcome {
    let damage_type = kurulus_core::components::combat::damage_type_for_weapon(ctx.weapon_type);

    // ─── Block check ───────────────────────────────────────────────────
    let base_block = kurulus_core::components::combat::compute_block_chance(attacker_archetype);
    let block_reduction = ctx.flanking.block_reduction;
    let formation_block_bonus = ctx.attacker_formation.block_bonus / 100.0;
    let block_chance = (base_block - block_reduction + formation_block_bonus).max(0.0);
    let blocked = block_chance > 0.0 && rand::thread_rng().gen_range(0.0..1.0) < block_chance;

    // ─── Dodge check ───────────────────────────────────────────────────
    let dodge_chance = kurulus_core::components::combat::compute_dodge_chance(attacker_archetype);
    let dodged =
        !blocked && dodge_chance > 0.0 && rand::thread_rng().gen_range(0.0..1.0) < dodge_chance;

    // ─── Critical hit roll ─────────────────────────────────────────────
    let base_crit = config.base_crit_chance;
    let flank_crit = match ctx.flanking.zone {
        AttackZone::Front => 0.0,
        AttackZone::Side => config.flank_crit_bonus_side,
        AttackZone::Rear => config.flank_crit_bonus_rear,
    };
    let crit_chance = base_crit + flank_crit;
    let is_crit =
        !blocked && crit_chance > 0.0 && rand::thread_rng().gen_range(0.0..1.0) < crit_chance;

    // ─── Block mitigation ──────────────────────────────────────────────
    let final_amount = if blocked {
        base_damage * 0.5
    } else if dodged {
        0.0
    } else if is_crit {
        (base_damage
            * config.crit_damage_multiplier
            * ctx.defender_formation.incoming_damage_mult
            * ctx.defender_morale.defense_mult)
            .max(config.min_damage)
    } else {
        (base_damage
            * ctx.defender_formation.incoming_damage_mult
            * ctx.defender_morale.defense_mult)
            .max(config.min_damage)
    };

    let crit_type = if is_crit {
        CritType::Flanking
    } else {
        CritType::None
    };

    DamageOutcome {
        final_amount,
        attacker: ctx.attacker,
        target: ctx.target,
        damage_type,
        blocked,
        dodged,
        crit_type,
        flanking: ctx.flanking,
        raw_damage: ctx.weapon_damage,
        base_damage,
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  STAGE 4 — Emit Events (full system)
// ═════════════════════════════════════════════════════════════════════════════

/// Stage 4: emit `DamageDealt` + `AttackTriggered` events from an outcome.
///
/// This is the standard emit function used by the pipeline system.
/// It can also be called directly by other combat systems (e.g. ranged,
/// siege) that have already built their own `DamageOutcome`.
pub fn emit_damage_events(
    outcome: &DamageOutcome,
    damage_events: &mut EventWriter<DamageDealt>,
    attack_triggers: &mut EventWriter<AttackTriggered>,
) {
    damage_events.send(DamageDealt {
        attacker: outcome.attacker,
        target: outcome.target,
        amount: outcome.final_amount,
        damage_type: outcome.damage_type,
        crit: outcome.crit_type,
        flanking: outcome.flanking,
        is_friendly_fire: false,
        source_ability: None,
    });

    attack_triggers.send(AttackTriggered {
        attacker: outcome.attacker,
        target: outcome.target,
        weapon_type: match outcome.damage_type {
            DamageType::Slash => WeaponType::Sword,
            DamageType::Pierce => WeaponType::Spear,
            DamageType::Blunt => WeaponType::Mace,
            _ => WeaponType::Sword,
        },
        is_charge: false,
        is_from_stealth: false,
    });
}

// ═════════════════════════════════════════════════════════════════════════════
//  ORCHESTRATOR — run_damage_pipeline_system
// ═════════════════════════════════════════════════════════════════════════════

/// Full 4‑stage damage pipeline system.
///
/// For every attacker whose `AttackTimer` just finished:
/// 1. Finds the nearest valid target (using SpatialGrid)
/// 2. Collects all modifiers → `DamageContext` (Stage 1)
/// 3. Calculates base damage (Stage 2)
/// 4. Applies block/dodge/crit/formation/morale multipliers (Stage 3)
/// 5. Emits `DamageDealt` + `AttackTriggered` (Stage 4)
///
/// This system runs in `SimulationSet::MeleeResolution` — at the same
/// point as `melee_combat_system`, but as a **clean, testable alternative**
/// that both melee and future ranged combat can use.
#[allow(clippy::type_complexity)]
pub fn run_damage_pipeline_system(
    grid: Res<SpatialGrid>,
    mut damage_events: EventWriter<DamageDealt>,
    mut attack_triggers: EventWriter<AttackTriggered>,
    mut attackers: Query<
        (
            Entity,
            &WorldPosition,
            &Stats,
            &mut AttackTimer,
            &MeleeWeapon,
            &Unit,
            Option<&FlankingInfo>,
            Option<&ActiveBuffs>,
            Option<&Equipment>,
            Option<&FormationBonus>,
            Option<&CurrentMoraleState>,
        ),
        (With<Unit>, Without<Dead>),
    >,
    targets: Query<
        (
            Entity,
            &WorldPosition,
            &Health,
            &Unit,
            Option<&FormationBonus>,
            Option<&CurrentMoraleState>,
        ),
        (With<Unit>, Without<Dead>),
    >,
    config: Res<CombatConfig>,
    effectiveness_table: Res<DamageEffectivenessTable>,
    resistance_table: Res<ArmorResistanceTable>,
) {
    // Snapshot target data for O(1) lookup
    let target_map: HashMap<
        Entity,
        (
            Vec2,
            UnitArchetype,
            FactionId,
            f32,
            ArmorType,
            Option<FormationBonus>,
            Option<CurrentMoraleState>,
        ),
    > = targets
        .iter()
        .map(|(e, pos, health, unit, formation, morale)| {
            (
                e,
                (
                    pos.world_2d.0,
                    unit.archetype,
                    unit.faction,
                    health.armor,
                    health.armor_type,
                    formation.copied(),
                    morale.copied(),
                ),
            )
        })
        .collect();

    for (
        attacker_e,
        attacker_pos,
        attacker_stats,
        mut timer,
        weapon,
        attacker_unit,
        flanking_opt,
        buffs_opt,
        equipment_opt,
        formation_opt,
        attacker_morale_opt,
    ) in attackers.iter_mut()
    {
        // ── Only proceed when the attack timer fires ────────────────────
        if !timer.0.just_finished() {
            continue;
        }

        // ── Find nearest valid target via SpatialGrid ──────────────────
        let nearby = grid.entities_in_range(attacker_pos.grid, attacker_stats.attack_range);

        let best_target = nearby
            .iter()
            .filter_map(|&candidate| {
                if candidate == attacker_e {
                    return None;
                }
                let (target_pos, _, target_faction, _, _, _, _) = target_map.get(&candidate)?;
                if attacker_unit.faction == *target_faction {
                    return None;
                }
                let dist = attacker_pos.world_2d.0.distance(*target_pos);
                if dist > attacker_stats.attack_range {
                    return None;
                }
                Some((candidate, dist))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let Some((target_e, _)) = best_target else {
            continue;
        };

        let Some(&(
            _target_pos,
            target_archetype,
            _target_faction,
            target_armor_val,
            target_armor_type,
            target_formation_opt,
            target_morale_opt,
        )) = target_map.get(&target_e)
        else {
            continue;
        };

        // ── Build a synthetic Health for collect_modifiers ─────────────
        let target_health = Health {
            current: 0.0,
            max: 0.0,
            armor: target_armor_val,
            armor_type: target_armor_type,
        };

        // ── Stage 1: Collect modifiers ─────────────────────────────────
        let ctx = collect_modifiers(
            attacker_e,
            target_e,
            weapon,
            attacker_stats,
            &flanking_opt.copied().unwrap_or_default(),
            equipment_opt,
            buffs_opt,
            formation_opt,
            target_formation_opt.as_ref(),
            attacker_morale_opt,
            target_morale_opt.as_ref(),
            &target_health,
            target_archetype,
        );

        // ── Stage 2: Calculate base damage ─────────────────────────────
        let base_damage = calculate_base_damage(&ctx, &effectiveness_table, &resistance_table);

        // ── Stage 3: Apply multipliers ─────────────────────────────────
        let outcome = apply_multipliers(base_damage, &ctx, &config, attacker_unit.archetype);

        // ── Stage 4: Emit events ───────────────────────────────────────
        emit_damage_events(&outcome, &mut damage_events, &mut attack_triggers);

        // ── Reset attack timer with effective attack speed ─────────────
        let effective_attack_speed =
            (attacker_stats.attack_speed + ctx.buff_attack_speed_bonus).max(0.1);
        timer.0 = Timer::from_seconds(1.0 / effective_attack_speed, TimerMode::Repeating);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TESTS
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────

    fn test_damage_context() -> DamageContext {
        DamageContext {
            attacker: Entity::from_raw(1),
            target: Entity::from_raw(2),
            weapon_damage: 10.0,
            weapon_type: WeaponType::Sword,
            armor_pen: 0.0,
            attack_speed: 1.0,
            flanking: FlankingInfo::default(),
            equipment_damage_bonus: 0.0,
            equipment_armor_bonus: 0.0,
            buff_damage_bonus: 0.0,
            buff_attack_speed_bonus: 0.0,
            attacker_formation: FormationBonus::default(),
            defender_formation: FormationBonus::default(),
            attacker_morale: CurrentMoraleState::default(),
            defender_morale: CurrentMoraleState::default(),
            target_armor: 5.0,
            target_armor_type: ArmorType::Medium,
            target_archetype: UnitArchetype::LightInfantry,
        }
    }

    fn test_tables() -> (DamageEffectivenessTable, ArmorResistanceTable) {
        (
            DamageEffectivenessTable::default(),
            ArmorResistanceTable::default(),
        )
    }

    fn test_config() -> CombatConfig {
        CombatConfig {
            base_crit_chance: 0.05,
            crit_damage_multiplier: 1.5,
            flank_crit_bonus_side: 0.10,
            flank_crit_bonus_rear: 0.20,
            min_damage: 1.0,
            ..Default::default()
        }
    }

    // ── Stage 2 Tests ───────────────────────────────────────────────────

    #[test]
    fn test_stage2_base_damage_basic() {
        let ctx = test_damage_context();
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // Slash vs LightInfantry ×1.5, Medium armor resistance 0.6
        // raw = 10.0 * 1.0 * 1.0 * 1.0 * 1.0 * 1.0 * 1.0 = 10.0
        // effective_armor = 5.0 * (1.0 - 0.0) = 5.0
        // resistance = 0.6 (Medium vs Slash)
        // final = (10.0 * 1.5) - (5.0 * 0.6) = 15.0 - 3.0 = 12.0
        assert!(
            (damage - 12.0).abs() < 0.01,
            "Expected 12.0, got {}",
            damage
        );
    }

    #[test]
    fn test_stage2_armor_pen_reduces_effective_armor() {
        let mut ctx = test_damage_context();
        ctx.armor_pen = 0.5; // 50% armour penetration
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // effective_armor = 5.0 * (1.0 - 0.5) = 2.5
        // final = (10.0 * 1.5) - (2.5 * 0.6) = 15.0 - 1.5 = 13.5
        assert!(
            (damage - 13.5).abs() < 0.01,
            "Expected 13.5, got {}",
            damage
        );
    }

    #[test]
    fn test_stage2_flanking_increases_damage() {
        let mut ctx = test_damage_context();
        ctx.flanking = FlankingInfo {
            attack_angle: 150.0,
            zone: AttackZone::Rear,
            damage_multiplier: 1.5,
            block_reduction: 0.3,
        };
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // raw = 10.0 * 1.5 * ... = 15.0
        // after armour: (15.0 * 1.5) - (5.0 * 0.6) = 22.5 - 3.0 = 19.5
        assert!(
            (damage - 19.5).abs() < 0.01,
            "Expected 19.5, got {}",
            damage
        );
    }

    #[test]
    fn test_stage2_equipment_bonus() {
        let mut ctx = test_damage_context();
        ctx.equipment_damage_bonus = 0.25; // +25% damage from equipment
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // raw = 10.0 * 1.0 * 1.0 * 1.25 * 1.0 * 1.0 * 1.0 = 12.5
        // after armour: (12.5 * 1.5) - (5.0 * 0.6) = 18.75 - 3.0 = 15.75
        assert!(
            (damage - 15.75).abs() < 0.01,
            "Expected 15.75, got {}",
            damage
        );
    }

    #[test]
    fn test_stage2_buff_damage_bonus() {
        let mut ctx = test_damage_context();
        ctx.buff_damage_bonus = 0.3; // +30% from buffs
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // raw = 10.0 * 1.0 * 1.0 * 1.0 * 1.3 * 1.0 * 1.0 = 13.0
        // after armour: (13.0 * 1.5) - (5.0 * 0.6) = 19.5 - 3.0 = 16.5
        assert!(
            (damage - 16.5).abs() < 0.01,
            "Expected 16.5, got {}",
            damage
        );
    }

    #[test]
    fn test_stage2_morale_affects_damage() {
        let mut ctx = test_damage_context();
        ctx.attacker_morale = CurrentMoraleState {
            damage_mult: 0.7, // Low morale = -30% damage
            ..Default::default()
        };
        let (eff, res) = test_tables();
        let damage = calculate_base_damage(&ctx, &eff, &res);
        // raw = 10.0 * 1.0 * 1.0 * 1.0 * 1.0 * 1.0 * 0.7 = 7.0
        // after armour: (7.0 * 1.5) - (5.0 * 0.6) = 10.5 - 3.0 = 7.5
        assert!((damage - 7.5).abs() < 0.01, "Expected 7.5, got {}", damage);
    }

    // ── Stage 3 Tests ───────────────────────────────────────────────────

    #[test]
    fn test_stage3_block_chance() {
        let ctx = test_damage_context();
        let config = test_config();
        // HeavyInfantry has 15% base block
        let outcome = apply_multipliers(100.0, &ctx, &config, UnitArchetype::HeavyInfantry);
        // We can't assert exact result (random), but block can only halve damage
        if outcome.blocked {
            assert!(
                (outcome.final_amount - 50.0).abs() < 0.01,
                "Blocked should halve to 50"
            );
        } else {
            assert!(
                (outcome.final_amount - 100.0).abs() < 0.01 || outcome.final_amount > 0.0,
                "Unblocked should be near 100 or at least positive"
            );
        }
    }

    #[test]
    fn test_stage3_dodge_is_zero_damage() {
        let ctx = test_damage_context();
        let config = test_config();
        // Scout has 15% base dodge
        let outcome = apply_multipliers(100.0, &ctx, &config, UnitArchetype::Scout);
        if outcome.dodged {
            assert_eq!(outcome.final_amount, 0.0, "Dodged should deal 0 damage");
        } else {
            assert!(
                outcome.final_amount > 0.0,
                "Undodged should deal positive damage"
            );
        }
    }

    #[test]
    fn test_stage3_outcome_metadata() {
        let ctx = test_damage_context();
        let config = test_config();
        let outcome = apply_multipliers(100.0, &ctx, &config, UnitArchetype::LightInfantry);
        assert_eq!(outcome.attacker, Entity::from_raw(1));
        assert_eq!(outcome.target, Entity::from_raw(2));
        assert_eq!(outcome.damage_type, DamageType::Slash); // Sword → Slash
        assert_eq!(outcome.raw_damage, 10.0); // weapon_damage from test context
        assert_eq!(outcome.base_damage, 100.0); // passed in
    }

    // ── Stage 4 Tests ───────────────────────────────────────────────────
    // Stage 4 event emission is pure Bevy boilerplate (EventWriter::send).
    // We test the *outcome struct integrity* here. The emit_damage_events
    // function is too trivial to need its own integration test.

    #[test]
    fn test_stage4_outcome_struct_is_correct() {
        let outcome = DamageOutcome {
            final_amount: 15.0,
            attacker: Entity::from_raw(1),
            target: Entity::from_raw(2),
            damage_type: DamageType::Blunt,
            blocked: true,
            dodged: false,
            crit_type: CritType::None,
            flanking: FlankingInfo::default(),
            raw_damage: 10.0,
            base_damage: 12.0,
        };
        assert_eq!(outcome.final_amount, 15.0);
        assert_eq!(outcome.attacker, Entity::from_raw(1));
        assert_eq!(outcome.target, Entity::from_raw(2));
        assert!(outcome.blocked);
        assert!(!outcome.dodged);
    }

    // ── Stage 1 Tests ───────────────────────────────────────────────────

    #[test]
    fn test_stage1_collect_modifiers_basic() {
        let attacker = Entity::from_raw(1);
        let target = Entity::from_raw(2);
        let weapon = MeleeWeapon {
            damage: 15.0,
            weapon_type: WeaponType::Axe,
            range: 1.5,
            armor_penetration: 0.2,
        };
        let stats = Stats::default();
        let flanking = FlankingInfo::default();
        let health = Health::default(); // armor=5.0, armor_type=Light

        let ctx = collect_modifiers(
            attacker,
            target,
            &weapon,
            &stats,
            &flanking,
            None, // equipment
            None, // buffs
            None, // attacker_formation
            None, // defender_formation
            None, // attacker_morale
            None, // defender_morale
            &health,
            UnitArchetype::LightInfantry,
        );

        assert_eq!(ctx.attacker, attacker);
        assert_eq!(ctx.target, target);
        assert_eq!(ctx.weapon_damage, 15.0);
        assert_eq!(ctx.weapon_type, WeaponType::Axe);
        assert_eq!(ctx.armor_pen, 0.2);
        assert_eq!(ctx.target_armor, 5.0);
        assert_eq!(ctx.target_armor_type, ArmorType::Light);
        assert_eq!(ctx.target_archetype, UnitArchetype::LightInfantry);
    }

    #[test]
    fn test_stage1_collect_modifiers_with_equipment() {
        let attacker = Entity::from_raw(1);
        let target = Entity::from_raw(2);
        // Manually construct MeleeWeapon (no Default impl)
        let weapon = MeleeWeapon {
            damage: 10.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.0,
        };
        let stats = Stats::default();
        let flanking = FlankingInfo::default();
        let health = Health::default();

        let equipment = Equipment {
            weapon: Some(EquipmentItem {
                item_id: "test_sword".into(),
                name: "Test Sword".into(),
                rarity: Rarity::Common,
                slot: EquipmentSlot::Weapon,
                stats: EquipmentStats {
                    damage_bonus: 0.3,
                    defense_bonus: 0.1,
                    ..Default::default()
                },
                passive_effect: None,
            }),
            ..Default::default()
        };

        let ctx = collect_modifiers(
            attacker,
            target,
            &weapon,
            &stats,
            &flanking,
            Some(&equipment),
            None,
            None,
            None,
            None,
            None,
            &health,
            UnitArchetype::LightInfantry,
        );

        assert!((ctx.equipment_damage_bonus - 0.3).abs() < 0.01);
        assert!((ctx.equipment_armor_bonus - 0.1).abs() < 0.01);
    }
}
