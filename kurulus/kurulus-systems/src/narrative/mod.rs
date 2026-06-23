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
                    // ── M1 tutorial objective trackers ─────────────────
                    track_m1_build_objective_system,
                    track_m1_recruit_objective_system,
                    track_m1_kill_objective_system,
                    track_m1_camera_objective_system,
                    // ── M2 objective trackers ──────────────────────────
                    track_m2_build_barracks_objective_system,
                    track_m2_recruit_infantry_objective_system,
                    track_m2_destroy_watchtower_objective_system,
                    track_m2_capture_region_objective_system,
                    // ── M3 objective trackers ──────────────────────────
                    track_m3_survive_wave_objective_system,
                    track_m3_recruit_mounted_objective_system,
                    track_m3_defeat_mongols_objective_system,
                    // ── M4 objective trackers ──────────────────────────
                    track_m4_survive_wave_objective_system,
                    track_m4_destroy_sentry_objective_system,
                    track_m4_rescue_ally_objective_system,
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    // ── Post-mission rewards (applied before campaign save) ──
                    apply_mission_rewards_system,
                    // ── M5 objective trackers ──────────────────────────
                    track_m5_build_barracks_objective_system,
                    track_m5_recruit_infantry_objective_system,
                    track_m5_destroy_watchtower_objective_system,
                    track_m5_kill_enemy_hero_objective_system,
                    // ── M6 objective trackers ──────────────────────────
                    track_m6_kill_enemy_hero_objective_system,
                    track_m6_survive_wave_objective_system,
                    track_m6_capture_region_objective_system,
                    // ── Campaign persistence ───────────────────────────
                    campaign_save_system,
                    campaign_restore_system,
                    // ── Shared systems ─────────────────────────────────
                    mission_objective_check_system,
                    mission_win_loss_system,
                    dialogue_trigger_system,
                    mission_intro_activator_system,
                    mission_state_machine_system,
                ),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  M4 — The Mongol Scouts (Chapter 2 — first Mongol encounter)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 4 playable scenario: Mongol scout patrol spotted near the settlement.
