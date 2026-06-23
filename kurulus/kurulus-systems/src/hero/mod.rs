use bevy::prelude::*;
use kurulus_core::prelude::*;

pub mod state_death;
pub mod talents;
pub mod equipment;
pub mod named_units;

#[derive(Event, Clone, Debug)]
pub struct HeroLeveledUp {
    pub hero: Entity,
    pub new_level: u8,
}

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<XpConfig>()
            .init_resource::<AdministrativeMindBuff>()
            .init_resource::<CulturalRenaissanceBuff>()
            .init_resource::<ActiveHero>()
            .init_resource::<HeroDeathPenalties>()
            .init_resource::<equipment::RecipeDatabase>()
            .init_resource::<EquipmentLootTable>()
            .init_resource::<NamedUnitRoster>()
            .add_event::<HeroLeveledUp>()
            .add_event::<HeroSwitchEvent>()
            .add_event::<HeroKnockedOut>()
            .add_event::<HeroRecovering>()
            .add_event::<HeroReturnedToIdle>()
            .add_event::<HeroKilledInMission>()
            .add_event::<HeroDeathBlow>()
            .add_event::<TalentChoiceRequest>()
            .add_event::<TalentChoiceSelected>()
            .add_event::<SpecializationRequest>()
            .add_event::<SpecializationSelected>()
            .add_event::<state_death::DeathBlowAttempt>()
            // ── Phase 2.5-2.7: Equipment events ──
            .add_event::<equipment::EquipItem>()
            .add_event::<equipment::UnequipItem>()
            .add_event::<equipment::ItemUnequipped>()
            .add_event::<equipment::CraftItem>()
            .add_event::<equipment::ItemCrafted>()
            .add_event::<equipment::CraftingFailed>()
            .add_event::<equipment::LootDropped>()
            // ── Phase 3: Named unit events ──
            .add_event::<named_units::NamedUnitKilled>()
            .register_type::<ActiveHero>()
            .register_type::<HeroDeathPenalties>()
            .register_type::<DeathPenalty>()
            .register_type::<Equipment>()
            .register_type::<EquipmentItem>()
            .register_type::<EquipmentSlot>()
            .register_type::<Rarity>()
            .register_type::<EquipmentStats>()
            .register_type::<HeroState>()
            .register_type::<HeroDeathState>()
            .register_type::<NamedUnit>()
            .register_type::<EquipmentLootTable>()
            .register_type::<equipment::RecipeDatabase>()
            .register_type::<equipment::CraftingRecipe>()
            .register_type::<NamedUnitAbilities>()
            .register_type::<NamedUnitStatsBlock>()
            .register_type::<NamedUnitRoster>()
            .register_type::<NamedUnitAbilityId>()
            .add_systems(
                FixedUpdate,
                (
                    hero_resource_regen_system,
                    buff_tick_system,
                    xp_gain_system,
                    level_up_system,
                    skill_tree_system,
                    hero_ability_system,
                    cooldown_tick_system,
                    admin_mind_tick_system,
                    cultural_renaissance_tick_system,
                    hero_switch_system,
                ),
            )
            // ── Phase 2: State machine + death flow ──
            .add_systems(FixedUpdate, (
                state_death::hero_state_machine_system,
                state_death::hero_death_penalty_system,
            ))
            // ── Phase 2: Talent + specialization ──
            .add_systems(FixedUpdate, (
                talents::hero_talent_request_system,
                talents::hero_talent_apply_system,
                talents::hero_specialization_apply_system,
            ))
            // ── Phase 2.5-2.7: Equipment + Crafting + Loot ──
            .add_systems(FixedUpdate, (
                equipment::equip_item_system,
                equipment::unequip_item_system,
                equipment::equipment_apply_system,
                equipment::crafting_system,
                equipment::loot_drop_system,
            ))
            // ── Phase 3: Named units ──
            .add_systems(FixedUpdate, (
                named_units::named_unit_stats_system,
                named_units::named_unit_permadeath_system,
                named_units::named_unit_ability_system,
            ))
            // Initialize recipes, loot table, and named unit roster on startup
            .add_systems(Startup, (
                equipment::initialize_recipes,
                equipment::initialize_loot_table,
                named_units::initialize_named_unit_roster,
            ));
    }
}

