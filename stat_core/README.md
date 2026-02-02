# stat_core

Stat aggregation, damage calculation, and combat resolution for action RPGs.

## Quick Start

```rust
use stat_core::{StatBlock, DamagePacket, CombatResult, EquipmentSlot, default_skills};
use stat_core::damage::calculate_damage;
use stat_core::combat::resolve_damage;
use rand::thread_rng;

// Create characters
let mut player = StatBlock::new();
player.weapon_physical_min = 80.0;
player.weapon_physical_max = 120.0;
player.global_physical_damage.add_increased(0.50);  // 50% increased

let mut enemy = StatBlock::new();
enemy.max_life.base = 500.0;
enemy.current_life = 500.0;
enemy.armour.base = 200.0;

// Combat
let skills = default_skills();
let skill = skills.get("heavy_strike").unwrap();
let mut rng = thread_rng();

let packet = calculate_damage(&player, skill, "player".into(), &mut rng);
let (enemy, result) = resolve_damage(&enemy, &packet);

println!("Dealt {} damage", result.total_damage);
```

## Core Concepts

### StatValue

All stats use the triple-modifier formula:
```
Final = (base + flat) × (1 + Σincreased) × Π(1 + more)
```

```rust
let mut damage = StatValue::new(100.0);
damage.add_flat(20.0);       // +20
damage.add_increased(0.50);  // 50% increased (additive)
damage.add_more(0.20);       // 20% more (multiplicative)
// = (100 + 20) × 1.50 × 1.20 = 216
```

### Equipment

```rust
use stat_core::EquipmentSlot;
use loot_core::{Config, Generator};

let generator = Generator::new(Config::load_from_dir("config")?);
let sword = generator.generate("iron_sword", 42)?;

let mut player = StatBlock::new();
player.equip(EquipmentSlot::MainHand, sword);  // Stats auto-rebuild
player.unequip(EquipmentSlot::MainHand);       // Returns the item
```

### Defense Mechanics

| Defense | Formula |
|---------|---------|
| Armour | `Reduction% = Armour / (Armour + 10 × Damage)` |
| Evasion | `Damage Cap = Accuracy / (1 + Evasion/1000)` |
| Resistance | `Final = Damage × (1 - Resist + Pen)`, capped at 75% |

### Effects

Buffs, debuffs, and ailments use a unified `Effect` type. Status effects are config-driven via `config/dots.toml`:

```rust
// Tick effects over time
let (new_enemy, tick_result) = enemy.tick_effects(delta_time);
println!("DoT dealt {} damage", tick_result.dot_damage);
```

## Configuration

```
config/
├── constants.toml  # Game balance constants
├── dots.toml       # Status effect definitions
└── skills.toml     # Skill definitions
```

## License

MIT