/// Alaeddin joins as a playable hero. Stealth-focused mission.
pub fn setup_mission4_scenario(commands: &mut Commands) {
    use std::collections::HashMap;

    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero, level 3)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 3,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        Health {
            current: 300.0,
            max: 300.0,
            armor: 15.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.4,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 40.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.4,
        },
        AttackTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![],
            available_points: 2,
        },
    ));

    // Alaeddin Bey (hero, level 1 — joins in this mission)
    commands.spawn((
        Hero {
            hero_id: HeroId::Alaeddin,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![AbilityId::AdministrativeMind, AbilityId::TaxReform],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(6, 6), CELL_SIZE),
        Health {
            current: 150.0,
            max: 150.0,
            armor: 6.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 18.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 80.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 18.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.2,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Alaeddin,
            nodes: vec![],
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
            WorldPosition::new(IVec2::new(4 + i as i32, 7), CELL_SIZE),
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
        WorldPosition::new(IVec2::new(3, 8), CELL_SIZE),
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

    // Player Oba (Level 2)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 2,
            max_level: 5,
            name: "Kayı Obağı — Frontier Outpost".into(),
        },
        WorldPosition::new(IVec2::new(5, 9), CELL_SIZE),
        Health {
            current: 750.0,
            max: 750.0,
            armor: 25.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // Stable (for mounted unit recruitment later)
    commands.spawn((
        Building {
            building_type: BuildingType::Stable,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: "Kayı Stable".into(),
        },
        WorldPosition::new(IVec2::new(7, 9), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Mongol scouts) ────────────────────────────────

    // 5 Mongol Light Cavalry (scout patrol)
    for i in 0..5 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Mongol,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(
                IVec2::new(20 + (i % 3) as i32, 4 + (i / 3) as i32 * 3),
                CELL_SIZE,
            ),
            Health {
                current: 70.0,
                max: 70.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 6.0,
                attack_range: 2.0,
                attack_damage: 18.0,
                attack_speed: 1.3,
                vision_range: 8.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 18.0,
                weapon_type: WeaponType::Spear,
                range: 2.0,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
        ));
    }

    // Mongol Sentry Post (Watchtower — DestroyWall objective)
    commands.spawn((
        Building {
            building_type: BuildingType::Watchtower,
            faction: FactionId::Mongol,
            level: 1,
            max_level: 1,
            name: "Mongol Sentry Post".into(),
        },
        WorldPosition::new(IVec2::new(22, 6), CELL_SIZE),
        Health {
            current: 250.0,
            max: 250.0,
            armor: 25.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Rescue target ───────────────────────────────────────────────

    // Captured Kayi scout (RescueAlly objective)
    commands.spawn((
        RescueTarget { extracted: false },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(21, 5), CELL_SIZE), // near Mongol camp
        Health {
            current: 40.0,
            max: 40.0,
            armor: 2.0,
            armor_type: ArmorType::Light,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 8.0,
            attack_speed: 0.8,
            vision_range: 5.0,
            morale: 20.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 8.0,
            weapon_type: WeaponType::Unarmed,
            range: 1.0,
            armor_penetration: 0.0,
        },
        AttackTimer(Timer::from_seconds(1.5, TimerMode::Repeating)),
    ));

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive the Mongol scout assault for 30 seconds".into(),
            progress: 0.0,
            required_progress: 30.0,
            completed: false,
            rewards: MissionRewards {
                gold: 15,
                xp: 25,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::DestroyWall,
            objective_type: ObjectiveType::Destroy,
            description: "Destroy the Mongol Sentry Post".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 35,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RescueAlly,
            objective_type: ObjectiveType::Rescue,
            description: "Rescue the captured Kayi scout".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 25,
                xp: 40,
                sovereignty_points: 3,
                item_unlock: Some("Mongol_Scout_Map".into()),
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission04,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(300.0), // 5-minute time limit
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  M5 — Siege of Kulucahisar (Chapter 3 — full base-building siege)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 5 playable scenario: besiege the Byzantine fort at Kulucahisar.
pub fn setup_mission5_scenario(commands: &mut Commands) {
    use std::collections::HashMap;

    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero, level 3)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 3,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        Health {
            current: 350.0,
            max: 350.0,
            armor: 18.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 45.0,
            attack_speed: 1.4,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 45.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.4,
        },
        AttackTimer(Timer::from_seconds(0.65, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![],
            available_points: 3,
        },
    ));

    // Orhan Bey (hero, level 2)
    commands.spawn((
        Hero {
            hero_id: HeroId::Orhan,
            level: 2,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::TacticalGenius,
                AbilityId::ShieldWall,
                AbilityId::CavalryCharge,
                AbilityId::FortressBuilder,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(3, 5), CELL_SIZE),
        Health {
            current: 220.0,
            max: 220.0,
            armor: 10.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 30.0,
            attack_speed: 1.2,
            vision_range: 10.0,
            morale: 95.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 30.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.3,
        },
        AttackTimer(Timer::from_seconds(0.85, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Orhan,
            nodes: vec![],
            available_points: 2,
        },
    ));

    // 5 Kayi Light Infantry
    for i in 0..5 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(3 + i as i32, 6), CELL_SIZE),
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
        WorldPosition::new(IVec2::new(2, 8), CELL_SIZE),
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

    // Player Oba (Level 3)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 3,
            max_level: 5,
            name: "Kayı Obağı — Siege Camp".into(),
        },
        WorldPosition::new(IVec2::new(5, 8), CELL_SIZE),
        Health {
            current: 1000.0,
            max: 1000.0,
            armor: 30.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Byzantine Kulucahisar garrison) ────────────────

    // 8 Byzantine Infantry (garrison)
    for i in 0..8 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(
                IVec2::new(20 + (i % 4) as i32, 4 + (i / 4) as i32 * 3),
                CELL_SIZE,
            ),
            Health {
                current: 90.0,
                max: 90.0,
                armor: 5.0,
                armor_type: ArmorType::Medium,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.15,
            },
            AttackTimer(Timer::from_seconds(1.1, TimerMode::Repeating)),
        ));
    }

    // 2 Byzantine Archers (on the wall)
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::Archer,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(22, 8 + i as i32), CELL_SIZE),
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 7.0,
                attack_damage: 15.0,
                attack_speed: 0.7,
                vision_range: 9.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Bow,
                range: 7.0,
                armor_penetration: 0.0,
            },
            AttackTimer(Timer::from_seconds(1.4, TimerMode::Repeating)),
        ));
    }

    // Byzantine Captain (the "enemy hero" — KillEnemyHero objective target)
    commands.spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(21, 5), CELL_SIZE),
        Health {
            current: 250.0,
            max: 250.0,
            armor: 20.0,
            armor_type: ArmorType::Heavy,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 2.0,
            attack_damage: 30.0,
            attack_speed: 1.1,
            vision_range: 8.0,
            morale: 90.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 30.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.35,
        },
        AttackTimer(Timer::from_seconds(0.9, TimerMode::Repeating)),
    ));

    // Byzantine Watchtower (DestroyWall objective target)
    commands.spawn((
        Building {
            building_type: BuildingType::Watchtower,
            faction: FactionId::Byzantine,
            level: 1,
            max_level: 1,
            name: "Kulucahisar Watchtower".into(),
        },
        WorldPosition::new(IVec2::new(23, 6), CELL_SIZE),
        Health {
            current: 400.0,
            max: 400.0,
            armor: 35.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::BuildBarracks,
            objective_type: ObjectiveType::Build,
            description: "Construct a Barracks near the siege camp".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 15,
                xp: 25,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RecruitTenUnits,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit 5 Light Infantry from the Barracks".into(),
            progress: 0.0,
            required_progress: 5.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 30,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::DestroyWall,
            objective_type: ObjectiveType::Destroy,
            description: "Destroy the Kulucahisar Watchtower".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 30,
                xp: 45,
                sovereignty_points: 2,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::KillEnemyHero,
            objective_type: ObjectiveType::Kill,
            description: "Kill the Byzantine Captain commanding the fort".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 60,
                xp: 100,
                sovereignty_points: 4,
                item_unlock: Some("Kulucahisar_Key".into()),
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission05,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(480.0), // 8-minute time limit
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  M6 — Field of Blood (Chapter 3 — culmination battle)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 6 playable scenario: the decisive battle against the Byzantine army.
/// All 3 heroes (Osman, Orhan, Alaeddin) are playable.
pub fn setup_mission6_scenario(commands: &mut Commands) {
    use std::collections::HashMap;

    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero, level 4)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 4,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 5), CELL_SIZE),
        Health {
            current: 400.0,
            max: 400.0,
            armor: 20.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 50.0,
            attack_speed: 1.5,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 50.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.45,
        },
        AttackTimer(Timer::from_seconds(0.6, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![],
            available_points: 4,
        },
    ));

    // Orhan Bey (hero, level 3)
    commands.spawn((
        Hero {
            hero_id: HeroId::Orhan,
            level: 3,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::TacticalGenius,
                AbilityId::ShieldWall,
                AbilityId::CavalryCharge,
                AbilityId::FortressBuilder,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        Health {
            current: 280.0,
            max: 280.0,
            armor: 14.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 35.0,
            attack_speed: 1.3,
            vision_range: 10.0,
            morale: 95.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 35.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.35,
        },
        AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Orhan,
            nodes: vec![],
            available_points: 3,
        },
    ));

    // Alaeddin Bey (hero, level 2)
    commands.spawn((
        Hero {
            hero_id: HeroId::Alaeddin,
            level: 2,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::AdministrativeMind,
                AbilityId::TaxReform,
                AbilityId::DiplomaticImmunity,
                AbilityId::CulturalRenaissance,
            ],
            active_cooldowns: HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Defensive,
        },
        WorldPosition::new(IVec2::new(6, 6), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 1.5,
            attack_damage: 22.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 85.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 22.0,
            weapon_type: WeaponType::Sword,
            range: 1.5,
            armor_penetration: 0.25,
        },
        AttackTimer(Timer::from_seconds(0.95, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Alaeddin,
            nodes: vec![],
            available_points: 2,
        },
    ));

    // 8 Kayi Light Infantry
    for i in 0..8 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(
                IVec2::new(4 + (i % 4) as i32, 7 + (i / 4) as i32),
                CELL_SIZE,
            ),
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

    // 2 Kayi Light Cavalry
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(3 + i as i32 * 2, 9), CELL_SIZE),
            Health {
                current: 100.0,
                max: 100.0,
                armor: 4.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 6.0,
                attack_range: 2.0,
                attack_damage: 15.0,
                attack_speed: 1.2,
                vision_range: 7.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Spear,
                range: 2.0,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(0.9, TimerMode::Repeating)),
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
        WorldPosition::new(IVec2::new(2, 10), CELL_SIZE),
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

    // Player Oba (Level 3)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 3,
            max_level: 5,
            name: "Kayı Obağı — Field HQ".into(),
        },
        WorldPosition::new(IVec2::new(6, 10), CELL_SIZE),
        Health {
            current: 1000.0,
            max: 1000.0,
            armor: 30.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Byzantine army) ────────────────────────────────

    // 12 Byzantine Infantry
    for i in 0..12 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(
                IVec2::new(22 + (i % 4) as i32, 3 + (i / 4) as i32 * 3),
                CELL_SIZE,
            ),
            Health {
                current: 90.0,
                max: 90.0,
                armor: 5.0,
                armor_type: ArmorType::Medium,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 65.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.15,
            },
            AttackTimer(Timer::from_seconds(1.1, TimerMode::Repeating)),
        ));
    }

    // 4 Byzantine Archers
    for i in 0..4 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::Archer,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(24, 7 + i as i32), CELL_SIZE),
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 7.0,
                attack_damage: 15.0,
                attack_speed: 0.7,
                vision_range: 9.0,
                morale: 55.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Bow,
                range: 7.0,
                armor_penetration: 0.0,
            },
            AttackTimer(Timer::from_seconds(1.4, TimerMode::Repeating)),
        ));
    }

    // 2 Byzantine Cavalry
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(20, 5 + i as i32 * 5), CELL_SIZE),
            Health {
                current: 120.0,
                max: 120.0,
                armor: 8.0,
                armor_type: ArmorType::Medium,
            },
            Stats {
                move_speed: 5.0,
                attack_range: 2.0,
                attack_damage: 18.0,
                attack_speed: 1.1,
                vision_range: 7.0,
                morale: 70.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 18.0,
                weapon_type: WeaponType::Spear,
                range: 2.0,
                armor_penetration: 0.2,
            },
            AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ));
    }

    // Byzantine Commander (the "enemy hero" — KillEnemyHero objective)
    commands.spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(23, 5), CELL_SIZE),
        Health {
            current: 350.0,
            max: 350.0,
            armor: 25.0,
            armor_type: ArmorType::Heavy,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.3,
            vision_range: 9.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 40.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.4,
        },
        AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
    ));

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::KillEnemyHero,
            objective_type: ObjectiveType::Kill,
            description: "Kill the Byzantine Commander leading the army".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 40,
                xp: 60,
                sovereignty_points: 3,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive the Byzantine onslaught for 60 seconds".into(),
            progress: 0.0,
            required_progress: 60.0,
            completed: false,
            rewards: MissionRewards {
                gold: 30,
                xp: 50,
                sovereignty_points: 2,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::CaptureRegion,
            objective_type: ObjectiveType::Capture,
            description: "Defeat all Byzantine forces on the field".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 100,
                xp: 150,
                sovereignty_points: 6,
                item_unlock: Some("Byzantine_Standard".into()),
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission06,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(600.0), // 10-minute time limit
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  M1 — Tutorial (Kayı Camp)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 1 playable scenario with Osman, his camp, and enemies.
pub fn setup_mission1_scenario(commands: &mut Commands) {
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
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: std::collections::HashMap::new(),
            ..Default::default()
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
                    description: "Unlocks CallOfTheTribe".into(),
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
                    description: "Unlocks Lionheart".into(),
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
            iron: 0,
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

// ═══════════════════════════════════════════════════════════════════════════
//  M2 — Byzantine Fort (Osman + Orhan siege)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 2 playable scenario: Osman & Orhan besiege a Byzantine fort.
/// Available as a public function so the test harness or future campaign
/// progression can call it.
pub fn setup_mission2_scenario(commands: &mut Commands) {
    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero, level 2)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 2,
            xp: 0,
            skill_points: 1,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: std::collections::HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(4, 4), CELL_SIZE),
        Health {
            current: 250.0,
            max: 250.0,
            armor: 12.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 35.0,
            attack_speed: 1.3,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 35.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.35,
        },
        AttackTimer(Timer::from_seconds(0.75, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![
                SkillNode {
                    id: SkillNodeId::SwordMastery,
                    name: "Sword Mastery".into(),
                    description: "Unlocks CallOfTheTribe".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::LeaderAura,
                    name: "Leader's Aura".into(),
                    description: "Unlocks WarCry".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::TribalUnity,
                    name: "Tribal Unity".into(),
                    description: "Unlocks Lionheart".into(),
                    cost: 1,
                    prerequisites: vec![SkillNodeId::SwordMastery],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::Sovereignty,
                    name: "Sovereignty".into(),
                    description: "Unlocks Trees' Vision".into(),
                    cost: 2,
                    prerequisites: vec![SkillNodeId::LeaderAura],
                    unlocked: true,
                },
            ],
            available_points: 0,
        },
    ));

    // Orhan Bey (hero, level 1)
    commands.spawn((
        Hero {
            hero_id: HeroId::Orhan,
            level: 1,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![AbilityId::TacticalGenius, AbilityId::ShieldWall],
            active_cooldowns: std::collections::HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(3, 5), CELL_SIZE),
        Health {
            current: 180.0,
            max: 180.0,
            armor: 8.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 25.0,
            attack_speed: 1.1,
            vision_range: 10.0,
            morale: 90.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 25.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.25,
        },
        AttackTimer(Timer::from_seconds(0.9, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Orhan,
            nodes: vec![
                SkillNode {
                    id: SkillNodeId::SiegeMastery,
                    name: "Siege Mastery".into(),
                    description: "Unlocks FortressBuilder".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: false,
                },
                SkillNode {
                    id: SkillNodeId::ArmyOrganization,
                    name: "Army Organization".into(),
                    description: "Unlocks TacticalGenius".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::Fortification,
                    name: "Fortification".into(),
                    description: "Unlocks ShieldWall".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::Logistics,
                    name: "Logistics".into(),
                    description: "Passive: faster recruitment".into(),
                    cost: 1,
                    prerequisites: vec![SkillNodeId::ArmyOrganization],
                    unlocked: false,
                },
            ],
            available_points: 0,
        },
    ));

    // 4 Kayi Light Infantry
    for i in 0..4 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(3 + i as i32, 6), CELL_SIZE),
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
        WorldPosition::new(IVec2::new(2, 7), CELL_SIZE),
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

    // Player Oba (Level 2 — already upgraded)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 2,
            max_level: 5,
            name: "Kayı Obağı — Fort".into(),
        },
        WorldPosition::new(IVec2::new(5, 8), CELL_SIZE),
        Health {
            current: 750.0,
            max: 750.0,
            armor: 25.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Byzantine Fort garrison) ──────────────────────

    // 6 Byzantine Infantry (garrison)
    for i in 0..6 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(18 + i as i32, 5 + (i % 2) as i32), CELL_SIZE),
            Health {
                current: 90.0,
                max: 90.0,
                armor: 5.0,
                armor_type: ArmorType::Medium,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 1.5,
                attack_damage: 12.0,
                attack_speed: 0.9,
                vision_range: 6.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 12.0,
                weapon_type: WeaponType::Spear,
                range: 1.5,
                armor_penetration: 0.15,
            },
            AttackTimer(Timer::from_seconds(1.1, TimerMode::Repeating)),
        ));
    }

    // 2 Byzantine Archers (on the wall)
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::Archer,
                faction: FactionId::Byzantine,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(20, 8 + i as i32), CELL_SIZE),
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 3.0,
                attack_range: 7.0,
                attack_damage: 15.0,
                attack_speed: 0.7,
                vision_range: 9.0,
                morale: 50.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 15.0,
                weapon_type: WeaponType::Bow,
                range: 7.0,
                armor_penetration: 0.0,
            },
            AttackTimer(Timer::from_seconds(1.4, TimerMode::Repeating)),
        ));
    }

    // 1 Byzantine Captain (mini-boss)
    commands.spawn((
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Byzantine,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(19, 4), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 15.0,
            armor_type: ArmorType::Heavy,
        },
        Stats {
            move_speed: 3.5,
            attack_range: 2.0,
            attack_damage: 25.0,
            attack_speed: 1.0,
            vision_range: 8.0,
            morale: 80.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 25.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.3,
        },
        AttackTimer(Timer::from_seconds(1.0, TimerMode::Repeating)),
    ));

    // Byzantine Watchtower (destroy objective target)
    commands.spawn((
        Building {
            building_type: BuildingType::Watchtower,
            faction: FactionId::Byzantine,
            level: 1,
            max_level: 1,
            name: "Byzantine Watchtower".into(),
        },
        WorldPosition::new(IVec2::new(21, 6), CELL_SIZE),
        Health {
            current: 300.0,
            max: 300.0,
            armor: 30.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::BuildBarracks,
            objective_type: ObjectiveType::Build,
            description: "Construct a Barracks near the Oba".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 15,
                xp: 25,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RecruitTenUnits,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit 5 Light Infantry from the Barracks".into(),
            progress: 0.0,
            required_progress: 5.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 30,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::DestroyWall,
            objective_type: ObjectiveType::Destroy,
            description: "Destroy the Byzantine Watchtower".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 25,
                xp: 40,
                sovereignty_points: 2,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::CaptureRegion,
            objective_type: ObjectiveType::Capture,
            description: "Defeat all Byzantine forces in the region".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 50,
                xp: 80,
                sovereignty_points: 3,
                item_unlock: Some("Byzantine_Banner".into()),
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission02,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: None,
        time_limit: Some(480.0), // 8-minute time limit
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  M3 — Mongol Raid (defend the settlement)
// ═══════════════════════════════════════════════════════════════════════════

/// Spawns the Mission 3 playable scenario: Mongol raiders attack the settlement.
/// Available as a public function so the test harness or future campaign
/// progression can call it.
pub fn setup_mission3_scenario(commands: &mut Commands) {
    // ── Player forces ─────────────────────────────────────────────────

    // Osman Bey (hero, level 3)
    commands.spawn((
        Hero {
            hero_id: HeroId::Osman,
            level: 3,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![
                AbilityId::WarCry,
                AbilityId::CallOfTheTribe,
                AbilityId::Lionheart,
                AbilityId::DreamOfTheTree,
            ],
            active_cooldowns: std::collections::HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(6, 5), CELL_SIZE),
        Health {
            current: 300.0,
            max: 300.0,
            armor: 15.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.0,
            attack_range: 2.0,
            attack_damage: 40.0,
            attack_speed: 1.4,
            vision_range: 10.0,
            morale: 100.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 40.0,
            weapon_type: WeaponType::Sword,
            range: 2.0,
            armor_penetration: 0.4,
        },
        AttackTimer(Timer::from_seconds(0.7, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Osman,
            nodes: vec![
                SkillNode {
                    id: SkillNodeId::SwordMastery,
                    name: "Sword Mastery".into(),
                    description: "Unlocks CallOfTheTribe".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::LeaderAura,
                    name: "Leader's Aura".into(),
                    description: "Unlocks WarCry".into(),
                    cost: 1,
                    prerequisites: vec![],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::TribalUnity,
                    name: "Tribal Unity".into(),
                    description: "Unlocks Lionheart".into(),
                    cost: 1,
                    prerequisites: vec![SkillNodeId::SwordMastery],
                    unlocked: true,
                },
                SkillNode {
                    id: SkillNodeId::Sovereignty,
                    name: "Sovereignty".into(),
                    description: "Unlocks Trees' Vision".into(),
                    cost: 2,
                    prerequisites: vec![SkillNodeId::LeaderAura],
                    unlocked: true,
                },
            ],
            available_points: 0,
        },
    ));

    // Bamsi Beyrek (hero, level 2)
    commands.spawn((
        Hero {
            hero_id: HeroId::Bamsi,
            level: 2,
            xp: 0,
            skill_points: 0,
            unlocked_abilities: vec![],
            active_cooldowns: std::collections::HashMap::new(),
            ..Default::default()
        },
        Unit {
            archetype: UnitArchetype::LightInfantry,
            faction: FactionId::Kayi,
            formation: FormationType::Line,
            stance: CombatStance::Aggressive,
        },
        WorldPosition::new(IVec2::new(5, 4), CELL_SIZE),
        Health {
            current: 220.0,
            max: 220.0,
            armor: 12.0,
            armor_type: ArmorType::Medium,
        },
        Stats {
            move_speed: 4.5,
            attack_range: 2.0,
            attack_damage: 28.0,
            attack_speed: 1.3,
            vision_range: 10.0,
            morale: 95.0,
            morale_max: 100.0,
        },
        MeleeWeapon {
            damage: 28.0,
            weapon_type: WeaponType::Spear,
            range: 2.0,
            armor_penetration: 0.2,
        },
        AttackTimer(Timer::from_seconds(0.85, TimerMode::Repeating)),
        SkillTree {
            hero_id: HeroId::Bamsi,
            nodes: vec![],
            available_points: 1,
        },
    ));

    // 2 Kayi Light Infantry
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightInfantry,
                faction: FactionId::Kayi,
                formation: FormationType::Line,
                stance: CombatStance::Defensive,
            },
            WorldPosition::new(IVec2::new(4 + i as i32, 6), CELL_SIZE),
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

    // Player Oba (Level 1)
    commands.spawn((
        Building {
            building_type: BuildingType::Oba,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 5,
            name: "Kayı Obağı — Frontier".into(),
        },
        WorldPosition::new(IVec2::new(6, 8), CELL_SIZE),
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
            iron: 0,
            population: 0,
        },
    ));

    // A Stable (pre-built so player can recruit mounted units)
    commands.spawn((
        Building {
            building_type: BuildingType::Stable,
            faction: FactionId::Kayi,
            level: 1,
            max_level: 3,
            name: "Kayı Stable".into(),
        },
        WorldPosition::new(IVec2::new(4, 8), CELL_SIZE),
        Health {
            current: 200.0,
            max: 200.0,
            armor: 10.0,
            armor_type: ArmorType::Fortification,
        },
        BuildCost {
            food: 0,
            gold: 0,
            wood: 0,
            stone: 0,
            iron: 0,
            population: 0,
        },
    ));

    // ── Enemy forces (Mongol raiders) ───────────────────────────────

    // 8 Mongol Light Cavalry (high speed, hit-and-run)
    for i in 0..8 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Mongol,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(
                IVec2::new(22 + (i % 4) as i32, 4 + (i / 4) as i32 * 4),
                CELL_SIZE,
            ),
            Health {
                current: 70.0,
                max: 70.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 6.0,
                attack_range: 2.0,
                attack_damage: 18.0,
                attack_speed: 1.3,
                vision_range: 8.0,
                morale: 60.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 18.0,
                weapon_type: WeaponType::Spear,
                range: 2.0,
                armor_penetration: 0.1,
            },
            AttackTimer(Timer::from_seconds(0.8, TimerMode::Repeating)),
        ));
    }

    // 2 Mongol Horse Archers (ranged harass)
    for i in 0..2 {
        commands.spawn((
            Unit {
                archetype: UnitArchetype::LightCavalry,
                faction: FactionId::Mongol,
                formation: FormationType::Line,
                stance: CombatStance::Aggressive,
            },
            WorldPosition::new(IVec2::new(24, 6 + i as i32 * 3), CELL_SIZE),
            Health {
                current: 60.0,
                max: 60.0,
                armor: 2.0,
                armor_type: ArmorType::Light,
            },
            Stats {
                move_speed: 6.5,
                attack_range: 6.0,
                attack_damage: 14.0,
                attack_speed: 0.9,
                vision_range: 9.0,
                morale: 55.0,
                morale_max: 100.0,
            },
            MeleeWeapon {
                damage: 14.0,
                weapon_type: WeaponType::Bow,
                range: 6.0,
                armor_penetration: 0.0,
            },
            AttackTimer(Timer::from_seconds(1.1, TimerMode::Repeating)),
        ));
    }

    // ── Mission State ─────────────────────────────────────────────────

    let objectives = vec![
        Objective {
            objective_id: ObjectiveId::SurviveWave,
            objective_type: ObjectiveType::Survive,
            description: "Survive the initial Mongol assault for 90 seconds".into(),
            progress: 0.0,
            required_progress: 90.0,
            completed: false,
            rewards: MissionRewards {
                gold: 20,
                xp: 30,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::RecruitTenUnits,
            objective_type: ObjectiveType::Recruit,
            description: "Recruit 3 Light Cavalry from the Stable".into(),
            progress: 0.0,
            required_progress: 3.0,
            completed: false,
            rewards: MissionRewards {
                gold: 25,
                xp: 35,
                sovereignty_points: 1,
                item_unlock: None,
            },
        },
        Objective {
            objective_id: ObjectiveId::Custom(2),
            objective_type: ObjectiveType::Kill,
            description: "Defeat all Mongol raiders".into(),
            progress: 0.0,
            required_progress: 1.0,
            completed: false,
            rewards: MissionRewards {
                gold: 60,
                xp: 100,
                sovereignty_points: 4,
                item_unlock: Some("Mongol_Horse_Breed".into()),
            },
        },
    ];

    commands.spawn(MissionStateComponent {
        mission_id: MissionId::Mission03,
        phase: MissionPhase::Intro,
        objectives,
        current_objective_index: 0,
        timer: Some(0.0),
        time_limit: Some(300.0), // 5-minute time limit
    });
}

