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
            // Startup: spawn the M1 scenario once
            .add_systems(Startup, setup_mission1_scenario)
            .add_systems(
                FixedUpdate,
                (
                    // Update objective progress based on game events
                    track_build_objective_system,
                    track_recruit_objective_system,
                    track_kill_objective_system,
                    track_camera_objective_system,
                    // Check if objectives are now complete
                    mission_objective_check_system,
                    // Check win/loss conditions
                    mission_win_loss_system,
                    // Dialogue triggers
                    dialogue_trigger_system,
                    // Mission state machine
                    mission_state_machine_system,
                ),
            );
    }
}

// ─── M1 Scenario setup ─────────────────────────────────────────────────────

/// Spawns the Mission 1 playable scenario with Osman, his camp, and enemies.
fn setup_mission1_scenario(mut commands: Commands) {
    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 1,
            xp: 0,
            skill_points: 1,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::SovereignStrike,
                AbilityId::LionsHeart,
                AbilityId::TreesVision,
            ],
            active_cooldowns: std::collections::HashMap::new(),
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 30.0,
            attack_speed: 1.2,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 30.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.3,
        },
        AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![
                SkillNode {
                    id: SkillNodeId::SwordMastery,
                    name: "Sword Mastery".into(),
                    description: "Unlocks SovereignStrike".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: false,
                },
                SkillNode {
                    id: SkillNodeId::LeaderAura,
                    name: "Leader's Aura".into(),
                    description: "Unlocks WarCry".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: false,
                },
                SkillNode {
                    id: SkillNodeId::TribalUnity,
                    name: "Tribal Unity".into(),
                    description: "Unlocks LionsHeart".into(),
                    cost: 1,
                    prerequisites: vec![SkillNodeId::SwordMastery],
                    unlocked: false,
                },
                SkillNode {
                    id: SkillNodeId::Sovereignty,
                    name: "Sovereignty".into(),
                    description: "Unlocks Trees' Vision".into(),
                    cost: 2,
                    prerequisites: vec![SkillNodeId::LeaderAura],
                    unlocked: false,
                },
            ],
            available_points: 1,
        },
    ));

    // 3 Kayi Light Infantry
    for i in 0..3 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(4 + i as i32, 4), CELL_SIZE),
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.5,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 1.0,
                vision_range: 6.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Sword,
                range: 1.5,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    // 1 Worker
    commands.spawn((
        Unit {
            archetype: UnitArchetype::Worker,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(3, 6), CELL_SIZE),
        Health {
            current: 50.0,
            max: 50.0,
            armor: 2.0,
            armor_type: ArmorType::Light,
        },
        Stats {
            move_speed: 3.0,
            attack_range: 1.0,
            attack_damage: 3.0,
            attack_speed: 0.5,
            vision_range: 4.0,
            morale: 30.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 3.0,
            weapon_type: WeaponType::Unarmed,
            range: 1.0,
            armor_penetration: 0.0,
        },
        AttackTimer(Timer::from_seconds(2.0, TimerMode::Repeating)),
    ));

    // Player Oba (Level 1)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 5,
            name: "Kayı Obağı".into(),
        },
        WorldPosition::new(IVec2::new(5, 7), CELL_SIZE),
        Health {
            current: 500.0,
            max: 500.0,
            armor: 20.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Byzantine) ────────────────────────────────────

    // 5 Byzantine Light Infantry
    for i in 0..5 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(18 + i as i32, 5 + (i % 2) as i32), CELL_SIZE),
            Health {
                current: 80.0,
                max: 80.0,
                armor: 3.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 10.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 40.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 10.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(1.1, TimerMode::Repeating)),
        ));
    }

    // 1 Byzantine Archer
    commands.spawn((
        Unit {
            archetype: UnitArchetype::Archer,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(20, 7), CELL_SIZE),
        Health {
            current: 60.0,
            max: 60.0,
            armor: 2.0,
            armor_type: ArmorType::Light,
        },
        Stats {
            move_speed: 3.0,
            attack_range: 6.0,
            attack_damage: 15.0,
            attack_speed: 0.7,
            vision_range: 8.0,
            morale: 40.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 15.0,
            weapon_type: WeaponType::Bow,
            range: 6.0,
            armor_penetration: 0.0,
        },
        AttackTimer(Timer::from_seconds(1.4, TimerMode::Repeating)),
    ));

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::TutorialMoveCamera,
            objective_type: ObjectiveType::ReachLocation,
            description: "Pan the camera to explore the camp".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 5,
                xp: 10,
                sovereignty_points: 0,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::TutorialBuildOba,
            objective_type: ObjectiveType::Build,
            description: "Build a Pasture near the Oba".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 10,
                xp: 20,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::TutorialRecruitUnit,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit a Light Infantry from the Oba".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 10,
                xp: 20,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::TutorialAttackEnemy,
            objective_type: ObjectiveType::Kill,
            description: "Defeat the Byzantine camp".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 30,
                xp: 50,
                sovereignty_points: 3,
                item_unlock: None,
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission01,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: None,
        time_limit: None,
    });

    // MissionStarted will be sent on the first frame by the state machine
    // when it detects a freshly spawned mission.
}

// ─── Objective progress tracking ───────────────────────────────────────────