// ─── Hero resource regen ────────────────────────────────────────────────────

/// Regenerates hero-specific ability resources each tick.
/// Osman (Inspiration): +1/s base, +3/s in combat
/// Turgut (Fury): +0.8/s base, +2 on taking damage (handled by combat system)
/// Buran (Focus): +1.5/s base, +5 while stealthed and stationary (handled by stealth system)
fn hero_resource_regen_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut heroes: Query<&mut Hero>,
) {
    let delta = time.delta_secs();
    for mut hero in heroes.iter_mut() {
        let regen_amount = hero.resource_regen * delta;
        hero.resource_current = (hero.resource_current + regen_amount).min(hero.resource_max);
    }
}

// ─── Buff system ────────────────────────────────────────────────────────────

/// Ticks all active buff durations and removes expired ones.
/// NOTE: Stat bonuses from buffs are applied ONCE at buff creation time
/// (in the ability execute functions). The tick system does NOT modify stats
/// to avoid cumulative inflation. Buff removal is handled by removal handlers.
fn buff_tick_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut commands: Commands,
    mut entities: Query<(Entity, &mut ActiveBuffs)>,
) {
    for (_entity, mut active_buffs) in entities.iter_mut() {
        let delta = time.delta_secs();
        let mut expired = Vec::new();

        for (i, buff) in active_buffs.buffs.iter_mut().enumerate() {
            buff.duration -= delta;
            if buff.duration <= 0.0 {
                expired.push(i);
            }
        }

        // Remove expired buffs (reverse order to keep indices valid)
        for i in expired.into_iter().rev() {
            active_buffs.buffs.remove(i);
        }

        if active_buffs.buffs.is_empty() {
            commands.entity(_entity).remove::<ActiveBuffs>();
        }
    }
}

// ─── XP gain ────────────────────────────────────────────────────────────────

