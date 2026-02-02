# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Build
cargo build              # Debug build
cargo build --release    # Release build

# Test
cargo test               # Run all tests
cargo test <name>        # Run specific test by name

# Lint & Format
cargo clippy             # Run lints
cargo fmt                # Format code
cargo fmt --check        # Check formatting without changes
```

## Development Environment

This project uses Nix flakes for reproducible development. Enter the dev shell with:

```bash
nix develop
```

This provides: Rust stable toolchain, cargo-watch, cargo-edit, rust-analyzer, and git.

## Architecture

Cargo workspace with two library crates:

- **loot_core/**: Item generation - base types, affixes, currencies, uniques, and configuration loading
- **stat_core/**: Stat management - aggregates stats from multiple sources, handles damage calculation, defense mechanics, DoT systems, and combat resolution

### Key Design Concepts

**loot_core - Item Generation:**
- **Seed-based storage**: Items store a seed + operation history, replayed deterministically to reconstruct full stats
- **Tag-based affix weighting**: Items and affixes have tags; matching tags increase spawn weight
- **Immutable operations**: `apply_currency` returns a new item rather than mutating
- **Result-based errors**: `GeneratorError` provides context on failures (unknown base types, currency errors, etc.)

**stat_core - Stat Management:**
- **StatBlock**: Central struct aggregating stats from multiple sources via the `StatSource` trait
- **StatValue**: Individual stat with base/flat/increased/more modifiers following PoE-style formula: `(base + flat) * (1 + sum(increased)) * product(1 + more)`
- **DamagePacketGenerator**: Skill definitions that create `DamagePacket` outputs with damage, crit, penetration, and status effects
- **Defense calculations**: Armour (physical mitigation), Evasion (one-shot protection cap), Resistance (elemental reduction with caps/penetration)
- **Effect system**: Unified `Effect` type for buffs, debuffs, and ailments with configurable stacking behavior

### Configuration System

Both crates use TOML-based configuration loaded at runtime:

**loot_core config structure:**
```
config/
  base_types/     # [[base_types]] arrays - item base definitions
  affixes/        # [[affixes]] arrays - prefix/suffix modifiers
  affix_pools/    # [[pools]] arrays - groups of affixes for currencies
  currencies/     # [[currencies]] arrays - orbs that modify items
  uniques/        # [unique] + optional [recipe] per file
  names.toml      # [rare_names] prefixes/suffixes for rare item names
```

**stat_core config structure:**
```
config/
  constants.toml  # Game balance constants (armour, evasion, resistance formulas)
  skills.toml     # Skill definitions for damage calculation
  dots.toml       # DoT type configurations (poison, bleed, burn, etc.)
```

### Key Patterns

**Global Constants (stat_core):**
- `OnceLock<GameConstants>` pattern for thread-safe lazy initialization
- `init_constants(path)` to load from file, `ensure_constants_initialized()` for tests
- Access via `constants()` function

**HashMap-based Extensibility:**
- `StatusEffectData` uses `HashMap<StatusEffect, StatusEffectStats>` instead of explicit fields
- `StatusConversions` uses `HashMap<DamageType, f64>` for damage-to-status conversions
- Adding new status effects requires only config changes, not code changes

**Error Handling (loot_core):**
- `GeneratorError` enum with variants: `UnknownBaseType`, `UnknownUnique`, `UniqueBaseTypeNotFound`, `Currency`
- `ConfigError` with file path context for IO and parse errors
- Functions return `Result<T, Error>` instead of `Option<T>`

### Key Files

**loot_core:**
- `src/config.rs` - Configuration loading and all config structs
- `src/generator.rs` - Item generation logic, `Generator` struct
- `src/item.rs` - `Item`, `Modifier`, `Defenses` structs
- `src/storage.rs` - Binary encoding/decoding for items
- `src/types.rs` - Core enums: `StatType`, `DamageType`, `StatusEffect`, `Rarity`, `ItemClass`

**stat_core:**
- `src/stat_block/mod.rs` - `StatBlock` struct, `StatusEffectData`
- `src/stat_block/aggregator.rs` - `StatAccumulator`, `StatusEffectStats`, `StatusConversions`
- `src/damage/calculation.rs` - Damage calculation from StatBlock + skill
- `src/damage/packet.rs` - `DamagePacket`, `PendingStatusEffect`
- `src/combat/resolution.rs` - Apply damage to defender, status effect application
- `src/defense/` - Armour, evasion, resistance calculations
- `src/types.rs` - `Effect`, `EffectType`, `AilmentStacking`
- `src/config/constants.rs` - Global game constants with `OnceLock`
