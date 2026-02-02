//! DoT (Damage over Time) system

mod active;
pub mod tick;
mod types;

pub use active::ActiveDoT;
pub use tick::apply_dot;
pub use types::{DotConfig, DotStacking};

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
