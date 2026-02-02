//! Prelude module for convenient imports
//!
//! ```rust
//! use stat_core::prelude::*;
//! ```

// Core types
pub use crate::stat_block::{StatBlock, StatValue};
pub use crate::types::{ActiveBuff, ActiveStatusEffect, AilmentStacking, Effect, EffectType, EquipmentSlot, SkillTag, StatMod, TickResult};

// Damage system
pub use crate::damage::{DamagePacket, DamagePacketGenerator, BaseDamage};

// Combat
pub use crate::combat::{CombatResult, DamageTaken};

// DoT system
pub use crate::dot::{DotRegistry, ActiveDoT, DotConfig};

// Sources
pub use crate::source::{BuffSource, GearSource, StatSource};

// Config
pub use crate::config::default_skills;

// Re-exports from loot_core
pub use loot_core::types::{DamageType, StatusEffect, StatType, ItemClass, Rarity};
pub use loot_core::Item;
