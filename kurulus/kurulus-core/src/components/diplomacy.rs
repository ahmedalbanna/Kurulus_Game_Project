use super::unit::FactionId;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Relations {
    pub faction_a: FactionId,
    pub faction_b: FactionId,
    pub standing: f32,
    pub trend: RelationTrend,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Treaty {
    pub treaty_type: TreatyType,
    pub parties: Vec<FactionId>,
    pub duration: f32,
    pub remaining: f32,
    pub terms: TreatyTerms,
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct TradeRoute {
    pub source: FactionId,
    pub destination: FactionId,
    pub goods: Vec<TradeGood>,
    pub frequency: f32,
    pub last_exchange: f32,
    pub security: f32,
}

// --- Enums ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationTrend {
    Improving,
    Stable,
    Deteriorating,
    Hostile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TreatyType {
    NonAggression,
    Alliance,
    TradeAgreement,
    MilitaryAccess,
    Vassalage,
    Peace,
    DefensivePact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreatyTerms {
    pub requires_tribute: bool,
    pub tribute_amount: u32,
    pub requires_military_support: bool,
    pub shared_vision: bool,
    pub border_guarantees: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TradeGood {
    pub good_type: GoodType,
    pub quantity: u32,
    pub price: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GoodType {
    Food,
    Gold,
    Wood,
    Stone,
    Horses,
    Iron,
    Silk,
    Spice,
}