/// TutorialBuildOba: progress = 1 when a Pasture is built by the player.
fn track_build_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    construction_sites: Query<&ConstructionSite>,
) {
    for mut mission in missions.iter_mut() {
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        // Check objective type *before* borrowing the field
        let is_target = mission.objectives[idx].objective_id == ObjectiveId::TutorialBuildOba
            && !mission.objectives[idx].completed;
        if !is_target {
            continue;
        }
        // Simplified: if no construction sites exist, building is done
        if construction_sites.iter().count() == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// TutorialRecruitUnit: progress = 1 when at least one Kayi unit exists
/// beyond the starting units (i.e., any unit with a count > starting units).
fn track_recruit_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    let kayi_count = units
        .iter()
        .filter(|u| u.faction == FactionId::Kayi)
        .count();
    let recruited = kayi_count > 5; // starting: 1 Osman + 3 Inf + 1 Worker = 5

    if !recruited {
        return;
    }

    for mut mission in missions.iter_mut() {
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id == ObjectiveId::TutorialRecruitUnit
            && !mission.objectives[idx].completed
        {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// TutorialAttackEnemy: progress = 1 when all Byzantine units are dead.
fn track_kill_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit, Without<Dead>>,
) {
    let byz_alive = units
        .iter()
        .filter(|u| u.faction == FactionId::Byzantine)
        .count();
    let all_dead = byz_alive == 0;

    if !all_dead {
        return;
    }

    for mut mission in missions.iter_mut() {
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id == ObjectiveId::TutorialAttackEnemy
            && !mission.objectives[idx].completed
        {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// TutorialMoveCamera: progress = 1 on first frame (simulated — a real game
/// would wait for camera pan input).
fn track_camera_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    mut first_frame: Local<bool>,
) {
    if *first_frame {
        return;
    }
    *first_frame = true;

    for mut mission in missions.iter_mut() {
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id == ObjectiveId::TutorialMoveCamera
            && !mission.objectives[idx].completed
        {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ─── Objective completion checking ─────────────────────────────────────────

/// Checks mission objectives for completion.
fn mission_objective_check_system(
    mut objective_events: EventWriter<ObjectiveCompleted>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    // Use a Vec to defer event sending (avoids borrow conflicts)
    let mut completed: Vec<(ObjectiveId, MissionId)> = Vec::new();

    // SAFETY: we are iterating mutable references but not aliasing.
    // We collect mutation targets first, then apply.
    for mut mission in missions.iter_mut() {
        let idx = mission.current_objective_index;
        if idx >= mission.objectives.len() {
            continue;
        }
        let should_complete = mission.objectives[idx].progress
            >= mission.objectives[idx].required_progress
            && !mission.objectives[idx].completed;

        if should_complete {
            let obj_id = mission.objectives[idx].objective_id;
            let mis_id = mission.mission_id;
            mission.objectives[idx].completed = true;
            mission.current_objective_index =
                (idx + 1).min(mission.objectives.len());
            completed.push((obj_id, mis_id));
        }
    }

    for (obj_id, mis_id) in completed {
        objective_events.send(ObjectiveCompleted {
            objective_id: obj_id,
            mission_id: mis_id,
        });
    }
}

// ─── Win / Loss ─────────────────────────────────────────────────────────────

/// Checks win/loss conditions for Mission 1.
fn mission_win_loss_system(
    heroes: Query<&Hero>,
    _units: Query<(&Unit, Option<&Dead>)>,
    buildings: Query<(&Building, Option<&Dead>)>,
    mut end_events: EventWriter<MissionEnded>,
    missions: Query<&MissionStateComponent>,
) {
    let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
    let oba_alive = buildings.iter().any(|(b, dead)| {
        b.building_type == BuildingType::Oba && dead.is_none()
    });
    let osman_dead = !osman_alive;
    let oba_destroyed = !oba_alive;

    for mission in missions.iter() {
        // Loss: Osman dead or Oba destroyed
        if osman_dead || oba_destroyed {
            end_events.send(MissionEnded {
                mission_id: mission.mission_id,
                success: false,
            });
            continue;
        }

        // Win: all objectives completed
        if mission.objectives.iter().all(|o| o.completed) {
            end_events.send(MissionEnded {
                mission_id: mission.mission_id,
                success: true,
            });
        }
    }
}

// ─── Dialogue triggers ─────────────────────────────────────────────────────

/// Triggers dialogue based on game events.
fn dialogue_trigger_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut started_events: EventReader<MissionStarted>,
    mut dialogue_events: EventWriter<DialogueTriggered>,
) {
    for _event in started_events.read() {
        dialogue_events.send(DialogueTriggered {
            dialogue_id: "mission1_intro".into(),
            trigger_type: kurulus_core::components::DialogueTrigger::OnMissionStart,
        });
    }
    for event in objective_events.read() {
        dialogue_events.send(DialogueTriggered {
            dialogue_id: format!("obj_complete_{:?}", event.objective_id),
            trigger_type: kurulus_core::components::DialogueTrigger::OnObjectiveComplete,
        });
    }
}

// ─── Mission state machine ─────────────────────────────────────────────────

/// Manages mission state machine (Loading -> Active -> Complete/Failed).
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
