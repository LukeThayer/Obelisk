//! Configuration loading from TOML files

mod constants;
mod dots;
mod skills;

pub use constants::GameConstants;
pub use dots::load_dot_configs;
pub use skills::{default_skills, load_skill_configs};

use std::fs;
use std::path::Path;
use thiserror::Error;

/// Configuration loading error
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}

/// Load a TOML file and deserialize it
pub fn load_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config: T = toml::from_str(&content)?;
    Ok(config)
}

/// Load a TOML string and deserialize it
pub fn parse_toml<T: serde::de::DeserializeOwned>(content: &str) -> Result<T, ConfigError> {
    let config: T = toml::from_str(content)?;
    Ok(config)
}
