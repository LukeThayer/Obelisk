# stat_core

A game mechanics library for stat management, damage calculation, and combat resolution in action RPGs. Built with the triple-modifier (Flat/Increased/More) stacking model.

## Quick Start

```rust
use stat_core::{
    StatBlock, DamagePacketGenerator,
    damage::calculate_damage,
    combat::resolve_damage,
    types::EquipmentSlot,
};
use loot_core::{Config, Generator};
use rand::thread_rng;
use std::path::Path;

fn main() {
    // Load loot generator and create a weapon
    let config = Config::load_from_dir(Path::new("config")).unwrap();
    let generator = Generator::new(config);
    let sword = generator.generate("iron_sword", 42).unwrap();

    // Create player and equip weapon
    let mut player = StatBlock::new();
    player.max_life.base = 100.0;
    player.current_life = 100.0;
    player.equip(EquipmentSlot::MainHand, sword);

    println!("Equipped weapon: {:?}", player.equipped(EquipmentSlot::MainHand).map(|i| &i.name));

    // Create enemy
    let mut enemy = StatBlock::new();
    enemy.max_life.base = 500.0;
    enemy.current_life = 500.0;
    enemy.armour.base = 200.0;

    // Attack
    let skill = DamagePacketGenerator::basic_attack();
    let mut rng = thread_rng();
    let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
    let (enemy, result) = resolve_damage(&enemy, &packet);

    println!("Dealt {} damage, enemy has {} HP remaining",
        result.total_damage, enemy.current_life);

    // Unequip weapon (stats automatically recalculated)
    let removed = player.unequip(EquipmentSlot::MainHand);
    println!("Unequipped: {:?}", removed.map(|i| i.name));
}
```

## Installation

```toml
[dependencies]
stat_core = { path = "../stat_core" }
```

---

## Core Concepts

### StatValue: Triple-Modifier Model

All stats use `StatValue` with the formula:

```
Final = (base + flat) × (1 + increased) × ∏(1 + more[i])
```

```rust
let mut damage = StatValue::new(100.0);
damage.add_flat(20.0);           // +20 flat
damage.add_increased(0.50);      // 50% increased (additive with other increased)
damage.add_more(0.20);           // 20% more (multiplicative)
// Result: (100 + 20) × 1.50 × 1.20 = 216
```

### StatBlock

Complete character state including:
- **Resources**: life, mana, energy shield
- **Attributes**: strength, dexterity, intelligence, etc.
- **Defenses**: armour, evasion, resistances
- **Offense**: damage stats, crit, penetration
- **Weapon**: min/max damage, attack speed, crit chance
- **Effects**: active buffs, debuffs, and ailments

---

## Skills

Skills are defined with `DamagePacketGenerator`:

```rust
let fireball = DamagePacketGenerator {
    id: "fireball".into(),
    name: "Fireball".into(),
    base_damages: vec![BaseDamage::new(DamageType::Fire, 100.0, 150.0)],
    weapon_effectiveness: 0.0,    // Spell - no weapon damage
    damage_effectiveness: 1.0,    // Full scaling
    base_crit_chance: 6.0,
    tags: vec![SkillTag::Spell, SkillTag::Fire],
    ..Default::default()
};
```

Key fields:
| Field | Description |
|-------|-------------|
| `weapon_effectiveness` | 0.0 = spell, 1.0 = full weapon damage |
| `damage_effectiveness` | Multiplier for all damage scaling |
| `damage_conversions` | Convert physical → elemental before scaling |
| `status_conversions` | % of damage that applies status effects |

---

## Unified Effect System

All buffs, debuffs, and ailments use a single `Effect` type:

```rust
use stat_core::{Effect, StatusEffect};

// Create a poison effect using helper constructor
let poison = Effect::poison(50.0, "attacker"); // 50 DPS

// Apply to target
enemy.add_effect(poison);

// Tick effects over time (immutable API)
let (new_enemy, result) = enemy.tick_effects(1.0);
println!("DoT dealt {} damage", result.dot_damage);
```

### Ailment Helpers

```rust
Effect::poison(dps, source)   // Chaos DoT, unlimited stacking
Effect::bleed(dps, source)    // Physical DoT, limited stacking
Effect::burn(dps, source)     // Fire DoT, strongest only
Effect::freeze(mag, source)   // Immobilize
Effect::chill(mag, source)    // Slow
Effect::shock(mag, source)    // Increased damage taken
Effect::fear(mag, source)     // Flee
Effect::slow(mag, source)     // Movement penalty
```

