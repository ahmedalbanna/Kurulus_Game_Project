use bevy::prelude::*;
use kurulus_core::prelude::*;

pub struct BaseBuildingPlugin;

impl Plugin for BaseBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecruitmentTimer>()
            .add_systems(
                FixedUpdate,
                (
                    construction_progress_system.in_set(SimulationSet::ConstructionProgress),
                    complete_construction_system,
                    building_upgrade_system,
                    process_recruit_orders_system,
                    recruitment_progress_system,
                ),
            );
    }
}

// ─── Construction progress ─────────────────────────────────────────────────

/// Tick construction progress and consume resources from the stockpile
/// proportionally as the building is built.
fn construction_progress_system(
    game_time: Res<GameTime>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut sites: Query<(&mut ConstructionSite, &BuildCost)>,
) {
    for (mut site, cost) in sites.iter_mut() {
        if site.progress >= site.max_progress {
            continue;
        }

        let increment = game_time.time_scale * 10.0;
        let new_progress = (site.progress + increment).min(site.max_progress);

        // Consume resources proportionally to progress made this tick
        let progress_fraction = (new_progress - site.progress) / site.max_progress;
        stockpile.food = (stockpile.food - cost.food as f32 * progress_fraction).max(0.0);
        stockpile.gold = (stockpile.gold - cost.gold as f32 * progress_fraction).max(0.0);
        stockpile.wood = (stockpile.wood - cost.wood as f32 * progress_fraction).max(0.0);
        stockpile.stone = (stockpile.stone - cost.stone as f32 * progress_fraction).max(0.0);

        site.progress = new_progress;
    }
}

