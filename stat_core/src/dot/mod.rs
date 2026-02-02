//! DoT (Damage over Time) system

mod active;
pub mod tick;
mod types;

pub use active::ActiveDoT;
pub use tick::apply_dot;
pub use types::{DotConfig, DotStacking};

use loot_core::types::DamageType;
use std::collections::HashMap;

/// DoT type registry
#[derive(Debug, Clone, Default)]
pub struct DotRegistry {
    /// Mapping from DoT type ID to configuration
    configs: HashMap<String, DotConfig>,
}

impl DotRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        DotRegistry {
            configs: HashMap::new(),
        }
    }

    /// Register a DoT type
    pub fn register(&mut self, config: DotConfig) {
        self.configs.insert(config.id.clone(), config);
    }

    /// Get a DoT configuration by ID
    pub fn get(&self, id: &str) -> Option<&DotConfig> {
        self.configs.get(id)
    }

    /// Load default DoT types
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Ignite/Burn - fire DoT, strongest only
        registry.register(DotConfig {
            id: "burn".to_string(),
            name: "Burn".to_string(),
            damage_type: DamageType::Fire,
            stacking: DotStacking::StrongestOnly,
            base_duration: 4.0,
            tick_rate: 0.5,
            base_damage_percent: 0.25, // 25% of status damage as DPS
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Poison - chaos DoT, unlimited stacking
        registry.register(DotConfig {
            id: "poison".to_string(),
            name: "Poison".to_string(),
            damage_type: DamageType::Chaos,
            stacking: DotStacking::Unlimited,
            base_duration: 2.0,
            tick_rate: 0.33,
            base_damage_percent: 0.20, // 20% of status damage as DPS
            max_stacks: 999,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Bleed - physical DoT, limited stacking
        registry.register(DotConfig {
            id: "bleed".to_string(),
            name: "Bleed".to_string(),
            damage_type: DamageType::Physical,
            stacking: DotStacking::Limited {
                max_stacks: 8,
                stack_effectiveness: 0.5,
            },
            base_duration: 5.0,
            tick_rate: 1.0,
            base_damage_percent: 0.20, // 20% of status damage as DPS
            max_stacks: 8,
            stack_effectiveness: 0.5,
            moving_multiplier: 2.0, // Bleed deals double damage while moving
        });

        // Freeze - cold status, no DoT damage
        registry.register(DotConfig {
            id: "freeze".to_string(),
            name: "Freeze".to_string(),
            damage_type: DamageType::Cold,
            stacking: DotStacking::StrongestOnly,
            base_duration: 0.5,
            tick_rate: 0.1,
            base_damage_percent: 0.0, // Non-damaging
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Chill - cold status, no DoT damage
        registry.register(DotConfig {
            id: "chill".to_string(),
            name: "Chill".to_string(),
            damage_type: DamageType::Cold,
            stacking: DotStacking::StrongestOnly,
            base_duration: 2.0,
            tick_rate: 0.5,
            base_damage_percent: 0.0, // Non-damaging
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Static - lightning status, no DoT damage
        registry.register(DotConfig {
            id: "static".to_string(),
            name: "Static".to_string(),
            damage_type: DamageType::Lightning,
            stacking: DotStacking::Limited {
                max_stacks: 3,
                stack_effectiveness: 1.0,
            },
            base_duration: 1.0,
            tick_rate: 0.25,
            base_damage_percent: 0.0, // Non-damaging (shock effect)
            max_stacks: 3,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Fear - chaos status, no DoT damage
        registry.register(DotConfig {
            id: "fear".to_string(),
            name: "Fear".to_string(),
            damage_type: DamageType::Chaos,
            stacking: DotStacking::StrongestOnly,
            base_duration: 1.5,
            tick_rate: 0.5,
            base_damage_percent: 0.0, // Non-damaging
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        // Slow - physical/cold status, no DoT damage
        registry.register(DotConfig {
            id: "slow".to_string(),
            name: "Slow".to_string(),
            damage_type: DamageType::Physical,
            stacking: DotStacking::StrongestOnly,
            base_duration: 3.0,
            tick_rate: 0.5,
            base_damage_percent: 0.0, // Non-damaging
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        });

        registry
    }

    /// Get the base damage percent for a status effect
    pub fn get_base_damage_percent(&self, status: loot_core::types::StatusEffect) -> f64 {
        use loot_core::types::StatusEffect;
        let id = match status {
            StatusEffect::Poison => "poison",
            StatusEffect::Bleed => "bleed",
            StatusEffect::Burn => "burn",
            StatusEffect::Freeze => "freeze",
            StatusEffect::Chill => "chill",
            StatusEffect::Static => "static",
            StatusEffect::Fear => "fear",
            StatusEffect::Slow => "slow",
        };
        self.get(id).map(|c| c.base_damage_percent).unwrap_or(0.0)
    }

    /// Get the base duration for a status effect
    pub fn get_base_duration(&self, status: loot_core::types::StatusEffect) -> f64 {
        use loot_core::types::StatusEffect;
        let id = match status {
            StatusEffect::Poison => "poison",
            StatusEffect::Bleed => "bleed",
            StatusEffect::Burn => "burn",
            StatusEffect::Freeze => "freeze",
            StatusEffect::Chill => "chill",
            StatusEffect::Static => "static",
            StatusEffect::Fear => "fear",
            StatusEffect::Slow => "slow",
        };
        self.get(id).map(|c| c.base_duration).unwrap_or(2.0)
    }
}
