use bevy::prelude::*;
use kurulus_core::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
//  UTILITY AI (big-brain pattern) — scorer evaluation + action selection
//
//  The utility_thinker_system iterates all entities with a UtilityThinker,
//  evaluates every Consideration's ScorerConfig against the current game world,
//  and selects the action with the highest weighted score.
//
//  Integration: runs in SimulationSet::AIDecision, alongside the BT evaluator.
//  Entities should have EITHER a UtilityThinker OR a BehaviorTree, not both.
// ═══════════════════════════════════════════════════════════════════════════════

/// Main Utility AI evaluation system.
///
/// For each entity with a `UtilityThinker`:
/// 1. Check if enough time has passed since the last evaluation.
/// 2. If a "sticky" action is still running, skip evaluation.
/// 3. Evaluate every consideration's scorer using current game state.
/// 4. Select the action with the highest weighted score.
/// 5. Execute the selected action via event writers.
#[allow(clippy::too_many_arguments)]
pub fn utility_thinker_system(
    game_time: Res<GameTime>,
    grid: Res<SpatialGrid>,
    stockpile: Res<ResourceStockpile>,
    _diff_config: Res<AIDifficultyConfig>,
    _faction_config: Res<FactionConfig>,
    mut thinker_query: Query<(
        Entity,
        &mut UtilityThinker,
        &Health,
        &Stats,
        &WorldPosition,
        &FactionId,
    )>,
    unit_query: Query<(Entity, &WorldPosition, &Health, &FactionId, &Stats)>,
    building_query: Query<(&Building, &WorldPosition, &FactionId)>,
    mut move_events: EventWriter<MoveOrder>,
    mut attack_events: EventWriter<AttackOrder>,
    mut build_events: EventWriter<BuildOrder>,
    mut recruit_events: EventWriter<RecruitOrder>,
) {
    // Snapshot units for borrow-free lookup
    let unit_snapshot: Vec<(Entity, GridPos, Health, FactionId, Stats)> = unit_query
        .iter()
        .map(|(e, p, h, f, s)| (e, p.grid, h.clone(), *f, s.clone()))
        .collect();

    for (entity, mut thinker, health, stats, pos, faction_id) in thinker_query.iter_mut() {
        let current_time = game_time.total_seconds as f32;

        // If sticky action is still running, skip re-evaluation
        if thinker.sticky && thinker.action_running {
            // Consider the action "done" after one evaluation cycle; the next
            // evaluation will pick a new action. In a full implementation
            // this would track completion via event confirmation.
            thinker.action_running = false;
            continue;
        }

        // Cooldown check
        if current_time - thinker.last_evaluation < thinker.evaluation_interval {
            continue;
        }

        // Evaluate all considerations and pick the best
        let mut best_score = f32::NEG_INFINITY;
        let mut best_action: Option<UtilityAction> = None;

        for consideration in &thinker.considerations {
            let raw_score = evaluate_scorer(
                &consideration.scorer,
                entity,
                pos,
                health,
                stats,
                faction_id,
                &stockpile,
                &grid,
                &unit_snapshot,
                &building_query,
            );
            let weighted_score = raw_score * consideration.weight;

            if weighted_score > best_score {
                best_score = weighted_score;
                best_action = Some(consideration.action.clone());
            }
        }

        if let Some(action) = best_action {
            // Don't re-issue the same action if it's already running
            if thinker.action_running
                && thinker
                    .current_action
                    .as_ref()
                    .is_some_and(|ca| ca == &action)
            {
                thinker.last_evaluation = current_time;
                continue;
            }

            thinker.current_action = Some(action.clone());
            thinker.action_running = true;
            thinker.action_progress = 0.0;
            thinker.last_evaluation = current_time;

            execute_utility_action(
                &action,
                entity,
                pos,
                faction_id,
                &unit_snapshot,
                &grid,
                &stockpile,
                &mut move_events,
                &mut attack_events,
                &mut build_events,
                &mut recruit_events,
            );
        } else {
            thinker.last_evaluation = current_time;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
//  SCORER EVALUATION
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(clippy::too_many_arguments)]
fn evaluate_scorer(
    scorer: &ScorerConfig,
    entity: Entity,
    pos: &WorldPosition,
    health: &Health,
    stats: &Stats,
    faction_id: &FactionId,
    stockpile: &ResourceStockpile,
    grid: &SpatialGrid,
    unit_snapshot: &[(Entity, GridPos, Health, FactionId, Stats)],
    building_query: &Query<(&Building, &WorldPosition, &FactionId)>,
) -> f32 {
    match scorer {
        ScorerConfig::TargetThreat { range, falloff } => score_target_threat(
            entity,
            pos,
            faction_id,
            grid,
            unit_snapshot,
            *range,
            *falloff,
        ),
        ScorerConfig::LowHealth { threshold } => {
            let ratio = health.current / health.max;
            if ratio <= *threshold {
                // Score increases as health drops: 0.0 at threshold, 1.0 at 0 HP
                ((threshold - ratio) / threshold).clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        ScorerConfig::ResourceNeed {
            resource,
            threshold,
        } => {
            let current = match resource {
                AiUtilityResource::Food => stockpile.food,
                AiUtilityResource::Wood => stockpile.wood,
                AiUtilityResource::Stone => stockpile.stone,
                AiUtilityResource::Gold => stockpile.gold,
            };
            let thr = *threshold;
            if current <= thr {
                // Score increases as stockpile drops below threshold
                ((thr - current) / thr).clamp(0.0, 1.0)
            } else {
                0.0
            }
        }
        ScorerConfig::DistanceToEnemy { max_range } => {
            score_distance_to_enemy(pos, faction_id, grid, unit_snapshot, *max_range)
        }
        ScorerConfig::MoraleState => {
            // Low morale → high score (need to retreat / recover)
            1.0 - (stats.morale / stats.morale_max.clamp(1.0, f32::MAX)).clamp(0.0, 1.0)
        }
        ScorerConfig::ForceRatio {
            advantage_threshold,
        } => score_force_ratio(
            entity,
            pos,
            faction_id,
            grid,
            unit_snapshot,
            *advantage_threshold,
        ),
        ScorerConfig::HasBuilding { building } => {
            if has_building(building, *faction_id, building_query) {
                1.0
            } else {
                0.0
            }
        }
        ScorerConfig::IsUnderAttack => {
            // Check if any nearby enemy is in attack range
            let nearby = grid.entities_in_range(pos.grid, stats.vision_range);
            for &candidate in &nearby {
                if candidate == entity {
                    continue;
                }
                for (ue, _upos, _uh, uf, _us) in unit_snapshot.iter() {
                    if *ue == candidate && *uf != *faction_id {
                        return 1.0; // Enemy nearby → under attack
                    }
                }
            }
            0.0
        }
        ScorerConfig::PopulationCapReached => {
            if stockpile.population >= stockpile.max_population {
                1.0
            } else {
                0.0
            }
        }
        ScorerConfig::Constant(val) => val.clamp(0.0, 1.0),
        ScorerConfig::Custom(_) => {
            // Custom scorers can be expanded by checking a resource or blackboard
            0.0
        }
    }
}

fn score_target_threat(
    entity: Entity,
    pos: &WorldPosition,
    faction_id: &FactionId,
    grid: &SpatialGrid,
    unit_snapshot: &[(Entity, GridPos, Health, FactionId, Stats)],
    range: f32,
    falloff: f32,
) -> f32 {
    let nearby = grid.entities_in_range(pos.grid, range);
    let mut max_threat = 0.0f32;

    for &candidate in &nearby {
        if candidate == entity {
            continue;
        }
        for (ue, upos, uh, uf, us) in unit_snapshot.iter() {
            if *ue != candidate || *uf == *faction_id {
                continue;
            }
            let dist = (pos.grid.0 - upos.0).as_vec2().length();
            let distance_factor = if dist < 1.0 {
                1.0
            } else if falloff > 0.0 {
                1.0 - (dist / range).powf(falloff)
            } else {
                1.0 - (dist / range)
            };
            let health_ratio = uh.current / uh.max;
            let damage_factor = (us.attack_damage / 100.0).min(1.0);
            let threat = distance_factor * 0.5 + health_ratio * 0.3 + damage_factor * 0.2;
            max_threat = max_threat.max(threat);
        }
    }
    max_threat.clamp(0.0, 1.0)
}

fn score_distance_to_enemy(
    pos: &WorldPosition,
    faction_id: &FactionId,
    grid: &SpatialGrid,
    unit_snapshot: &[(Entity, GridPos, Health, FactionId, Stats)],
    max_range: f32,
) -> f32 {
    let nearby = grid.entities_in_range(pos.grid, max_range);
    let mut min_dist = max_range;

    for &candidate in &nearby {
        for (ue, upos, _uh, uf, _us) in unit_snapshot.iter() {
            if *ue != candidate || *uf == *faction_id {
                continue;
            }
            let dist = (pos.grid.0 - upos.0).as_vec2().length();
            min_dist = min_dist.min(dist);
        }
    }

    if min_dist >= max_range {
        0.0
    } else {
        // Closer enemies = higher score
        1.0 - (min_dist / max_range)
    }
}

fn score_force_ratio(
    entity: Entity,
    pos: &WorldPosition,
    faction_id: &FactionId,
    grid: &SpatialGrid,
    unit_snapshot: &[(Entity, GridPos, Health, FactionId, Stats)],
    advantage_threshold: f32,
) -> f32 {
    let nearby = grid.entities_in_range(pos.grid, 20.0);
    let mut allies = 0;
    let mut enemies = 0;

    for &candidate in &nearby {
        if candidate == entity {
            allies += 1;
            continue;
        }
        for (ue, _upos, _uh, uf, _us) in unit_snapshot.iter() {
            if *ue != candidate {
                continue;
            }
            if *uf == *faction_id {
                allies += 1;
            } else {
                enemies += 1;
            }
        }
    }

    let ratio = if enemies == 0 {
        advantage_threshold + 1.0 // way ahead
    } else {
        allies as f32 / enemies as f32
    };

    // Score 0-1: higher ratio = higher score (for attack actions)
    (ratio / advantage_threshold).clamp(0.0, 1.0)
}

fn has_building(
    building_type: &BuildingType,
    faction_id: FactionId,
    buildings: &Query<(&Building, &WorldPosition, &FactionId)>,
) -> bool {
    for (building, _pos, b_faction) in buildings.iter() {
        if building.building_type == *building_type && *b_faction == faction_id {
            return true;
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════════
//  ACTION EXECUTION
// ═══════════════════════════════════════════════════════════════════════════════

#[allow(clippy::too_many_arguments)]
fn execute_utility_action(
    action: &UtilityAction,
    entity: Entity,
    _pos: &WorldPosition,
    faction_id: &FactionId,
    unit_snapshot: &[(Entity, GridPos, Health, FactionId, Stats)],
    _grid: &SpatialGrid,
    _stockpile: &ResourceStockpile,
    move_events: &mut EventWriter<MoveOrder>,
    attack_events: &mut EventWriter<AttackOrder>,
    build_events: &mut EventWriter<BuildOrder>,
    recruit_events: &mut EventWriter<RecruitOrder>,
) {
    match action {
        UtilityAction::AttackNearest => {
            // Find nearest enemy by checking unit snapshot for enemies
            let my_pos = _pos.grid;
            let mut nearest: Option<(Entity, i32)> = None;
            for (ue, upos, _uh, uf, _us) in unit_snapshot.iter() {
                if *uf != *faction_id && *ue != entity {
                    let dist_sq = my_pos.0.distance_squared(upos.0);
                    match nearest {
                        None => nearest = Some((*ue, dist_sq)),
                        Some((_, best_dist)) if dist_sq < best_dist => {
                            nearest = Some((*ue, dist_sq));
                        }
                        _ => {}
                    }
                }
            }
            if let Some((target, _)) = nearest {
                attack_events.send(AttackOrder {
                    entity,
                    target,
                    aggressive: true,
                });
            }
        }
        UtilityAction::AttackEntity(target) => {
            attack_events.send(AttackOrder {
                entity,
                target: *target,
                aggressive: true,
            });
        }
        UtilityAction::Retreat(pos) => {
            move_events.send(MoveOrder {
                entity,
                target: *pos,
                formation: None,
            });
        }
        UtilityAction::Gather(resource) => {
            // Find gathering point based on resource type
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
        UtilityAction::Research(_tech) => {
            // Placeholder — would send a ResearchOrder event
        }
        UtilityAction::Wait(_duration) => {
            // Action progress will track the wait duration
        }
        UtilityAction::Flank(target) => {
            attack_events.send(AttackOrder {
                entity,
                target: *target,
                aggressive: true,
            });
        }
        UtilityAction::Custom(_) => {
            // Extension point for custom action logic
        }
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
            .init_resource::<GameTime>()
            .init_resource::<SpatialGrid>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<AIDifficultyConfig>()
            .init_resource::<FactionConfig>()
            .add_event::<MoveOrder>()
            .add_event::<AttackOrder>()
            .add_event::<BuildOrder>()
            .add_event::<RecruitOrder>();
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 60.0,
        )));
        app
    }

    /// Helper to advance the app by N fixed-update ticks.
    fn advance(app: &mut App, ticks: u32) {
        for _ in 0..ticks {
            app.update();
        }
    }

    #[test]
    fn test_utility_thinker_defaults() {
        let thinker = UtilityThinker::default();
        assert!(thinker.considerations.is_empty());
        assert!((thinker.evaluation_interval - 0.5).abs() < f32::EPSILON);
        assert!(thinker.sticky);
        assert!(thinker.current_action.is_none());
    }

    #[test]
    fn test_low_health_scorer() {
        let mut app = test_app();

        // Spawn an entity with UtilityThinker, low health
        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![Consideration {
                    name: "retreat_if_hurt".into(),
                    scorer: ScorerConfig::LowHealth { threshold: 0.5 },
                    action: UtilityAction::Retreat(GridPos(IVec2::new(0, 0))),
                    weight: 1.0,
                }],
                ..default()
            },
            Health {
                current: 20.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats::default(),
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        // Run the system
        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60); // 1 second at 60fps

        // After evaluation interval, the LowHealth scorer should fire
        let mut query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = query.iter(app.world()).collect();
        assert!(!thinkers.is_empty());
        // The thinker should have selected an action (Retreat due to low health)
        let thinker = thinkers[0];
        assert!(thinker.current_action.is_some());
        assert_eq!(
            thinker.current_action.as_ref().unwrap(),
            &UtilityAction::Retreat(GridPos(IVec2::new(0, 0)))
        );
    }

    #[test]
    fn test_constant_scorer_highest_wins() {
        let mut app = test_app();

        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![
                    Consideration {
                        name: "always_0.2".into(),
                        scorer: ScorerConfig::Constant(0.2),
                        action: UtilityAction::Wait(1.0),
                        weight: 1.0,
                    },
                    Consideration {
                        name: "always_0.9".into(),
                        scorer: ScorerConfig::Constant(0.9),
                        action: UtilityAction::AttackNearest,
                        weight: 1.0,
                    },
                ],
                ..default()
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats::default(),
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60);

        let mut think_query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = think_query.iter(app.world()).collect();
        assert!(!thinkers.is_empty());
        let thinker = thinkers[0];
        // The 0.9 constant should win over 0.2
        assert_eq!(
            thinker.current_action.as_ref().unwrap(),
            &UtilityAction::AttackNearest
        );
    }

    #[test]
    fn test_resource_need_scorer() {
        let mut app = test_app();

        // Set gold to very low
        app.world_mut().resource_mut::<ResourceStockpile>().gold = 5.0;

        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![Consideration {
                    name: "need_gold".into(),
                    scorer: ScorerConfig::ResourceNeed {
                        resource: AiUtilityResource::Gold,
                        threshold: 50.0,
                    },
                    action: UtilityAction::Gather(AiUtilityResource::Gold),
                    weight: 1.0,
                }],
                ..default()
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats::default(),
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60);

        let mut think_query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = think_query.iter(app.world()).collect();
        assert!(!thinkers.is_empty());
        let thinker = thinkers[0];
        assert_eq!(
            thinker.current_action.as_ref().unwrap(),
            &UtilityAction::Gather(AiUtilityResource::Gold)
        );
    }

    #[test]
    fn test_thinker_evaluation_interval() {
        let mut app = test_app();

        // Must set last_evaluation to current game time so the interval check works.
        let initial_time = app
            .world()
            .get_resource::<GameTime>()
            .map(|gt| gt.total_seconds as f32)
            .unwrap_or(0.0);
        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![Consideration {
                    name: "always_1.0".into(),
                    scorer: ScorerConfig::Constant(1.0),
                    action: UtilityAction::Wait(1.0),
                    weight: 1.0,
                }],
                evaluation_interval: 10.0, // Very long interval
                last_evaluation: initial_time,
                ..default()
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats::default(),
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60); // 1 second, not enough for 10s interval

        let mut think_query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = think_query.iter(app.world()).collect();
        let thinker = thinkers[0];
        // Should NOT have evaluated yet because interval is 10s
        assert!(thinker.current_action.is_none());
    }

    #[test]
    fn test_population_cap_scorer() {
        let mut app = test_app();

        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .population = 50;
        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .max_population = 50;

        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![Consideration {
                    name: "pop_capped".into(),
                    scorer: ScorerConfig::PopulationCapReached,
                    action: UtilityAction::Wait(1.0),
                    weight: 1.0,
                }],
                ..default()
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats::default(),
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60);

        let mut think_query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = think_query.iter(app.world()).collect();
        let thinker = thinkers[0];
        assert_eq!(
            thinker.current_action.as_ref().unwrap(),
            &UtilityAction::Wait(1.0)
        );
    }

    #[test]
    fn test_morale_scorer_low_morale() {
        let mut app = test_app();

        app.world_mut().spawn((
            UtilityThinker {
                considerations: vec![Consideration {
                    name: "low_morale".into(),
                    scorer: ScorerConfig::MoraleState,
                    action: UtilityAction::Retreat(GridPos(IVec2::new(10, 10))),
                    weight: 1.0,
                }],
                ..default()
            },
            Health {
                current: 100.0,
                max: 100.0,
                armor: 0.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                morale: 10.0,
                morale_max: 100.0,
                ..default()
            },
            WorldPosition::default(),
            FactionId::Kayi,
        ));

        app.add_systems(FixedUpdate, utility_thinker_system);
        advance(&mut app, 60);

        let mut think_query = app.world_mut().query::<&UtilityThinker>();
        let thinkers: Vec<&UtilityThinker> = think_query.iter(app.world()).collect();
        let thinker = thinkers[0];
        assert_eq!(
            thinker.current_action.as_ref().unwrap(),
            &UtilityAction::Retreat(GridPos(IVec2::new(10, 10)))
        );
    }
}
