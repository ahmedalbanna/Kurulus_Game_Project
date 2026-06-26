use crate::events::NotificationType;
use bevy::prelude::*;

// ─── Resource Bar State ─────────────────────────────────────────────────

/// Snapshot of resource counts for the top bar display.
/// Updated each tick by the top bar logic system (Phase 2).
#[derive(Resource, Clone, Debug)]
pub struct ResourceBarState {
    pub gold: u32,
    pub wood: u32,
    pub food: u32,
    pub iron: u32,
    pub stone: u32,
    pub population_current: u32,
    pub population_max: u32,
    pub sovereignty_points: u32,
    /// Per-resource deltas for arrow indicators (+/-).
    pub gold_delta: i32,
    pub wood_delta: i32,
    pub food_delta: i32,
    pub iron_delta: i32,
    pub stone_delta: i32,
    /// Previous-frame resource values used to compute deltas.
    pub prev_gold: u32,
    pub prev_wood: u32,
    pub prev_food: u32,
    pub prev_iron: u32,
    pub prev_stone: u32,
    /// Pre-formatted in-game time string (e.g. "14:30").
    pub time_string: String,
    /// Current game speed multiplier.
    pub game_speed: f32,
}

impl Default for ResourceBarState {
    fn default() -> Self {
        Self {
            gold: 0,
            wood: 0,
            food: 0,
            iron: 0,
            stone: 0,
            population_current: 0,
            population_max: 0,
            sovereignty_points: 0,
            gold_delta: 0,
            wood_delta: 0,
            food_delta: 0,
            iron_delta: 0,
            stone_delta: 0,
            prev_gold: 0,
            prev_wood: 0,
            prev_food: 0,
            prev_iron: 0,
            prev_stone: 0,
            time_string: String::new(),
            game_speed: 1.0,
        }
    }
}

// ─── Notification Stack ─────────────────────────────────────────────────

/// Active notifications awaiting display in the notification stack.
/// Populated by the notification dispatch system; consumed by the
/// notification render system (Phase 3).
#[derive(Resource, Clone, Debug, Default)]
pub struct NotificationStack {
    pub entries: Vec<NotificationEntry>,
}

/// A single notification entry with its display data and lifecycle.
#[derive(Clone, Debug)]
pub struct NotificationEntry {
    pub notification_type: NotificationType,
    pub message: String,
    /// Seconds remaining before auto-dismiss.
    pub time_remaining: f32,
    /// Total duration this entry was spawned with.
    pub duration: f32,
}

impl NotificationEntry {
    pub fn new(notification_type: NotificationType, message: String, duration: f32) -> Self {
        Self {
            notification_type,
            message,
            time_remaining: duration,
            duration,
        }
    }
}

// ─── Objective Tracker ─────────────────────────────────────────────────

/// Current mission objectives for the objectives panel.
/// Populated by the objectives logic system (Phase 3).
#[derive(Resource, Clone, Debug, Default)]
pub struct ObjectiveTracker {
    pub primary: Vec<ObjectiveEntry>,
    pub secondary: Vec<ObjectiveEntry>,
}

/// A single mission objective entry for UI display.
#[derive(Clone, Debug)]
pub struct ObjectiveEntry {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub progress: Option<(u32, u32)>,
}

// ─── Hero Bar State ─────────────────────────────────────────────────────

/// Hero bar state — which heroes are available for display.
/// Populated by the hero bar logic system (Phase 3).
#[derive(Resource, Clone, Debug, Default)]
pub struct HeroBarState {
    pub heroes: Vec<HeroSlot>,
}

/// A single hero slot in the hero bar.
#[derive(Clone, Debug)]
pub struct HeroSlot {
    pub entity: Option<Entity>,
    pub name: String,
    pub health_pct: f32,
    pub mana_pct: f32,
    pub available: bool,
    pub hotkey: String,
}

impl Default for HeroSlot {
    fn default() -> Self {
        Self {
            entity: None,
            name: String::new(),
            health_pct: 1.0,
            mana_pct: 1.0,
            available: true,
            hotkey: String::new(),
        }
    }
}

// ─── Main Screen Layout ─────────────────────────────────────────────────────

