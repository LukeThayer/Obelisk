# loot_core

Item generation with seed-based determinism, affixes, and crafting currencies.

## Quick Start

```rust
use loot_core::{Config, Generator, Item, BinaryEncode, BinaryDecode};
use std::path::Path;

// Load config and create generator
let config = Config::load_from_dir(Path::new("config"))?;
let generator = Generator::new(config);

// Generate an item (same seed = same item)
let item = generator.generate("iron_sword", 12345)?;

// Apply currencies (immutable - returns new item)
let item = generator.apply_currency(&item, "transmute")?;  // Normal -> Magic
let item = generator.apply_currency(&item, "augment")?;    // Add affix

// Compact binary encoding (~30 bytes vs hundreds for JSON)
let bytes = item.encode_to_vec();
let loaded = Item::decode_from_slice(&bytes, &generator)?;
```

## Core Concepts

- **Seed-based determinism** - Items store seed + operations, reconstructed deterministically
- **Immutable operations** - `apply_currency` returns a new item
- **Data-driven** - All currencies and affixes defined in TOML
- **Tag-based weighting** - Matching tags increase affix spawn probability

## Configuration

```
config/
├── base_types/   # Item base definitions
├── affixes/      # Affix definitions with tiers
├── currencies/   # Currency effects
└── uniques/      # Unique item templates
```

## License

MIT
