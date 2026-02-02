//! stat_core - Core stat management library for game entities
//!
//! This library provides:
//! - StatBlock: Aggregated stats from multiple sources
//! - DamagePacketGenerator: Skill/ability damage configuration
//! - DamagePacket: Calculated damage output
//! - Damage Resolution: Processing incoming damage against defenses
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use stat_core::prelude::*;
//! use loot_core::{Config, Generator};
//!
//! // Create player and equip items
//! let mut player = StatBlock::with_id("player");
//! let generator = Generator::new(Config::load("config/").unwrap());
//! player.equip(EquipmentSlot::MainHand, generator.generate("iron_sword", 12345).unwrap());
//!
//! // Combat
//! let skills = default_skills();
//! let dot_registry = DotRegistry::new();
//! let packet = player.attack(&skills["heavy_strike"], &dot_registry);
//!
//! let mut enemy = StatBlock::with_id("goblin");
//! let result = enemy.receive_damage(&packet, &dot_registry);
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

// Re-export core types for convenience
pub use combat::{CombatResult, DamageTaken};
pub use defense::calculate_damage_cap;
pub use damage::{
    BaseDamage, DamagePacket, DamagePacketGenerator, DotApplication, FinalDamage, PendingDoT,
    PendingStatusEffect,
};
pub use dot::{ActiveDoT, DotConfig, DotStacking};
pub use source::{BaseStatsSource, BuffSource, GearSource, SkillTreeSource, StatSource};
pub use stat_block::{StatAccumulator, StatBlock, StatValue, StatusConversions, StatusEffectStats, StatusEffectData};
pub use types::{ActiveBuff, ActiveStatusEffect, AilmentStacking, Effect, EffectType, EquipmentSlot, SkillNodeId, SkillTag, StatMod, TickResult};
pub use config::default_skills;

// Re-export loot_core types for convenience
pub use loot_core::types::{Attribute, DamageType, DefenseType, ItemClass, Rarity, StatType, StatusEffect};
pub use loot_core::item::Modifier;
pub use loot_core::Item;
