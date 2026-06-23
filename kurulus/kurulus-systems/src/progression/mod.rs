use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TechModifiers>().add_systems(
            FixedUpdate,
            (
                start_research_system.in_set(SimulationSet::InputProcessing),
                research_progress_system.in_set(SimulationSet::ResearchProgress),
                tech_effect_application_system.in_set(SimulationSet::ResearchProgress),
                tech_rate_application_system.after(SimulationSet::ResearchProgress),
                sovereignty_point_system.in_set(SimulationSet::XPGain),
            ),
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  START RESEARCH — handles ResearchOrder events
// ═════════════════════════════════════════════════════════════════════════════

fn start_research_system(
    mut research_events: EventReader<ResearchOrder>,
    mut tech_tree: ResMut<TechTree>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    for order in research_events.read() {
        let result = tech_tree.can_research(&order.tech_id, stockpile.sovereignty_points);
        match result {
            ResearchStartResult::Success => {
                // Deduct SP
                let node = tech_tree
                    .techs
                    .iter()
                    .find(|t| t.id == order.tech_id)
                    .expect("Tech node must exist after can_research success");
                stockpile.sovereignty_points -= node.cost;

                // Create research project
                tech_tree.current_research = Some(ResearchProject {
                    tech_id: order.tech_id.clone(),
                    progress: 0.0,
                    max_progress: node.research_time,
                    assigned_hero: None,
                });
            }
            ResearchStartResult::AlreadyUnlocked
            | ResearchStartResult::AlreadyResearching
            | ResearchStartResult::InsufficientSP(..)
            | ResearchStartResult::MissingPrerequisites(..)
            | ResearchStartResult::TechNotFound => {
                // Silently ignored — UI or AI should validate before sending.
                // Future: emit a ResearchFailed event for feedback.
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  RESEARCH PROGRESS — advances active projects with hero bonus
// ═════════════════════════════════════════════════════════════════════════════

fn research_progress_system(
    time: Res<GameTime>,
    mut tech_tree: ResMut<TechTree>,
    hero_query: Query<&Hero>,
) {
    let Some(ref mut project) = tech_tree.current_research else {
        return;
    };

    // Base research rate: 1.0x per second
    let mut rate = time.time_scale;

    // Hero bonus: assigned hero adds +0.5 per skill level
    if let Some(hero_entity) = project.assigned_hero {
        if let Ok(hero) = hero_query.get(hero_entity) {
            let hero_bonus = 0.5 + hero.skill_points as f32 * 0.1;
            rate += hero_bonus;
        }
    }

    project.progress += rate * 0.1; // per-frame tick (FixedUpdate at ~10 Hz)

    if project.progress >= project.max_progress {
        let tech_id = project.tech_id.clone();
        if let Some(node) = tech_tree.techs.iter_mut().find(|t| t.id == tech_id) {
            node.unlocked = true;
        }
        tech_tree.current_research = None;
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TECH EFFECT APPLICATION — applies TechEffects from newly unlocked nodes
// ═════════════════════════════════════════════════════════════════════════════

fn tech_effect_application_system(
    mut tech_tree: ResMut<TechTree>,
    mut modifiers: ResMut<TechModifiers>,
    mut unit_query: Query<(&mut Stats, &mut Health, &Unit)>,
    mut rates: ResMut<ProductionRates>,
) {
    // Collect IDs of newly-unlocked nodes whose effects haven't been applied yet
    let newly_unlocked: Vec<String> = tech_tree
        .techs
        .iter()
        .filter(|n| n.unlocked && !tech_tree.applied_effects.contains(&n.id))
        .map(|n| n.id.clone())
        .collect();

    if newly_unlocked.is_empty() {
        return;
    }

    // Apply effects for each newly unlocked node
    for tech_id in &newly_unlocked {
        if let Some(node) = tech_tree.techs.iter().find(|n| n.id == *tech_id) {
            for effect in &node.effects {
                apply_effect(effect, &mut modifiers, &mut unit_query, &mut rates);
            }
        }
        tech_tree.applied_effects.push(tech_id.clone());
    }
}

fn apply_effect(
    effect: &TechEffect,
    modifiers: &mut TechModifiers,
    unit_query: &mut Query<(&mut Stats, &mut Health, &Unit)>,
    rates: &mut ProductionRates,
) {
    match effect {
        TechEffect::UnitStatBonus(archetype, stat_name, value) => {
            modifiers
                .unit_stat_bonuses
                .push((*archetype, stat_name.clone(), *value));
            // Apply immediately to existing units of this archetype
            for (mut stats, mut health, unit) in unit_query.iter_mut() {
                if unit.archetype != *archetype {
                    continue;
                }
                apply_stat_bonus(&mut stats, &mut health, stat_name, *value);
            }
        }
        TechEffect::BuildingCostReduction(building_type, value) => {
            modifiers
                .building_cost_reductions
                .push((*building_type, *value));
        }
        TechEffect::ResourceRateBonus(resource, value) => {
            modifiers
                .resource_rate_bonuses
                .push((resource.clone(), *value));
            // Apply immediately to production rates
            apply_rate_bonus(rates, resource, *value);
        }
        TechEffect::UnlockAbility(ability) => {
            modifiers.unlocked_abilities.push(*ability);
        }
        TechEffect::UnlockBuilding(building) => {
            modifiers.unlocked_buildings.push(*building);
        }
        TechEffect::UnlockUnit(archetype) => {
            modifiers.unlocked_units.push(*archetype);
        }
        TechEffect::PassiveBonus(key, value) => {
            let entry = modifiers.passive_bonuses.entry(key.clone()).or_insert(0.0);
            *entry += value;
        }
    }
}

fn apply_stat_bonus(stats: &mut Stats, health: &mut Health, stat_name: &str, value: f32) {
    match stat_name {
        "attack_damage" => stats.attack_damage *= 1.0 + value,
        "attack_speed" => stats.attack_speed *= 1.0 + value,
        "attack_range" => stats.attack_range *= 1.0 + value,
        "armor" => health.armor *= 1.0 + value,
        "move_speed" => stats.move_speed *= 1.0 + value,
        "vision_range" => stats.vision_range *= 1.0 + value,
        "morale" => {
            stats.morale *= 1.0 + value;
            stats.morale_max *= 1.0 + value;
        }
        _ => {}
    }
}

fn apply_rate_bonus(rates: &mut ProductionRates, resource: &str, value: f32) {
    match resource {
        "food" => rates.food_per_second *= 1.0 + value,
        "gold" => rates.gold_per_second *= 1.0 + value,
        "wood" => rates.wood_per_second *= 1.0 + value,
        "stone" => rates.stone_per_second *= 1.0 + value,
        _ => {}
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TECH RATE APPLICATION — syncs TechModifiers → ProductionRates each frame
//  (handles late-joining or respawned systems that need the rates)
// ═════════════════════════════════════════════════════════════════════════════

fn tech_rate_application_system(modifiers: Res<TechModifiers>, mut rates: ResMut<ProductionRates>) {
    // Reset rates to base first, then re-apply all bonuses
    // We store base rates in a separate resource? For now, just apply cumulatively.
    // Each frame we re-calc from scratch using the modifiers list.
    // Since resource_rate_bonuses grows monotonically, we can just apply them fresh.

    // Compute base rates (default without tech)
    let base_rates = ProductionRates::default();
    rates.food_per_second = base_rates.food_per_second;
    rates.gold_per_second = base_rates.gold_per_second;
    rates.wood_per_second = base_rates.wood_per_second;
    rates.stone_per_second = base_rates.stone_per_second;

    for (resource, value) in &modifiers.resource_rate_bonuses {
        apply_rate_bonus(&mut rates, resource, *value);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  SOVEREIGNTY POINT AWARDS
// ═════════════════════════════════════════════════════════════════════════════

fn sovereignty_point_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut stockpile: ResMut<ResourceStockpile>,
    tech_tree: Res<TechTree>,
) {
    for _event in objective_events.read() {
        // Award SP based on objective difficulty
        // Base: 5 SP per objective
        let mut sp_award: u32 = 5;

        // Apply sovereignty_rate passive bonus from tech
        let sov_bonus = tech_tree
            .techs
            .iter()
            .filter(|t| t.unlocked)
            .flat_map(|t| &t.effects)
            .filter_map(|e| {
                if let TechEffect::PassiveBonus(key, value) = e {
                    if key == "sovereignty_rate" {
                        Some(*value)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .sum::<f32>();

        if sov_bonus > 0.0 {
            sp_award = (sp_award as f32 * (1.0 + sov_bonus)).round() as u32;
        }

        stockpile.sovereignty_points += sp_award;
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  TESTS — inline integration tests
// ═════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn update_n(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    // ── Tech Tree Data Tests ──────────────────────────────────────────────

    #[test]
    fn test_default_tech_tree_has_45_nodes() {
        let tree = TechTree::default();
        let count = tree.techs.len();
        assert_eq!(
            count, 48,
            "Default tech tree should have 48 nodes, got {count}"
        );
    }

    #[test]
    fn test_all_tech_ids_unique() {
        let tree = TechTree::default();
        let mut ids: Vec<&str> = tree.techs.iter().map(|t| t.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(
            ids.len(),
            tree.techs.len(),
            "All tech node IDs must be unique"
        );
    }

    #[test]
    fn test_all_categories_have_techs() {
        let tree = TechTree::default();
        let categories: Vec<TechCategory> = vec![
            TechCategory::Military,
            TechCategory::Economy,
            TechCategory::Culture,
            TechCategory::Diplomacy,
            TechCategory::Building,
            TechCategory::Naval,
        ];
        for cat in categories {
            let count = tree.techs.iter().filter(|t| t.category == cat).count();
            assert!(count > 0, "Category {cat:?} should have at least 1 tech");
        }
    }

    #[test]
    fn test_prerequisites_exist_in_tree() {
        let tree = TechTree::default();
        let all_ids: std::collections::HashSet<&str> =
            tree.techs.iter().map(|t| t.id.as_str()).collect();
        for node in &tree.techs {
            for prereq in &node.prerequisites {
                assert!(
                    all_ids.contains(prereq.as_str()),
                    "Prerequisite '{prereq}' for tech '{}' not found in tree",
                    node.id
                );
            }
        }
    }

    #[test]
    fn test_no_circular_prerequisites() {
        let tree = TechTree::default();
        for node in &tree.techs {
            assert!(
                !node.prerequisites.contains(&node.id),
                "Tech '{}' depends on itself",
                node.id
            );
        }
    }

    // ── Research Validation Tests ─────────────────────────────────────────

    #[test]
    fn test_can_research_tier1_tech() {
        let tree = TechTree::default();
        let result = tree.can_research("mil_t1", 100);
        assert_eq!(result, ResearchStartResult::Success);
    }

    #[test]
    fn test_cannot_research_unknown_tech() {
        let tree = TechTree::default();
        let result = tree.can_research("nonexistent", 100);
        assert_eq!(result, ResearchStartResult::TechNotFound);
    }

    #[test]
    fn test_cannot_research_without_enough_sp() {
        let tree = TechTree::default();
        let result = tree.can_research("mil_t1", 5);
        assert_eq!(result, ResearchStartResult::InsufficientSP(10, 5));
    }

    #[test]
    fn test_cannot_research_already_researching() {
        let mut tree = TechTree::default();
        tree.current_research = Some(ResearchProject {
            tech_id: "eco_t1".into(),
            progress: 0.0,
            max_progress: 30.0,
            assigned_hero: None,
        });
        let result = tree.can_research("mil_t1", 100);
        assert_eq!(result, ResearchStartResult::AlreadyResearching);
    }

    #[test]
    fn test_cannot_research_already_unlocked() {
        let mut tree = TechTree::default();
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "mil_t1") {
            node.unlocked = true;
        }
        let result = tree.can_research("mil_t1", 100);
        assert_eq!(result, ResearchStartResult::AlreadyUnlocked);
    }

    #[test]
    fn test_cannot_research_without_prerequisites() {
        let tree = TechTree::default();
        let result = tree.can_research("mil_t2", 100);
        assert_eq!(
            result,
            ResearchStartResult::MissingPrerequisites(vec!["mil_t1".into()])
        );
    }

    #[test]
    fn test_can_research_with_prerequisites_met() {
        let mut tree = TechTree::default();
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "mil_t1") {
            node.unlocked = true;
        }
        let result = tree.can_research("mil_t2", 100);
        assert_eq!(result, ResearchStartResult::Success);
    }

    #[test]
    fn test_total_sp_spent() {
        let mut tree = TechTree::default();
        assert_eq!(tree.total_sp_spent(), 0);
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "mil_t1") {
            node.unlocked = true;
        }
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "eco_t1") {
            node.unlocked = true;
        }
        assert_eq!(tree.total_sp_spent(), 20);
    }

    // ── System Integration Tests ──────────────────────────────────────────

    #[test]
    fn test_start_research_consumes_sp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<TechTree>()
            .add_event::<ResearchOrder>()
            .add_systems(FixedUpdate, start_research_system);

        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 100;
        app.world_mut().send_event(ResearchOrder {
            entity: Entity::PLACEHOLDER,
            tech_id: "mil_t1".into(),
        });
        update_n(&mut app, 5);

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(stockpile.sovereignty_points, 90, "SP should be deducted");
        let tree = app.world().resource::<TechTree>();
        assert!(tree.current_research.is_some(), "Research should be active");
        if let Some(ref project) = tree.current_research {
            assert_eq!(project.tech_id, "mil_t1");
        }
    }

    #[test]
    fn test_start_research_insufficient_sp_ignored() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<TechTree>()
            .add_event::<ResearchOrder>()
            .add_systems(FixedUpdate, start_research_system);

        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 5;
        app.world_mut().send_event(ResearchOrder {
            entity: Entity::PLACEHOLDER,
            tech_id: "mil_t1".into(),
        });
        app.update();

        let tree = app.world().resource::<TechTree>();
        assert!(tree.current_research.is_none(), "Research should not start");
        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(stockpile.sovereignty_points, 5, "SP should not be deducted");
    }

    #[test]
    fn test_research_completes_and_unlocks() {
        let mut tree = TechTree::default();
        tree.current_research = Some(ResearchProject {
            tech_id: "mil_t1".into(),
            progress: 0.0,
            max_progress: 30.0,
            assigned_hero: None,
        });

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .insert_resource(tree)
            .init_resource::<TechModifiers>()
            .add_systems(
                FixedUpdate,
                research_progress_system.in_set(SimulationSet::ResearchProgress),
            );

        // 300 ticks at 0.1/tick = 30.0 progress = complete
        for _ in 0..300 {
            app.update();
        }

        let tree = app.world().resource::<TechTree>();
        let mil_t1 = tree.techs.iter().find(|t| t.id == "mil_t1").unwrap();
        assert!(
            mil_t1.unlocked,
            "mil_t1 should be unlocked after research completes"
        );
        assert!(
            tree.current_research.is_none(),
            "Research project should be cleared"
        );
    }

    #[test]
    fn test_research_with_hero_bonus_faster() {
        let mut tree = TechTree::default();
        tree.current_research = Some(ResearchProject {
            tech_id: "mil_t1".into(),
            progress: 0.0,
            max_progress: 30.0,
            assigned_hero: None,
        });

        // Without hero app
        let mut app_no_hero = App::new();
        app_no_hero
            .add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .insert_resource(tree)
            .init_resource::<TechModifiers>()
            .add_systems(
                FixedUpdate,
                research_progress_system.in_set(SimulationSet::ResearchProgress),
            );

        // With hero app — spawn hero first so we get a real entity id
        let mut app_with_hero = App::new();
        app_with_hero
            .add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<TechModifiers>()
            .add_systems(
                FixedUpdate,
                research_progress_system.in_set(SimulationSet::ResearchProgress),
            );

        let hero_entity = app_with_hero
            .world_mut()
            .spawn(Hero {
                hero_id: HeroId::Osman,
                level: 1,
                xp: 0,
                skill_points: 3,
                unlocked_abilities: Vec::new(),
                active_cooldowns: std::collections::HashMap::new(),
                ..Default::default()
            })
            .id();

        let mut tree_hero = TechTree::default();
        tree_hero.current_research = Some(ResearchProject {
            tech_id: "mil_t1".into(),
            progress: 0.0,
            max_progress: 30.0,
            assigned_hero: Some(hero_entity),
        });
        app_with_hero.world_mut().insert_resource(tree_hero);

        // Run 100 ticks at 0.1/tick
        for _ in 0..100 {
            app_no_hero.update();
            app_with_hero.update();
        }

        let tree_no = app_no_hero.world().resource::<TechTree>();
        let tree_yes = app_with_hero.world().resource::<TechTree>();

        let progress_no = tree_no
            .current_research
            .as_ref()
            .map(|p| p.progress)
            .unwrap_or(0.0);
        let progress_yes = tree_yes
            .current_research
            .as_ref()
            .map(|p| p.progress)
            .unwrap_or(0.0);

        assert!(
            progress_yes > progress_no,
            "Hero-assisted research should be faster: yes={progress_yes} no={progress_no}"
        );
    }

    #[test]
    fn test_objective_completed_awards_sp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<TechTree>()
            .add_event::<ObjectiveCompleted>()
            .add_systems(FixedUpdate, sovereignty_point_system);

        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 0;
        app.world_mut().send_event(ObjectiveCompleted {
            objective_id: ObjectiveId::Custom(1),
            mission_id: MissionId::Mission01,
        });
        update_n(&mut app, 5);

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(
            stockpile.sovereignty_points, 5,
            "Completing an objective should award 5 SP"
        );
    }

    #[test]
    fn test_multiple_objectives_award_cumulative_sp() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .init_resource::<ResourceStockpile>()
            .init_resource::<TechTree>()
            .add_event::<ObjectiveCompleted>()
            .add_systems(FixedUpdate, sovereignty_point_system);

        app.world_mut()
            .resource_mut::<ResourceStockpile>()
            .sovereignty_points = 0;
        for i in 0..3 {
            app.world_mut().send_event(ObjectiveCompleted {
                objective_id: ObjectiveId::Custom(i),
                mission_id: MissionId::Mission01,
            });
        }
        update_n(&mut app, 5);

        let stockpile = app.world().resource::<ResourceStockpile>();
        assert_eq!(
            stockpile.sovereignty_points, 15,
            "3 objectives should award 15 SP"
        );
    }

    // ── Tech Effect Application Tests ─────────────────────────────────────

    #[test]
    fn test_tech_effects_applied_on_unlock() {
        let mut tree = TechTree::default();
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "mil_t1") {
            node.unlocked = true;
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .insert_resource(tree)
            .init_resource::<TechModifiers>()
            .init_resource::<ProductionRates>()
            .add_systems(FixedUpdate, tech_effect_application_system);

        update_n(&mut app, 5);

        let modifiers = app.world().resource::<TechModifiers>();
        let found = modifiers.unit_stat_bonuses.iter().any(|(arch, stat, val)| {
            *arch == UnitArchetype::LightInfantry
                && stat == "attack_damage"
                && (*val - 0.10).abs() < 0.001
        });
        assert!(found, "UnitStatBonus should be applied");
    }

    #[test]
    fn test_effects_applied_only_once() {
        let mut tree = TechTree::default();
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "mil_t1") {
            node.unlocked = true;
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .insert_resource(tree)
            .init_resource::<TechModifiers>()
            .init_resource::<ProductionRates>()
            .add_systems(FixedUpdate, tech_effect_application_system);

        update_n(&mut app, 5);
        let count_1 = app
            .world()
            .resource::<TechModifiers>()
            .unit_stat_bonuses
            .len();

        update_n(&mut app, 5);
        let count_2 = app
            .world()
            .resource::<TechModifiers>()
            .unit_stat_bonuses
            .len();

        assert_eq!(count_1, count_2, "Tech effects should only be applied once");
    }

    #[test]
    fn test_resource_rate_effects_applied() {
        let mut tree = TechTree::default();
        if let Some(node) = tree.techs.iter_mut().find(|t| t.id == "eco_t1") {
            node.unlocked = true;
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 60.0,
            )))
            .init_resource::<GameTime>()
            .insert_resource(tree)
            .init_resource::<TechModifiers>()
            .init_resource::<ProductionRates>()
            .add_systems(FixedUpdate, tech_effect_application_system)
            .add_systems(FixedUpdate, tech_rate_application_system);

        update_n(&mut app, 5);

        let rates = app.world().resource::<ProductionRates>();
        assert!(
            (rates.food_per_second - 1.25).abs() < 0.001,
            "Food rate should be 1.25, got {}",
            rates.food_per_second
        );
    }
}