/// Grants XP to heroes from kills and objective completions.
/// Checks CulturalRenaissanceBuff for the XP multiplier.
fn xp_gain_system(
    mut death_events: EventReader<UnitDied>,
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut heroes: Query<&mut Hero>,
    renaissance: Res<CulturalRenaissanceBuff>,
) {
    let xp_mult: f32 = if renaissance.active { 1.15 } else { 1.0 };

    // Kill XP
    for event in death_events.read() {
        if let Ok(mut hero) = heroes.get_mut(event.killer) {
            let base_xp = 20u32.saturating_mul(hero.level as u32);
            let bonus = (base_xp as f32 * (xp_mult - 1.0)).round() as u32;
            hero.xp = hero.xp.saturating_add(base_xp + bonus);
        }
    }

    // Objective XP (granted to every hero)
    for _event in objective_events.read() {
        for mut hero in heroes.iter_mut() {
            let base_xp = 50;
            let bonus = (base_xp as f32 * (xp_mult - 1.0)).round() as u32;
            hero.xp = hero.xp.saturating_add(base_xp + bonus);
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
        // Osman tree
        SkillNodeId::SwordMastery => Some(AbilityId::CallOfTheTribe),
        SkillNodeId::LeaderAura => Some(AbilityId::WarCry),
        SkillNodeId::GuerrillaTactics => None,
        SkillNodeId::WarLeadership => None,
        SkillNodeId::TribalUnity => Some(AbilityId::Lionheart),
        SkillNodeId::EpicWarcry => Some(AbilityId::WarCry),
        SkillNodeId::ShadowStrike => Some(AbilityId::CallOfTheTribe),
        SkillNodeId::Sovereignty => Some(AbilityId::DreamOfTheTree),
        // Orhan tree
        SkillNodeId::SiegeMastery => Some(AbilityId::FortressBuilder),
        SkillNodeId::ArmyOrganization => Some(AbilityId::TacticalGenius),
        SkillNodeId::Fortification => Some(AbilityId::ShieldWall),
        SkillNodeId::Logistics => None,
        SkillNodeId::MilitaryReform => Some(AbilityId::CavalryCharge),
        SkillNodeId::GrandArmy => Some(AbilityId::TacticalGenius),
        // Alaeddin tree
        SkillNodeId::Taxation => Some(AbilityId::TaxReform),
        SkillNodeId::Bureaucracy => Some(AbilityId::AdministrativeMind),
        SkillNodeId::Diplomacy => Some(AbilityId::DiplomaticImmunity),
        SkillNodeId::CulturalPatronage => Some(AbilityId::CulturalRenaissance),
        SkillNodeId::LegalCode => Some(AbilityId::TaxReform),
        SkillNodeId::GrandVizier => Some(AbilityId::AdministrativeMind),
        // Turgut tree
        SkillNodeId::GroundPounder => Some(AbilityId::GroundStrike),
        SkillNodeId::Unyielding => Some(AbilityId::IronWall),
        SkillNodeId::BattleRoar => Some(AbilityId::EarthquakeChallenge),
        SkillNodeId::StoneSkin => Some(AbilityId::IronWall),
        // Buran tree
        SkillNodeId::EagleEye => Some(AbilityId::BlindingShot),
        SkillNodeId::SilentStep => Some(AbilityId::ShadowLightness),
        SkillNodeId::EmberArrow => Some(AbilityId::FlamingArrow),
        SkillNodeId::ShadowForm => Some(AbilityId::ShadowLightness),
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
/// Uses a single mutable hero query to avoid B0001 conflicts.
#[allow(clippy::too_many_arguments)]
fn hero_ability_system(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityUsed>,
    mut heroes: Query<(Entity, &mut Hero, &WorldPosition)>,
    units: Query<(Entity, &WorldPosition, &Unit)>,
    mut healths: Query<&mut Health>,
    mut stats_q: Query<&mut Stats>,
    mut damage_events: EventWriter<DamageDealt>,
    buildings: Query<(Entity, &Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut productions: ResMut<ProductionRates>,
    mut renaissance: ResMut<CulturalRenaissanceBuff>,
    mut admin_buff: ResMut<AdministrativeMindBuff>,
) {
    // Phase 1: collect valid activations
    let mut activations: Vec<Activation> = Vec::new();
    for event in ability_events.read() {
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
            // Osman
            AbilityId::WarCry => {
                execute_warcry(&mut commands, act.caster, &act.hero_pos, &mut stats_q)
            }
            AbilityId::CallOfTheTribe => execute_sovereign_strike(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &act.target,
                &units,
                &mut damage_events,
            ),
            AbilityId::Lionheart => {
                execute_lions_heart(&mut commands, act.caster, &act.hero_pos, &mut healths)
            }
            AbilityId::DreamOfTheTree => 45.0,
            // Orhan
            AbilityId::TacticalGenius => execute_tactical_genius(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &units,
                &mut stats_q,
            ),
            AbilityId::ShieldWall => execute_shield_wall(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &units,
                &mut stats_q,
            ),
            AbilityId::CavalryCharge => execute_cavalry_charge(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &act.target,
                &units,
                &mut stats_q,
                &mut damage_events,
            ),
            AbilityId::FortressBuilder => execute_fortress_builder(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &buildings,
            ),
            // Alaeddin
            AbilityId::AdministrativeMind => execute_administrative_mind(
                &mut productions,
                &mut admin_buff,
            ),
            AbilityId::TaxReform => {
                execute_tax_reform(&mut stockpile, &units);
                90.0
            }
            AbilityId::DiplomaticImmunity => execute_diplomatic_immunity(
                &mut commands,
                act.caster,
                &act.hero_pos,
                &buildings,
            ),
            AbilityId::CulturalRenaissance => execute_cultural_renaissance(
                &mut heroes,
                &mut renaissance,
            ),
            // Turgut (not yet implemented — stubs return placeholder cooldowns)
            AbilityId::GroundStrike => {
                execute_ground_strike(&mut commands, act.caster, &act.hero_pos, &units, &mut damage_events)
            }
            AbilityId::EarthquakeChallenge => {
                execute_earthquake_challenge(&mut commands, act.caster, &act.hero_pos, &units, &mut stats_q, &mut damage_events)
            }
            AbilityId::IronWall => {
                0.0 // Passive — handled by stat calculation system
            }
            AbilityId::BullsWrath => {
                execute_bulls_wrath(&mut commands, act.caster, &act.hero_pos, &act.target, &units, &mut stats_q, &mut damage_events)
            }
            // Buran (not yet implemented — stubs return placeholder cooldowns)
            AbilityId::FlamingArrow => {
                execute_flaming_arrow(&mut commands, act.caster, &act.hero_pos, &act.target, &units, &mut damage_events)
            }
            AbilityId::BlindingShot => {
                execute_blinding_shot(&mut commands, act.caster, &act.target, &units, &mut damage_events)
            }
            AbilityId::ShadowLightness => {
                0.0 // Passive — handled by stealth system
            }
            AbilityId::DeathVolley => {
                execute_death_volley(&mut commands, act.caster, &act.hero_pos, &act.target, &units, &mut damage_events)
            }
            // Generic (not yet implemented)
            AbilityId::FirstAid | AbilityId::Scout | AbilityId::Sabotage | AbilityId::Inspire => {
                30.0
            }
        };
        cooldowns.push((act.caster, act.ability_id, cooldown));
    }

    // Phase 3: set cooldowns
    for (caster, ability_id, duration) in &cooldowns {
        if let Ok((_entity, mut hero, _pos)) = heroes.get_mut(*caster) {
            hero.active_cooldowns.insert(*ability_id, *duration);
        }
    }
}

// ─── Osman ability implementations ──────────────────────────────────────────

/// WarCry: morale boost + damage buff for all units in radius.
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

/// CallOfTheTribe: summons 5 alps to the hero's location.
fn execute_sovereign_strike(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
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
    if let Ok((_, target_pos, _unit)) = units.get(target_entity) {
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

/// Lionheart: heal self + damage reduction.
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

    let mut buff = Buff::new(AbilityId::Lionheart, DURATION);
    buff.damage_reduction = DR;
    commands
        .entity(caster)
        .insert(ActiveBuffs { buffs: vec![buff] });

    30.0
}

// ─── Turgut ability implementations ─────────────────────────────────────────

/// GroundStrike: AoE blunt damage + knockback.
fn execute_ground_strike(
    _commands: &mut Commands,
    caster: Entity,
    caster_pos: &WorldPosition,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    const RADIUS: f32 = 4.0 * CELL_SIZE;
    const DAMAGE: f32 = 60.0;

    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);

    for (entity, pos, unit) in units.iter() {
        if unit.faction != caster_faction {
            let dist = pos.world_2d.0.distance(caster_pos.world_2d.0);
            if dist <= RADIUS {
                damage_events.send(DamageDealt {
                    attacker: caster,
                    target: entity,
                    amount: DAMAGE,
                    damage_type: DamageType::Melee,
                });
            }
        }
    }

    10.0
}

/// EarthquakeChallenge: leap to target, taunt, self defense buff.
fn execute_earthquake_challenge(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: &WorldPosition,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    stats_q: &mut Query<&mut Stats>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    const LEAP_RADIUS: f32 = 8.0 * CELL_SIZE;
    const DAMAGE: f32 = 40.0;
    const DEFENSE_BOOST: f32 = 0.25;
    const DURATION: f32 = 5.0;

    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);

    // AoE damage on landing
    for (entity, pos, unit) in units.iter() {
        if unit.faction != caster_faction {
            let dist = pos.world_2d.0.distance(caster_pos.world_2d.0);
            if dist <= LEAP_RADIUS {
                damage_events.send(DamageDealt {
                    attacker: caster,
                    target: entity,
                    amount: DAMAGE,
                    damage_type: DamageType::Melee,
                });
            }
        }
    }

    // Self defense buff
    if let Ok(mut stats) = stats_q.get_mut(caster) {
        stats.attack_damage *= 1.0 + DEFENSE_BOOST;
    }

    // Taunt + defense buff via buff component
    let mut buff = Buff::new(AbilityId::EarthquakeChallenge, DURATION);
    buff.defense_bonus = DEFENSE_BOOST;
    commands
        .entity(caster)
        .insert(ActiveBuffs { buffs: vec![buff] });

    18.0
}

/// BullsWrath: dash forward, deal AoE damage, enter enrage state.
fn execute_bulls_wrath(
    commands: &mut Commands,
    caster: Entity,
    caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    _stats_q: &mut Query<&mut Stats>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    const DASH_DISTANCE: f32 = 10.0;
    const BASE_DAMAGE: f32 = 80.0;
    const ENRAGE_DURATION: f32 = 12.0;

    // Get target position
    let target_pos = match target {
        AbilityTarget::Position(gp) => Vec2::new(gp.0.x as f32, gp.0.y as f32),
        AbilityTarget::Entity(e) => {
            if let Ok((_, pos, _)) = units.get(*e) {
                pos.world_2d.0
            } else {
                return 20.0;
            }
        }
        _ => return 20.0,
    };

    // Move caster toward target
    let dir = (target_pos - caster_pos.world_2d.0).normalize_or_zero();
    let dest = caster_pos.world_2d.0 + dir * DASH_DISTANCE;
    let gp = IVec2::new(
        (dest.x / CELL_SIZE).round() as i32,
        (dest.y / CELL_SIZE).round() as i32,
    );
    commands.entity(caster).insert(WorldPosition::new(gp, CELL_SIZE));

    // AoE damage on impact
    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);
    let impact_radius = 2.0 * CELL_SIZE;

    for (entity, pos, unit) in units.iter() {
        if unit.faction != caster_faction {
            let dist = pos.world_2d.0.distance(dest);
            if dist <= impact_radius {
                damage_events.send(DamageDealt {
                    attacker: caster,
                    target: entity,
                    amount: BASE_DAMAGE,
                    damage_type: DamageType::Melee,
                });
            }
        }
    }

    // Enrage buff: +100% attack speed, +40% damage, -30% defense
    let mut buff = Buff::new(AbilityId::BullsWrath, ENRAGE_DURATION);
    buff.attack_speed_bonus = 1.0;
    buff.damage_bonus = 0.4;
    buff.damage_reduction = -0.30; // negative = more damage taken
    commands
        .entity(caster)
        .insert(ActiveBuffs { buffs: vec![buff] });

    150.0
}

// ─── Buran ability implementations ──────────────────────────────────────────

/// FlamingArrow: fire DoT + area reveal.
fn execute_flaming_arrow(
    _commands: &mut Commands,
    _caster: Entity,
    caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    const RANGE: f32 = 35.0 * CELL_SIZE;
    const BASE_DAMAGE: f32 = 30.0;

    let target_entity = match target {
        AbilityTarget::Entity(e) => *e,
        _ => return 8.0,
    };

    // Check range
    if let Ok((_, target_pos, _)) = units.get(target_entity) {
        let dist = target_pos.world_2d.0.distance(caster_pos.world_2d.0);
        if dist > RANGE {
            return 8.0; // out of range, quick cooldown
        }
    }

    damage_events.send(DamageDealt {
        attacker: _caster,
        target: target_entity,
        amount: BASE_DAMAGE,
        damage_type: DamageType::Ranged,
    });

    8.0
}

/// BlindingShot: blind + silence a single target.
fn execute_blinding_shot(
    _commands: &mut Commands,
    caster: Entity,
    target: &AbilityTarget,
    _units: &Query<(Entity, &WorldPosition, &Unit)>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    let target_entity = match target {
        AbilityTarget::Entity(e) => *e,
        _ => return 12.0,
    };

    damage_events.send(DamageDealt {
        attacker: caster,
        target: target_entity,
        amount: 20.0,
        damage_type: DamageType::Ranged,
    });

    12.0
}

/// DeathVolley: 20 arrows rain on target area.
fn execute_death_volley(
    _commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    const RADIUS: f32 = 5.0 * CELL_SIZE; // 10m diameter
    const ARROW_DAMAGE: f32 = 15.0;

    let center = match target {
        AbilityTarget::Position(gp) => Vec2::new(gp.0.x as f32, gp.0.y as f32),
        AbilityTarget::Entity(e) => {
            if let Ok((_, pos, _)) = units.get(*e) {
                pos.world_2d.0
            } else {
                return 100.0;
            }
        }
        _ => return 100.0,
    };

    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);

    for (entity, pos, unit) in units.iter() {
        if unit.faction != caster_faction {
            let dist = pos.world_2d.0.distance(center);
            if dist <= RADIUS {
                damage_events.send(DamageDealt {
                    attacker: caster,
                    target: entity,
                    amount: ARROW_DAMAGE,
                    damage_type: DamageType::Ranged,
                });
            }
        }
    }

    100.0
}

