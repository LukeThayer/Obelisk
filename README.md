# Obelisk

A Rust game mechanics library for action RPGs. Two crates that work together:

- **loot_core** - Item generation with seed-based determinism, affixes, and crafting currencies
- **stat_core** - Stat aggregation, damage calculation, and combat resolution

## Quick Start

```rust
use loot_core::{Config, Generator};
use stat_core::{StatBlock, EquipmentSlot, default_skills};
use stat_core::damage::calculate_damage;
use stat_core::combat::resolve_damage;
use rand::thread_rng;
use std::path::Path;

fn main() {
    // Generate items with loot_core
    let config = Config::load_from_dir(Path::new("config")).unwrap();
    let generator = Generator::new(config);

    let sword = generator.generate("iron_sword", 12345).unwrap();
    let sword = generator.apply_currency(&sword, "transmute").unwrap(); // Add affixes

    // Equip items and manage stats with stat_core
    let mut player = StatBlock::new();
    player.max_life.base = 100.0;
    player.current_life = 100.0;
    player.equip(EquipmentSlot::MainHand, sword);

    let mut enemy = StatBlock::new();
    enemy.max_life.base = 500.0;
    enemy.current_life = 500.0;
    enemy.armour.base = 200.0;

    // Combat - load skill from config
    let skills = default_skills();
    let skill = skills.get("heavy_strike").unwrap();
    let mut rng = thread_rng();

    let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
    let (enemy, result) = resolve_damage(&enemy, &packet);

    println!("Dealt {} damage, enemy has {} HP", result.total_damage, enemy.current_life);
}
```

## Crate Details

See individual READMEs for full API documentation:
- [loot_core/README.md](loot_core/README.md) - Item generation, currencies, binary serialization
- [stat_core/README.md](stat_core/README.md) - StatBlock, damage formulas, defense mechanics, DoT system

## License

MIT
