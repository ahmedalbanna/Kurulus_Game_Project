use bevy::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct SelectionChanged {
    pub selected: Vec<Entity>,
    pub previous: Vec<Entity>,
}

// ─── Selection Events (from render layer) ──────────────────────────────

/// Emitted by the render layer when the user clicks on an entity or empty ground.
/// `entity` = `Some(Entity)` if the click hit a selectable entity,
/// `None` if clicking on empty ground (deselect all).
#[derive(Event, Clone, Debug)]
pub struct SelectEvent {
    pub entity: Option<Entity>,
    pub modifiers: SelectModifiers,
}

/// Keyboard modifiers held during an input event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// Emitted by the render layer when the user drags a selection rectangle.
/// The render layer is responsible for determining which entities fall
/// inside the screen-space rectangle (frustum culling + raycasting).
#[derive(Event, Clone, Debug)]
pub struct BoxSelectEvent {
    pub entities: Vec<Entity>,
    /// If true, adds to the current selection instead of replacing it (Shift held).
    pub additive: bool,
}

/// Emitted by the render layer when the user interacts with control groups
/// (Ctrl+1-9, 1-9, Shift+1-9).
#[derive(Event, Clone, Debug)]
pub struct ControlGroupEvent {
    pub group: u8,
    pub action: ControlGroupAction,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ControlGroupAction {
    /// Ctrl+number: store current selection into this group.
    Create,
    /// Number press: replace selection with this group's entities.
    Recall,
    /// Shift+number: add this group's entities to current selection.
    Append,
}

#[derive(Event, Clone, Debug)]
pub struct PanelOpened {
    pub panel_type: PanelType,
}

#[derive(Event, Clone, Debug)]
pub struct PanelClosed {
    pub panel_type: PanelType,
}

#[derive(Event, Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<f32>,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PanelType {
    UnitInfo,
    BuildingInfo,
    HeroInfo,
    TechTree,
    Diplomacy,
    MissionLog,
    MiniMap,
    BuildMenu,
    Recruitment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
    Achievement,
}

// ─── Formation Events ──────────────────────────────────────────────────────

/// Emitted when a formation breaks (from charge, casualties, or ambush).
#[derive(Event, Clone, Debug)]
pub struct FormationBreakEvent {
    pub leader: Entity,
    pub followers: Vec<Entity>,
    pub duration: f32,
    pub panic: bool,
}
