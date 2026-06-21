use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct HeroPlugin;

impl Plugin for HeroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                hero_ability_system,
                cooldown_tick_system,
                xp_gain_system,
                skill_tree_system,
            ),
        );
    }
}

/// Activates hero abilities based on AbilityUsed events
fn hero_ability_system(
    _commands: Commands,
    mut ability_events: EventReader<AbilityUsed>,
    mut heroes: Query<&mut Hero>,
    _abilities: Query<&Ability>,
) {
    for event in ability_events.read() {
        if let Ok(mut hero) = heroes.get_mut(event.caster) {
            // Check cooldowns
            if let Some(cooldown) = hero.active_cooldowns.get(&event.ability_id) {
                if *cooldown > 0.0 {
                    continue;
                }
            }
            // Apply ability effect
            hero.active_cooldowns.insert(event.ability_id, 0.0);
        }
    }
}

/// Ticks cooldowns for all hero abilities
fn cooldown_tick_system(time: Res<bevy::time::Time>, mut heroes: Query<&mut Hero>) {
    for mut hero in heroes.iter_mut() {
        for (_ability_id, cooldown) in hero.active_cooldowns.iter_mut() {
            *cooldown = (*cooldown - time.delta_secs()).max(0.0);
        }
    }
}

/// Grants XP to heroes for various actions
fn xp_gain_system(
    _death_events: EventReader<UnitDied>,
    _objective_events: EventReader<ObjectiveCompleted>,
    _heroes: Query<&mut Hero>,
) {
    // TODO: Implement XP gain
}

/// Manages skill tree progression
fn skill_tree_system(_heroes: Query<(&Hero, &mut SkillTree)>) {
    // TODO: Implement skill tree UI interaction with optional input
}
