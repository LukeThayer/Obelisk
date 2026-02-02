//! Game constants configuration

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::OnceLock;

use super::ConfigError;

/// Global game constants instance
static GAME_CONSTANTS: OnceLock<GameConstants> = OnceLock::new();

/// Initialize the global game constants from a TOML file
///
/// Must be called once at startup before any combat calculations.
/// Returns error if already initialized or if loading fails.
pub fn init_constants(path: &Path) -> Result<(), ConfigError> {
    let constants = GameConstants::load_from_path(path)?;
    GAME_CONSTANTS
        .set(constants)
        .map_err(|_| ConfigError::ValidationError("GameConstants already initialized".to_string()))
}

/// Initialize the global game constants with default values
///
/// Useful for tests or when no config file is available.
pub fn init_constants_default() -> Result<(), ConfigError> {
    GAME_CONSTANTS
        .set(GameConstants::default())
        .map_err(|_| ConfigError::ValidationError("GameConstants already initialized".to_string()))
}

/// Get a reference to the global game constants
///
/// Panics if constants have not been initialized via `init_constants()` or `init_constants_default()`.
pub fn constants() -> &'static GameConstants {
    GAME_CONSTANTS
        .get()
        .expect("GameConstants not initialized - call init_constants() or init_constants_default() first")
}

/// Check if constants have been initialized
pub fn constants_initialized() -> bool {
    GAME_CONSTANTS.get().is_some()
}

/// Ensure constants are initialized with defaults (idempotent, useful for tests)
///
/// If constants are already initialized, this does nothing.
/// If not initialized, initializes with default values.
pub fn ensure_constants_initialized() {
    GAME_CONSTANTS.get_or_init(GameConstants::default);
}

/// Tunable game constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConstants {
    #[serde(default)]
    pub resistances: ResistanceConstants,
    #[serde(default)]
    pub armour: ArmourConstants,
    #[serde(default)]
    pub evasion: EvasionConstants,
    #[serde(default)]
    pub crit: CritConstants,
    #[serde(default)]
    pub leech: LeechConstants,
    #[serde(default)]
    pub energy_shield: EnergyShieldConstants,
}

impl Default for GameConstants {
    fn default() -> Self {
        GameConstants {
            resistances: ResistanceConstants::default(),
            armour: ArmourConstants::default(),
            evasion: EvasionConstants::default(),
            crit: CritConstants::default(),
            leech: LeechConstants::default(),
            energy_shield: EnergyShieldConstants::default(),
        }
    }
}

impl GameConstants {
    /// Load constants from a TOML file
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let constants: GameConstants = toml::from_str(&content)?;
        Ok(constants)
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
pub struct EvasionConstants {
    /// Scaling factor for evasion formula: cap = accuracy / (1 + evasion / scale_factor)
    #[serde(default = "default_scale_factor")]
    pub scale_factor: f64,
}

impl Default for EvasionConstants {
    fn default() -> Self {
        EvasionConstants {
            scale_factor: 1000.0,
        }
    }
}

fn default_scale_factor() -> f64 {
    1000.0
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
