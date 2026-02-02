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

## Development Environment

This project uses Nix flakes for reproducible development. Enter the dev shell with:

```bash
nix develop
```

This provides: Rust stable toolchain, cargo-watch, cargo-edit, rust-analyzer, and git.

## Architecture

Cargo workspace with three crates:

- **loot_core/**: Library crate for item generation - base types, affixes, currencies, and configuration loading
- **stat_core/**: Library crate for stat management - aggregates stats from multiple sources (gear, buffs, skill tree), handles damage calculation, defense mechanics, and DoT systems

### Key Design Concepts

**loot_core - Item Generation:**
- **Seed-based storage**: Items store a seed + operation history, replayed deterministically to reconstruct full stats
- **Tag-based affix weighting**: Items and affixes have tags; matching tags increase spawn weight
- **Immutable operations**: `apply_currency` returns a new item rather than mutating
- **Modular config**: TOML files in `config/` for base_types, affixes, currencies, uniques

**stat_core - Stat Management:**
- **StatBlock**: Aggregates stats from multiple sources (gear, buffs, skill tree, base stats) via the `StatSource` trait
- **DamagePacketGenerator**: Skill definitions that create `DamagePacket` outputs
- **Defense calculations**: Armour (physical mitigation), Evasion (entropy-based avoidance), Resistance (elemental reduction with caps)
- **DoT system**: Damage-over-time with configurable stacking behavior

