//! # AI Integration Layer — BT ↔ Utility Coordination
//!
//! Bridges the **Behavior Tree** (strategic planning) and **Utility AI**
//! (tactical execution) systems so AI units make coherent decisions.
//!
//! ## Architecture
//!
//! ```text
//!                  ┌──────────────────┐
//!                  │  BT Evaluation   │ ← goals, strategies, long-term plans
//!                  │ (bt_evaluate)    │
//!                  └────────┬─────────┘
//!                           │ current_goal, strategy
//!                           ▼
//!                  ┌──────────────────┐
//!                  │  AI Integration  │ ← resolves conflicts, bridges data
//!                  │ (this module)    │
//!                  └────────┬─────────┘
//!                           │ tactical_context (urgency, threats)
//!                           ▼
//!                  ┌──────────────────┐
//!                  │ Utility AI       │ ← moment-to-moment action selection
//!                  │ (thinker)        │
//!                  └────────┬─────────┘
//!                           │ unified action
//!                           ▼
//!                  ┌──────────────────┐
//!                  │  Event Writers   │ → MoveOrder / AttackOrder / etc.
//!                  └──────────────────┘
//! ```

use bevy::prelude::*;
use kurulus_core::prelude::*;
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
//  CONFIGURATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Controls how BT and Utility AI are coordinated for an entity or faction.
#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct AIIntegrationConfig {
    /// Priority scheme for resolving conflicting decisions.
    pub priority: AIPriority,
    /// Whether to expose BT blackboard data (goal, strategy) to utility scorers
    /// via the `AIBridgeState` resource.
    pub bridge_blackboard: bool,
    /// Weight given to BT strategic input when computing utility scores.
    /// 0.0 = ignore BT, 1.0 = full BT influence.
    pub bt_influence: f32,
}

impl Default for AIIntegrationConfig {
    fn default() -> Self {
        Self {
            priority: AIPriority::UtilityTactical,
            bridge_blackboard: true,
            bt_influence: 0.3,
        }
    }
}

/// How BT and Utility decisions are reconciled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIPriority {
    /// Utility action wins in tactical situations (under attack, low health).
    /// BT provides strategic goals but Utility can override for survival.
    UtilityTactical,
    /// BT strategy constrains Utility — Utility only picks actions aligned
    /// with the current BT goal.
    BTStrategic,
}

