use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct BaseBuildingPlugin;

impl Plugin for BaseBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                construction_progress_system.in_set(SimulationSet::ConstructionProgress),
                building_upgrade_system,
                unit_recruitment_system,
            ),
        );
    }
}

/// Progresses construction of buildings over time
fn construction_progress_system(
    time: Res<GameTime>,
    mut sites: Query<(&mut ConstructionSite, &BuildCost)>,
    _stockpile: ResMut<ResourceStockpile>,
) {
    for (mut site, _cost) in sites.iter_mut() {
        if site.progress < site.max_progress {
            site.progress += time.time_scale * 10.0;
        }
    }
}

/// Handles building level upgrades
fn building_upgrade_system(
    _commands: Commands,
    _buildings: Query<&mut Building>,
    _stockpile: Res<ResourceStockpile>,
) {
    // TODO: Implement building upgrades
}

/// Processes recruitment orders from buildings
fn unit_recruitment_system(
    _recruit_events: EventReader<RecruitOrder>,
    _buildings: Query<&Building>,
    _commands: Commands,
) {
    // TODO: Implement unit recruitment
}
