use bevy::prelude::*;
use kurulus_core::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct HeroLeveledUp {
    pub hero: Entity,
    pub new_level: u8,
}

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<XpConfig>()
            .add_event::<HeroLeveledUp>()
            .add_systems(
                FixedUpdate,
                (
                    buff_tick_system,
                    xp_gain_system,
                    level_up_system,
                    skill_tree_system,
                    hero_ability_system,
                    cooldown_tick_system,
                ),
            );
    }
}

// ─── Buff system ────────────────────────────────────────────────────────────

/// Ticks all active buff durations and removes expired ones.
fn buff_tick_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut commands: Commands,
    mut entities: Query<(Entity, &mut ActiveBuffs, &mut Stats)>,
) {
    for (entity, mut active_buffs, mut stats) in entities.iter_mut() {
        let delta = time.delta_secs();
        let mut expired = Vec::new();

        // Reset attack damage to remove any stale buff contributions before
        // re-applying.  This is a simplification — a full game would store
        // a "base" copy of stats.
        // We just apply them additively for now.

        for (i, buff) in active_buffs.buffs.iter_mut().enumerate() {
            buff.duration -= delta;
            stats.attack_damage = (stats.attack_damage + buff.damage_bonus).max(1.0);

            if buff.duration <= 0.0 {
                expired.push(i);
            }
        }

        // Remove expired buffs (reverse order to keep indices valid)
        for i in expired.into_iter().rev() {
            active_buffs.buffs.remove(i);
        }

        if active_buffs.buffs.is_empty() {
            commands.entity(entity).remove::<ActiveBuffs>();
        }
    }
}

// ─── XP gain ────────────────────────────────────────────────────────────────

/// Grants XP to heroes from kills and objective completions.
fn xp_gain_system(
    mut death_events: EventReader<UnitDied>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut heroes: Query<&mut Hero>,
) {
    // Kill XP
    for event in death_events.read() {
        if let Ok(mut hero) = heroes.get_mut(event.killer) {
            let xp_gain = 20u32.saturating_mul(hero.level as u32);
            hero.xp = hero.xp.saturating_add(xp_gain);
        }
    }

    // Objective XP (granted to every hero — Phase 2 has only Osman)
    for _event in objective_events.read() {
        for mut hero in heroes.iter_mut() {
            hero.xp = hero.xp.saturating_add(50);
        }
    }
}

// ─── Level up ───────────────────────────────────────────────────────────────

/// Checks if any hero has enough XP to advance.
/// On level-up: +1 skill point, +5% max HP, full heal.
fn level_up_system(
    xp_config: Res<XpConfig>,
    mut heroes: Query<(Entity, &mut Hero)>,
    mut healths: Query<&mut Health>,
    mut level_up_events: EventWriter<HeroLeveledUp>,
) {
    for (entity, mut hero) in heroes.iter_mut() {
        loop {
            let required_xp = xp_config.xp_for_level(hero.level);
            if required_xp == u32::MAX || hero.xp < required_xp {
                break;
            }
            hero.xp = hero.xp.saturating_sub(required_xp);
            hero.level += 1;
            hero.skill_points += 1;

            if let Ok(mut health) = healths.get_mut(entity) {
                health.max *= 1.05;
                health.current = health.max;
            }

            level_up_events.send(HeroLeveledUp {
                hero: entity,
                new_level: hero.level,
            });
        }
    }
}

// ─── Skill tree ─────────────────────────────────────────────────────────────

/// Auto-unlocks the first available skill node whose prerequisites are met.
fn skill_tree_system(mut heroes: Query<(&mut Hero, &mut SkillTree)>) {
    for (mut hero, mut tree) in heroes.iter_mut() {
        if hero.skill_points == 0 {
            continue;
        }

        // Collect IDs of already unlocked nodes
        let unlocked_ids: Vec<SkillNodeId> = tree
            .nodes
            .iter()
            .filter(|n| n.unlocked)
            .map(|n| n.id)
            .collect();

        // Find the first unlockable node
        let candidate_idx = tree.nodes.iter().position(|node| {
            if node.unlocked {
                return false;
            }
            if hero.skill_points < node.cost {
                return false;
            }
            node.prerequisites
                .iter()
                .all(|prereq| unlocked_ids.contains(prereq))
        });

        if let Some(idx) = candidate_idx {
            let node = &mut tree.nodes[idx];
            node.unlocked = true;
            hero.skill_points -= node.cost;

            if let Some(ability_id) = skill_node_to_ability(node.id) {
                if !hero.unlocked_abilities.contains(&ability_id) {
                    hero.unlocked_abilities.push(ability_id);
                }
            }
        }
    }
}

fn skill_node_to_ability(node: SkillNodeId) -> Option<AbilityId> {
    match node {
        SkillNodeId::SwordMastery => Some(AbilityId::SovereignStrike),
        SkillNodeId::LeaderAura => Some(AbilityId::WarCry),
        SkillNodeId::GuerrillaTactics => None,
        SkillNodeId::WarLeadership => None,
        SkillNodeId::TribalUnity => Some(AbilityId::LionsHeart),
        SkillNodeId::EpicWarcry => Some(AbilityId::WarCry),
        SkillNodeId::ShadowStrike => Some(AbilityId::SovereignStrike),
        SkillNodeId::Sovereignty => Some(AbilityId::TreesVision),
        _ => None,
    }
}

