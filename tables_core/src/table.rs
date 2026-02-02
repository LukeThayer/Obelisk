use crate::config::{CountConfig, EntryConfig, TableFileConfig};
use crate::drop::Drop;
use crate::RollError;
use rand::Rng;

/// A drop table with weighted roll counts and entries
#[derive(Debug, Clone)]
pub struct DropTable {
    pub id: String,
    rolls: Vec<RollOption>,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
struct RollOption {
    count: u32,
    weight: u32,
}

#[derive(Debug, Clone)]
struct Entry {
    entry_type: EntryType,
    weight: u32,
    rarity_bonus: u32,
    min_level: Option<u32>,
    max_level: Option<u32>,
}

#[derive(Debug, Clone)]
enum EntryType {
    NoDrop,
    Item {
        base_type: String,
        currencies: Vec<String>,
    },
    Currency {
        id: String,
        count: CountRange,
    },
    Unique {
        id: String,
    },
    Table {
        id: String,
    },
}

#[derive(Debug, Clone, Copy)]
struct CountRange {
    min: u32,
    max: u32,
}

impl DropTable {
    /// Parse a drop table from config
    pub fn from_config(config: TableFileConfig) -> Result<Self, RollError> {
        let rolls: Vec<RollOption> = if config.table.rolls.is_empty() {
            // Default to 1 roll if none specified
            vec![RollOption {
                count: 1,
                weight: 1,
            }]
        } else {
            config
                .table
                .rolls
                .into_iter()
                .map(|r| RollOption {
                    count: r.count,
                    weight: r.weight,
                })
                .collect()
        };

        let entries: Vec<Entry> = config
            .entries
            .into_iter()
            .map(Entry::from_config)
            .collect::<Result<_, _>>()?;

        Ok(DropTable {
            id: config.table.id,
            rolls,
            entries,
        })
    }

    /// Roll this table and return the drops
    pub fn roll<R: Rng>(
        &self,
        rarity_mult: f64,
        quantity_mult: f64,
        level: u32,
        rng: &mut R,
        registry: &crate::DropTableRegistry,
        depth: u32,
    ) -> Result<Vec<Drop>, RollError> {
        const MAX_DEPTH: u32 = 10;
        if depth > MAX_DEPTH {
            return Err(RollError::CycleDetected(self.id.clone()));
        }

        // Select base roll count from weighted options
        let base_rolls = self.select_roll_count(rng);

        // Apply quantity multiplier with fractional chance
        let roll_count = apply_quantity_mult(base_rolls, quantity_mult, rng);

        let mut drops = Vec::new();

        for _ in 0..roll_count {
            // Filter entries by level
            let valid_entries: Vec<&Entry> = self
                .entries
                .iter()
                .filter(|e| e.level_valid(level))
                .collect();

            if valid_entries.is_empty() {
                continue;
            }

            // Calculate effective weights with rarity bonus
            let weights: Vec<f64> = valid_entries
                .iter()
                .map(|e| e.weight as f64 + e.rarity_bonus as f64 * rarity_mult)
                .collect();

            let total_weight: f64 = weights.iter().sum();
            if total_weight <= 0.0 {
                continue;
            }

            // Weighted random selection
            let mut roll = rng.gen::<f64>() * total_weight;
            let mut selected_idx = 0;
            for (i, &w) in weights.iter().enumerate() {
                roll -= w;
                if roll <= 0.0 {
                    selected_idx = i;
                    break;
                }
            }

            let entry = valid_entries[selected_idx];

            match &entry.entry_type {
                EntryType::NoDrop => continue,
                EntryType::Item {
                    base_type,
                    currencies,
                } => {
                    drops.push(Drop::Item {
                        base_type: base_type.clone(),
                        currencies: currencies.clone(),
                    });
                }
                EntryType::Unique { id } => {
                    drops.push(Drop::Unique { id: id.clone() });
                }
                EntryType::Currency { id, count } => {
                    let base_count = rng.gen_range(count.min..=count.max);
                    let final_count = apply_quantity_mult(base_count, quantity_mult, rng);
                    if final_count > 0 {
                        drops.push(Drop::Currency {
                            id: id.clone(),
                            count: final_count,
                        });
                    }
                }
                EntryType::Table { id } => {
                    let nested_table = registry
                        .get(id)
                        .ok_or_else(|| RollError::UnknownTable(id.clone()))?;
                    let nested_drops = nested_table.roll(
                        rarity_mult,
                        quantity_mult,
                        level,
                        rng,
                        registry,
                        depth + 1,
                    )?;
                    drops.extend(nested_drops);
                }
            }
        }

        Ok(drops)
    }

