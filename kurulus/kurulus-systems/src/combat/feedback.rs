use bevy::prelude::*;
use kurulus_core::prelude::*;

// ═════════════════════════════════════════════════════════════════════════════
//  HIT FEEDBACK SYSTEM (Phase 5e — NEW)
// ═════════════════════════════════════════════════════════════════════════════

/// Reads damage/combat events and emits `HitFeedbackEvent` for the UI layer.
/// The UI reads these events to display floating damage numbers, block/dodge
/// indicators, critical hit flashes, etc.
pub fn hit_feedback_system(
    mut damage_events: EventReader<DamageDealt>,
    mut charge_impact_events: EventReader<ChargeImpact>,
    mut ambush_events: EventReader<AmbushTriggered>,
    mut ff_events: EventReader<RangedFriendlyFire>,
    positions: Query<&WorldPosition>,
    mut hit_feedback: EventWriter<HitFeedbackEvent>,
) {
    // ─── Process DamageDealt events ─────────────────────────────────────
    for ev in damage_events.read() {
        let pos = positions
            .get(ev.target)
            .map(|p| p.world_2d.0)
            .unwrap_or_default();

        let feedback_type = if ev.is_friendly_fire {
            HitFeedbackType::FriendlyFire
        } else if ev.crit != CritType::None {
            HitFeedbackType::Critical
        } else {
            HitFeedbackType::Hit
        };

        hit_feedback.send(HitFeedbackEvent {
            entity: ev.target,
            feedback_type,
            amount: ev.amount,
            position: pos,
        });
    }

    // ─── Process ChargeImpact events ────────────────────────────────────
    for ev in charge_impact_events.read() {
        let pos = positions
            .get(ev.target)
            .map(|p| p.world_2d.0)
            .unwrap_or_default();

        hit_feedback.send(HitFeedbackEvent {
            entity: ev.target,
            feedback_type: HitFeedbackType::ChargeImpact,
            amount: ev.damage,
            position: pos,
        });
    }

    // ─── Process AmbushTriggered events ─────────────────────────────────
    for ev in ambush_events.read() {
        let pos = positions
            .get(ev.target)
            .map(|p| p.world_2d.0)
            .unwrap_or_default();

        hit_feedback.send(HitFeedbackEvent {
            entity: ev.target,
            feedback_type: HitFeedbackType::AmbushBonus,
            amount: ev.damage_bonus,
            position: pos,
        });
    }

    // ─── Process RangedFriendlyFire events ──────────────────────────────
    for ev in ff_events.read() {
        let pos = positions
            .get(ev.target)
            .map(|p| p.world_2d.0)
            .unwrap_or_default();

        hit_feedback.send(HitFeedbackEvent {
            entity: ev.target,
            feedback_type: HitFeedbackType::FriendlyFire,
            amount: ev.damage,
            position: pos,
        });
    }
}
