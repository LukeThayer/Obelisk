# tables_core

Drop table configuration and rolling for loot generation. Tables are defined in TOML and return drop specifications that integrate with `loot_core`.

## Usage

### Loading Tables

```rust
use tables_core::DropTableRegistry;
use std::path::Path;

// Load all tables from a directory (recursively)
let tables = DropTableRegistry::load(Path::new("config/tables"))?;

// Check if a table exists
if tables.contains("goblin") {
    println!("Goblin table loaded");
}

// List all loaded tables
for id in tables.table_ids() {
    println!("Loaded table: {}", id);
}
```

### Rolling Drops

```rust
use tables_core::{DropTableRegistry, DropsExt};
use rand::thread_rng;

let tables = DropTableRegistry::load(Path::new("config/tables"))?;
let mut rng = thread_rng();

// Roll the table
let drops = tables.roll(
    "goblin",       // table id
    1.0,            // rarity multiplier
    1.0,            // quantity multiplier
    15,             // player/area level
    &mut rng,
)?;

// Process items
for item in drops.get_items() {
    println!("Dropped item: {}", item.base_type);
    println!("  Apply currencies: {:?}", item.currencies);
}

// Process currencies
for currency in drops.get_currencies() {
    println!("Dropped {} x {}", currency.count, currency.id);
}

// Process uniques
for unique in drops.get_uniques() {
    println!("Dropped unique: {}", unique.id);
}
```

### Integration with loot_core

```rust
use tables_core::{DropTableRegistry, DropsExt};
use loot_core::{Generator, Config};

let config = Config::load_from_dir(Path::new("config"))?;
let generator = Generator::new(config);
let tables = DropTableRegistry::load(Path::new("config/tables"))?;

let mut rng = rand::thread_rng();
let drops = tables.roll("boss", 1.5, 1.0, 50, &mut rng)?;

// Generate items and apply currencies
for item in drops.get_items() {
    let mut generated = generator.generate(item.base_type, rng.gen())?;
    for currency_id in item.currencies {
        generated = generator.apply_currency(&generated, currency_id)?;
    }
    inventory.add(generated);
}

// Add currencies to player
for currency in drops.get_currencies() {
    player.add_currency(currency.id, currency.count);
}

// Generate uniques
for unique in drops.get_uniques() {
    let generated = generator.generate_unique(unique.id, rng.gen())?;
    inventory.add(generated);
}
```

## How Drop Tables Work

### Table Structure

Each table has:
- **id**: Unique identifier for the table
- **rolls**: Weighted options for how many times to roll
- **entries**: Weighted list of possible drops

```toml
[table]
id = "goblin"

[[table.rolls]]
count = 1
weight = 50    # 50% chance for 1 roll

[[table.rolls]]
count = 2
weight = 30    # 30% chance for 2 rolls

[[table.rolls]]
count = 3
weight = 20    # 20% chance for 3 rolls

[[entries]]
type = "item"
base_type = "iron_sword"
currencies = ["transmute", "augment"]
weight = 10
# ... more entries
```

### Entry Types

| Type | Description | Fields |
|------|-------------|--------|
| `no_drop` | Nothing drops | `weight` |
| `item` | Generate an item | `base_type`, `currencies`, `weight`, `rarity_bonus`, `min_level`, `max_level` |
| `currency` | Drop currency | `id`, `count`, `weight`, `rarity_bonus`, `min_level`, `max_level` |
| `unique` | Drop a unique item | `id`, `weight`, `rarity_bonus`, `min_level`, `max_level` |
| `table` | Roll another table | `id`, `weight`, `rarity_bonus`, `min_level`, `max_level` |

### Level Filtering

Entries can specify level requirements:

```toml
[[entries]]
type = "item"
base_type = "iron_sword"
weight = 10
min_level = 1
max_level = 30    # Only drops for levels 1-30

[[entries]]
type = "unique"
id = "titans_grip"
weight = 1
min_level = 50    # Only drops at level 50+
```

Entries outside the current level range are excluded from the roll.

### Multipliers

#### Rarity Multiplier

Increases the effective weight of entries with `rarity_bonus`:

```
effective_weight = weight + (rarity_bonus × rarity_mult)
```

Example with `rarity_mult = 2.0`:
```toml
[[entries]]
type = "item"
weight = 10
rarity_bonus = 20
# effective_weight = 10 + (20 × 2.0) = 50
```

Use higher rarity multipliers for magic find bonuses or special events.

#### Quantity Multiplier

Scales both roll count and currency amounts using floor + fractional chance:

```
scaled = base × quantity_mult
result = floor(scaled) + (1 if random < frac(scaled) else 0)
```

Examples:
- `rolls=2, quantity_mult=1.5` → 3 guaranteed rolls
- `rolls=2, quantity_mult=1.3` → 2 guaranteed + 60% chance for 3rd
- `currency count=10, quantity_mult=1.5` → 15 guaranteed

### Nested Tables

Tables can reference other tables for modular drop pools:

```toml
# gems_common.toml
[table]
id = "gems_common"

[[entries]]
type = "currency"
id = "imbue_fire"
weight = 30

[[entries]]
type = "currency"
id = "imbue_life"
weight = 30
```

```toml
# goblin.toml
[table]
id = "goblin"

[[entries]]
type = "table"
id = "gems_common"    # Rolls the gems_common table
weight = 15
```

Nested tables inherit the same `rarity_mult`, `quantity_mult`, and `level` parameters. Cycle detection prevents infinite loops (max depth of 10).

### Currency Counts

Currency entries support single values or ranges:

```toml
# Fixed count
[[entries]]
type = "currency"
id = "transmute"
count = 1
weight = 20

# Random range
[[entries]]
type = "currency"
id = "chaos"
count = [1, 5]    # Drops 1-5, then scaled by quantity_mult
weight = 10
```

## Configuration Directory

Recommended structure:

```
config/
└── tables/
    ├── monsters/
    │   ├── goblin.toml
    │   └── boss.toml
    ├── chests/
    │   └── common.toml
    └── gems_common.toml
```

Tables are loaded recursively, so subdirectories are supported.

## Error Handling

```rust
use tables_core::{DropTableRegistry, ConfigError, RollError};

// Loading errors
match DropTableRegistry::load(Path::new("config/tables")) {
    Ok(tables) => { /* use tables */ }
    Err(ConfigError::Io { error, path }) => {
        eprintln!("Failed to read {:?}: {}", path, error);
    }
    Err(ConfigError::Parse { error, path }) => {
        eprintln!("Invalid TOML in {:?}: {}", path, error);
    }
    Err(ConfigError::Validation { message, path }) => {
        eprintln!("Invalid table in {:?}: {}", path, message);
    }
}

// Rolling errors
match tables.roll("goblin", 1.0, 1.0, 10, &mut rng) {
    Ok(drops) => { /* process drops */ }
    Err(RollError::UnknownTable(id)) => {
        eprintln!("Table not found: {}", id);
    }
    Err(RollError::CycleDetected(id)) => {
        eprintln!("Circular reference at table: {}", id);
    }
    Err(RollError::InvalidEntryType(t)) => {
        eprintln!("Unknown entry type: {}", t);
    }
}
```
