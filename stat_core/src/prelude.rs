//! Prelude module for convenient imports
//!
//! ```rust
//! use stat_core::prelude::*;
//! ```

// Core types
pub use crate::stat_block::StatBlock;
pub use crate::types::{Effect, EquipmentSlot};

// Damage system
pub use crate::damage::{BaseDamage, DamagePacket, DamagePacketGenerator};

// Combat
pub use crate::combat::CombatResult;

// DoT system
pub use crate::dot::DotRegistry;

// Config
pub use crate::config::{default_skills, init_constants, init_constants_default};

// Sources (for advanced use)
pub use crate::source::StatSource;

// Re-exports from loot_core
pub use loot_core::{DamageType, Item, StatType, StatusEffect};
