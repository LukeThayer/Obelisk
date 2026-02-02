//! stat_core - Core stat management library for game entities
//!
//! This library provides:
//! - StatBlock: Aggregated stats from multiple sources
//! - DamagePacketGenerator: Skill/ability damage configuration
//! - DamagePacket: Calculated damage output
//! - Combat resolution: Processing incoming damage against defenses
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use stat_core::prelude::*;
//! use loot_core::{Config, Generator};
//!
//! // Create player and equip items
//! let mut player = StatBlock::with_id("player");
//! let config = Config::load_from_dir(Path::new("config/")).unwrap();
//! let generator = Generator::new(config);
//! let item = generator.generate("iron_sword", 12345).unwrap();
//! player.equip(EquipmentSlot::MainHand, item);
//!
//! // Create a skill and attack
//! let skill = DamagePacketGenerator::new("slash")
//!     .with_base_damage(BaseDamage::weapon());
//! let packet = player.attack(&skill);
//!
//! // Resolve damage against enemy
//! let mut enemy = StatBlock::with_id("goblin");
//! let (new_enemy, result) = enemy.receive_damage(&packet);
//! println!("Dealt {} damage!", result.total_damage);
//! ```

pub mod combat;
pub mod config;
pub mod damage;
pub mod defense;
pub mod dot;
pub mod prelude;
pub mod source;
pub mod stat_block;
pub mod types;

// Core API - what most users need
pub use stat_block::StatBlock;
pub use damage::{BaseDamage, DamagePacket, DamagePacketGenerator};
pub use combat::CombatResult;
pub use types::{Effect, EquipmentSlot};

// Configuration
pub use config::{default_skills, init_constants, init_constants_default};
pub use dot::DotRegistry;

// Advanced: Custom stat sources
pub use source::StatSource;

// Re-export commonly needed loot_core types
pub use loot_core::{DamageType, Item, StatType, StatusEffect};
