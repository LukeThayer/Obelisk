//! Combat resolution - Apply damage packets to stat blocks

mod resolution;
mod result;

pub use resolution::{resolve_damage, resolve_damage_with_rng};
pub use result::{CombatResult, DamageTaken};
