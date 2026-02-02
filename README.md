# Obelisk

A Rust game mechanics library for action RPGs. Three crates that work together:

- **loot_core** - Item generation with seed-based determinism, affixes, and crafting currencies
- **stat_core** - Stat aggregation, damage calculation, and combat resolution
- **tables_core** - Drop table configuration and weighted loot rolling

## Quick Start

```rust
use loot_core::{Config, Generator};
use tables_core::{DropTableRegistry, Drop};
use stat_core::{StatBlock, EquipmentSlot, default_skills};
use stat_core::damage::calculate_damage;
use stat_core::combat::resolve_damage;
use rand::thread_rng;
use std::path::Path;

fn main() {
    let mut rng = thread_rng();

    // Load configurations
    let config = Config::load_from_dir(Path::new("config")).unwrap();
    let generator = Generator::new(config);
    let tables = DropTableRegistry::load(Path::new("config/tables")).unwrap();

    // Roll a drop table for loot
    let drops = tables.roll("goblin", 1.0, 1.0, 15, &mut rng).unwrap();

    for drop in drops {
        match drop {
            Drop::Item { base_type, currencies } => {
                // Generate item and apply currencies
                let mut item = generator.generate(&base_type, rng.gen()).unwrap();
                for currency in currencies {
                    item = generator.apply_currency(&item, &currency).unwrap();
                }
                println!("Dropped: {}", item.name());
            }
            Drop::Currency { id, count } => {
                println!("Dropped {} x {}", count, id);
            }
            Drop::Unique { id } => {
                let item = generator.generate_unique(&id, rng.gen()).unwrap();
                println!("Dropped unique: {}", item.name());
            }
        }
    }

    // Equip items and manage stats with stat_core
    let mut player = StatBlock::new();
    player.max_life.base = 100.0;
    player.current_life = 100.0;

    let mut enemy = StatBlock::new();
    enemy.max_life.base = 500.0;
    enemy.current_life = 500.0;
    enemy.armour.base = 200.0;

    // Combat
    let skills = default_skills();
    let skill = skills.get("heavy_strike").unwrap();

    let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
    let (enemy, result) = resolve_damage(&enemy, &packet);

    println!("Dealt {} damage, enemy has {} HP", result.total_damage, enemy.current_life);
}
```

## Crate Details

See individual READMEs for full API documentation:
- [loot_core/README.md](loot_core/README.md) - Item generation, currencies, binary serialization
- [stat_core/README.md](stat_core/README.md) - StatBlock, damage formulas, defense mechanics, DoT system
- [tables_core/README.md](tables_core/README.md) - Drop tables, weighted rolls, rarity/quantity multipliers

## License

MIT
