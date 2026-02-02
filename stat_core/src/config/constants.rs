//! Game constants configuration

use serde::{Deserialize, Serialize};

/// Tunable game constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstants {
    pub resistances: ResistanceConstants,
    pub armour: ArmourConstants,
    pub crit: CritConstants,
    pub leech: LeechConstants,
    pub energy_shield: EnergyShieldConstants,
}

impl Default for GameConstants {
    fn default() -> Self {
        GameConstants {
            resistances: ResistanceConstants::default(),
            armour: ArmourConstants::default(),
            crit: CritConstants::default(),
            leech: LeechConstants::default(),
            energy_shield: EnergyShieldConstants::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResistanceConstants {
    /// Maximum resistance percentage (100 = immunity)
    #[serde(default = "default_max_cap")]
    pub max_cap: f64,
    /// Minimum resistance (can go negative)
    #[serde(default = "default_min_value")]
    pub min_value: f64,
    /// Penetration effectiveness vs capped resistance
    #[serde(default = "default_pen_vs_capped")]
    pub penetration_vs_capped: f64,
}

impl Default for ResistanceConstants {
    fn default() -> Self {
        ResistanceConstants {
            max_cap: 100.0,
            min_value: -200.0,
            penetration_vs_capped: 0.5,
        }
    }
}

fn default_max_cap() -> f64 {
    100.0
}
fn default_min_value() -> f64 {
    -200.0
}
fn default_pen_vs_capped() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmourConstants {
    /// Formula constant: reduction = armour / (armour + constant * damage)
    #[serde(default = "default_damage_constant")]
    pub damage_constant: f64,
}

impl Default for ArmourConstants {
    fn default() -> Self {
        ArmourConstants {
            damage_constant: 5.0,
        }
    }
}

fn default_damage_constant() -> f64 {
    5.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CritConstants {
    /// Base critical strike multiplier (1.5 = 150%)
    #[serde(default = "default_base_multiplier")]
    pub base_multiplier: f64,
}

impl Default for CritConstants {
    fn default() -> Self {
        CritConstants {
            base_multiplier: 1.5,
        }
    }
}

fn default_base_multiplier() -> f64 {
    1.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeechConstants {
    /// Maximum life leeched per second as percentage of max life
    #[serde(default = "default_max_leech_rate")]
    pub max_life_leech_rate: f64,
    /// Maximum mana leeched per second as percentage of max mana
    #[serde(default = "default_max_leech_rate")]
    pub max_mana_leech_rate: f64,
}

impl Default for LeechConstants {
    fn default() -> Self {
        LeechConstants {
            max_life_leech_rate: 0.20,
            max_mana_leech_rate: 0.20,
        }
    }
}

fn default_max_leech_rate() -> f64 {
    0.20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyShieldConstants {
    /// Whether ES takes damage before life
    #[serde(default = "default_damage_priority")]
    pub damage_priority: String,
}

impl Default for EnergyShieldConstants {
    fn default() -> Self {
        EnergyShieldConstants {
            damage_priority: "first".to_string(),
        }
    }
}

fn default_damage_priority() -> String {
    "first".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_constants() {
        let constants = GameConstants::default();
        assert!((constants.resistances.max_cap - 100.0).abs() < f64::EPSILON);
        assert!((constants.armour.damage_constant - 5.0).abs() < f64::EPSILON);
        assert!((constants.crit.base_multiplier - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_constants() {
        let toml = r#"
[resistances]
max_cap = 100
min_value = -200
penetration_vs_capped = 0.5

[armour]
damage_constant = 5.0

[crit]
base_multiplier = 1.5

[leech]
max_life_leech_rate = 0.20
max_mana_leech_rate = 0.20

[energy_shield]
damage_priority = "first"
"#;

        let constants: GameConstants = toml::from_str(toml).unwrap();
        assert!((constants.resistances.max_cap - 100.0).abs() < f64::EPSILON);
    }
}