/// Main screen layout configuration for positioning all HUD panels.
#[derive(Resource, Clone, Debug)]
pub struct MainScreenLayout {
    /// Height of the top resource bar.
    pub top_bar_height: f32,
    /// Width of the hero selection panel.
    pub hero_panel_width: f32,
    /// Height of the hero selection panel.
    pub hero_panel_height: f32,
    /// Width of the unit stats display panel.
    pub unit_stats_width: f32,
    /// Height of the unit stats display panel.
    pub unit_stats_height: f32,
    /// Width of the action bar (building/unit buttons).
    pub action_bar_width: f32,
    /// Height of the action bar.
    pub action_bar_height: f32,
    /// Width of the command panel (Attack, Patrol, Guard, Stop).
    pub command_panel_width: f32,
    /// Height of the command panel.
    pub command_panel_height: f32,
    /// Size of the minimap (width = height).
    pub minimap_size: f32,
    /// General padding value for spacing.
    pub padding: f32,
    /// Settings button size.
    pub settings_button_size: f32,
}

impl Default for MainScreenLayout {
    fn default() -> Self {
        Self {
            top_bar_height: 40.0,
            hero_panel_width: 300.0,
            hero_panel_height: 130.0,
            unit_stats_width: 300.0,
            unit_stats_height: 70.0,
            action_bar_width: 500.0,
            action_bar_height: 80.0,
            command_panel_width: 130.0,
            command_panel_height: 170.0,
            minimap_size: 180.0,
            padding: 8.0,
            settings_button_size: 36.0,
        }
    }
}

// ─── Unit Stats State ────────────────────────────────────────────────────────

/// Snapshot of the currently selected unit's stats for display.
#[derive(Resource, Clone, Debug)]
pub struct UnitStatsState {
    /// Attack/damage value displayed.
    pub damage: u32,
    /// Defense/armor value displayed.
    pub defense: u32,
    /// Movement speed value displayed.
    pub speed: u32,
    /// Optional name of the selected unit type.
    pub selected_unit_name: Option<String>,
    /// A battle cry or flavour text shown below the stats.
    pub battle_cry: String,
}

impl Default for UnitStatsState {
    fn default() -> Self {
        Self {
            damage: 0,
            defense: 0,
            speed: 0,
            selected_unit_name: None,
            battle_cry: String::from("God is Great! Cry"),
        }
    }
}

// ─── Action Bar State ───────────────────────────────────────────────────────

/// State of the action bar — which actions are available based on selection.
#[derive(Resource, Clone, Debug, Default)]
pub struct ActionBarState {
    /// Row 1 actions (e.g. Move, Flag).
    pub row1: Vec<ActionSlot>,
    /// Row 2 actions (e.g. Heroes, Blacksmith, Oba, Barracks, Alpa, Horse Archer).
    pub row2: Vec<ActionSlot>,
}

/// A single action slot in the action bar.
#[derive(Clone, Debug)]
pub struct ActionSlot {
    /// The type of action this slot represents.
    pub action_type: ActionType,
    /// Display label for the action.
    pub label: String,
    /// Whether this action is currently available.
    pub available: bool,
    /// Optional hotkey character.
    pub hotkey: Option<String>,
    /// Font icon or character to display.
    pub icon: String,
}

impl Default for ActionSlot {
    fn default() -> Self {
        Self {
            action_type: ActionType::None,
            label: String::new(),
            available: true,
            hotkey: None,
            icon: String::new(),
        }
    }
}

/// Types of actions available in the game UI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionType {
    None,
    Move,
    Attack,
    Patrol,
    Guard,
    Stop,
    Build,
    Train,
    Repair,
    Gather,
    HeroAbility,
    Flag,
    Heroes,
    Blacksmith,
    Oba,
    Barracks,
    Alpa,
    HorseArcher,
}

