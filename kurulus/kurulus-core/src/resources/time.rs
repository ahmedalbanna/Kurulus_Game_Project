use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::events::world::SeasonType;

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct GameTime {
    pub total_seconds: f64,
    pub minute: u8,
    pub hour: u8,
    pub day: u32,
    pub season: SeasonType,
    pub year: u32,
    pub time_scale: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            total_seconds: 0.0,
            minute: 0,
            hour: 8,
            day: 1,
            season: SeasonType::Spring,
            year: 1280,
            time_scale: 1.0,
        }
    }
}

impl GameTime {
    pub const MINUTES_PER_DAY: u32 = 24 * 60;
    pub const DAYS_PER_SEASON: u32 = 30;
    pub const SEASONS_PER_YEAR: u32 = 4;

    pub fn advance(&mut self, delta: f64) {
        self.total_seconds += delta * self.time_scale as f64;
        let total_minutes = (self.total_seconds / 60.0) as u32;
        self.minute = (total_minutes % 60) as u8;
        self.hour = ((total_minutes / 60) % 24) as u8;
        self.day = (total_minutes / (24 * 60)) % Self::DAYS_PER_SEASON + 1;
        let season_index = (total_minutes / (24 * 60 * Self::DAYS_PER_SEASON)) % Self::SEASONS_PER_YEAR;
        self.season = match season_index {
            0 => SeasonType::Spring,
            1 => SeasonType::Summer,
            2 => SeasonType::Autumn,
            _ => SeasonType::Winter,
        };
        self.year = 1280 + (total_minutes / (24 * 60 * Self::DAYS_PER_SEASON * Self::SEASONS_PER_YEAR)) as u32;
    }

    pub fn is_night(&self) -> bool {
        self.hour < 6 || self.hour >= 20
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct TickTimer {
    pub tick_interval: f32,
    pub accumulator: f32,
}

impl Default for TickTimer {
    fn default() -> Self {
        Self {
            tick_interval: 0.1,
            accumulator: 0.0,
        }
    }
}
