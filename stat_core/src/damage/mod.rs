//! Damage system - DamagePacketGenerator and DamagePacket

mod calculation;
mod generator;
mod packet;

pub use calculation::{calculate_damage, calculate_skill_dps};
pub use generator::{BaseDamage, DamagePacketGenerator, DotApplication, SkillStatusConversions};
pub use packet::{DamagePacket, FinalDamage, PendingDoT, PendingStatusEffect};