// ─── Orhan ability implementations ──────────────────────────────────────────

/// TacticalGenius: AoE +20% attack speed for all friendly units for 12s.
fn execute_tactical_genius(
    commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    stats_q: &mut Query<&mut Stats>,
) -> f32 {
    const DURATION: f32 = 12.0;
    const SPEED_BONUS: f32 = 0.20;
    const RADIUS: f32 = 8.0 * CELL_SIZE;

    // Find caster's faction
    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);

    // Apply buff to all friendly units (same faction) within radius
    for (e, pos, unit) in units.iter() {
        if unit.faction == caster_faction {
            let dist = pos.world_2d.0.distance(_caster_pos.world_2d.0);
            if dist <= RADIUS {
                let mut buff = Buff::new(AbilityId::TacticalGenius, DURATION);
                buff.attack_speed_bonus = SPEED_BONUS;
                commands.entity(e).insert(ActiveBuffs {
                    buffs: vec![buff],
                });
            }
        }
    }

    // Morale bonus to Orhan himself
    if let Ok(mut stats) = stats_q.get_mut(caster) {
        stats.morale = (stats.morale + 15.0).min(stats.morale_max);
    }

    60.0
}

/// ShieldWall: AoE +50% damage resistance for friendly units for 10s.
fn execute_shield_wall(
    commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    stats_q: &mut Query<&mut Stats>,
) -> f32 {
    const DURATION: f32 = 10.0;
    const DR: f32 = 0.50;
    const RADIUS: f32 = 6.0 * CELL_SIZE;

    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);

    for (e, pos, unit) in units.iter() {
        if unit.faction == caster_faction {
            let dist = pos.world_2d.0.distance(_caster_pos.world_2d.0);
            if dist <= RADIUS {
                let mut buff = Buff::new(AbilityId::ShieldWall, DURATION);
                buff.damage_reduction = DR;
                commands.entity(e).insert(ActiveBuffs {
                    buffs: vec![buff],
                });
            }
        }
    }

    // Boost Orhan's own morale
    if let Ok(mut stats) = stats_q.get_mut(caster) {
        stats.morale = (stats.morale + 10.0).min(stats.morale_max);
    }

    45.0
}