### Stacking Rules

| Mode | Behavior | Used By |
|------|----------|---------|
| `StrongestOnly` | Only strongest applies | Burn, Freeze, Chill |
| `Unlimited` | All instances stack | Poison |
| `Limited` | Up to max stacks | Bleed, Shock |

---

## Defense Mechanics

### Armour
Reduces physical damage with diminishing returns:
```
Reduction% = Armour / (Armour + 10 × Damage)
```

### Evasion
Caps maximum damage from a single hit:
```
Damage Cap = Accuracy / (1 + Evasion/1000)
```

### Resistance
Flat percentage reduction (capped at 75%):
```
Final = Damage × (1 - (Resistance - Penetration))
```

---

## Combat Flow

Both functions use immutable APIs that return new state:

```
1. calculate_damage(&attacker, &skill, source_id, &mut rng) → DamagePacket
   - Roll weapon/skill damage
   - Apply damage conversions
   - Apply scaling (increased/more)
   - Calculate crit
   - Generate pending status effects

2. resolve_damage(&defender, &packet) → (StatBlock, CombatResult)
   - Apply resistances
   - Apply armour (physical only)
   - Apply evasion cap
   - Damage ES, then life
   - Roll status effect applications
   - Return new defender state + damage breakdown
```

---

## Stat Sources

There are two ways to manage equipment and stat sources:

### Simple: Built-in Equip/Unequip

StatBlock has built-in equipment management that automatically rebuilds stats:

```rust
use stat_core::{StatBlock, types::EquipmentSlot};
use loot_core::{Config, Generator};

let config = Config::load_from_dir("config").unwrap();
let generator = Generator::new(config);

let mut player = StatBlock::new();
player.max_life.base = 100.0;

// Equip items - stats rebuild automatically
let sword = generator.generate("iron_sword", 42).unwrap();
let helmet = generator.generate("iron_helmet", 43).unwrap();

player.equip(EquipmentSlot::MainHand, sword);
player.equip(EquipmentSlot::Helmet, helmet);

// Check what's equipped
if let Some(weapon) = player.equipped(EquipmentSlot::MainHand) {
    println!("Wielding: {}", weapon.name);
}

// Unequip - stats rebuild automatically, item returned
let removed_helmet = player.unequip(EquipmentSlot::Helmet);
println!("Removed: {:?}", removed_helmet.map(|i| i.name));

// Iterate all equipped items
for (slot, item) in player.all_equipped() {
    println!("{:?}: {}", slot, item.name);
}
```

### Advanced: External Source Management

For complex scenarios (skill trees, auras, temporary buffs), manage sources externally:

```rust
use stat_core::{
    StatBlock,
    source::{BaseStatsSource, GearSource, StatSource},
    types::EquipmentSlot,
};
use loot_core::{Config, Generator};

let config = Config::load_from_dir("config").unwrap();
let generator = Generator::new(config);

// Create sources
let base_stats = BaseStatsSource::new(10);  // Level 10
let sword = generator.generate("iron_sword", 42).unwrap();
let helmet = generator.generate("iron_helmet", 43).unwrap();

let mut sources: Vec<Box<dyn StatSource>> = vec![
    Box::new(base_stats),
    Box::new(GearSource::new(EquipmentSlot::MainHand, sword)),
    Box::new(GearSource::new(EquipmentSlot::Helmet, helmet)),
];

// Build stats from sources
let mut player = StatBlock::new();
player.rebuild_from_sources(&sources);
player.current_life = player.computed_max_life();

// To remove a source: filter it out and rebuild
sources.retain(|s| s.id() != "gear_helmet");  // Remove helmet
player.rebuild_from_sources(&sources);

// To add a source: push and rebuild
let shield = generator.generate("wooden_shield", 44).unwrap();
sources.push(Box::new(GearSource::new(EquipmentSlot::OffHand, shield)));
player.rebuild_from_sources(&sources);
```

### Priority Order

Sources are applied from lowest to highest priority. This ensures base stats are set before equipment modifies them.

