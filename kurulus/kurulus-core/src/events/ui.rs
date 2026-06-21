use bevy::prelude::*;

#[derive(Event, Clone, Debug)]
pub struct SelectionChanged {
    pub selected: Vec<Entity>,
    pub previous: Vec<Entity>,
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
