//! DoT configuration loading with global registry support

use crate::dot::{DotConfig, DotRegistry};
use super::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

/// Global DoT registry instance
static DOT_REGISTRY: OnceLock<DotRegistry> = OnceLock::new();

/// Container for DoT configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotsConfig {
    #[serde(rename = "dot_types")]
    pub dot_types: Vec<DotConfig>,
}

/// Initialize the global DoT registry from a config file
pub fn init_dot_registry(path: &Path) -> Result<(), ConfigError> {
    let registry = load_dot_configs(path)?;
    DOT_REGISTRY.set(registry).ok();
    Ok(())
}

/// Initialize the global DoT registry with default path (config/dots.toml)
pub fn init_dot_registry_default() -> Result<(), ConfigError> {
    init_dot_registry(Path::new("config/dots.toml"))
}

/// Get a reference to the global DoT registry
/// Panics if not initialized - call init_dot_registry first
pub fn dot_registry() -> &'static DotRegistry {
    DOT_REGISTRY.get().expect("DoT registry not initialized. Call init_dot_registry() first.")
}

/// Check if the DoT registry has been initialized
pub fn dot_registry_initialized() -> bool {
    DOT_REGISTRY.get().is_some()
}

/// Ensure the DoT registry is initialized (for tests)
/// Uses an empty registry if not already initialized
pub fn ensure_dot_registry_initialized() {
    DOT_REGISTRY.get_or_init(DotRegistry::new);
}

/// Load DoT configurations from a TOML file (returns registry, doesn't set global)
pub fn load_dot_configs(path: &Path) -> Result<DotRegistry, ConfigError> {
    let config: DotsConfig = super::load_toml(path)?;

    let mut registry = DotRegistry::new();
    for dot in config.dot_types {
        registry.register(dot);
    }

    Ok(registry)
}

/// Parse DoT configurations from a TOML string (for testing)
pub fn parse_dot_configs(toml: &str) -> Result<DotRegistry, ConfigError> {
    let config: DotsConfig = super::parse_toml(toml)?;

    let mut registry = DotRegistry::new();
    for dot in config.dot_types {
        registry.register(dot);
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dots() {
        let toml = r#"
[[dot_types]]
id = "ignite"
name = "Ignite"
damage_type = "fire"
base_duration = 4.0
tick_rate = 0.5

[dot_types.stacking]
type = "strongest_only"

[[dot_types]]
id = "poison"
name = "Poison"
damage_type = "chaos"
base_duration = 2.0
tick_rate = 0.33

[dot_types.stacking]
type = "unlimited"

[[dot_types]]
id = "bleed"
name = "Bleed"
damage_type = "physical"
base_duration = 5.0
tick_rate = 1.0
moving_multiplier = 2.0

[dot_types.stacking]
type = "limited"
max_stacks = 8
stack_effectiveness = 0.5
"#;

        let registry = parse_dot_configs(toml).unwrap();
        assert!(registry.get("ignite").is_some());
        assert!(registry.get("poison").is_some());
        assert!(registry.get("bleed").is_some());

        let bleed = registry.get("bleed").unwrap();
        assert!((bleed.moving_multiplier - 2.0).abs() < f64::EPSILON);
    }
}
