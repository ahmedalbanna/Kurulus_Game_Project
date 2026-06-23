use bevy::prelude::*;
use kurulus_core::prelude::*;
use kurulus_core::resources::world_map::{WorldMapHover, WorldMapSelection};
use std::fmt::Write;

/// Colour for the scout count badge (reserved for future use).
#[allow(dead_code)]
const SCOUT_BADGE_COLOR: Color = Color::srgb(0.20, 0.85, 0.40);

// ── Constants ──────────────────────────────────────────────────────────────

const TOOLTIP_PADDING: UiRect = UiRect::px(8.0, 8.0, 6.0, 6.0);
const LEGEND_PADDING: UiRect = UiRect::px(10.0, 10.0, 8.0, 8.0);
const PANEL_PADDING: UiRect = UiRect::px(12.0, 12.0, 10.0, 10.0);

const FONT_SIZE_TOOLTIP: f32 = 12.0;
const FONT_SIZE_LEGEND: f32 = 10.0;
const FONT_SIZE_BODY: f32 = 12.0;

const BG_COLOR: Color = Color::srgba(0.08, 0.08, 0.12, 0.88);
const TEXT_COLOR: Color = Color::srgb(0.90, 0.90, 0.92);

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Region / POI Tooltip
// ═══════════════════════════════════════════════════════════════════════════════

/// Renders a floating tooltip near the mouse cursor showing information about
/// the hovered region or POI on the world map.
///
/// Priority: POI tooltip > Region tooltip.  If hovering over a POI tile, the
/// POI details are shown.  Otherwise region details are shown.
pub fn world_map_tooltip_system(
    state: Res<State<AppState>>,
    hover: Res<WorldMapHover>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
    tooltips: Query<Entity, With<WorldMapTooltipRoot>>,
) {
    if *state.get() != AppState::WorldMap {
        // Despawn tooltip when not in world map
        for entity in tooltips.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Build tooltip text
    let body = build_tooltip_text(&hover, &world_map);
    let show_tooltip = body.is_some();

    let existing = tooltips.get_single().ok();

    match (show_tooltip, existing) {
        (true, None) => {
            // Spawn
            let text = body.unwrap_or_default();
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    left: Val::Px(10.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    padding: TOOLTIP_PADDING,
                    ..default()
                },
                BackgroundColor(BG_COLOR),
                Text::new(&text),
                TextFont {
                    font_size: FONT_SIZE_TOOLTIP,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                WorldMapTooltipRoot,
                Name::new("WorldMapTooltip"),
            ));
        }
        (true, Some(entity)) => {
            // Update
            let text = body.unwrap_or_default();
            commands.entity(entity).insert(Text::new(&text));
        }
        (false, Some(entity)) => {
            // Despawn
            commands.entity(entity).despawn();
        }
        (false, None) => {
            // Nothing
        }
    }
}