    fn select_roll_count<R: Rng>(&self, rng: &mut R) -> u32 {
        let total_weight: u32 = self.rolls.iter().map(|r| r.weight).sum();
        if total_weight == 0 {
            return 1;
        }

        let mut roll = rng.gen_range(0..total_weight);
        for option in &self.rolls {
            if roll < option.weight {
                return option.count;
            }
            roll -= option.weight;
        }

        self.rolls.last().map(|r| r.count).unwrap_or(1)
    }
}

impl Entry {
    fn from_config(config: EntryConfig) -> Result<Self, RollError> {
        let entry_type = match config.entry_type.as_str() {
            "no_drop" => EntryType::NoDrop,
            "item" => EntryType::Item {
                base_type: config.base_type.unwrap_or_default(),
                currencies: config.currencies,
            },
            "unique" => EntryType::Unique {
                id: config.id.unwrap_or_default(),
            },
            "currency" => {
                let count = config.count.unwrap_or(CountConfig::Single(1));
                EntryType::Currency {
                    id: config.id.unwrap_or_default(),
                    count: CountRange {
                        min: count.min(),
                        max: count.max(),
                    },
                }
            }
            "table" => EntryType::Table {
                id: config.id.unwrap_or_default(),
            },
            _ => {
                return Err(RollError::InvalidEntryType(config.entry_type));
            }
        };

        Ok(Entry {
            entry_type,
            weight: config.weight,
            rarity_bonus: config.rarity_bonus,
            min_level: config.min_level,
            max_level: config.max_level,
        })
    }

    fn level_valid(&self, level: u32) -> bool {
        if let Some(min) = self.min_level {
            if level < min {
                return false;
            }
        }
        if let Some(max) = self.max_level {
            if level > max {
                return false;
            }
        }
        true
    }
}

/// Apply quantity multiplier with fractional chance for extra
fn apply_quantity_mult<R: Rng>(base: u32, mult: f64, rng: &mut R) -> u32 {
    let scaled = base as f64 * mult;
    let guaranteed = scaled.floor() as u32;
    let fraction = scaled.fract();

    if rng.gen::<f64>() < fraction {
        guaranteed + 1
    } else {
        guaranteed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_quantity_mult_no_fraction() {
        let mut rng = rand::thread_rng();
        // 2 * 1.5 = 3.0, no fraction
        let result = apply_quantity_mult(2, 1.5, &mut rng);
        assert_eq!(result, 3);
    }

    #[test]
    fn test_apply_quantity_mult_with_fraction() {
        // Test that over many iterations, 2 * 1.3 averages to ~2.6
        let mut rng = rand::thread_rng();
        let mut total = 0u64;
        let iterations = 10000;

        for _ in 0..iterations {
            total += apply_quantity_mult(2, 1.3, &mut rng) as u64;
        }

        let avg = total as f64 / iterations as f64;
        // Should be around 2.6, allow some variance
        assert!(avg > 2.4 && avg < 2.8, "Average was {}", avg);
    }

    #[test]
    fn test_level_filtering() {
        let entry = Entry {
            entry_type: EntryType::NoDrop,
            weight: 100,
            rarity_bonus: 0,
            min_level: Some(10),
            max_level: Some(30),
        };

        assert!(!entry.level_valid(5));
        assert!(entry.level_valid(10));
        assert!(entry.level_valid(20));
        assert!(entry.level_valid(30));
        assert!(!entry.level_valid(31));
    }

    #[test]
    fn test_level_filtering_no_limits() {
        let entry = Entry {
            entry_type: EntryType::NoDrop,
            weight: 100,
            rarity_bonus: 0,
            min_level: None,
            max_level: None,
        };

        assert!(entry.level_valid(0));
        assert!(entry.level_valid(100));
    }
}