/// CavalryCharge: dash to target, dealing AoE damage + knockback on arrival.
fn execute_cavalry_charge(
    commands: &mut Commands,
    caster: Entity,
    _caster_pos: &WorldPosition,
    target: &AbilityTarget,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
    stats_q: &mut Query<&mut Stats>,
    damage_events: &mut EventWriter<DamageDealt>,
) -> f32 {
    // Get target position
    let target_pos = match target {
        AbilityTarget::Position(gp) => Vec2::new(gp.0.x as f32, gp.0.y as f32),
        AbilityTarget::Entity(e) => {
            if let Ok((_, pos, _)) = units.get(*e) {
                pos.world_2d.0
            } else {
                return 20.0; // can't find target, short cooldown
            }
        }
        _ => return 20.0,
    };

    // Move caster to target position
    let gp = IVec2::new(
        (target_pos.x / CELL_SIZE).round() as i32,
        (target_pos.y / CELL_SIZE).round() as i32,
    );
    commands.entity(caster).insert(WorldPosition::new(gp, CELL_SIZE));

    // Deal AoE damage to all enemies near the destination
    let caster_faction = units
        .get(caster)
        .map(|(_, _, u)| u.faction)
        .unwrap_or(FactionId::Kayi);
    let aoe_radius = 1.5 * CELL_SIZE;
    let charge_damage = 40.0;

    for (e, pos, unit) in units.iter() {
        if unit.faction != caster_faction {
            let dist = pos.world_2d.0.distance(target_pos);
            if dist <= aoe_radius {
                damage_events.send(DamageDealt {
                    attacker: caster,
                    target: e,
                    amount: charge_damage,
                    damage_type: DamageType::Melee,
                });
            }
        }
    }

    // Morale bonus to Orhan
    if let Ok(mut stats) = stats_q.get_mut(caster) {
        stats.morale = (stats.morale + 10.0).min(stats.morale_max);
    }

    20.0
}