/// Build the tooltip text string from current hover state.
/// Returns `None` when there is nothing to show.
fn build_tooltip_text(hover: &WorldMapHover, world_map: &WorldMap) -> Option<String> {
    // POI takes priority
    if let Some(poi_id) = hover.hovered_poi {
        if let Some(poi) = world_map
            .points_of_interest
            .iter()
            .find(|p| p.poi_id == poi_id)
        {
            let mut s = String::new();
            let _ = write!(s, "📍 {}\n", poi.name);
            let _ = write!(s, "{}", poi.poi_type.display_name());
            if poi.discovered {
                s.push_str(" (Discovered)");
            } else {
                s.push_str(" (Undiscovered)");
            }
            s.push('\n');
            let _ = write!(s, "{}", poi.description);
            if let Some(rid) = poi.region {
                if let Some(reg) = world_map.regions.iter().find(|r| r.region_id == rid) {
                    let _ = write!(s, "\nRegion: {}", reg.name);
                }
            }
            return Some(s);
        }
    }

    // Region tooltip
    if let Some(region_id) = hover.hovered_region {
        if let Some(region) = world_map.regions.iter().find(|r| r.region_id == region_id) {
            let mut s = String::new();
            let _ = write!(s, "🏛 {} [{:?}]\n", region.name, region.owner);
            let _ = write!(s, "Strategic Value: {}\n", region.strategic_value);

            let r = &region.resources;
            let _ = write!(s, "Food: {:.0} | Gold: {:.0}\n", r.food, r.gold);
            let _ = write!(s, "Wood: {:.0} | Stone: {:.0}\n", r.wood, r.stone);
            let _ = write!(
                s,
                "Garrison: {:.1}  Pop Cap: {}",
                region.garrison_strength, r.population_capacity
            );

            return Some(s);
        }
    }

    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Map Legend
// ═══════════════════════════════════════════════════════════════════════════════

/// Renders a legend overlay in the bottom-left corner of the world map,
/// explaining terrain colours, fog states, POI icons, and the player marker.
pub fn world_map_legend_system(
    state: Res<State<AppState>>,
    scouts: Query<&ScoutParty>,
    mut commands: Commands,
    legends: Query<Entity, With<WorldMapLegendRoot>>,
) {
    let in_world_map = *state.get() == AppState::WorldMap;
    let existing = legends.get_single().ok();

    match (in_world_map, existing) {
        (true, None) => {
            commands.spawn((
                legend_node(),
                BackgroundColor(BG_COLOR),
                Text::new(legend_text(scouts.iter().count())),
                TextFont {
                    font_size: FONT_SIZE_LEGEND,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                WorldMapLegendRoot,
                Name::new("WorldMapLegend"),
            ));
        }
        (true, Some(entity)) => {
            commands
                .entity(entity)
                .insert(Text::new(legend_text(scouts.iter().count())));
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
        }
        (false, None) => {}
    }
}

fn legend_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(8.0),
        left: Val::Px(12.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Start,
        padding: LEGEND_PADDING,
        ..default()
    }
}

fn legend_text(scout_count: usize) -> String {
    let mut s = String::new();
    // Terrain
    s.push_str("=== Terrain ===\n");
    s.push_str("██ Grassland\n██ Forest\n██ Mountain\n██ Water\n██ Desert\n██ Snow\n");
    // Fog
    s.push_str("\n=== Fog of War ===\n");
    s.push_str("█ Unexplored (hidden)\n");
    s.push_str("█ Explored (dimmed)\n");
    s.push_str("█ Visible (clear)\n");
    // POI
    s.push_str("\n=== Points of Interest ===\n");
    s.push_str("● Discovered (gold)\n");
    s.push_str("● Undiscovered (grey)\n");
    // Scouts
    s.push_str("\n=== Scouts ===\n");
    let _ = write!(s, "● Active: {}\n", scout_count);
    s.push_str("● Encounter (orange)\n");
    // Player
    s.push_str("\n=== Player ===\n");
    s.push_str("♦ Current Position\n");
    // Controls
    s.push_str("\n=== Controls ===\n");
    s.push_str(
        "M: Toggle map\nDrag: Pan view\nScroll: Zoom\nClick: Select region\nR-Click/Esc: Deselect",
    );
    s
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3. Selection Info Panel
// ═══════════════════════════════════════════════════════════════════════════════

/// When a region is selected (via `WorldMapSelection`), shows a detailed info
/// panel on the right side of the screen with the region's full data.
pub fn world_map_selection_panel_system(
    state: Res<State<AppState>>,
    selection: Res<WorldMapSelection>,
    world_map: Res<WorldMap>,
    mut commands: Commands,
    panels: Query<Entity, With<WorldMapSelectionRoot>>,
) {
    if *state.get() != AppState::WorldMap {
        for entity in panels.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let body = selection
        .selected_region
        .and_then(|rid| world_map.regions.iter().find(|r| r.region_id == rid))
        .map(|region| build_selection_panel_text(region, &world_map));

    let existing = panels.get_single().ok();

    match (body.is_some(), existing) {
        (true, None) => {
            let text = body.unwrap_or_default();
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(12.0),
                    top: Val::Px(60.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    padding: PANEL_PADDING,
                    min_width: Val::Px(220.0),
                    ..default()
                },
                BackgroundColor(BG_COLOR),
                Text::new(&text),
                TextFont {
                    font_size: FONT_SIZE_BODY,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                WorldMapSelectionRoot,
                Name::new("WorldMapSelectionPanel"),
            ));
        }
        (true, Some(entity)) => {
            let text = body.unwrap_or_default();
            commands.entity(entity).insert(Text::new(&text));
        }
        (false, Some(entity)) => {
            commands.entity(entity).despawn();
        }
        (false, None) => {}
    }
}

fn build_selection_panel_text(region: &RegionDef, world_map: &WorldMap) -> String {
    let mut s = String::new();

    // Title
    let _ = write!(s, "🏰 {}\n", region.name);
    s.push_str(&"─".repeat(24));
    s.push('\n');

    // Status & Owner
    let _ = write!(s, "Owner: {:?}\n", region.owner);

    let status = match region.owner {
        FactionId::Kayi => "Owned",
        _ => "Hostile",
    };
    let _ = write!(s, "Status: {}\n", status);

    let stars = "⭐".repeat(region.strategic_value as usize);
    let _ = write!(s, "Value:  {}\n\n", stars);

    // Resources
    s.push_str("Resources\n");
    let r = &region.resources;
    append_resource_bar(&mut s, "Food", r.food);
    append_resource_bar(&mut s, "Gold", r.gold);
    append_resource_bar(&mut s, "Wood", r.wood);
    append_resource_bar(&mut s, "Stone", r.stone);
    let _ = write!(s, "\nPop Cap: {}", r.population_capacity);

    // Military
    let _ = write!(s, "\n\nMilitary\n");
    let garrison_pct = (region.garrison_strength * 100.0) as u32;
    let _ = write!(s, "Garrison: {}%\n", garrison_pct);

    // POIs in this region
    let region_pois: Vec<_> = world_map
        .points_of_interest
        .iter()
        .filter(|p| p.region == Some(region.region_id))
        .collect();
    if !region_pois.is_empty() {
        let _ = write!(s, "\nPOIs ({})\n", region_pois.len());
        for poi in &region_pois {
            let _ = write!(s, "• {}\n", poi.name);
        }
    }

    s
}

/// Appends a resource bar line like: "Food:  ██████░░░  200"
fn append_resource_bar(buf: &mut String, label: &str, value: f32) {
    let filled = ((value / 500.0).clamp(0.0, 1.0) * 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);
    let _ = write!(
        buf,
        "{}: {} {:.0}\n",
        label,
        "█".repeat(filled) + &"░".repeat(empty),
        value
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world_map(resources: RegionResources, owner: FactionId, sv: u32, gs: f32) -> WorldMap {
        let mut wm = WorldMap::default();
        wm.regions = vec![RegionDef {
            region_id: RegionId::Sogut,
            name: "Söğüt".into(),
            tiles: vec![IVec2::new(10, 10), IVec2::new(11, 10)],
            owner,
            resources,
            strategic_value: sv,
            garrison_strength: gs,
        }];
        wm.points_of_interest = vec![PointOfInterest {
            poi_id: PoiId::KayiCamp,
            name: "Kayı Camp".into(),
            position: IVec2::new(10, 10),
            poi_type: PoiType::Camp,
            discovered: true,
            description: "The ancestral camp of the Kayı tribe.".into(),
            region: Some(RegionId::Sogut),
        }];
        wm
    }

    #[test]
    fn test_build_tooltip_text_region() {
        let resources = RegionResources {
            food: 200.0,
            gold: 150.0,
            wood: 100.0,
            stone: 80.0,
            population_capacity: 500,
        };
        let wm = make_world_map(resources, FactionId::Kayi, 3, 0.2);
        let hover = WorldMapHover {
            hovered_tile: Some(IVec2::new(10, 10)),
            hovered_region: Some(RegionId::Sogut),
            hovered_poi: Some(PoiId::KayiCamp),
        };

        // POI takes priority
        let text = build_tooltip_text(&hover, &wm);
        assert!(text.is_some());
        let t = text.unwrap();
        assert!(t.contains("Kayı Camp"), "Should show POI name: {}", t);
        assert!(
            t.contains("Discovered"),
            "Discovered POI should show status: {}",
            t
        );
    }

    #[test]
    fn test_build_tooltip_text_region_only() {
        let resources = RegionResources {
            food: 100.0,
            gold: 50.0,
            wood: 30.0,
            stone: 20.0,
            population_capacity: 200,
        };
        let wm = make_world_map(resources, FactionId::Byzantine, 5, 0.8);
        let hover = WorldMapHover {
            hovered_tile: Some(IVec2::new(11, 10)),
            hovered_region: Some(RegionId::Sogut),
            hovered_poi: None,
        };

        let text = build_tooltip_text(&hover, &wm);
        assert!(text.is_some());
        let t = text.unwrap();
        assert!(t.contains("Söğüt"), "Should show region name: {}", t);
        assert!(t.contains("Byzantine"), "Should show owner: {}", t);
        assert!(t.contains("Strategic Value: 5"), "Should show SV: {}", t);
        assert!(t.contains("Garrison"), "Should show garrison: {}", t);
    }

    #[test]
    fn test_build_tooltip_text_no_hover() {
        let resources = RegionResources::default();
        let wm = make_world_map(resources, FactionId::Kayi, 1, 0.0);
        let hover = WorldMapHover::default();

        let text = build_tooltip_text(&hover, &wm);
        assert!(text.is_none(), "Should be None when nothing is hovered");
    }

    #[test]
    fn test_legend_text_contains_all_sections() {
        let text = legend_text(2); // 2 active scouts
        assert!(text.contains("Terrain"), "Should have terrain section");
        assert!(text.contains("Fog of War"), "Should have fog section");
        assert!(
            text.contains("Points of Interest"),
            "Should have POI section"
        );
        assert!(text.contains("Scouts"), "Should have scouts section");
        assert!(text.contains("Active: 2"), "Should show scout count");
        assert!(text.contains("Encounter"), "Should show encounter info");
        assert!(text.contains("Player"), "Should have player section");
        assert!(text.contains("Controls"), "Should have controls section");
    }

    #[test]
    fn test_legend_text_scout_count_zero() {
        let text = legend_text(0);
        assert!(text.contains("Active: 0"), "Should show 0 scouts");
    }

    #[test]
    fn test_resource_bar_length() {
        let mut buf = String::new();
        append_resource_bar(&mut buf, "Food", 250.0);
        assert!(buf.contains("Food"), "Should contain label");
        assert!(buf.contains("250"), "Should contain value");
        // 250/500 = 0.5 = 5 filled bars out of 10
        assert!(buf.contains("█████"), "Should have ~5 filled bars");
    }

    #[test]
    fn test_selection_panel_contains_region_info() {
        let resources = RegionResources {
            food: 300.0,
            gold: 200.0,
            wood: 100.0,
            stone: 50.0,
            population_capacity: 400,
        };
        let mut wm = make_world_map(resources, FactionId::Kayi, 6, 0.5);
        // Add a second POI in the region
        wm.points_of_interest.push(PointOfInterest {
            poi_id: PoiId::DomanicPlateau,
            name: "Domaniç Plateau".into(),
            position: IVec2::new(12, 10),
            poi_type: PoiType::Village,
            discovered: false,
            description: "A high plateau used for summer pastures.".into(),
            region: Some(RegionId::Sogut),
        });

        let region = wm.regions.first().unwrap();
        let text = build_selection_panel_text(region, &wm);

        assert!(
            text.contains("Söğüt"),
            "Title should contain region name: {}",
            text
        );
        assert!(text.contains("Kayı"), "Owner should appear: {}", text);
        assert!(text.contains("Status"), "Status section: {}", text);
        assert!(text.contains("Value"), "Value section: {}", text);
        assert!(text.contains("Resources"), "Resources section: {}", text);
        assert!(text.contains("Military"), "Military section: {}", text);
        assert!(text.contains("POIs"), "POIs section: {}", text);
        assert!(text.contains("Kayı Camp"), "Should list POI: {}", text);
        assert!(
            text.contains("Domaniç Plateau"),
            "Should list second POI: {}",
            text
        );
    }
}
