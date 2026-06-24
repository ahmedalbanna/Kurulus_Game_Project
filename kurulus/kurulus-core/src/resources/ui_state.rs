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