/// FortressBuilder: instantly upgrade nearest Oba by one level.
fn execute_fortress_builder(
    commands: &mut Commands,
    _caster: Entity,
    caster_pos: &WorldPosition,
    buildings: &Query<(Entity, &Building, &WorldPosition)>,
) -> f32 {
    // Find nearest Oba
    let mut nearest_oba: Option<(Entity, f32)> = None;
    for (entity, building, pos) in buildings.iter() {
        if building.building_type == BuildingType::Oba
            && building.level < building.max_level
        {
            let dist = pos
                .world_2d
                .0
                .distance(caster_pos.world_2d.0);
            match nearest_oba {
                Some((_, best_dist)) if dist < best_dist => {
                    nearest_oba = Some((entity, dist));
                }
                None => {
                    nearest_oba = Some((entity, dist));
                }
                _ => {}
            }
        }
    }

    if let Some((entity, _dist)) = nearest_oba {
        // Upgrade the Oba by one level
        if let Ok((_, building, _)) = buildings.get(entity) {
            let new_level = (building.level + 1).min(building.max_level);
            commands.entity(entity).insert(Building {
                level: new_level,
                ..(*building).clone()
            });
        }
    }

    120.0 // long cooldown — powerful ability
}

// ─── Alaeddin ability implementations ───────────────────────────────────────

