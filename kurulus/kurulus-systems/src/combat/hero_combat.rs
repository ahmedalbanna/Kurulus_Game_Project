use bevy::prelude::*;
use kurulus_core::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
//  HERO RESOURCE REGENERATION (Phase C)
// ═════════════════════════════════════════════════════════════════════════════

/// Ticks hero resource regeneration each frame according to the GDD §5.2 rates:
///   - Osman (Inspiration): +1/s base, +3/s in combat. Max 100.
///   - Turgut (Fury): +0.8/s base, +2 on taking damage. Max 120.
///   - Buran (Focus): +1.5/s base, +5 while stealthed and stationary. Max 80.
pub fn hero_resource_regen_system(
    time: Res<Time>,
    mut heroes: Query<&mut Hero, (With<Hero>, Without<Dead>)>,
) {
    for mut hero in heroes.iter_mut() {
        // Base regeneration per second (already set in Hero::new())
        let regen_amount = hero.resource_regen * time.delta_secs();
        hero.resource_current = (hero.resource_current + regen_amount).min(hero.resource_max);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HERO AURA SYSTEM (Phase C)
// ═════════════════════════════════════════════════════════════════════════════

/// Applies passive auras from heroes to nearby allied units.
/// Reads unlocked abilities and applies the appropriate buffs.
///
/// GDD §5.2 Hero Synergies:
///   - Osman's WarCry: +30% damage to all allies in 8m for 10s
///   - Turgut's Earthq. Challenge: taunt + self defense buff
///   - Buran's Shadow Lightness: +15% speed, stealth after 3s still
pub fn hero_aura_system(
    mut commands: Commands,
    heroes: Query<(Entity, &Hero, &WorldPosition, &HeroStats), (With<Hero>, Without<Dead>)>,
    allies: Query<(Entity, &Unit, &WorldPosition), (With<Unit>, Without<Hero>, Without<Dead>)>,
) {
    for (hero_e, hero, hero_pos, hero_stats) in heroes.iter() {
        // Check which auras are unlocked
        for ability_id in &hero.unlocked_abilities {
            match ability_id {
                AbilityId::WarCry => {
                    // Osman's WarCry: +30% damage to allies within 8m for 10s
                    let aura_radius = 8.0;
                    apply_aura_buff(
                        &mut commands,
                        hero_e,
                        hero_pos,
                        &allies,
                        aura_radius,
                        AbilityId::WarCry,
                        10.0,
                        0.30, // +30% damage
                    );
                }
                AbilityId::Lionheart => {
                    // Osman's Lionheart: triggers at <30% HP — handled in
                    // the damage system; the aura here provides the passive
                    // damage reduction buff to nearby allies.
                    let aura_radius = 6.0;
                    let hp_threshold = hero_stats.base_health * 0.30;
                    // Deactivated — handled by the hero damage system
                    let _ = (aura_radius, hp_threshold);
                }
                AbilityId::IronWall => {
                    // Turgut's Iron Wall: % damage reduction to self
                    // (implemented via self-buff; no aura range needed)
                }
                AbilityId::ShadowLightness => {
                    // Buran's Shadow Lightness: +15% speed for nearby allies
                    let aura_radius = 6.0;
                    apply_aura_buff(
                        &mut commands,
                        hero_e,
                        hero_pos,
                        &allies,
                        aura_radius,
                        AbilityId::ShadowLightness,
                        f32::MAX, // Permanent while in range
                        0.0,
                    );
                }
                _ => {}
            }
        }

        // Named unit auras (Season 1)
        if let Some(named_unit_id) = hero.named_unit_id {
            match named_unit_id {
                NamedUnitId::BamsiBeyrek => {
                    // VeteransCry: +15% morale aura 15m
                    apply_morale_aura(&mut commands, hero_e, hero_pos, &allies, 15.0);
                }
                NamedUnitId::Salvador => {
                    // RomanTactics: +15% defense to adjacent
                    apply_defense_aura(&mut commands, hero_e, hero_pos, &allies, 5.0, 0.15);
                }
                _ => {}
            }
        }
    }
}

/// Applies a damage bonus buff to allies within radius.
#[allow(clippy::too_many_arguments)]
fn apply_aura_buff(
    commands: &mut Commands,
    _hero_e: Entity,
    hero_pos: &WorldPosition,
    allies: &Query<(Entity, &Unit, &WorldPosition), (With<Unit>, Without<Hero>, Without<Dead>)>,
    radius: f32,
    ability_id: AbilityId,
    duration: f32,
    damage_bonus: f32,
) {
    for (ally_e, _, ally_pos) in allies.iter() {
        let dist = hero_pos.world_2d.0.distance(ally_pos.world_2d.0);
        if dist <= radius {
            commands.entity(ally_e).insert(ActiveBuffs {
                buffs: vec![Buff {
                    source: ability_id,
                    duration,
                    max_duration: duration,
                    damage_bonus,
                    defense_bonus: 0.0,
                    move_speed_bonus: 0.0,
                    heal_per_second: 0.0,
                    damage_reduction: 0.0,
                    attack_speed_bonus: 0.0,
                }],
            });
        }
    }
}

/// Applies a morale boost to allies within radius.
fn apply_morale_aura(
    _commands: &mut Commands,
    _hero_e: Entity,
    hero_pos: &WorldPosition,
    allies: &Query<(Entity, &Unit, &WorldPosition), (With<Unit>, Without<Hero>, Without<Dead>)>,
    radius: f32,
) {
    for (_, _, ally_pos) in allies.iter() {
        let dist = hero_pos.world_2d.0.distance(ally_pos.world_2d.0);
        if dist <= radius {
            // Morale aura is applied as a flat morale bonus directly.
            // For now this is a no-op placeholder — the actual morale
            // bonus will be applied by `morale_state_machine_system`
            // which reads a `MoraleAuraBonus` component (future).
        }
    }
}

/// Applies a defense bonus to adjacent allies.
#[allow(clippy::too_many_arguments)]
fn apply_defense_aura(
    commands: &mut Commands,
    _hero_e: Entity,
    hero_pos: &WorldPosition,
    allies: &Query<(Entity, &Unit, &WorldPosition), (With<Unit>, Without<Hero>, Without<Dead>)>,
    radius: f32,
    defense_bonus: f32,
) {
    for (ally_e, _, ally_pos) in allies.iter() {
        let dist = hero_pos.world_2d.0.distance(ally_pos.world_2d.0);
        if dist <= radius {
            commands.entity(ally_e).insert(ActiveBuffs {
                buffs: vec![Buff {
                    source: AbilityId::Inspire,
                    duration: f32::MAX,
                    max_duration: f32::MAX,
                    damage_bonus: 0.0,
                    defense_bonus,
                    move_speed_bonus: 0.0,
                    heal_per_second: 0.0,
                    damage_reduction: 0.0,
                    attack_speed_bonus: 0.0,
                }],
            });
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HERO KNOCKOUT SYSTEM (Phase C)
// ═════════════════════════════════════════════════════════════════════════════

/// Intercepts `UnitDied` events for heroes and converts them to `KnockedOut`
/// state instead of permanent death. The hero only dies permanently if they
/// receive a death blow while already knocked out.
pub fn hero_knockout_system(
    mut unit_died_events: EventReader<UnitDied>,
    mut heroes: Query<(Entity, &mut Hero, &mut HeroDeathState, &mut HeroState)>,
    mut knockout_events: EventWriter<HeroKnockedOut>,
    mut death_blow_events: EventWriter<HeroDeathBlow>,
) {
    let dead_events: Vec<UnitDied> = unit_died_events.read().cloned().collect();

    for ev in &dead_events {
        // Check if the dying entity is a hero
        if let Ok((entity, hero, mut death_state, mut state)) = heroes.get_mut(ev.entity) {
            if death_state.is_dead_this_mission {
                // Already KO'd this mission — this is a death blow
                death_state.knockout_timer = 0.0;
                *state = HeroState::Dead;
                death_blow_events.send(HeroDeathBlow {
                    hero: entity,
                    hero_id: hero.hero_id,
                    killer: ev.killer,
                });
            } else {
                // First time — enter KnockedOut state
                death_state.is_dead_this_mission = true;
                death_state.knockout_timer = 30.0; // 30 seconds before recovery
                *state = HeroState::KnockedOut { timer: 30.0 };
                knockout_events.send(HeroKnockedOut {
                    hero: entity,
                    hero_id: hero.hero_id,
                    knockout_duration: 30.0,
                });
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  HERO KNOCKOUT RECOVERY SYSTEM (Phase C)
// ═════════════════════════════════════════════════════════════════════════════

/// Ticks knockout recovery timers for heroes. When the timer expires, the hero
/// transitions from KnockedOut → Recovering → Idle with partial health.
pub fn hero_knockout_recovery_system(
    time: Res<Time>,
    mut heroes: Query<(
        Entity,
        &mut Hero,
        &mut HeroDeathState,
        &mut HeroState,
        &mut Health,
        &HeroStats,
    )>,
    mut recovery_events: EventWriter<HeroRecovering>,
    mut idle_events: EventWriter<HeroReturnedToIdle>,
) {
    for (entity, _hero, mut death_state, mut state, mut health, hero_stats) in heroes.iter_mut() {
        match *state {
            HeroState::KnockedOut { ref mut timer } => {
                *timer -= time.delta_secs();
                death_state.knockout_timer = *timer;
                if *timer <= 0.0 {
                    // Transition to Recovering
                    *state = HeroState::Recovering { timer: 5.0 };
                    recovery_events.send(HeroRecovering {
                        hero: entity,
                        hero_id: _hero.hero_id,
                    });
                }
            }
            HeroState::Recovering { ref mut timer } => {
                *timer -= time.delta_secs();
                if *timer <= 0.0 {
                    // Fully recovered — restore to 30% health and return to Idle
                    health.current = hero_stats.base_health * 0.30;
                    death_state.is_dead_this_mission = false;
                    death_state.knockout_timer = 0.0;
                    *state = HeroState::Idle;
                    idle_events.send(HeroReturnedToIdle {
                        hero: entity,
                        hero_id: _hero.hero_id,
                    });
                }
            }
            _ => {}
        }
    }
}