// ═══════════════════════════════════════════════════════════════════════════
//  M1 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// TutorialBuildOba: progress = 1 when a Pasture is built by the player.
fn track_m1_build_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    construction_sites: Query<&ConstructionSite>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission01 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
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
/// beyond the starting units.
fn track_m1_recruit_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission01 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::TutorialRecruitUnit
            || mission.objectives[idx].completed
        {
            continue;
        }

        let kayi_count = units
            .iter()
            .filter(|u| u.faction == FactionId::Kayi)
            .count();
        if kayi_count > 5 {
            // starting: 1 Osman + 3 Inf + 1 Worker = 5
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// TutorialAttackEnemy: progress = 1 when all Byzantine units are dead.
fn track_m1_kill_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission01 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::TutorialAttackEnemy
            || mission.objectives[idx].completed
        {
            continue;
        }

        let byz_alive = units
            .iter()
            .filter(|u| u.faction == FactionId::Byzantine)
            .count();
        if byz_alive == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// TutorialMoveCamera: progress = 1 on first frame (simulated).
fn track_m1_camera_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    mut first_frame: Local<bool>,
) {
    if *first_frame {
        return;
    }
    *first_frame = true;

    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission01 {
            continue;
        }
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

// ═══════════════════════════════════════════════════════════════════════════
//  M2 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// BuildBarracks: progress = 1 when a Barracks building exists for Kayi.
fn track_m2_build_barracks_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    buildings: Query<&Building>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission02 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::BuildBarracks
            || mission.objectives[idx].completed
        {
            continue;
        }

        // Check if a Kayi Barracks exists
        let has_barracks = buildings
            .iter()
            .any(|b| b.building_type == BuildingType::Barracks && b.faction == FactionId::Kayi);
        if has_barracks {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// RecruitTenUnits (M2): progress counts Kayi units above the starting count.
/// Starting: Osman + Orhan + 4 infantry + 1 Worker = 7. Progress = min(actual - 7, 5).
fn track_m2_recruit_infantry_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission02 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::RecruitTenUnits
            || mission.objectives[idx].completed
        {
            continue;
        }

        let kayi_count = units
            .iter()
            .filter(|u| u.faction == FactionId::Kayi)
            .count();
        // Starting count: 7 (2 heroes + 4 infantry + 1 worker)
        let recruited = kayi_count.saturating_sub(7) as f32;
        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress = recruited.min(required);
    }
}