impl ActionType {
    /// Get the display label for this action type.
    pub fn label(&self) -> &'static str {
        match self {
            ActionType::None => "",
            ActionType::Move => "Move",
            ActionType::Attack => "Attack",
            ActionType::Patrol => "Patrol",
            ActionType::Guard => "Guard",
            ActionType::Stop => "Stop",
            ActionType::Build => "Build",
            ActionType::Train => "Train",
            ActionType::Repair => "Repair",
            ActionType::Gather => "Gather",
            ActionType::HeroAbility => "Ability",
            ActionType::Flag => "Flag",
            ActionType::Heroes => "Heroes",
            ActionType::Blacksmith => "Blacksmith",
            ActionType::Oba => "Oba",
            ActionType::Barracks => "Barracks",
            ActionType::Alpa => "Alpa",
            ActionType::HorseArcher => "H.Archer",
        }
    }

    /// Get the display icon character for this action type.
    pub fn icon(&self) -> &'static str {
        match self {
            ActionType::None => "",
            ActionType::Move => "🚩",
            ActionType::Attack => "⚔",
            ActionType::Patrol => "🔄",
            ActionType::Guard => "🛡",
            ActionType::Stop => "⏹",
            ActionType::Build => "🔨",
            ActionType::Train => "🎓",
            ActionType::Repair => "🔧",
            ActionType::Gather => "🌾",
            ActionType::HeroAbility => "⭐",
            ActionType::Flag => "🏴",
            ActionType::Heroes => "👤",
            ActionType::Blacksmith => "⚒",
            ActionType::Oba => "🏠",
            ActionType::Barracks => "🏰",
            ActionType::Alpa => "🐴",
            ActionType::HorseArcher => "🏹",
        }
    }
}

// ─── HUD Settings (Phase 4: Polish) ─────────────────────────────────────────

/// HUD accessibility and display settings.
/// Controls font scaling, per-panel visibility, and colourblind-friendly mode.
#[derive(Resource, Clone, Debug)]
pub struct HudSettings {
    /// Font size multiplier (0.5 – 2.0, default 1.0).
    pub font_scale: f32,
    /// Colourblind-friendly display mode.
    pub colorblind_mode: ColorblindMode,
    /// Per-panel visibility toggles.
    pub panel_visibility: Vec<(HudPanel, bool)>,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            font_scale: 1.0,
            colorblind_mode: ColorblindMode::Normal,
            panel_visibility: vec![
                (HudPanel::TopBar, true),
                (HudPanel::SelectionPanel, true),
                (HudPanel::CommandBar, true),
                (HudPanel::HeroBar, true),
                (HudPanel::Notifications, true),
                (HudPanel::Objectives, true),
                (HudPanel::Minimap, true),
                (HudPanel::HeroPanel, true),
                (HudPanel::UnitStats, true),
                (HudPanel::ActionBar, true),
                (HudPanel::CommandPanel, true),
                (HudPanel::SettingsButton, true),
            ],
        }
    }
}

impl HudSettings {
    /// Returns `true` if the given panel is currently visible.
    pub fn is_visible(&self, panel: HudPanel) -> bool {
        self.panel_visibility
            .iter()
            .find(|(p, _)| *p == panel)
            .map(|(_, v)| *v)
            .unwrap_or(true)
    }

    /// Set visibility for a panel by name string (for key-binding hooks).
    pub fn set_panel_by_name(&mut self, name: &str, visible: bool) {
        for (panel, v) in &mut self.panel_visibility {
            if panel.name() == name {
                *v = visible;
                return;
            }
        }
    }
}

/// Identifies each HUD panel for visibility toggles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HudPanel {
    TopBar,
    SelectionPanel,
    CommandBar,
    HeroBar,
    Notifications,
    Objectives,
    Minimap,
    HeroPanel,
    UnitStats,
    ActionBar,
    CommandPanel,
    SettingsButton,
}

impl HudPanel {
    /// Stable string key for this panel (used in settings serialisation).
    pub fn name(self) -> &'static str {
        match self {
            HudPanel::TopBar => "top_bar",
            HudPanel::SelectionPanel => "selection_panel",
            HudPanel::CommandBar => "command_bar",
            HudPanel::HeroBar => "hero_bar",
            HudPanel::Notifications => "notifications",
            HudPanel::Objectives => "objectives",
            HudPanel::Minimap => "minimap",
            HudPanel::HeroPanel => "hero_panel",
            HudPanel::UnitStats => "unit_stats",
            HudPanel::ActionBar => "action_bar",
            HudPanel::CommandPanel => "command_panel",
            HudPanel::SettingsButton => "settings_button",
        }
    }
}

/// Colourblind-friendly display mode for the HUD.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ColorblindMode {
    /// Default — no colourblind accommodations.
    #[default]
    Normal,
    /// Red-blind (difficulty distinguishing red).
    Protanopia,
    /// Green-blind (difficulty distinguishing green).
    Deuteranopia,
    /// Blue-blind (difficulty distinguishing blue).
    Tritanopia,
    /// Total colour blindness (greyscale).
    Achromatopsia,
}
