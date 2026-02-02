use serde::Deserialize;

/// TOML configuration for a drop table file
#[derive(Debug, Deserialize)]
pub struct TableFileConfig {
    pub table: TableConfig,
    #[serde(default)]
    pub entries: Vec<EntryConfig>,
}

/// Configuration for the table itself
#[derive(Debug, Deserialize)]
pub struct TableConfig {
    pub id: String,
    #[serde(default)]
    pub rolls: Vec<RollConfig>,
}

/// Weighted roll count option
#[derive(Debug, Deserialize)]
pub struct RollConfig {
    pub count: u32,
    pub weight: u32,
}

/// Configuration for a single entry in the drop table
#[derive(Debug, Deserialize)]
pub struct EntryConfig {
    #[serde(rename = "type")]
    pub entry_type: String,
    #[serde(default)]
    pub weight: u32,
    #[serde(default)]
    pub rarity_bonus: u32,
    #[serde(default)]
    pub min_level: Option<u32>,
    #[serde(default)]
    pub max_level: Option<u32>,

    // Item-specific fields
    #[serde(default)]
    pub base_type: Option<String>,
    #[serde(default)]
    pub currencies: Vec<String>,

    // Currency-specific fields
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub count: Option<CountConfig>,
    // For single count value (not a range)
    // This allows `count = 5` instead of `count = [5, 5]`
}

/// Count can be a single value or a range [min, max]
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum CountConfig {
    Single(u32),
    Range([u32; 2]),
}

impl CountConfig {
    pub fn min(&self) -> u32 {
        match self {
            CountConfig::Single(v) => *v,
            CountConfig::Range([min, _]) => *min,
        }
    }

    pub fn max(&self) -> u32 {
        match self {
            CountConfig::Single(v) => *v,
            CountConfig::Range([_, max]) => *max,
        }
    }
}