/// DestroyWall: progress = 1 when the Byzantine Watchtower is destroyed.
fn track_m2_destroy_watchtower_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    mut dead_events: EventReader<UnitDied>,
    buildings: Query<&Building>,
) {
    // Check if the watchtower was destroyed
    let watchtower_down = dead_events.read().any(|ev| {
        buildings.get(ev.entity).is_ok_and(|b| {
            b.building_type == BuildingType::Watchtower && b.faction == FactionId::Byzantine
        })
    }) || buildings
        .iter()
        .filter(|b| {
            b.building_type == BuildingType::Watchtower && b.faction == FactionId::Byzantine
        })
        .count()
        == 0;

    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission02 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::DestroyWall
            || mission.objectives[idx].completed
        {
            continue;
        }

        if watchtower_down {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// CaptureRegion: progress = 1 when all Byzantine units are dead.
fn track_m2_capture_region_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission02 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::CaptureRegion
            || mission.objectives[idx].completed
        {
            continue;
        }

        let byz_alive = units
            .iter()
            .filter(|u| u.faction == FactionId::Byzantine)
            .count();
        if byz_alive == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  M3 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// SurviveWave: progress = time elapsed (in seconds) capped at required_progress.
fn track_m3_survive_wave_objective_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission03 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::SurviveWave
            || mission.objectives[idx].completed
        {
            continue;
        }

        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress =
            (mission.objectives[idx].progress + time.delta_secs()).min(required);
    }
}

