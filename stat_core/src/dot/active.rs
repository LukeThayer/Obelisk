//! ActiveDoT - Tracking active DoT instances on an entity

use loot_core::types::DamageType;
use serde::{Deserialize, Serialize};

/// An active DoT instance on an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDoT {
    /// DoT type ID (e.g., "ignite", "poison", "bleed")
    pub dot_type: String,
    /// Source entity ID (for stacking rules)
    pub source_id: String,
    /// Damage type of this DoT
    pub damage_type: DamageType,
    /// Damage per tick
    pub damage_per_tick: f64,
    /// Time between ticks
    pub tick_rate: f64,
    /// Time until next tick
    pub time_until_tick: f64,
    /// Duration remaining
    pub duration_remaining: f64,
    /// Total duration (for percentage calculations)
    pub total_duration: f64,
    /// Effectiveness multiplier (for stacking)
    pub effectiveness: f64,
    /// Whether this is the "strongest" instance for stacking purposes
    pub is_strongest: bool,
}

impl ActiveDoT {
    /// Create a new active DoT
    pub fn new(
        dot_type: String,
        source_id: String,
        damage_type: DamageType,
        damage_per_tick: f64,
        tick_rate: f64,
        duration: f64,
    ) -> Self {
        ActiveDoT {
            dot_type,
            source_id,
            damage_type,
            damage_per_tick,
            tick_rate,
            time_until_tick: tick_rate,
            duration_remaining: duration,
            total_duration: duration,
            effectiveness: 1.0,
            is_strongest: true,
        }
    }

    /// Get DPS (damage per second) of this DoT
    pub fn dps(&self) -> f64 {
        if self.tick_rate <= 0.0 {
            return 0.0;
        }
        (self.damage_per_tick / self.tick_rate) * self.effectiveness
    }

    /// Get total remaining damage
    pub fn total_remaining_damage(&self) -> f64 {
        let remaining_ticks = (self.duration_remaining / self.tick_rate).ceil();
        remaining_ticks * self.damage_per_tick * self.effectiveness
    }

    /// Check if the DoT is still active
    pub fn is_active(&self) -> bool {
        self.duration_remaining > 0.0
    }

    /// Get percentage of duration remaining
    pub fn duration_percent(&self) -> f64 {
        if self.total_duration <= 0.0 {
            return 0.0;
        }
        (self.duration_remaining / self.total_duration * 100.0).clamp(0.0, 100.0)
    }

    /// Refresh the DoT duration (for reapplication)
    pub fn refresh(&mut self, new_duration: f64, new_damage_per_tick: f64) {
        // For strongest-only stacking, take the higher damage
        if new_damage_per_tick > self.damage_per_tick {
            self.damage_per_tick = new_damage_per_tick;
        }
        // Always refresh duration
        self.duration_remaining = new_duration;
        self.total_duration = new_duration;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_dot_dps() {
        let dot = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            50.0,  // 50 damage per tick
            0.5,   // tick every 0.5 seconds
            4.0,   // 4 second duration
        );

        // DPS = 50 / 0.5 = 100
        assert!((dot.dps() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_active_dot_total_damage() {
        let dot = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            50.0,
            0.5,
            4.0,
        );

        // 8 ticks * 50 damage = 400 total
        assert!((dot.total_remaining_damage() - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_active_dot_refresh() {
        let mut dot = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            50.0,
            0.5,
            4.0,
        );

        // Refresh with lower damage - should keep higher
        dot.refresh(4.0, 30.0);
        assert!((dot.damage_per_tick - 50.0).abs() < 0.01);

        // Refresh with higher damage - should update
        dot.refresh(4.0, 70.0);
        assert!((dot.damage_per_tick - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_active_dot_effectiveness() {
        let mut dot = ActiveDoT::new(
            "bleed".to_string(),
            "player".to_string(),
            DamageType::Physical,
            100.0,
            1.0,
            5.0,
        );
        dot.effectiveness = 0.5;

        // DPS = 100 * 0.5 = 50
        assert!((dot.dps() - 50.0).abs() < 0.01);
    }
}
