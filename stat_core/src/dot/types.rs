//! DoT type definitions

use loot_core::types::DamageType;
use serde::{Deserialize, Serialize};

/// DoT stacking behavior
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DotStacking {
    /// Only the strongest instance deals damage
    StrongestOnly,
    /// All instances stack and deal damage independently
    Unlimited,
    /// Strongest + up to N stacks at reduced effectiveness
    Limited {
        max_stacks: u32,
        /// Effectiveness of additional stacks (e.g., 0.5 = 50%)
        stack_effectiveness: f64,
    },
}

/// Configuration for a DoT type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotConfig {
    /// Unique identifier (e.g., "ignite", "poison", "bleed")
    pub id: String,
    /// Display name
    pub name: String,
    /// The damage type this DoT deals
    pub damage_type: DamageType,
    /// How instances stack
    pub stacking: DotStacking,
    /// Base duration in seconds
    pub base_duration: f64,
    /// Time between damage ticks
    pub tick_rate: f64,
    /// Base damage percent - what percentage of status damage becomes DoT DPS
    /// For example, 0.20 means 20% of status damage becomes DPS
    #[serde(default)]
    pub base_damage_percent: f64,
    /// Maximum number of stacks (for limited stacking)
    #[serde(default = "default_max_stacks")]
    pub max_stacks: u32,
    /// Effectiveness of additional stacks
    #[serde(default = "default_stack_effectiveness")]
    pub stack_effectiveness: f64,
    /// Damage multiplier while target is moving (for bleed)
    #[serde(default = "default_moving_multiplier")]
    pub moving_multiplier: f64,
}

fn default_max_stacks() -> u32 {
    1
}

fn default_stack_effectiveness() -> f64 {
    1.0
}

fn default_moving_multiplier() -> f64 {
    1.0
}

impl DotConfig {
    /// Calculate the number of ticks for this DoT's base duration
    pub fn base_tick_count(&self) -> u32 {
        (self.base_duration / self.tick_rate).ceil() as u32
    }

    /// Calculate total duration accounting for tick alignment
    pub fn actual_duration(&self) -> f64 {
        self.base_tick_count() as f64 * self.tick_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_stacking_serialization() {
        let stacking = DotStacking::Limited {
            max_stacks: 8,
            stack_effectiveness: 0.5,
        };
        let json = serde_json::to_string(&stacking).unwrap();
        assert!(json.contains("limited"));
    }

    #[test]
    fn test_dot_config_tick_count() {
        let config = DotConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            damage_type: DamageType::Fire,
            stacking: DotStacking::StrongestOnly,
            base_duration: 4.0,
            tick_rate: 0.5,
            base_damage_percent: 0.25,
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        };

        // 4.0 / 0.5 = 8 ticks
        assert_eq!(config.base_tick_count(), 8);
    }
}