| Source | Priority | What It Provides |
|--------|----------|------------------|
| `BaseStatsSource` | -100 | Life (+12/level), mana (+6/level), base attributes (10 each) |
| `GearSource` | 0 | Weapon damage, armour, evasion, affixes from items |
| `SkillTreeSource` | 100 | Passive skill bonuses |
| Custom buffs | 200+ | Temporary stat modifications |

### BaseStatsSource

Provides level-scaled base stats:

```rust
let base = BaseStatsSource::new(level);
// Level 1:  62 life, 46 mana
// Level 10: 170 life, 100 mana
// Formula: 50 + (12 × level) life, 40 + (6 × level) mana
```

### GearSource

Converts `loot_core::Item` into stat modifiers. Each GearSource has an ID of `gear_{slot}`:

```rust
use loot_core::currency::apply_currency;

// Generate and craft an item
let mut sword = generator.generate("iron_sword", 42).unwrap();
let transmute = generator.config().currencies.get("transmute").unwrap();
apply_currency(&generator, &mut sword, transmute, &mut rng).unwrap();

// Create gear source (ID will be "gear_main_hand")
let gear = GearSource::new(EquipmentSlot::MainHand, sword);
```

GearSource handles:
- **Weapon stats**: Base damage, attack speed, crit chance
- **Armour/Evasion**: From body armour, shields, helmets
- **Affixes**: Both prefixes and suffixes with their stat modifiers
- **Local vs Global scope**: Local modifiers (e.g., "increased physical damage" on weapons) apply to the item; global modifiers apply to all stats

### Custom Sources

Implement `StatSource` for custom stat providers:

```rust
use stat_core::{source::StatSource, stat_block::StatAccumulator};

struct AuraSource {
    increased_damage: f64,
}

impl StatSource for AuraSource {
    fn id(&self) -> &str { "aura" }
    fn priority(&self) -> i32 { 150 }  // After gear, before temporary buffs

    fn apply(&self, acc: &mut StatAccumulator) {
        acc.global_physical_damage.add_increased(self.increased_damage);
    }
}

// Add aura
sources.push(Box::new(AuraSource { increased_damage: 0.30 }));
player.rebuild_from_sources(&sources);

// Remove aura later
sources.retain(|s| s.id() != "aura");
player.rebuild_from_sources(&sources);
```

---

## Configuration

Load skills from TOML:

```toml
[[skills]]
id = "heavy_strike"
name = "Heavy Strike"
tags = ["attack", "melee", "physical"]
weapon_effectiveness = 1.0
damage_effectiveness = 1.5

[skills.status_conversions]
physical_to_bleed = 0.25
```

```rust
use stat_core::default_skills;

let skills = default_skills();
let heavy_strike = skills.get("heavy_strike").unwrap();
```

---

## Full Example: Combat Loop

```rust
use stat_core::{
    StatBlock, DamagePacketGenerator,
    damage::calculate_damage,
    combat::resolve_damage,
};
use rand::thread_rng;

fn main() {
    let mut player = StatBlock::new();
    player.weapon_physical_min = 80.0;
    player.weapon_physical_max = 120.0;
    player.weapon_attack_speed = 1.4;
    player.global_physical_damage.add_increased(0.75);

    let mut enemy = StatBlock::new();
    enemy.max_life.base = 1000.0;
    enemy.current_life = 1000.0;
    enemy.armour.base = 500.0;
    enemy.fire_resistance.base = 30.0;

    let skill = DamagePacketGenerator::basic_attack();
    let mut rng = thread_rng();
    let mut time = 0.0;

    while enemy.is_alive() && time < 10.0 {
        // Attack (immutable - returns new enemy state)
        let packet = calculate_damage(&player, &skill, "player".into(), &mut rng);
        let (new_enemy, result) = resolve_damage(&enemy, &packet);
        enemy = new_enemy;

        println!("[{:.1}s] {} damage → {:.0} HP",
            time, result.total_damage, enemy.current_life);

        // Tick effects (also immutable)
        if !enemy.effects.is_empty() {
            let (new_enemy, tick) = enemy.tick_effects(0.5);
            enemy = new_enemy;
            if tick.dot_damage > 0.0 {
                println!("  DoT: {} damage", tick.dot_damage);
            }
        }

        time += 1.0 / player.computed_attack_speed();
    }

    println!("Combat ended at {:.1}s - Enemy {}",
        time, if enemy.is_alive() { "survived" } else { "defeated" });
}
```

---

## License

MIT