/// When a `ConstructionSite` reaches full progress, replace it with a
/// `Building` component so the structure becomes operational.
fn complete_construction_system(
    mut commands: Commands,
    sites: Query<(Entity, &ConstructionSite, &Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    for (entity, site, building_template, pos) in sites.iter() {
        if site.progress < site.max_progress {
            continue;
        }

        // Calculate max_population contribution from Oba
        let pop_bonus = if building_template.building_type == BuildingType::Oba {
            match building_template.level {
                1 => 5,
                2 => 10,
                3 => 15,
                4 => 25,
                _ => 40,
            }
        } else {
            0
        };
        stockpile.max_population += pop_bonus;

        // Remove ConstructionSite, keep Building and other components
        commands.entity(entity).remove::<ConstructionSite>();

        // Log completion
        info!(
            "Construction complete: {:?} at {:?}",
            building_template.building_type,
            pos.grid.0
        );
    }
}

// ─── Building upgrades ─────────────────────────────────────────────────────

/// Handles building level upgrades (triggered by a `BuildOrder` with an
/// existing building as target, or via direct component mutation).
///
/// For Phase 2, the upgrade cost is hardcoded per building type.
fn building_upgrade_system(
    mut buildings: Query<(&mut Building, &BuildCost), Without<UpgradeChain>>,
    mut stockpile: ResMut<ResourceStockpile>,
    mut production_rates: ResMut<ProductionRates>,
) {
    for (mut building, upgrade_cost) in buildings.iter_mut() {
        if building.level >= building.max_level {
            continue;
        }

        // Upgrade is triggered when a player sends a BuildOrder targeting
        // an existing building.  In Phase 2 we simulate this automatically
        // when resources are sufficient (for testing).
        // A full game would use BuildOrder events.

        let can_upgrade = stockpile.food >= upgrade_cost.food as f32
            && stockpile.gold >= upgrade_cost.gold as f32
            && stockpile.wood >= upgrade_cost.wood as f32
            && stockpile.stone >= upgrade_cost.stone as f32;

        if !can_upgrade {
            continue;
        }

        // Deduct cost
        stockpile.food -= upgrade_cost.food as f32;
        stockpile.gold -= upgrade_cost.gold as f32;
        stockpile.wood -= upgrade_cost.wood as f32;
        stockpile.stone -= upgrade_cost.stone as f32;

        // Apply level
        building.level += 1;

        // Oba-specific: increase max_population per level
        if building.building_type == BuildingType::Oba {
            stockpile.max_population += 5; // flat bonus per level
            production_rates.food_per_second *= 1.2; // Oba upgrades improve food production
        }

        info!(
            "{:?} upgraded to level {}",
            building.building_type, building.level
        );

        // For now, upgrade one at a time per frame
        break;
    }
}

// ─── Recruitment ───────────────────────────────────────────────────────────

/// Tracks the recruitment timer so units are spawned one at a time with a
/// delay between them.
#[derive(Resource, Default)]
pub struct RecruitmentTimer {
    pub active_recruitments: Vec<ActiveRecruitment>,
}

pub struct ActiveRecruitment {
    pub building: Entity,
    pub archetype: UnitArchetype,
    pub remaining: u32,
    pub timer: f32,
    pub interval: f32, // seconds between each unit
}

/// Reads `RecruitOrder` events and starts recruitment queues on buildings.
fn process_recruit_orders_system(
    mut orders: EventReader<RecruitOrder>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<&Building>,
) {
    for order in orders.read() {
        let Ok(building) = buildings.get(order.building) else {
            continue;
        };

        // Only Oba can recruit in Phase 2
        if building.building_type != BuildingType::Oba {
            continue;
        }

        let interval = match order.archetype {
            UnitArchetype::Worker => 5.0,
            UnitArchetype::LightInfantry => 8.0,
            _ => 10.0,
        };

        // Add to recruitment queue
        timer.active_recruitments.push(ActiveRecruitment {
            building: order.building,
            archetype: order.archetype,
            remaining: order.count,
            timer: 0.0,
            interval,
        });
    }
}

/// Processes the recruitment queue, spawning units when their timer reaches
/// the interval threshold.
fn recruitment_progress_system(
    mut commands: Commands,
    time: Res<bevy::time::Time<bevy::time::Fixed>>,
    mut timer: ResMut<RecruitmentTimer>,
    buildings: Query<(&Building, &WorldPosition)>,
    mut stockpile: ResMut<ResourceStockpile>,
) {
    let mut completed = Vec::new();

    for (idx, recruitment) in timer.active_recruitments.iter_mut().enumerate() {
        if recruitment.remaining == 0 {
            completed.push(idx);
            continue;
        }

        recruitment.timer += time.delta_secs();

        if recruitment.timer >= recruitment.interval {
            recruitment.timer -= recruitment.interval;
            recruitment.remaining -= 1;

            // Spawn the unit near the building
            if let Ok((_building, pos)) = buildings.get(recruitment.building) {
                let spawn_offset = IVec2::new(
                    pos.grid.0.x + 2, // spawn to the right of the building
                    pos.grid.0.y,
                );

                let (health, stats, weapon) = unit_archetype_stats(recruitment.archetype);
                let attack_speed = stats.attack_speed;

                commands.spawn((
                    Unit {
                        archetype: recruitment.archetype,
                        faction: FactionId::Kayi,
                        formation: FormationType::Line,
                        stance: CombatStance::Defensive,
                    },
                    WorldPosition::new(spawn_offset, CELL_SIZE),
                    health,
                    stats,
                    weapon,
                    AttackTimer(Timer::from_seconds(
                        1.0 / attack_speed.max(0.1),
                        TimerMode::Repeating,
                    )),
                ));

                // Consume population
                stockpile.population += 1;

                info!("Recruited {:?} at {:?}", recruitment.archetype, spawn_offset);
            }
        }
    }

    // Remove completed recruitments (reverse order)
    for idx in completed.into_iter().rev() {
        timer.active_recruitments.remove(idx);
    }
}

/// Returns default health, stats, and weapon for a given archetype.
fn unit_archetype_stats(
    archetype: UnitArchetype,
) -> (Health, Stats, MeleeWeapon) {
    match archetype {
        UnitArchetype::LightInfantry => (
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
        ),
        UnitArchetype::Worker => (
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
        ),
        UnitArchetype::Archer => (
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
        ),
        _ => (
            Health::default(),
            Stats::default(),
            MeleeWeapon {
                damage: 10.0,
                weapon_type: WeaponType::Unarmed,
                range: 1.0,
                armor_penetration: 0.0,
            },
        ),
    }
}
