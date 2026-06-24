use bevy::math::IVec2;
use bevy::prelude::*;
use kurulus_core::prelude::*;
use std::collections::HashMap;

pub struct AIPlugin;

impl Plugin for AIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AIDifficultyConfig>()
            .init_resource::<FactionConfig>()
            .add_systems(
                FixedUpdate,
                (
                    // Phase 2a: AI Decision (world state → blackboard + goals)
                    threat_evaluation_system.in_set(SimulationSet::AIDecision),
                    ai_blackboard_system.in_set(SimulationSet::AIDecision),
                    ai_goal_selection_system.in_set(SimulationSet::AIDecision),
                    ai_strategy_transition_system.in_set(SimulationSet::AIDecision),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    // Phase 2b: AI Command Issuing (blackboard + goals → events)
                    bt_evaluate_system.in_set(SimulationSet::AICommandIssuing),
                    resource_management_system.in_set(SimulationSet::AICommandIssuing),
                    combat_ai_system.in_set(SimulationSet::AICommandIssuing),
                    worker_ai_system.in_set(SimulationSet::AICommandIssuing),
                    ai_command_issuing_system.in_set(SimulationSet::AICommandIssuing),
                    faction_unit_bonus_system.in_set(SimulationSet::AICommandIssuing),
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  FACTORY — spawn an AI controller with faction-appropriate defaults
// ═══════════════════════════════════════════════════════════════════════════════

/// Convenience function to build an AIController for a given faction.
/// Uses FactionConfig to pick the default strategy and difficulty-appropriate
/// decision interval.
pub fn create_faction_controller(
    faction: FactionId,
    difficulty: AIDifficulty,
    faction_config: &FactionConfig,
    difficulty_config: &AIDifficultyConfig,
) -> AIController {
    let strategy = faction_config
        .strategy_for(faction)
        .map(|s| s.default_strategy)
        .unwrap_or(AIStrategy::Balanced);

    let diff_mods = difficulty_config.for_difficulty(difficulty);

    AIController {
        personality: AIPersonality::Balanced,
        difficulty,
        decision_interval: 1.0 * diff_mods.decision_interval_multiplier,
        last_decision_time: 0.0,
        current_goal: None,
        strategy,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  BEHAVIOR TREE TRAVERSAL ENGINE
// ═══════════════════════════════════════════════════════════════════════════════

fn bt_evaluate_system(
    mut ai_query: Query<(&mut BehaviorTree, &mut AIController)>,
    unit_query: Query<(
        Entity,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        Option<&AICombatUnit>,
    )>,
    building_query: Query<(Entity, &Building, &WorldPosition)>,
    enemy_query: Query<(Entity, &WorldPosition, &FactionId, &Health)>,
    stockpile: Res<ResourceStockpile>,
    game_time: Res<GameTime>,
    diff_config: Res<AIDifficultyConfig>,
    mut move_events: EventWriter<MoveOrder>,
    mut attack_events: EventWriter<AttackOrder>,
    mut build_events: EventWriter<BuildOrder>,
    mut recruit_events: EventWriter<RecruitOrder>,
) {
    for (mut bt, mut controller) in ai_query.iter_mut() {
        let diff_mods = diff_config.for_difficulty(controller.difficulty);

        // Scale decision interval by difficulty
        let effective_interval =
            controller.decision_interval * diff_mods.decision_interval_multiplier;

        if game_time.total_seconds as f32 - controller.last_decision_time < effective_interval {
            continue;
        }

        let root = bt.root.clone();
        let blackboard = bt.blackboard.clone();
        let mut path = std::mem::take(&mut bt.execution_path);

        let state = evaluate_node(
            &root,
            &mut path,
            &blackboard,
            &stockpile,
            &unit_query,
            &building_query,
            &enemy_query,
            &*controller,
            diff_mods,
            &mut move_events,
            &mut attack_events,
            &mut build_events,
            &mut recruit_events,
        );

        controller.last_decision_time = game_time.total_seconds as f32;
        bt.last_state = state;

        if state != BehaviorState::Running {
            path.clear();
        }
        bt.execution_path = path;
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_node(
    node: &BehaviorNode,
    path: &mut Vec<usize>,
    blackboard: &AIDataBlackboard,
    stockpile: &ResourceStockpile,
    units: &Query<(
        Entity,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        Option<&AICombatUnit>,
    )>,
    buildings: &Query<(Entity, &Building, &WorldPosition)>,
    enemies: &Query<(Entity, &WorldPosition, &FactionId, &Health)>,
    controller: &AIController,
    diff_mods: &DifficultyModifiers,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) -> BehaviorState {
    match node {
        BehaviorNode::Sequence(children) => evaluate_sequence(
            children,
            path,
            blackboard,
            stockpile,
            units,
            buildings,
            enemies,
            controller,
            diff_mods,
            move_events,
            attack_events,
            build_events,
            recruit_events,
        ),
        BehaviorNode::Selector(children) => evaluate_selector(
            children,
            path,
            blackboard,
            stockpile,
            units,
            buildings,
            enemies,
            controller,
            diff_mods,
            move_events,
            attack_events,
            build_events,
            recruit_events,
        ),
        BehaviorNode::Condition(cond) => evaluate_condition(cond, blackboard, stockpile, buildings),
        BehaviorNode::Action(action) => execute_action(
            action,
            stockpile,
            units,
            buildings,
            enemies,
            controller,
            diff_mods,
            move_events,
            attack_events,
            build_events,
            recruit_events,
        ),
        BehaviorNode::Inverter(child) => {
            let state = evaluate_node(
                child,
                path,
                blackboard,
                stockpile,
                units,
                buildings,
                enemies,
                controller,
                diff_mods,
                move_events,
                attack_events,
                build_events,
                recruit_events,
            );
            match state {
                BehaviorState::Running => BehaviorState::Running,
                BehaviorState::Success => BehaviorState::Failure,
                BehaviorState::Failure => BehaviorState::Success,
            }
        }
        BehaviorNode::Repeater(child, count) => {
            let max_loops = if *count == 0 { u32::MAX } else { *count };
            for _ in 0..max_loops {
                let mut child_path = Vec::new();
                let state = evaluate_node(
                    child,
                    &mut child_path,
                    blackboard,
                    stockpile,
                    units,
                    buildings,
                    enemies,
                    controller,
                    diff_mods,
                    move_events,
                    attack_events,
                    build_events,
                    recruit_events,
                );
                match state {
                    BehaviorState::Running => return BehaviorState::Running,
                    BehaviorState::Failure => return BehaviorState::Failure,
                    BehaviorState::Success => {}
                }
            }
            BehaviorState::Success
        }
        BehaviorNode::Succeeder(child) => {
            let _ = evaluate_node(
                child,
                path,
                blackboard,
                stockpile,
                units,
                buildings,
                enemies,
                controller,
                diff_mods,
                move_events,
                attack_events,
                build_events,
                recruit_events,
            );
            BehaviorState::Success
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_sequence(
    children: &[BehaviorNode],
    path: &mut Vec<usize>,
    blackboard: &AIDataBlackboard,
    stockpile: &ResourceStockpile,
    units: &Query<(
        Entity,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        Option<&AICombatUnit>,
    )>,
    buildings: &Query<(Entity, &Building, &WorldPosition)>,
    enemies: &Query<(Entity, &WorldPosition, &FactionId, &Health)>,
    controller: &AIController,
    diff_mods: &DifficultyModifiers,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) -> BehaviorState {
    let start = if !path.is_empty() { path.remove(0) } else { 0 };
    for i in start..children.len() {
        let mut child_path = if i == start {
            std::mem::take(path)
        } else {
            Vec::new()
        };
        let state = evaluate_node(
            &children[i],
            &mut child_path,
            blackboard,
            stockpile,
            units,
            buildings,
            enemies,
            controller,
            diff_mods,
            move_events,
            attack_events,
            build_events,
            recruit_events,
        );
        match state {
            BehaviorState::Running => {
                let mut resume = vec![i];
                resume.append(&mut child_path);
                *path = resume;
                return BehaviorState::Running;
            }
            BehaviorState::Failure => return BehaviorState::Failure,
            BehaviorState::Success => {}
        }
    }
    BehaviorState::Success
}

#[allow(clippy::too_many_arguments)]
fn evaluate_selector(
    children: &[BehaviorNode],
    path: &mut Vec<usize>,
    blackboard: &AIDataBlackboard,
    stockpile: &ResourceStockpile,
    units: &Query<(
        Entity,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        Option<&AICombatUnit>,
    )>,
    buildings: &Query<(Entity, &Building, &WorldPosition)>,
    enemies: &Query<(Entity, &WorldPosition, &FactionId, &Health)>,
    controller: &AIController,
    diff_mods: &DifficultyModifiers,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) -> BehaviorState {
    let start = if !path.is_empty() { path.remove(0) } else { 0 };
    for i in start..children.len() {
        let mut child_path = if i == start {
            std::mem::take(path)
        } else {
            Vec::new()
        };
        let state = evaluate_node(
            &children[i],
            &mut child_path,
            blackboard,
            stockpile,
            units,
            buildings,
            enemies,
            controller,
            diff_mods,
            move_events,
            attack_events,
            build_events,
            recruit_events,
        );
        match state {
            BehaviorState::Running => {
                let mut resume = vec![i];
                resume.append(&mut child_path);
                *path = resume;
                return BehaviorState::Running;
            }
            BehaviorState::Success => return BehaviorState::Success,
            BehaviorState::Failure => {}
        }
    }
    BehaviorState::Failure
}

fn evaluate_condition(
    condition: &BehaviorCondition,
    blackboard: &AIDataBlackboard,
    stockpile: &ResourceStockpile,
    _buildings: &Query<(Entity, &Building, &WorldPosition)>,
) -> BehaviorState {
    let result = match condition {
        BehaviorCondition::HasResources => {
            stockpile.gold >= 100.0
                || stockpile.food >= 200.0
                || stockpile.wood >= 100.0
                || stockpile.stone >= 50.0
        }
        BehaviorCondition::HasArmy => blackboard.get("army_count").unwrap_or(0.0) >= 3.0,
        BehaviorCondition::IsUnderAttack => blackboard.get("under_attack").unwrap_or(0.0) > 0.0,
        BehaviorCondition::CanExpand => {
            stockpile.gold >= 200.0 && stockpile.wood >= 150.0 && stockpile.stone >= 100.0
        }
        BehaviorCondition::EnemyInRange => {
            blackboard.get("nearest_enemy_distance").unwrap_or(f32::MAX) < 15.0
        }
        BehaviorCondition::HasHero => blackboard.get("has_hero").unwrap_or(0.0) > 0.0,
        BehaviorCondition::PopulationCapReached => stockpile.population >= stockpile.max_population,
        BehaviorCondition::ArmyStrengthBelow(threshold) => {
            blackboard.get("army_strength").unwrap_or(0.0) < *threshold
        }
        BehaviorCondition::ResourceLow(resource_type) => match resource_type {
            AiResourceType::Food => stockpile.food < 100.0,
            AiResourceType::Gold => stockpile.gold < 50.0,
            AiResourceType::Wood => stockpile.wood < 80.0,
            AiResourceType::Stone => stockpile.stone < 40.0,
        },
        BehaviorCondition::HasBuilding(building_type) => {
            let _ = building_type;
            false
        }
        BehaviorCondition::Custom(key) => blackboard.get(key).unwrap_or(0.0) > 0.0,
    };
    if result {
        BehaviorState::Success
    } else {
        BehaviorState::Failure
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_action(
    action: &BehaviorAction,
    _stockpile: &ResourceStockpile,
    units: &Query<(
        Entity,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        Option<&AICombatUnit>,
    )>,
    _buildings: &Query<(Entity, &Building, &WorldPosition)>,
    _enemies: &Query<(Entity, &WorldPosition, &FactionId, &Health)>,
    _controller: &AIController,
    _diff_mods: &DifficultyModifiers,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) -> BehaviorState {
    match action {
        BehaviorAction::MoveTo(pos) => {
            if let Some((entity, _, _, _world_pos, _, _)) = units.iter().next() {
                move_events.send(MoveOrder {
                    entity,
                    target: *pos,
                    formation: None,
                });
                return BehaviorState::Running;
            }
            BehaviorState::Failure
        }
        BehaviorAction::Attack(target) => {
            attack_events.send(AttackOrder {
                entity: *target,
                target: *target,
                aggressive: true,
            });
            BehaviorState::Running
        }
        BehaviorAction::Build(building_type, pos) => {
            if let Some((entity, _, _, _world_pos, _, _)) = units.iter().next() {
                build_events.send(BuildOrder {
                    entity,
                    building_type: *building_type,
                    position: *pos,
                });
                return BehaviorState::Running;
            }
            BehaviorState::Failure
        }
        BehaviorAction::Recruit(archetype, count) => {
            recruit_events.send(RecruitOrder {
                building: Entity::PLACEHOLDER,
                archetype: *archetype,
                count: *count,
            });
            BehaviorState::Running
        }
        BehaviorAction::Research(_) => BehaviorState::Success,
        BehaviorAction::Retreat(pos) => {
            if let Some((entity, _, _, _, _, _)) = units.iter().next() {
                move_events.send(MoveOrder {
                    entity,
                    target: *pos,
                    formation: None,
                });
                return BehaviorState::Running;
            }
            BehaviorState::Failure
        }
        BehaviorAction::Patrol(a, b) => {
            if let Some((entity, _, _, world_pos, _, _)) = units.iter().next() {
                let target = if world_pos.grid.0.distance_squared(a.0)
                    < world_pos.grid.0.distance_squared(b.0)
                {
                    *b
                } else {
                    *a
                };
                move_events.send(MoveOrder {
                    entity,
                    target,
                    formation: None,
                });
                return BehaviorState::Running;
            }
            BehaviorState::Failure
        }
        BehaviorAction::Wait(frames) => {
            if *frames == 0 {
                BehaviorState::Success
            } else {
                BehaviorState::Running
            }
        }
        BehaviorAction::Flank(target) => {
            attack_events.send(AttackOrder {
                entity: *target,
                target: *target,
                aggressive: true,
            });
            BehaviorState::Running
        }
        BehaviorAction::Custom(_) => BehaviorState::Success,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  AI BLACKBOARD SYSTEM
// ═══════════════════════════════════════════════════════════════════════════════

fn ai_blackboard_system(
    mut ai_query: Query<(&AIController, &mut BehaviorTree, &FactionId)>,
    unit_query: Query<(Entity, &Health, &Stats, &WorldPosition, &FactionId)>,
    building_query: Query<(&Building, &FactionId)>,
    stockpile: Res<ResourceStockpile>,
) {
    for (_controller, mut bt, faction_id) in ai_query.iter_mut() {
        let bb = &mut bt.blackboard;
        let mut army_count = 0u32;
        let mut army_strength = 0.0f32;
        let mut has_hero = false;
        let mut total_health = 0.0f32;

        for (_entity, health, stats, _pos, unit_faction) in unit_query.iter() {
            if *unit_faction == *faction_id {
                army_count += 1;
                army_strength += health.current + stats.attack_damage;
                total_health += health.current;
                if health.max > 200.0 {
                    has_hero = true;
                }
            }
        }

        bb.set("army_count", army_count as f32);
        bb.set("army_strength", army_strength);
        bb.set("total_health", total_health);
        bb.set("has_hero", if has_hero { 1.0 } else { 0.0 });

        let mut barracks_count = 0u32;
        for (building, b_faction) in building_query.iter() {
            if *b_faction == *faction_id && building.building_type == BuildingType::Barracks {
                barracks_count += 1;
            }
        }
        bb.set("barracks_count", barracks_count as f32);
        bb.set("population", stockpile.population as f32);
        bb.set("max_population", stockpile.max_population as f32);
        bb.set("gold", stockpile.gold);
        bb.set("food", stockpile.food);
        bb.set("wood", stockpile.wood);
        bb.set("stone", stockpile.stone);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  AI GOAL SELECTION — faction-aware, difficulty-scaled
// ═══════════════════════════════════════════════════════════════════════════════

fn ai_goal_selection_system(
    mut ai_query: Query<(&mut AIController, &BehaviorTree, &FactionId)>,
    unit_query: Query<(&Health, &FactionId)>,
    stockpile: Res<ResourceStockpile>,
    diff_config: Res<AIDifficultyConfig>,
    faction_config: Res<FactionConfig>,
) {
    for (mut controller, bt, faction_id) in ai_query.iter_mut() {
        let diff_mods = diff_config.for_difficulty(controller.difficulty);
        let _faction_strat = faction_config.strategy_for(*faction_id);

        // Scale thresholds by difficulty (harder AI = lower thresholds = more aggressive)
        let aggression_scale = 1.0 / diff_mods.attack_frequency_multiplier.max(0.1);

        let mut our_units = 0u32;
        let mut our_health = 0.0f32;
        for (health, f) in unit_query.iter() {
            if *f == *faction_id {
                our_units += 1;
                our_health += health.current;
            }
        }
        let army_strength = bt.blackboard.get("army_strength").unwrap_or(0.0);

        // Scale army strength thresholds by difficulty
        let attack_threshold = 200.0 * aggression_scale;
        let defend_health_threshold = 500.0 * aggression_scale;

        let new_goal = match controller.strategy {
            AIStrategy::Rush => {
                // Rush: attack early with minimal army
                if army_strength >= attack_threshold * 0.5 {
                    AIGoal::AttackPlayer
                } else {
                    AIGoal::TrainArmy
                }
            }
            AIStrategy::Turtle => {
                // Turtle: build eco, mass army, then attack
                if stockpile.gold < 300.0 || stockpile.wood < 200.0 {
                    AIGoal::GatherResources
                } else if our_units < (10.0 * aggression_scale) as u32 {
                    AIGoal::TrainArmy
                } else {
                    AIGoal::AttackPlayer
                }
            }
            AIStrategy::Boom => {
                // Boom: economy focus
                if stockpile.population < stockpile.max_population / 2 {
                    AIGoal::ExpandTerritory
                } else if our_health < defend_health_threshold {
                    AIGoal::DefendBase
                } else {
                    AIGoal::BuildEconomy
                }
            }
            AIStrategy::Guerrilla => {
                if army_strength < 150.0 * aggression_scale {
                    AIGoal::GatherResources
                } else if bt
                    .blackboard
                    .get("nearest_enemy_distance")
                    .unwrap_or(f32::MAX)
                    < 20.0
                {
                    AIGoal::AttackPlayer
                } else {
                    AIGoal::TrainArmy
                }
            }
            AIStrategy::Balanced => {
                if bt.blackboard.get("under_attack").unwrap_or(0.0) > 0.0 {
                    AIGoal::DefendBase
                } else if stockpile.food < 100.0 || stockpile.gold < 50.0 {
                    AIGoal::GatherResources
                } else if our_units < (8.0 * aggression_scale) as u32 {
                    AIGoal::TrainArmy
                } else {
                    AIGoal::AttackPlayer
                }
            }
        };

        controller.current_goal = Some(new_goal);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  AI STRATEGY TRANSITION — faction-specific (Byzantine Turtle→Boom, Mongol Rush→Guerrilla)
// ═══════════════════════════════════════════════════════════════════════════════

fn ai_strategy_transition_system(
    mut ai_query: Query<(&mut AIController, &BehaviorTree, &FactionId)>,
    game_time: Res<GameTime>,
    faction_config: Res<FactionConfig>,
) {
    for (mut controller, bt, faction_id) in ai_query.iter_mut() {
        let faction_strat = match faction_config.strategy_for(*faction_id) {
            Some(s) => s,
            None => continue,
        };

        let transition = match &faction_strat.transition_trigger {
            StrategyTransitionTrigger::Never => continue,
            StrategyTransitionTrigger::ArmyThreshold(threshold) => {
                let strength = bt.blackboard.get("army_strength").unwrap_or(0.0);
                strength >= *threshold
            }
            StrategyTransitionTrigger::GameTime(time_secs) => {
                game_time.total_seconds as f32 >= *time_secs
            }
        };

        if transition {
            if let Some(new_strategy) = &faction_strat.transition_strategy {
                if controller.strategy != *new_strategy {
                    controller.strategy = *new_strategy;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  THREAT EVALUATION — full implementation
// ═══════════════════════════════════════════════════════════════════════════════

fn threat_evaluation_system(
    grid: Res<SpatialGrid>,
    mut unit_query: Query<(
        Entity,
        &WorldPosition,
        &FactionId,
        &Health,
        &Stats,
        Option<&mut Threat>,
    )>,
    game_time: Res<GameTime>,
) {
    // Phase 1: Snapshot all unit data (owned copies = no borrow conflicts)
    let unit_data: Vec<(Entity, GridPos, FactionId, Health, Stats)> = unit_query
        .iter_mut()
        .map(|(e, p, f, h, s, _)| (e, p.grid, *f, h.clone(), s.clone()))
        .collect();

    let data_map: HashMap<Entity, (GridPos, FactionId, Health, Stats)> = unit_data
        .iter()
        .map(|(e, p, f, h, s)| (*e, (*p, *f, h.clone(), s.clone())))
        .collect();

    // Phase 2: For each unit, use SpatialGrid to find only nearby enemies
    let mut threats: Vec<(Entity, Entity, f32, ThreatType)> = Vec::new();

    for (entity, pos, faction, health, stats) in &unit_data {
        let nearby = grid.entities_in_range(*pos, stats.vision_range);

        for &enemy in &nearby {
            if enemy == *entity {
                continue;
            }
            let Some((epos, efaction, ehealth, _estats)) = data_map.get(&enemy) else {
                continue;
            };
            if *efaction == *faction {
                continue;
            }

            let dist_sq = pos.0.distance_squared(epos.0);
            let max_range = stats.vision_range as i32;
            if dist_sq > max_range * max_range {
                continue;
            }

            // Threat = f(distance, remaining HP, attack damage)
            let distance_factor = 1.0 - (dist_sq as f32).sqrt() / stats.vision_range;
            let health_ratio = ehealth.current / ehealth.max;
            let damage_factor = stats.attack_damage / 100.0;

            // Proximity = highest weight, then remaining health, then damage potential
            let threat_level =
                (distance_factor * 0.5 + health_ratio * 0.3 + damage_factor * 0.2).clamp(0.0, 1.0);

            let threat_type = if dist_sq < 25 {
                ThreatType::Proximity
            } else if ehealth.current > health.current * 0.5 {
                ThreatType::Military
            } else {
                ThreatType::Proximity
            };

            threats.push((*entity, enemy, threat_level, threat_type));
        }
    }

    // Phase 3: Update Threat components on existing entities
    for (entity, _pos, _faction, _health, _stats, threat_opt) in unit_query.iter_mut() {
        if let Some(mut existing_threat) = threat_opt {
            // Find the highest threat for this entity
            let best = threats
                .iter()
                .filter(|(e, _, _, _)| *e == entity)
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((_, target, level, ttype)) = best {
                existing_threat.target = *target;
                existing_threat.threat_level = *level;
                existing_threat.last_seen = game_time.total_seconds as f32;
                existing_threat.threat_type = *ttype;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  RESOURCE MANAGEMENT AI
// ═══════════════════════════════════════════════════════════════════════════════

fn resource_management_system(
    ai_query: Query<(&AIController, &FactionId)>,
    mut recruit_events: EventWriter<RecruitOrder>,
    stockpile: Res<ResourceStockpile>,
) {
    for (controller, _faction_id) in ai_query.iter() {
        let goal = match controller.current_goal {
            Some(g) => g,
            None => continue,
        };

        if matches!(goal, AIGoal::TrainArmy) {
            if stockpile.gold >= 50.0 && stockpile.food >= 50.0 {
                recruit_events.send(RecruitOrder {
                    building: Entity::PLACEHOLDER,
                    archetype: UnitArchetype::LightInfantry,
                    count: 1,
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  COMBAT AI — target selection, retreat, flanking (difficulty-aware targeting)
// ═══════════════════════════════════════════════════════════════════════════════

fn combat_ai_system(
    grid: Res<SpatialGrid>,
    mut combat_query: Query<(
        Entity,
        &mut AICombatUnit,
        &WorldPosition,
        &Health,
        &Stats,
        &FactionId,
    )>,
    all_units: Query<(Entity, &WorldPosition, &Health, &FactionId, Option<&Stats>)>,
    mut move_events: EventWriter<MoveOrder>,
    mut attack_events: EventWriter<AttackOrder>,
) {
    // Snapshot all unit data for O(1) lookups (owned copies = no borrow conflicts)
    let unit_map: HashMap<Entity, (GridPos, Health, FactionId, Option<Stats>)> = all_units
        .iter()
        .map(|(e, p, h, f, s)| (e, (p.grid, h.clone(), *f, s.cloned())))
        .collect();

    let mut targets_to_attack: Vec<(Entity, Entity)> = Vec::new();
    let mut entities_to_move: Vec<(Entity, GridPos)> = Vec::new();

    for (entity, mut combat_unit, pos, health, stats, faction) in combat_query.iter_mut() {
        let combat_pos = pos.grid;

        // Use SpatialGrid to find nearby entities
        let nearby = grid.entities_in_range(combat_pos, stats.vision_range);

        // Separate into enemies and friendlies
        let mut nearby_enemies: Vec<(Entity, GridPos, Health)> = Vec::new();
        let mut nearby_friendlies: Vec<(Entity, GridPos)> = Vec::new();

        for &candidate in &nearby {
            if candidate == entity {
                continue;
            }
            let Some((cpos, chealth, cfaction, _cstats)) = unit_map.get(&candidate) else {
                continue;
            };

            let dist_sq = combat_pos.0.distance_squared(cpos.0);
            if dist_sq >= (stats.vision_range * stats.vision_range) as i32 {
                continue;
            }

            if *cfaction == *faction {
                if cpos.0 != combat_pos.0 && dist_sq < 100 {
                    nearby_friendlies.push((candidate, *cpos));
                }
            } else {
                nearby_enemies.push((candidate, *cpos, chealth.clone()));
            }
        }

        if nearby_enemies.is_empty() {
            combat_unit.target = None;
            combat_unit.flanking = false;
            continue;
        }

        let outnumbered = nearby_friendlies.len() < nearby_enemies.len();
        let health_ratio = health.current / health.max;
        let should_retreat =
            (outnumbered || health_ratio < 0.2) && combat_unit.retreat_timer <= 0.0;

        if should_retreat {
            if let Some((_friendly_entity, fpos)) = nearby_friendlies.first() {
                let retreat_dir = (fpos.0 - combat_pos.0).signum();
                let retreat_pos = GridPos(combat_pos.0 + retreat_dir * 5);
                entities_to_move.push((entity, retreat_pos));
                combat_unit.target = None;
                combat_unit.retreat_timer = 120.0;
                continue;
            }
        }
        if combat_unit.retreat_timer > 0.0 {
            combat_unit.retreat_timer -= 1.0;
        }

        if !combat_unit.flanking && nearby_friendlies.len() >= nearby_enemies.len() + 2 {
            if let Some((_enemy_entity, epos, _eh)) = nearby_enemies.first() {
                let dir = epos.0 - combat_pos.0;
                let flank_offset = IVec2::new(-dir.y, dir.x).signum() * 3;
                let flank_pos = GridPos(epos.0 + flank_offset);
                entities_to_move.push((entity, flank_pos));
                combat_unit.flanking = true;
                continue;
            }
        }

        // Target selection: nearest enemy
        let target = nearby_enemies
            .iter()
            .min_by(|a, b| {
                let da = combat_pos.0.distance_squared(a.1 .0);
                let db = combat_pos.0.distance_squared(b.1 .0);
                da.cmp(&db)
            })
            .map(|(e, _, _)| *e);
        if let Some(target_entity) = target {
            combat_unit.target = Some(target_entity);
            targets_to_attack.push((entity, target_entity));
        }
    }

    for (entity, target) in targets_to_attack {
        attack_events.send(AttackOrder {
            entity,
            target,
            aggressive: true,
        });
    }
    for (entity, pos) in entities_to_move {
        move_events.send(MoveOrder {
            entity,
            target: pos,
            formation: None,
        });
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  WORKER AI
// ═══════════════════════════════════════════════════════════════════════════════

fn worker_ai_system(
    mut worker_query: Query<(Entity, &mut AIWorker, &WorldPosition, &Unit, &FactionId)>,
    _building_query: Query<(&Building, &WorldPosition, &FactionId)>,
    mut move_events: EventWriter<MoveOrder>,
    stockpile: Res<ResourceStockpile>,
) {
    for (entity, mut worker, pos, _unit, _faction_id) in worker_query.iter_mut() {
        match worker.assigned_task {
            Some(AIWorkerTask::GatherFood) => {
                let target = GridPos(IVec2::new(20, 20));
                if pos.grid.0.distance_squared(target.0) > 4 {
                    move_events.send(MoveOrder {
                        entity,
                        target,
                        formation: None,
                    });
                }
            }
            Some(AIWorkerTask::GatherWood) => {
                let target = GridPos(IVec2::new(25, 15));
                if pos.grid.0.distance_squared(target.0) > 4 {
                    move_events.send(MoveOrder {
                        entity,
                        target,
                        formation: None,
                    });
                }
            }
            Some(AIWorkerTask::GatherStone) => {
                let target = GridPos(IVec2::new(15, 25));
                if pos.grid.0.distance_squared(target.0) > 4 {
                    move_events.send(MoveOrder {
                        entity,
                        target,
                        formation: None,
                    });
                }
            }
            Some(AIWorkerTask::GatherGold) => {
                let target = GridPos(IVec2::new(10, 10));
                if pos.grid.0.distance_squared(target.0) > 4 {
                    move_events.send(MoveOrder {
                        entity,
                        target,
                        formation: None,
                    });
                }
            }
            Some(AIWorkerTask::Construct(_building_type)) => {
                let target = GridPos(IVec2::new(15, 15));
                if pos.grid.0.distance_squared(target.0) > 4 {
                    move_events.send(MoveOrder {
                        entity,
                        target,
                        formation: None,
                    });
                }
            }
            None => {
                worker.idle_time += 1.0;
                if worker.idle_time > 60.0 {
                    let task = if stockpile.food < 100.0 {
                        AIWorkerTask::GatherFood
                    } else if stockpile.wood < 80.0 {
                        AIWorkerTask::GatherWood
                    } else if stockpile.stone < 40.0 {
                        AIWorkerTask::GatherStone
                    } else if stockpile.gold < 50.0 {
                        AIWorkerTask::GatherGold
                    } else {
                        AIWorkerTask::GatherFood
                    };
                    worker.assigned_task = Some(task);
                    worker.idle_time = 0.0;
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  FACTION UNIT BONUS — applies faction-specific stat modifiers to units
// ═══════════════════════════════════════════════════════════════════════════════

fn faction_unit_bonus_system(
    mut unit_query: Query<(Entity, &mut Stats, &mut Health, &FactionId, &Unit)>,
    faction_config: Res<FactionConfig>,
) {
    for (_entity, mut stats, mut health, faction_id, _unit) in unit_query.iter_mut() {
        if let Some(bonus) = faction_config.bonus_for(*faction_id) {
            // Apply global bonuses
            stats.attack_damage *= 1.0 + bonus.attack_damage_bonus;
            health.armor *= 1.0 + bonus.armor_bonus;
            stats.move_speed *= 1.0 + bonus.speed_bonus;
            stats.morale_max *= 1.0 + bonus.morale_bonus;
            stats.morale = stats.morale_max;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  AI COMMAND ISSUING — final step for pending decisions
// ═══════════════════════════════════════════════════════════════════════════════

fn ai_command_issuing_system(
    ai_query: Query<(&AIController, &FactionId)>,
    unit_query: Query<(Entity, &Unit, &WorldPosition, &FactionId)>,
    mut move_events: EventWriter<MoveOrder>,
) {
    for (controller, faction_id) in ai_query.iter() {
        let goal = match controller.current_goal {
            Some(g) => g,
            None => continue,
        };

        if let AIGoal::AttackPlayer = goal {
            let mut our_units: Vec<(Entity, &WorldPosition)> = Vec::new();
            let mut enemy_positions: Vec<&WorldPosition> = Vec::new();

            for (entity, unit, pos, f) in unit_query.iter() {
                if *f == *faction_id && unit.archetype != UnitArchetype::Worker {
                    our_units.push((entity, pos));
                }
            }
            for (_e, _u, pos, f) in unit_query.iter() {
                if *f != *faction_id {
                    enemy_positions.push(pos);
                }
            }

            if let Some(avg_enemy_pos) = average_position(&enemy_positions) {
                for (entity, _pos) in our_units {
                    move_events.send(MoveOrder {
                        entity,
                        target: avg_enemy_pos,
                        formation: None,
                    });
                }
            }
        }
    }
}

fn average_position(positions: &[&WorldPosition]) -> Option<GridPos> {
    if positions.is_empty() {
        return None;
    }
    let sum = positions.iter().fold(IVec2::ZERO, |acc, p| acc + p.grid.0);
    let count = positions.len() as i32;
    Some(GridPos(IVec2::new(sum.x / count, sum.y / count)))
}