/// AdministrativeMind: +50% resource production for 20s.
fn execute_administrative_mind(
    productions: &mut ResMut<ProductionRates>,
    buff: &mut ResMut<AdministrativeMindBuff>,
) -> f32 {
    const DURATION: f32 = 20.0;
    const MULTIPLIER: f32 = 1.5;

    // Store original rates (only if not already active)
    if !buff.active {
        buff.original_food_rate = productions.food_per_second;
        buff.original_gold_rate = productions.gold_per_second;
        buff.original_wood_rate = productions.wood_per_second;
        buff.original_stone_rate = productions.stone_per_second;

        // Apply multiplier
        productions.food_per_second *= MULTIPLIER;
        productions.gold_per_second *= MULTIPLIER;
        productions.wood_per_second *= MULTIPLIER;
        productions.stone_per_second *= MULTIPLIER;

        buff.remaining = DURATION;
        buff.active = true;
    }

    90.0
}

/// TaxReform: instant gold based on population + number of friendly units.
fn execute_tax_reform(
    stockpile: &mut ResMut<ResourceStockpile>,
    units: &Query<(Entity, &WorldPosition, &Unit)>,
) -> f32 {
    let population = stockpile.population;
    let friendly_count = units
        .iter()
        .filter(|(_, _, u)| u.faction == FactionId::Kayi)
        .count();

    let gold = (population as f32 * 5.0) + (friendly_count as f32 * 10.0);
    stockpile.gold += gold;

    90.0
}

