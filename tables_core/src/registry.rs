use crate::config::TableFileConfig;
use crate::drop::Drop;
use crate::table::DropTable;
use crate::{ConfigError, RollError};
use rand::Rng;
use std::collections::HashMap;
use std::path::Path;

/// Registry of all drop tables, loaded from TOML files
#[derive(Debug, Default)]
pub struct DropTableRegistry {
    tables: HashMap<String, DropTable>,
}

impl DropTableRegistry {
    /// Create an empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all drop tables from a directory (recursively)
    pub fn load(dir: &Path) -> Result<Self, ConfigError> {
        let mut registry = Self::new();
        registry.load_dir(dir)?;
        Ok(registry)
    }

    /// Load tables from a directory recursively
    fn load_dir(&mut self, dir: &Path) -> Result<(), ConfigError> {
        if !dir.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(|e| ConfigError::Io {
            error: e,
            path: Some(dir.to_path_buf()),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| ConfigError::Io {
                error: e,
                path: Some(dir.to_path_buf()),
            })?;
            let path = entry.path();

            if path.is_dir() {
                self.load_dir(&path)?;
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                self.load_file(&path)?;
            }
        }

        Ok(())
    }

    /// Load a single table file
    fn load_file(&mut self, path: &Path) -> Result<(), ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
            error: e,
            path: Some(path.to_path_buf()),
        })?;

        let config: TableFileConfig = toml::from_str(&content).map_err(|e| ConfigError::Parse {
            error: e,
            path: path.to_path_buf(),
        })?;

        let table = DropTable::from_config(config).map_err(|e| ConfigError::Validation {
            message: e.to_string(),
            path: path.to_path_buf(),
        })?;

        self.tables.insert(table.id.clone(), table);
        Ok(())
    }

    /// Get a table by ID
    pub fn get(&self, id: &str) -> Option<&DropTable> {
        self.tables.get(id)
    }

    /// Check if a table exists
    pub fn contains(&self, id: &str) -> bool {
        self.tables.contains_key(id)
    }

    /// List all table IDs
    pub fn table_ids(&self) -> impl Iterator<Item = &str> {
        self.tables.keys().map(|s| s.as_str())
    }

    /// Roll a table by ID
    pub fn roll<R: Rng>(
        &self,
        table_id: &str,
        rarity_mult: f64,
        quantity_mult: f64,
        level: u32,
        rng: &mut R,
    ) -> Result<Vec<Drop>, RollError> {
        let table = self
            .get(table_id)
            .ok_or_else(|| RollError::UnknownTable(table_id.to_string()))?;

        table.roll(rarity_mult, quantity_mult, level, rng, self, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_table(dir: &Path, name: &str, content: &str) {
        let path = dir.join(format!("{}.toml", name));
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_load_simple_table() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "test",
            r#"
[table]
id = "test"

[[entries]]
type = "no_drop"
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        assert!(registry.contains("test"));
    }

    #[test]
    fn test_roll_no_drop() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "empty",
            r#"
[table]
id = "empty"

[[entries]]
type = "no_drop"
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let drops = registry.roll("empty", 1.0, 1.0, 10, &mut rng).unwrap();
        assert!(drops.is_empty());
    }

    #[test]
    fn test_roll_item() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "items",
            r#"
[table]
id = "items"

[[entries]]
type = "item"
base_type = "iron_sword"
currencies = ["transmute"]
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let drops = registry.roll("items", 1.0, 1.0, 10, &mut rng).unwrap();

        assert_eq!(drops.len(), 1);
        match &drops[0] {
            Drop::Item {
                base_type,
                currencies,
            } => {
                assert_eq!(base_type, "iron_sword");
                assert_eq!(currencies, &vec!["transmute".to_string()]);
            }
            _ => panic!("Expected Item drop"),
        }
    }

    #[test]
    fn test_roll_currency() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "gold",
            r#"
[table]
id = "gold"

[[entries]]
type = "currency"
id = "gold"
count = [10, 10]
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let drops = registry.roll("gold", 1.0, 1.0, 10, &mut rng).unwrap();

        assert_eq!(drops.len(), 1);
        match &drops[0] {
            Drop::Currency { id, count } => {
                assert_eq!(id, "gold");
                assert_eq!(*count, 10);
            }
            _ => panic!("Expected Currency drop"),
        }
    }

    #[test]
    fn test_roll_unique() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "uniques",
            r#"
[table]
id = "uniques"

[[entries]]
type = "unique"
id = "starforge"
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let drops = registry.roll("uniques", 1.0, 1.0, 10, &mut rng).unwrap();

        assert_eq!(drops.len(), 1);
        match &drops[0] {
            Drop::Unique { id } => {
                assert_eq!(id, "starforge");
            }
            _ => panic!("Expected Unique drop"),
        }
    }

    #[test]
    fn test_nested_tables() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "inner",
            r#"
[table]
id = "inner"

