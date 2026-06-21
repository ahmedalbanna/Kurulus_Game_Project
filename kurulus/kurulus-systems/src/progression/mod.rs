use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                research_progress_system.in_set(SimulationSet::ResearchProgress),
                xp_gain_progression_system.in_set(SimulationSet::XPGain),
                sovereignty_point_system,
            ),
        );
    }
}

/// Progresses active research projects
fn research_progress_system(time: Res<GameTime>, mut tech_tree: ResMut<TechTree>) {
    if let Some(ref mut project) = tech_tree.current_research {
        project.progress += time.time_scale * 5.0;
        if project.progress >= project.max_progress {
            let tech_id = project.tech_id.clone();
            // Research complete
            if let Some(node) = tech_tree.techs.iter_mut().find(|t| t.id == tech_id) {
                node.unlocked = true;
            }
            tech_tree.current_research = None;
        }
    }
}

/// Awards XP for various game actions
fn xp_gain_progression_system(
    _death_events: EventReader<UnitDied>,
    _objective_events: EventReader<ObjectiveCompleted>,
    _heroes: Query<&mut Hero>,
) {
    // TODO: Implement XP tracking
}

/// Manages sovereignty points (meta-progression currency)
fn sovereignty_point_system(
    _objective_events: EventReader<ObjectiveCompleted>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    // TODO: Implement sovereignty point awards
}