/// RecruitTenUnits (M3 variant): progress counts LightCavalry units.
/// Starting LightCavalry count = 0. Progress = min(cavalry_count, 3).
fn track_m3_recruit_mounted_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission03 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::RecruitTenUnits
            || mission.objectives[idx].completed
        {
            continue;
        }

        let cavalry = units
            .iter()
            .filter(|u| u.faction == FactionId::Kayi && u.archetype == UnitArchetype::LightCavalry)
            .count();
        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress = (cavalry as f32).min(required);
    }
}

/// Custom(2): progress = 1 when all Mongol units are dead.
fn track_m3_defeat_mongols_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission03 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::Custom(2)
            || mission.objectives[idx].completed
        {
            continue;
        }

        let mongol_alive = units
            .iter()
            .filter(|u| u.faction == FactionId::Mongol)
            .count();
        if mongol_alive == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  M4 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// SurviveWave (M4): progress = time elapsed capped at required_progress (30s).
fn track_m4_survive_wave_objective_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission04 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::SurviveWave
            || mission.objectives[idx].completed
        {
            continue;
        }
        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress =
            (mission.objectives[idx].progress + time.delta_secs()).min(required);
    }
}

/// DestroyWall (M4 — Mongol Sentry Post): progress = 1 when the Mongol Watchtower is gone.
fn track_m4_destroy_sentry_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    mut dead_events: EventReader<UnitDied>,
    buildings: Query<&Building>,
) {
    let sentry_down = dead_events.read().any(|ev| {
        buildings.get(ev.entity).is_ok_and(|b| {
            b.building_type == BuildingType::Watchtower && b.faction == FactionId::Mongol
        })
    }) || buildings
        .iter()
        .filter(|b| b.building_type == BuildingType::Watchtower && b.faction == FactionId::Mongol)
        .count()
        == 0;

    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission04 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::DestroyWall
            || mission.objectives[idx].completed
        {
            continue;
        }
        if sentry_down {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// RescueAlly: progress = 1 when no entities with RescueTarget remain (scout was rescued).
fn track_m4_rescue_ally_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    rescue_targets: Query<&RescueTarget>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission04 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::RescueAlly
            || mission.objectives[idx].completed
        {
            continue;
        }
        // Progress = 1 when the rescue target entity is gone (despawned = rescued)
        if rescue_targets.iter().count() == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  M5 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// BuildBarracks (M5): progress = 1 when a Barracks exists for Kayi.
fn track_m5_build_barracks_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    buildings: Query<&Building>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission05 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::BuildBarracks
            || mission.objectives[idx].completed
        {
            continue;
        }
        if buildings
            .iter()
            .any(|b| b.building_type == BuildingType::Barracks && b.faction == FactionId::Kayi)
        {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// RecruitTenUnits (M5): progress counts Kayi units above the starting count.
/// Starting: Osman + Orhan + 5 infantry + 1 Worker = 8. Progress = min(actual - 8, 5).
fn track_m5_recruit_infantry_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission05 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::RecruitTenUnits
            || mission.objectives[idx].completed
        {
            continue;
        }
        let kayi_count = units
            .iter()
            .filter(|u| u.faction == FactionId::Kayi)
            .count();
        let recruited = kayi_count.saturating_sub(8) as f32;
        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress = recruited.min(required);
    }
}

/// DestroyWall (M5): progress = 1 when the Byzantine Watchtower is destroyed.
fn track_m5_destroy_watchtower_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    mut dead_events: EventReader<UnitDied>,
    buildings: Query<&Building>,
) {
    let watchtower_down = dead_events.read().any(|ev| {
        buildings.get(ev.entity).is_ok_and(|b| {
            b.building_type == BuildingType::Watchtower && b.faction == FactionId::Byzantine
        })
    }) || buildings
        .iter()
        .filter(|b| {
            b.building_type == BuildingType::Watchtower && b.faction == FactionId::Byzantine
        })
        .count()
        == 0;

    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission05 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::DestroyWall
            || mission.objectives[idx].completed
        {
            continue;
        }
        if watchtower_down {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// KillEnemyHero (M5): progress = 1 when the Byzantine Captain (HP > 150) is dead.
fn track_m5_kill_enemy_hero_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<(&Unit, &Health)>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission05 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::KillEnemyHero
            || mission.objectives[idx].completed
        {
            continue;
        }
        // Check if a Byzantine hero-type unit (max HP > 150) still exists
        let byz_hero_alive = units
            .iter()
            .any(|(u, h)| u.faction == FactionId::Byzantine && h.max > 150.0);
        if !byz_hero_alive {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  M6 — Objective progress tracking
// ═══════════════════════════════════════════════════════════════════════════

/// KillEnemyHero (M6): progress = 1 when the Byzantine Commander (HP > 200) is dead.
fn track_m6_kill_enemy_hero_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<(&Unit, &Health)>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission06 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::KillEnemyHero
            || mission.objectives[idx].completed
        {
            continue;
        }
        // Byzantine commander has max HP > 200
        let byz_commander_alive = units
            .iter()
            .any(|(u, h)| u.faction == FactionId::Byzantine && h.max > 200.0);
        if !byz_commander_alive {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

/// SurviveWave (M6): progress = time elapsed capped at required_progress (60s).
fn track_m6_survive_wave_objective_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission06 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::SurviveWave
            || mission.objectives[idx].completed
        {
            continue;
        }
        let required = mission.objectives[idx].required_progress;
        mission.objectives[idx].progress =
            (mission.objectives[idx].progress + time.delta_secs()).min(required);
    }
}

/// CaptureRegion (M6): progress = 1 when all Byzantine units are dead.
fn track_m6_capture_region_objective_system(
    mut missions: Query<&mut MissionStateComponent>,
    units: Query<&Unit>,
) {
    for mut mission in missions.iter_mut() {
        if mission.mission_id != MissionId::Mission06 {
            continue;
        }
        let idx = mission
            .current_objective_index
            .min(mission.objectives.len().saturating_sub(1));
        if idx >= mission.objectives.len() {
            continue;
        }
        if mission.objectives[idx].objective_id != ObjectiveId::CaptureRegion
            || mission.objectives[idx].completed
        {
            continue;
        }
        let byz_alive = units
            .iter()
            .filter(|u| u.faction == FactionId::Byzantine)
            .count();
        if byz_alive == 0 {
            mission.objectives[idx].progress = 1.0;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Shared systems
// ═══════════════════════════════════════════════════════════════════════════

/// Checks mission objectives for completion across all missions.
fn mission_objective_check_system(
    mut objective_events: EventWriter<ObjectiveCompleted>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    let mut completed: Vec<(ObjectiveId, MissionId)> = Vec::new();

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
            mission.current_objective_index = (idx + 1).min(mission.objectives.len());
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

/// Checks win/loss conditions for all missions.
fn mission_win_loss_system(
    heroes: Query<&Hero>,
    units: Query<&Unit>,
    buildings: Query<&Building>,
    mut end_events: EventWriter<MissionEnded>,
    missions: Query<&MissionStateComponent>,
) {
    // Collect alive heroes per faction
    let _kayi_heroes_alive = heroes
        .iter()
        .any(|h| matches!(h.hero_id, HeroId::Osman | HeroId::Orhan | HeroId::Bamsi));

    // Check Oba alive (any Kayi Oba still standing)
    let oba_alive = buildings
        .iter()
        .any(|b| b.building_type == BuildingType::Oba && b.faction == FactionId::Kayi);

    // Helper: check Kayi units
    let kayi_units_alive = units.iter().any(|u| u.faction == FactionId::Kayi);

    for mission in missions.iter() {
        // ── Loss conditions ─────────────────────────────────────────
        let loss = match mission.mission_id {
            MissionId::Mission01 => {
                // Osman dead OR Oba destroyed
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                !osman_alive || !oba_alive
            }
            MissionId::Mission02 => {
                // Osman dead OR Orhan dead OR Oba destroyed OR time limit
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                let orhan_alive = heroes.iter().any(|h| h.hero_id == HeroId::Orhan);
                let timed_out = mission
                    .time_limit
                    .zip(mission.timer)
                    .is_some_and(|(limit, elapsed)| elapsed >= limit);
                !osman_alive || !orhan_alive || !oba_alive || timed_out
            }
            MissionId::Mission03 => {
                // Osman dead OR Bamsi dead OR Oba destroyed OR all Kayi units dead
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                let bamsi_alive = heroes.iter().any(|h| h.hero_id == HeroId::Bamsi);
                let timed_out = mission
                    .time_limit
                    .zip(mission.timer)
                    .is_some_and(|(limit, elapsed)| elapsed >= limit);
                !osman_alive || !bamsi_alive || !oba_alive || !kayi_units_alive || timed_out
            }
            MissionId::Mission04 => {
                // Osman dead OR Alaeddin dead OR Oba destroyed OR time limit
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                let alaeddin_alive = heroes.iter().any(|h| h.hero_id == HeroId::Alaeddin);
                let timed_out = mission
                    .time_limit
                    .zip(mission.timer)
                    .is_some_and(|(limit, elapsed)| elapsed >= limit);
                !osman_alive || !alaeddin_alive || !oba_alive || timed_out
            }
            MissionId::Mission05 => {
                // Osman dead OR Orhan dead OR Oba destroyed OR all Kayi units dead OR time limit
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                let orhan_alive = heroes.iter().any(|h| h.hero_id == HeroId::Orhan);
                let timed_out = mission
                    .time_limit
                    .zip(mission.timer)
                    .is_some_and(|(limit, elapsed)| elapsed >= limit);
                !osman_alive || !orhan_alive || !oba_alive || !kayi_units_alive || timed_out
            }
            MissionId::Mission06 => {
                // All 3 heroes alive, Oba alive, at least one Kayi unit, no time out
                let osman_alive = heroes.iter().any(|h| h.hero_id == HeroId::Osman);
                let orhan_alive = heroes.iter().any(|h| h.hero_id == HeroId::Orhan);
                let alaeddin_alive = heroes.iter().any(|h| h.hero_id == HeroId::Alaeddin);
                let timed_out = mission
                    .time_limit
                    .zip(mission.timer)
                    .is_some_and(|(limit, elapsed)| elapsed >= limit);
                !osman_alive
                    || !orhan_alive
                    || !alaeddin_alive
                    || !oba_alive
                    || !kayi_units_alive
                    || timed_out
            }
        };

        if loss {
            end_events.send(MissionEnded {
                mission_id: mission.mission_id,
                success: false,
            });
            continue;
        }

        // ── Win conditions ──────────────────────────────────────────
        if mission.objectives.iter().all(|o| o.completed) {
            end_events.send(MissionEnded {
                mission_id: mission.mission_id,
                success: true,
            });
        }
    }
}

// ─── Dialogue triggers ─────────────────────────────────────────────────────

/// Triggers dialogue based on game events for all missions.
fn dialogue_trigger_system(
    mut objective_events: EventReader<ObjectiveCompleted>,
    mut started_events: EventReader<MissionStarted>,
    mut dialogue_events: EventWriter<DialogueTriggered>,
) {
    for event in started_events.read() {
        let dialogue_id = match event.mission_id {
            MissionId::Mission01 => "mission1_intro".into(),
            MissionId::Mission02 => "mission2_intro".into(),
            MissionId::Mission03 => "mission3_intro".into(),
            MissionId::Mission04 => "mission4_intro".into(),
            MissionId::Mission05 => "mission5_intro".into(),
            MissionId::Mission06 => "mission6_intro".into(),
        };
        dialogue_events.send(DialogueTriggered {
            dialogue_id,
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

// ─── Mission lifecycle systems ─────────────────────────────────────────────

/// Auto-activates freshly spawned missions (Intro → Active) and sends
/// MissionStarted. Also ticks mission timers.
fn mission_intro_activator_system(
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut started_events: EventWriter<MissionStarted>,
    mut missions: Query<&mut MissionStateComponent>,
) {
    for mut mission in missions.iter_mut() {
        // Activate Intro missions
        if mission.phase == MissionPhase::Intro {
            mission.phase = MissionPhase::Active;
            started_events.send(MissionStarted {
                mission_id: mission.mission_id,
            });
        }
        // Tick mission timers
        if let Some(ref mut timer) = mission.timer {
            *timer += time.delta_secs();
        }
    }
}

/// Saves resources on mission win and advances campaign state.
/// Runs before restore so the stockpile snapshot is taken before it's overwritten.
fn campaign_save_system(
    mut end_events: EventReader<MissionEnded>,
    mut campaign_events: EventWriter<CampaignAdvanced>,
    stockpile: Res<ResourceStockpile>,
    mut persistence: ResMut<ResourcePersistence>,
    mut campaign_state: ResMut<CampaignState>,
) {
    for event in end_events.read() {
        if !event.success {
            continue;
        }
        // Snapshot the current resource stockpile for the next mission
        persistence.saved_stockpile = Some(stockpile.clone());
        campaign_state.current_mission = Some(event.mission_id);
        campaign_state.complete_mission(event.mission_id);
        let next = CampaignState::next_mission(event.mission_id);
        campaign_events.send(CampaignAdvanced {
            completed_mission: event.mission_id,
            next_unlocked: next,
        });
    }
}

/// Restores saved resources to the stockpile when a new mission starts.
/// This is a separate system (uses ResMut<ResourceStockpile> instead of Res)
/// so it doesn't conflict with campaign_save_system.
fn campaign_restore_system(
    mut start_events: EventReader<MissionStarted>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut persistence: ResMut<ResourcePersistence>,
) {
    for _event in start_events.read() {
        if let Some(saved) = persistence.saved_stockpile.take() {
            *stockpile = saved;
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
//  POST-MISSION REWARDS
// ═════════════════════════════════════════════════════════════════════════════

/// On successful mission completion, awards gold and items from completed
/// objectives to the player's stockpile and campaign inventory.
/// Runs before `campaign_save_system` so the stockpile snapshot includes rewards.
fn apply_mission_rewards_system(
    mut end_events: EventReader<MissionEnded>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut campaign_state: ResMut<CampaignState>,
    mission_state_query: Query<&MissionStateComponent>,
    _objectives_query: Query<&Objective>,
    mut notification_events: EventWriter<Notification>,
) {
    for event in end_events.read() {
        if !event.success {
            continue;
        }

        // Find the mission state component for the completed mission
        let mission_state = mission_state_query
            .iter()
            .find(|ms| ms.mission_id == event.mission_id);

        let Some(ms) = mission_state else {
            continue;
        };

        // Sum rewards from all completed objectives
        let mut total_gold: u32 = 0;
        let mut items_unlocked: Vec<String> = Vec::new();

        for objective in &ms.objectives {
            if !objective.completed {
                continue;
            }
            total_gold += objective.rewards.gold;
            if let Some(ref item) = objective.rewards.item_unlock {
                if !items_unlocked.contains(item) {
                    items_unlocked.push(item.clone());
                }
            }
        }

        // Award gold to the stockpile
        if total_gold > 0 {
            stockpile.gold += total_gold as f32;
        }

        // Track item unlocks in campaign state
        for item in &items_unlocked {
            if !campaign_state.unlocked_items.contains(item) {
                campaign_state.unlocked_items.push(item.clone());
            }
        }

        // Fire notification
        if !items_unlocked.is_empty() {
            notification_events.send(Notification {
                message: format!(
                    "Mission complete! {} gold awarded. Items unlocked: {}",
                    total_gold,
                    items_unlocked.join(", ")
                ),
                notification_type: NotificationType::Success,
                duration: Some(5.0),
            });
        } else if total_gold > 0 {
            notification_events.send(Notification {
                message: format!("Mission complete! {} gold awarded.", total_gold),
                notification_type: NotificationType::Success,
                duration: Some(5.0),
            });
        }
    }
}

/// Manages mission state transitions based on MissionStarted and
/// MissionEnded events (Loading → Active → Complete/Failed).
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