/// DiplomaticImmunity: grant 30s of damage immunity to the nearest allied building.
fn execute_diplomatic_immunity(
    commands: &mut Commands,
    _caster: Entity,
    caster_pos: &WorldPosition,
    buildings: &Query<(Entity, &Building, &WorldPosition)>,
) -> f32 {
    const DURATION: f32 = 30.0;
    const RADIUS: f32 = 10.0 * CELL_SIZE;

    // Find nearest friendly building
    let mut nearest: Option<(Entity, f32)> = None;
    for (entity, building, pos) in buildings.iter() {
        if building.faction == FactionId::Kayi {
            let dist = pos.world_2d.0.distance(caster_pos.world_2d.0);
            if dist <= RADIUS {
                match nearest {
                    Some((_, best)) if dist < best => {
                        nearest = Some((entity, dist));
                    }
                    None => {
                        nearest = Some((entity, dist));
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some((entity, _)) = nearest {
        commands.entity(entity).insert(DiplomaticImmunity {
            remaining: DURATION,
        });
    }

    120.0 // long cooldown
}

/// CulturalRenaissance: +1 skill point to all heroes, +15% XP for 60s.
fn execute_cultural_renaissance(
    heroes: &mut Query<(Entity, &mut Hero, &WorldPosition)>,
    renaissance: &mut ResMut<CulturalRenaissanceBuff>,
) -> f32 {
    const DURATION: f32 = 60.0;

    // Grant +1 skill point to all heroes
    for (_e, mut hero, _pos) in heroes.iter_mut() {
        hero.skill_points += 1;
    }

    // Set the XP buff
    renaissance.remaining = DURATION;
    renaissance.active = true;

    120.0
}

// ─── Cooldown ticking ──────────────────────────────────────────────────────

/// Ticks cooldowns for all hero abilities.
fn cooldown_tick_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut heroes: Query<&mut Hero>,
) {
    for mut hero in heroes.iter_mut() {
        for (_ability_id, cooldown) in hero.active_cooldowns.iter_mut() {
            *cooldown = (*cooldown - time.delta_secs()).max(0.0);
        }
    }
}

// ─── Buff expiry systems ────────────────────────────────────────────────────

/// Ticks the AdministrativeMind production buff and restores rates when expired.
fn admin_mind_tick_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut buff: ResMut<AdministrativeMindBuff>,
    mut productions: ResMut<ProductionRates>,
) {
    if !buff.active {
        return;
    }

    buff.remaining -= time.delta_secs();
    if buff.remaining <= 0.0 {
        // Restore original rates
        productions.food_per_second = buff.original_food_rate;
        productions.gold_per_second = buff.original_gold_rate;
        productions.wood_per_second = buff.original_wood_rate;
        productions.stone_per_second = buff.original_stone_rate;

        buff.active = false;
        buff.remaining = 0.0;
    }
}

/// Ticks the CulturalRenaissance XP buff and deactivates when expired.
fn cultural_renaissance_tick_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut buff: ResMut<CulturalRenaissanceBuff>,
) {
    if !buff.active {
        return;
    }

    buff.remaining -= time.delta_secs();
    if buff.remaining <= 0.0 {
        buff.active = false;
        buff.remaining = 0.0;
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HERO SWITCH / SELECTION
// ═════════════════════════════════════════════════════════════════════════════

/// Handles `HeroSwitchEvent` to cycle the actively controlled hero.
/// Collects all Kayi heroes, sorts by `HeroId`, and cycles next/previous.
fn hero_switch_system(
    mut switch_events: EventReader<HeroSwitchEvent>,
    mut active_hero: ResMut<ActiveHero>,
    heroes: Query<(Entity, &Hero, &Unit)>,
    mut selection_events: EventWriter<SelectionChanged>,
) {
    // Collect all Kayi heroes sorted by HeroId for deterministic cycling
    let mut kayi_heroes: Vec<(Entity, &Hero)> = heroes
        .iter()
        .filter(|(_, _, unit)| unit.faction == FactionId::Kayi)
        .map(|(e, h, _)| (e, h))
        .collect::<Vec<_>>();

    if kayi_heroes.is_empty() {
        return;
    }

    // Sort by entity index for deterministic cycling regardless of spawn order
    kayi_heroes.sort_by_key(|(e, _)| e.to_bits());

    for event in switch_events.read() {
        match event.direction {
            Some(HeroSwitchDirection::Next) => {
                // Find current index and advance
                let current_idx = kayi_heroes
                    .iter()
                    .position(|(e, _)| Some(*e) == active_hero.entity)
                    .unwrap_or(usize::MAX);

                let next_idx = if current_idx == usize::MAX || current_idx + 1 >= kayi_heroes.len() {
                    0 // wrap to first
                } else {
                    current_idx + 1
                };

                let (new_entity, _) = kayi_heroes[next_idx];
                let prev = active_hero.entity;
                active_hero.entity = Some(new_entity);

                selection_events.send(SelectionChanged {
                    selected: vec![new_entity],
                    previous: prev.map_or(vec![], |p| vec![p]),
                });
            }
            Some(HeroSwitchDirection::Previous) => {
                let current_idx = kayi_heroes
                    .iter()
                    .position(|(e, _)| Some(*e) == active_hero.entity)
                    .unwrap_or(0);

                let prev_idx = if current_idx == 0 {
                    kayi_heroes.len() - 1 // wrap to last
                } else {
                    current_idx - 1
                };

                let (new_entity, _) = kayi_heroes[prev_idx];
                let prev = active_hero.entity;
                active_hero.entity = Some(new_entity);

                selection_events.send(SelectionChanged {
                    selected: vec![new_entity],
                    previous: prev.map_or(vec![], |p| vec![p]),
                });
            }
            None => {
                // Direct switch to target entity
                if let Some(target) = event.target {
                    if kayi_heroes.iter().any(|(e, _)| *e == target) {
                        let prev = active_hero.entity;
                        active_hero.entity = Some(target);

                        selection_events.send(SelectionChanged {
                            selected: vec![target],
                            previous: prev.map_or(vec![], |p| vec![p]),
                        });
                    }
                }
            }
        }
    }
}