// ─── Ability activation ─────────────────────────────────────────────────────

/// Internal representation of an ability activation request.
struct Activation {
    ability_id: AbilityId,
    caster: Entity,
    target: AbilityTarget,
    hero_pos: WorldPosition,
}

/// Reads `AbilityUsed` events and dispatches to ability implementations.
/// Uses a single mutable query to avoid B0001 conflicting access.
fn hero_ability_system(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityUsed>,
    mut heroes: Query<(Entity, &mut Hero, &WorldPosition)>,
    units: Query<(&WorldPosition, &Unit)>,
    mut healths: Query<&mut Health>,
    mut stats_q: Query<&mut Stats>,
    mut damage_events: EventWriter<DamageDealt>,
) {
    // Phase 1: collect valid activations (immutable borrow from the single query)
    let mut activations: Vec<Activation> = Vec::new();
    for event in ability_events.read() {
        // Use `get` and destructure with ref for immutability
        let Ok((entity, hero, pos)) = heroes.get(event.caster) else {
            continue;
        };
        if hero
            .active_cooldowns
            .get(&event.ability_id)
            .map_or(false, |c| *c > 0.0)
        {
            continue;
        }
        activations.push(Activation {
            ability_id: event.ability_id,
            caster: entity,
            target: event.target.clone(),
            hero_pos: *pos,
        });
    }

    // Phase 2: execute abilities
    let mut cooldowns: Vec<(Entity, AbilityId, f32)> = Vec::new();
    for act in &activations {
        let cooldown = match act.ability_id {
            AbilityId::WarCry => {
                execute_warcry(&mut commands, act.caster, &act.hero_pos, &mut stats_q)
            }
            AbilityId::SovereignStrike => execute_sovereign_strike(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &act.target,
                &units,
                &mut damage_events,
            ),
            AbilityId::LionsHeart => {
                execute_lions_heart(&mut commands, act.caster, &act.hero_pos, &mut healths)
            }
            AbilityId::TreesVision => 45.0,
            _ => continue,
        };
        cooldowns.push((act.caster, act.ability_id, cooldown));
    }

    // Phase 3: set cooldowns (mutable borrow from the single query)
    for (caster, ability_id, duration) in &cooldowns {
        if let Ok((_entity, mut hero, _pos)) = heroes.get_mut(*caster) {
            hero.active_cooldowns.insert(*ability_id, *duration);
        }
    }
}

// ─── Osman ability implementations ──────────────────────────────────────────

/// WarCry (Ultimate): morale boost + damage buff for all units in radius.
fn execute_warcry(
    commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    stats_q: &mut Query<&mut Stats>,
) -> f32 {
    const DURATION: f32 = 10.0;
    const MORALE_BOOST: f32 = 25.0;
    const DAMAGE_BONUS: f32 = 0.5;

    for mut stats in stats_q.iter_mut() {
        stats.morale = (stats.morale + MORALE_BOOST).min(stats.morale_max);
    }

    let mut buff = Buff::new(AbilityId::WarCry, DURATION);
    buff.damage_bonus = DAMAGE_BONUS;
    commands.entity(caster).insert(ActiveBuffs {
        buffs: vec![buff],
    });

    60.0
}

/// SovereignStrike: 3× damage + knockback.
fn execute_sovereign_strike(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(&WorldPosition, &Unit)>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    let target_entity = match target {
        AbilityTarget::Entity(e) => *e,
        _ => return 15.0,
    };

    let base_damage = 30.0;
    let damage = base_damage * 3.0;

    damage_events.send(DamageDealt {
        attacker: caster,
        target: target_entity,
        amount: damage,
        damage_type: DamageType::Melee,
    });

    // Knockback 2 cells
    if let Ok((target_pos, _unit)) = units.get(target_entity) {
        let dir = (target_pos.world_2d.0 - caster_pos.world_2d.0).normalize_or_zero();
        let new_x = (target_pos.world_2d.0.x + dir.x * 2.0 * CELL_SIZE) as i32;
        let new_y = (target_pos.world_2d.0.y + dir.y * 2.0 * CELL_SIZE) as i32;
        commands.entity(target_entity).insert(WorldPosition::new(
            IVec2::new(new_x, new_y),
            CELL_SIZE,
        ));
    }

    15.0
}

/// LionsHeart: heal self + damage reduction.
fn execute_lions_heart(
    commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    healths: &mut Query<&mut Health>,
) -> f32 {
    const HEAL: f32 = 50.0;
    const DURATION: f32 = 8.0;
    const DR: f32 = 0.5;

    if let Ok(mut health) = healths.get_mut(caster) {
        health.current = (health.current + HEAL).min(health.max);
    }

    let mut buff = Buff::new(AbilityId::LionsHeart, DURATION);
    buff.damage_reduction = DR;
    commands
        .entity(caster)
        .insert(ActiveBuffs { buffs: vec![buff] });

    30.0
}

// ─── Cooldown ticking ──────────────────────────────────────────────────────

/// Ticks cooldowns for all hero abilities.
fn cooldown_tick_system(time: Res<bevy::time::Time<bevy::time::Fixed>>, mut heroes: Query<&mut Hero>) {
    for mut hero in heroes.iter_mut() {
        for (_ability_id, cooldown) in hero.active_cooldowns.iter_mut() {
            *cooldown = (*cooldown - time.delta_secs()).max(0.0);
        }
    }
}
