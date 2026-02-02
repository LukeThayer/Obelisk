mod config;
mod drop;
mod registry;
mod table;

pub use drop::Drop;
pub use registry::DropTableRegistry;

use std::path::PathBuf;
use thiserror::Error;

/// Error loading drop table configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error reading '{path:?}': {error}")]
    Io {
        error: std::io::Error,
        path: Option<PathBuf>,
    },
    #[error("Parse error in '{path}': {error}")]
    Parse {
        error: toml::de::Error,
        path: PathBuf,
    },
    #[error("Validation error in '{path}': {message}")]
    Validation { message: String, path: PathBuf },
}

/// Error rolling a drop table
#[derive(Debug, Error)]
pub enum RollError {
    #[error("Unknown table: {0}")]
    UnknownTable(String),
    #[error("Cycle detected in table references: {0}")]
    CycleDetected(String),
    #[error("Invalid entry type: {0}")]
    InvalidEntryType(String),
}
