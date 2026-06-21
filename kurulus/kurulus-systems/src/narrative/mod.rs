use bevy::prelude::*;
use bevy_state::app::AppExtStates;
use kurulus_core::components::MissionStateComponent;
use kurulus_core::prelude::*;
use kurulus_core::states::MissionState as MissionStateEnum;

pub struct NarrativePlugin;

impl Plugin for NarrativePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<MissionStateEnum>()
            .add_event::<MissionStarted>()
            .add_event::<MissionEnded>()
            .add_systems(
                FixedUpdate,
                (
                    mission_objective_check_system,
                    dialogue_trigger_system,
                    mission_state_machine_system,
                ),
            );
    }
}

/// Checks mission objectives for completion
fn mission_objective_check_system(
    mut objective_events: EventWriter<ObjectiveCompleted>,
    missions: Query<&MissionStateComponent>,
) {
    for mission in missions.iter() {
        if let Some(objective) = mission.objectives.get(mission.current_objective_index) {
            if objective.progress >= objective.required_progress && !objective.completed {
                objective_events.send(ObjectiveCompleted {
                    objective_id: objective.objective_id,
                    mission_id: mission.mission_id,
                });
            }
        }
    }
}

/// Triggers dialogue based on game events
fn dialogue_trigger_system(
    _objective_events: EventReader<ObjectiveCompleted>,
    _dialogue_events: EventWriter<DialogueTriggered>,
) {
    // TODO: Implement dialogue triggering
}

/// Manages mission state machine (Loading -> Active -> Complete/Failed)
fn mission_state_machine_system(
    mut next_mission_state: ResMut<NextState<MissionStateEnum>>,
    _mission_state: Res<State<MissionStateEnum>>,
    mut started_events: EventReader<MissionStarted>,
    mut end_events: EventReader<MissionEnded>,
) {
    for _event in started_events.read() {
        next_mission_state.set(MissionStateEnum::Active);
    }
    for event in end_events.read() {
        if event.success {
            next_mission_state.set(MissionStateEnum::Complete);
        } else {
            next_mission_state.set(MissionStateEnum::Failed);
        }
    }
}
