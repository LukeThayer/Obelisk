//! DoT configuration loading

use crate::dot::{DotConfig, DotRegistry};
use super::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Container for DoT configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotsConfig {
    #[serde(rename = "dot_types")]
    pub dot_types: Vec<DotConfig>,
}

/// Load DoT configurations from a TOML file
pub fn load_dot_configs(path: &Path) -> Result<DotRegistry, ConfigError> {
    let config: DotsConfig = super::load_toml(path)?;

    let mut registry = DotRegistry::new();
    for dot in config.dot_types {
        registry.register(dot);
    }

    Ok(registry)
}

/// Load DoT configurations from a TOML string
pub fn parse_dot_configs(content: &str) -> Result<DotRegistry, ConfigError> {
    let config: DotsConfig = super::parse_toml(content)?;

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