[[entries]]
type = "currency"
id = "gold"
count = 5
weight = 100
"#,
        );
        create_test_table(
            dir.path(),
            "outer",
            r#"
[table]
id = "outer"

[[entries]]
type = "table"
id = "inner"
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let drops = registry.roll("outer", 1.0, 1.0, 10, &mut rng).unwrap();

        assert_eq!(drops.len(), 1);
        match &drops[0] {
            Drop::Currency { id, count } => {
                assert_eq!(id, "gold");
                assert_eq!(*count, 5);
            }
            _ => panic!("Expected Currency drop"),
        }
    }

    #[test]
    fn test_level_filtering() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "leveled",
            r#"
[table]
id = "leveled"

[[entries]]
type = "currency"
id = "low_level"
count = 1
weight = 100
max_level = 10

[[entries]]
type = "currency"
id = "high_level"
count = 1
weight = 100
min_level = 50
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();

        // At level 5, only low_level should drop
        let drops = registry.roll("leveled", 1.0, 1.0, 5, &mut rng).unwrap();
        assert_eq!(drops.len(), 1);
        assert!(matches!(&drops[0], Drop::Currency { id, .. } if id == "low_level"));

        // At level 60, only high_level should drop
        let drops = registry.roll("leveled", 1.0, 1.0, 60, &mut rng).unwrap();
        assert_eq!(drops.len(), 1);
        assert!(matches!(&drops[0], Drop::Currency { id, .. } if id == "high_level"));

        // At level 30, nothing matches (between 10 and 50)
        let drops = registry.roll("leveled", 1.0, 1.0, 30, &mut rng).unwrap();
        assert!(drops.is_empty());
    }

    #[test]
    fn test_unknown_table_error() {
        let registry = DropTableRegistry::new();
        let mut rng = rand::thread_rng();
        let result = registry.roll("nonexistent", 1.0, 1.0, 10, &mut rng);
        assert!(matches!(result, Err(RollError::UnknownTable(_))));
    }

    #[test]
    fn test_weighted_roll_count() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "weighted",
            r#"
[table]
id = "weighted"

[[table.rolls]]
count = 1
weight = 50

[[table.rolls]]
count = 2
weight = 50

[[entries]]
type = "currency"
id = "gold"
count = 1
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();

        // Over many iterations, should average ~1.5 drops
        let mut total = 0usize;
        let iterations = 10000;
        for _ in 0..iterations {
            let drops = registry.roll("weighted", 1.0, 1.0, 10, &mut rng).unwrap();
            total += drops.len();
        }

        let avg = total as f64 / iterations as f64;
        assert!(avg > 1.3 && avg < 1.7, "Average was {}", avg);
    }

    #[test]
    fn test_rarity_bonus() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "rarity",
            r#"
[table]
id = "rarity"

[[entries]]
type = "currency"
id = "common"
count = 1
weight = 100
rarity_bonus = 0

[[entries]]
type = "currency"
id = "rare"
count = 1
weight = 1
rarity_bonus = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();

        // With high rarity mult, "rare" should appear more often
        let mut rare_count = 0;
        let iterations = 1000;
        for _ in 0..iterations {
            let drops = registry.roll("rarity", 10.0, 1.0, 10, &mut rng).unwrap();
            if let Some(Drop::Currency { id, .. }) = drops.first() {
                if id == "rare" {
                    rare_count += 1;
                }
            }
        }

        // With weight 100 + 100*10 = 1100 for rare vs 100 for common
        // Rare should be ~91% of drops
        let rare_pct = rare_count as f64 / iterations as f64;
        assert!(rare_pct > 0.85, "Rare percentage was {}", rare_pct);
    }

    #[test]
    fn test_quantity_mult_currency() {
        let dir = TempDir::new().unwrap();
        create_test_table(
            dir.path(),
            "currency",
            r#"
[table]
id = "currency"

[[entries]]
type = "currency"
id = "gold"
count = [10, 10]
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();

        // With quantity_mult = 2.0, should get ~20 gold
        let mut total = 0u64;
        let iterations = 1000;
        for _ in 0..iterations {
            let drops = registry.roll("currency", 1.0, 2.0, 10, &mut rng).unwrap();
            if let Some(Drop::Currency { count, .. }) = drops.first() {
                total += *count as u64;
            }
        }

        let avg = total as f64 / iterations as f64;
        assert!(avg > 18.0 && avg < 22.0, "Average was {}", avg);
    }

    #[test]
    fn test_cycle_detection() {
        let dir = TempDir::new().unwrap();
        // Table A references Table B, which references Table A
        create_test_table(
            dir.path(),
            "table_a",
            r#"
[table]
id = "table_a"

[[entries]]
type = "table"
id = "table_b"
weight = 100
"#,
        );
        create_test_table(
            dir.path(),
            "table_b",
            r#"
[table]
id = "table_b"

[[entries]]
type = "table"
id = "table_a"
weight = 100
"#,
        );

        let registry = DropTableRegistry::load(dir.path()).unwrap();
        let mut rng = rand::thread_rng();
        let result = registry.roll("table_a", 1.0, 1.0, 10, &mut rng);
        assert!(matches!(result, Err(RollError::CycleDetected(_))));
    }
}
