//! StatSource - Trait and implementations for stat providers

mod base_stats;
mod buff;
mod gear;
mod skill_tree;

pub use base_stats::BaseStatsSource;
pub use buff::BuffSource;
pub use gear::GearSource;
pub use skill_tree::SkillTreeSource;

use crate::stat_block::StatAccumulator;

/// Trait for anything that contributes stats to a StatBlock
pub trait StatSource: Send + Sync {
    /// Unique identifier for this source
    fn id(&self) -> &str;

    /// Priority for application order (higher = applied later)
    /// Default priority is 0.
    /// Suggested priorities:
    /// - Base stats: -100
    /// - Gear: 0
    /// - Skill tree: 100
    /// - Buffs: 200
    fn priority(&self) -> i32 {
        0
    }

    /// Apply this source's stats to the accumulator
    fn apply(&self, stats: &mut StatAccumulator);
}
