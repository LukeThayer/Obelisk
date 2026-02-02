//! Skill configuration loading

use crate::damage::DamagePacketGenerator;
use super::ConfigError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Container for skill configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsConfig {
    #[serde(rename = "skills")]
    pub skills: Vec<DamagePacketGenerator>,
}

/// Load skill configurations from a TOML file
pub fn load_skill_configs(path: &Path) -> Result<HashMap<String, DamagePacketGenerator>, ConfigError> {
    let config: SkillsConfig = super::load_toml(path)?;

    let mut map = HashMap::new();
    for skill in config.skills {
        map.insert(skill.id.clone(), skill);
    }

    Ok(map)
}

/// Load skill configurations from a TOML string
pub fn parse_skill_configs(content: &str) -> Result<HashMap<String, DamagePacketGenerator>, ConfigError> {
    let config: SkillsConfig = super::parse_toml(content)?;

    let mut map = HashMap::new();
    for skill in config.skills {
        map.insert(skill.id.clone(), skill);
    }

    Ok(map)
}

/// Get default skill configurations
pub fn default_skills() -> HashMap<String, DamagePacketGenerator> {
    let toml = include_str!("../../config/skills.toml");
    parse_skill_configs(toml).unwrap_or_else(|_| {
        let mut map = HashMap::new();
        map.insert("basic_attack".to_string(), DamagePacketGenerator::basic_attack());
        map
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skills() {
        let toml = r#"
[[skills]]
id = "fireball"
name = "Fireball"
tags = ["spell", "fire", "projectile", "aoe"]
weapon_effectiveness = 0.0
damage_effectiveness = 1.0
attack_speed_modifier = 1.0
base_crit_chance = 6.0
crit_multiplier_bonus = 0.0

[[skills.base_damages]]
type = "fire"
min = 100
max = 180

[skills.status_conversions]
fire_to_burn = 0.50
"#;

        let skills = parse_skill_configs(toml).unwrap();
        assert!(skills.contains_key("fireball"));

        let fireball = &skills["fireball"];
        assert_eq!(fireball.name, "Fireball");
        assert!((fireball.weapon_effectiveness - 0.0).abs() < f64::EPSILON);
        assert!((fireball.status_conversions.fire_to_burn - 0.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_skills_loads_all() {
        let skills = super::default_skills();

        // Should have 12 skills from config
        assert_eq!(skills.len(), 12, "Expected 12 skills from config");

        // Check all expected skills are present
        let expected = [
            "basic_attack",
            "fireball",
            "heavy_strike",
            "molten_strike",
            "blade_vortex",
            "viper_strike",
            "ice_nova",
            "glacial_hammer",
            "lightning_strike",
            "wild_strike",
            "double_strike",
            "infernal_blow",
        ];

        for id in expected {
            assert!(skills.contains_key(id), "Missing skill: {}", id);
        }
    }
}