/// Runtime bridge state shared between BT and Utility.
/// Written by `bridge_bt_to_utility_system`, read by utility scorers.
#[derive(Resource, Clone, Debug, Default)]
pub struct AIBridgeState {
    /// Current strategic goal from BT (per entity).
    pub bt_goals: std::collections::HashMap<Entity, AIGoal>,
    /// Current BT strategy.
    pub bt_strategies: std::collections::HashMap<Entity, AIStrategy>,
    /// Whether the BT is in a "Running" state (in-progress plan).
    pub bt_running: std::collections::HashMap<Entity, bool>,
    /// Last BT action output (for coordination).
    pub bt_last_action: std::collections::HashMap<Entity, String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
//  SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════════

/// Phase 1: Bridge BT state → Utility context.
///
/// Reads every entity with both `BehaviorTree` + `AIController` +
/// `AIIntegrationConfig` and writes the BT's current goal+strategy into
/// the `AIBridgeState` resource, which utility scorers can then query.
///
/// Runs in `SimulationSet::AIDecision` (same phase as BT evaluation).
pub fn bridge_bt_to_utility_system(
    mut bridge: ResMut<AIBridgeState>,
    query: Query<(Entity, &BehaviorTree, &AIController, &AIIntegrationConfig)>,
) {
    bridge.bt_goals.clear();
    bridge.bt_strategies.clear();
    bridge.bt_running.clear();
    bridge.bt_last_action.clear();

    for (entity, bt, controller, config) in query.iter() {
        if !config.bridge_blackboard {
            continue;
        }

        if let Some(goal) = controller.current_goal {
            bridge.bt_goals.insert(entity, goal);
        }
        bridge.bt_strategies.insert(entity, controller.strategy);
        bridge
            .bt_running
            .insert(entity, bt.last_state == BehaviorState::Running);

        // Extract a descriptive action name from the last BT node output
        let action_name = match &bt.root {
            BehaviorNode::Action(action) => format!("{:?}", action),
            BehaviorNode::Sequence(_) => "Sequence".into(),
            BehaviorNode::Selector(_) => "Selector".into(),
            _ => "Unknown".into(),
        };
        bridge.bt_last_action.insert(entity, action_name);
    }
}

/// Phase 2: Resolve conflicting decisions between BT and Utility.
///
/// For each entity with `AIIntegrationConfig` + `UtilityThinker`:
/// - Reads the Utility thinker's current action
/// - Reads the BT bridge state (if available)
/// - Resolves according to the priority scheme
/// - Emits a SINGLE set of commands (no double-issuing)
///
/// Runs in `SimulationSet::AICommandIssuing` (same phase as command emission).
#[allow(clippy::too_many_arguments)]
pub fn resolve_ai_decision_system(
    mut commands: Commands,
    bridge: Res<AIBridgeState>,
    query: Query<(
        Entity,
        &UtilityThinker,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
        &AIIntegrationConfig,
        Option<&AIController>,
    )>,
    mut move_events: EventWriter<MoveOrder>,
    mut attack_events: EventWriter<AttackOrder>,
    mut build_events: EventWriter<BuildOrder>,
    mut recruit_events: EventWriter<RecruitOrder>,
) {
    for (entity, thinker, health, _stats, pos, faction_id, config, controller_opt) in query.iter() {
        // Get the BT goal (if any)
        let bt_goal = bridge.bt_goals.get(&entity).copied();
        let _bt_strategy = bridge.bt_strategies.get(&entity).copied();

        // Get the Utility's current action
        let Some(ref utility_action) = thinker.current_action else {
            // Utility hasn't decided yet — nothing to resolve
            continue;
        };

        // ── Check urgency — critical health overrides BT ────────────────
        let health_ratio = health.current / health.max;
        let is_critical = health_ratio < 0.3;

        // ── Resolve according to priority scheme ──────────────────────────
        let should_act = if is_critical && matches!(config.priority, AIPriority::BTStrategic) {
            // Critical health: Utility survival action overrides BT strategy
            true
        } else {
            match config.priority {
                AIPriority::UtilityTactical => {
                    // Utility always wins — it's the tactical override
                    true
                }
                AIPriority::BTStrategic => {
                    // Check if Utility action aligns with BT goal
                    match bt_goal {
                        Some(goal) => action_matches_goal(utility_action, &goal),
                        None => true, // No BT goal — let Utility decide
                    }
                }
            }
        };

        if !should_act {
            continue;
        }

        // ── Execute the resolved action ──────────────────────────────────
        execute_integrated_action(
            utility_action,
            entity,
            pos,
            faction_id,
            &thinker.considerations,
            controller_opt,
            &mut move_events,
            &mut attack_events,
            &mut build_events,
            &mut recruit_events,
        );

        // ── Mark action as handled ──────────────────────────────────────
        commands.entity(entity).insert(IntegrationActionExecuted {
            action: utility_action.clone(),
            tick: 0,
        });
    }
}

/// Component marking that the AI integration has emitted a command for this
/// entity this frame. Prevents double-emission if multiple systems read it.
#[derive(Component, Clone, Debug)]
pub struct IntegrationActionExecuted {
    pub action: UtilityAction,
    pub tick: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
//  HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns `true` if the Utility action aligns with the given BT goal.
fn action_matches_goal(action: &UtilityAction, goal: &AIGoal) -> bool {
    match (action, goal) {
        // Attack goals
        (UtilityAction::AttackNearest, AIGoal::AttackPlayer)
        | (UtilityAction::AttackEntity(_), AIGoal::AttackPlayer)
        | (UtilityAction::Flank(_), AIGoal::AttackPlayer) => true,

        // Economy goals
        (UtilityAction::Gather(_), AIGoal::BuildEconomy)
        | (UtilityAction::Gather(_), AIGoal::GatherResources) => true,
        (UtilityAction::Build(_, _), AIGoal::BuildEconomy)
        | (UtilityAction::Build(_, _), AIGoal::ExpandTerritory) => true,

        // Defensive goals
        (UtilityAction::Defend(_), AIGoal::DefendBase)
        | (UtilityAction::Patrol(_, _), AIGoal::DefendBase) => true,
        (UtilityAction::Retreat(_), AIGoal::DefendBase) => true,

        // Recruitment goals
        (UtilityAction::Recruit(_, _), AIGoal::TrainArmy) => true,

        // Research goals
        (UtilityAction::Research(_), AIGoal::ResearchTech) => true,

        // Fallback: block by default; only allow explicitly matched combinations
        _ => false,
    }
}

/// Execute an integration‑resolved action, emitting the appropriate Bevy event.
/// Mirrors `execute_utility_action` in `utility_ai.rs` but adds BT context
/// for smarter targeting.
#[allow(clippy::too_many_arguments)]
fn execute_integrated_action(
    action: &UtilityAction,
    entity: Entity,
    _pos: &WorldPosition,
    _faction_id: &FactionId,
    _considerations: &[Consideration],
    controller: Option<&AIController>,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) {
    // Use the BT controller's strategy to bias targeting if available
    let is_aggressive = controller
        .map(|c| matches!(c.strategy, AIStrategy::Rush | AIStrategy::Balanced))
        .unwrap_or(true);

    match action {
        UtilityAction::AttackNearest => {
            // The actual target-finding happens in the utility_thinker_system;
            // here we just need to emit a general AttackOrder if the thinker
            // already selected AttackNearest but needs to be interpreted.
            // For now, we emit a generic attack order — the combat system
            // resolves exact targeting.
            attack_events.send(AttackOrder {
                entity,
                target: Entity::PLACEHOLDER,
                aggressive: is_aggressive,
            });
        }
        UtilityAction::AttackEntity(target) => {
            attack_events.send(AttackOrder {
                entity,
                target: *target,
                aggressive: is_aggressive,
            });
        }
        UtilityAction::Retreat(target) => {
            // If BT goal is AttackPlayer, retreat is only allowed in critical
            // situations (handled by the integration logic above).
            move_events.send(MoveOrder {
                entity,
                target: *target,
                formation: None,
            });
        }
        UtilityAction::Gather(resource) => {
            let gather_pos = match resource {
                AiUtilityResource::Food => GridPos(IVec2::new(20, 20)),
                AiUtilityResource::Wood => GridPos(IVec2::new(25, 15)),
                AiUtilityResource::Stone => GridPos(IVec2::new(15, 25)),
                AiUtilityResource::Gold => GridPos(IVec2::new(10, 10)),
            };
            move_events.send(MoveOrder {
                entity,
                target: gather_pos,
                formation: None,
            });
        }
        UtilityAction::Build(building_type, pos) => {
            build_events.send(BuildOrder {
                entity,
                building_type: *building_type,
                position: *pos,
            });
        }
        UtilityAction::Patrol(a, _b) => {
            move_events.send(MoveOrder {
                entity,
                target: *a,
                formation: None,
            });
        }
        UtilityAction::Defend(pos) => {
            move_events.send(MoveOrder {
                entity,
                target: *pos,
                formation: None,
            });
        }
        UtilityAction::Recruit(archetype, count) => {
            recruit_events.send(RecruitOrder {
                building: Entity::PLACEHOLDER,
                archetype: *archetype,
                count: *count,
            });
        }
        UtilityAction::Research(_tech) => {}
        UtilityAction::Wait(_) => {}
        UtilityAction::Flank(target) => {
            attack_events.send(AttackOrder {
                entity,
                target: *target,
                aggressive: true,
            });
        }
        UtilityAction::Custom(_) => {}
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<AIBridgeState>()
            .add_event::<MoveOrder>()
            .add_event::<AttackOrder>()
            .add_event::<BuildOrder>()
            .add_event::<RecruitOrder>();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
        app
    }

    fn advance(app: &mut App, ticks: u32) {
        for _ in 0..ticks {
            app.update();
        }
    }

    // ── Bridge System Tests ─────────────────────────────────────────────

    #[test]
    fn test_bridge_copies_bt_goal() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                BehaviorTree {
                    root: BehaviorNode::Succeeder(Box::new(BehaviorNode::Action(
                        BehaviorAction::Wait(0),
                    ))),
                    execution_path: Vec::new(),
                    blackboard: AIDataBlackboard::default(),
                    last_state: BehaviorState::Success,
                },
                AIController {
                    current_goal: Some(AIGoal::AttackPlayer),
                    strategy: AIStrategy::Rush,
                    personality: AIPersonality::Balanced,
                    difficulty: AIDifficulty::Normal,
                    decision_interval: 1.0,
                    last_decision_time: 0.0,
                },
                AIIntegrationConfig::default(),
            ))
            .id();

        app.add_systems(Update, bridge_bt_to_utility_system);
        advance(&mut app, 1);

        let bridge = app.world().resource::<AIBridgeState>();
        assert_eq!(bridge.bt_goals.get(&entity), Some(&AIGoal::AttackPlayer));
        assert_eq!(bridge.bt_strategies.get(&entity), Some(&AIStrategy::Rush));
    }

    #[test]
    fn test_bridge_skips_when_disabled() {
        let mut app = test_app();
        app.world_mut().spawn((
            BehaviorTree {
                root: BehaviorNode::Succeeder(Box::new(BehaviorNode::Action(
                    BehaviorAction::Wait(0),
                ))),
                execution_path: Vec::new(),
                blackboard: AIDataBlackboard::default(),
                last_state: BehaviorState::Success,
            },
            AIController {
                current_goal: Some(AIGoal::AttackPlayer),
                strategy: AIStrategy::Rush,
                personality: AIPersonality::Balanced,
                difficulty: AIDifficulty::Normal,
                decision_interval: 1.0,
                last_decision_time: 0.0,
            },
            AIIntegrationConfig {
                bridge_blackboard: false, // disabled
                ..Default::default()
            },
        ));

        app.add_systems(Update, bridge_bt_to_utility_system);
        advance(&mut app, 1);

        let bridge = app.world().resource::<AIBridgeState>();
        assert!(
            bridge.bt_goals.is_empty(),
            "Should skip when bridge disabled"
        );
    }

    // ── Decision Resolution Tests ───────────────────────────────────────

    #[test]
    fn test_utility_tactical_wins_over_bt() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                UtilityThinker {
                    current_action: Some(UtilityAction::AttackNearest),
                    ..Default::default()
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 5.0,
                    armor_type: ArmorType::Light,
                },
                Stats::default(),
                WorldPosition::default(),
                FactionId::Byzantine,
                AIIntegrationConfig {
                    priority: AIPriority::UtilityTactical,
                    ..Default::default()
                },
            ))
            .id();

        // Seed the bridge with a conflicting BT goal
        app.world_mut().resource_mut::<AIBridgeState>().bt_goals = {
            let mut m = std::collections::HashMap::new();
            m.insert(entity, AIGoal::BuildEconomy); // BT says build
            m
        };

        app.add_systems(Update, resolve_ai_decision_system);
        advance(&mut app, 1);

        // In UtilityTactical mode, the Utility action should be emitted
        let reader = app.world().resource::<Events<AttackOrder>>();
        let mut cursor = reader.get_cursor();
        let attacks: Vec<_> = cursor.read(reader).collect();
        assert!(
            !attacks.is_empty(),
            "UtilityTactical should emit AttackOrder"
        );
    }

    #[test]
    fn test_bt_strategic_blocks_mismatched_action() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                UtilityThinker {
                    current_action: Some(UtilityAction::Gather(AiUtilityResource::Gold)),
                    ..Default::default()
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 5.0,
                    armor_type: ArmorType::Light,
                },
                Stats::default(),
                WorldPosition::default(),
                FactionId::Byzantine,
                AIIntegrationConfig {
                    priority: AIPriority::BTStrategic,
                    ..Default::default()
                },
            ))
            .id();

        // BT says attack — Utility says gather — should block goid
        app.world_mut().resource_mut::<AIBridgeState>().bt_goals = {
            let mut m = std::collections::HashMap::new();
            m.insert(entity, AIGoal::AttackPlayer);
            m
        };

        app.add_systems(Update, resolve_ai_decision_system);
        advance(&mut app, 1);

        // Gather does not match AttackPlayer goal → should NOT emit MoveOrder
        let reader = app.world().resource::<Events<MoveOrder>>();
        let mut cursor = reader.get_cursor();
        let moves: Vec<_> = cursor.read(reader).collect();
        assert!(
            moves.is_empty(),
            "BTStrategic should block mismatched gather action"
        );
    }

    #[test]
    fn test_bt_strategic_allows_matched_action() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                UtilityThinker {
                    current_action: Some(UtilityAction::AttackNearest),
                    ..Default::default()
                },
                Health {
                    current: 50.0,
                    max: 100.0,
                    armor: 5.0,
                    armor_type: ArmorType::Light,
                },
                Stats::default(),
                WorldPosition::default(),
                FactionId::Byzantine,
                AIIntegrationConfig {
                    priority: AIPriority::BTStrategic,
                    ..Default::default()
                },
            ))
            .id();

        // BT says attack — Utility says attack — should match
        app.world_mut().resource_mut::<AIBridgeState>().bt_goals = {
            let mut m = std::collections::HashMap::new();
            m.insert(entity, AIGoal::AttackPlayer);
            m
        };

        app.add_systems(Update, resolve_ai_decision_system);
        advance(&mut app, 1);

        let reader = app.world().resource::<Events<AttackOrder>>();
        let mut cursor = reader.get_cursor();
        let attacks: Vec<_> = cursor.read(reader).collect();
        assert!(
            !attacks.is_empty(),
            "Matched action should emit AttackOrder"
        );
    }

    #[test]
    fn test_critical_health_overrides_bt_strategic() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                UtilityThinker {
                    current_action: Some(UtilityAction::Retreat(GridPos(IVec2::new(0, 0)))),
                    ..Default::default()
                },
                Health {
                    current: 10.0, // 10% health → critical
                    max: 100.0,
                    armor: 5.0,
                    armor_type: ArmorType::Light,
                },
                Stats::default(),
                WorldPosition::default(),
                FactionId::Byzantine,
                AIIntegrationConfig {
                    priority: AIPriority::BTStrategic,
                    ..Default::default()
                },
            ))
            .id();

        // BT says attack — but health is critical → retreat should win
        app.world_mut().resource_mut::<AIBridgeState>().bt_goals = {
            let mut m = std::collections::HashMap::new();
            m.insert(entity, AIGoal::AttackPlayer);
            m
        };

        app.add_systems(Update, resolve_ai_decision_system);
        advance(&mut app, 1);

        let reader = app.world().resource::<Events<MoveOrder>>();
        let mut cursor = reader.get_cursor();
        let moves: Vec<_> = cursor.read(reader).collect();
        assert!(
            !moves.is_empty(),
            "Critical health should override BT strategy → emit MoveOrder"
        );
    }

    #[test]
    fn test_integration_action_executed_marker() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((
                UtilityThinker {
                    current_action: Some(UtilityAction::AttackNearest),
                    ..Default::default()
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                    armor: 5.0,
                    armor_type: ArmorType::Light,
                },
                Stats::default(),
                WorldPosition::default(),
                FactionId::Byzantine,
                AIIntegrationConfig::default(),
            ))
            .id();

        app.world_mut().resource_mut::<AIBridgeState>().bt_goals = {
            let mut m = std::collections::HashMap::new();
            m.insert(entity, AIGoal::AttackPlayer);
            m
        };

        app.add_systems(Update, resolve_ai_decision_system);
        advance(&mut app, 1);

        // Verify the marker component was inserted
        let mut marker_query = app.world_mut().query::<&IntegrationActionExecuted>();
        let markers: Vec<&IntegrationActionExecuted> = marker_query.iter(app.world()).collect();
        assert_eq!(markers.len(), 1, "Should have exactly one marker");
        assert_eq!(markers[0].action, UtilityAction::AttackNearest);
    }

    // ── action_matches_goal Tests ───────────────────────────────────────

    #[test]
    fn test_action_matches_goal_for_attack() {
        assert!(action_matches_goal(
            &UtilityAction::AttackNearest,
            &AIGoal::AttackPlayer
        ));
        assert!(action_matches_goal(
            &UtilityAction::AttackEntity(Entity::from_raw(42)),
            &AIGoal::AttackPlayer
        ));
        assert!(action_matches_goal(
            &UtilityAction::Flank(Entity::from_raw(42)),
            &AIGoal::AttackPlayer
        ));
    }

    #[test]
    fn test_action_matches_goal_for_economy() {
        assert!(action_matches_goal(
            &UtilityAction::Gather(AiUtilityResource::Gold),
            &AIGoal::BuildEconomy
        ));
        assert!(action_matches_goal(
            &UtilityAction::Build(BuildingType::Market, GridPos(IVec2::new(0, 0))),
            &AIGoal::BuildEconomy
        ));
    }

    #[test]
    fn test_action_matches_goal_for_defense() {
        assert!(action_matches_goal(
            &UtilityAction::Defend(GridPos(IVec2::new(0, 0))),
            &AIGoal::DefendBase
        ));
        assert!(action_matches_goal(
            &UtilityAction::Patrol(GridPos(IVec2::new(0, 0)), GridPos(IVec2::new(5, 5))),
            &AIGoal::DefendBase
        ));
    }

    #[test]
    fn test_ai_integration_config_defaults() {
        let config = AIIntegrationConfig::default();
        assert_eq!(config.priority, AIPriority::UtilityTactical);
        assert!(config.bridge_blackboard);
        assert!((config.bt_influence - 0.3).abs() < f32::EPSILON);
    }
}
