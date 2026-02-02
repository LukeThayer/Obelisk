//! BaseStatsSource - Stats from character level

use crate::source::StatSource;
use crate::stat_block::StatAccumulator;

/// Stats from base character level
pub struct BaseStatsSource {
    /// Character level (1-100)
    pub level: u32,
}

impl BaseStatsSource {
    /// Create a new base stats source
    pub fn new(level: u32) -> Self {
        BaseStatsSource { level }
    }
}

impl StatSource for BaseStatsSource {
    fn id(&self) -> &str {
        "base_stats"
    }

    fn priority(&self) -> i32 {
        -100 // Base stats apply first
    }

    fn apply(&self, stats: &mut StatAccumulator) {
        // Life and mana scale with level
        stats.life_flat += (self.level as f64 - 1.0) * 12.0; // +12 life per level
        stats.mana_flat += (self.level as f64 - 1.0) * 6.0; // +6 mana per level

        // Base attributes (10 each)
        stats.strength_flat += 10.0;
        stats.dexterity_flat += 10.0;
        stats.intelligence_flat += 10.0;
        stats.constitution_flat += 10.0;
        stats.wisdom_flat += 10.0;
        stats.charisma_flat += 10.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_stats_level_scaling() {
        let source = BaseStatsSource::new(10);
        let mut acc = StatAccumulator::new();
        source.apply(&mut acc);

        // Level 10 = 9 levels of scaling
        // Life: 9 * 12 = 108
        assert!((acc.life_flat - 108.0).abs() < 0.01);

        // Mana: 9 * 6 = 54
        assert!((acc.mana_flat - 54.0).abs() < 0.01);
    }

    #[test]
    fn test_base_stats_attributes() {
        let source = BaseStatsSource::new(1);
        let mut acc = StatAccumulator::new();
        source.apply(&mut acc);

        // All attributes start at 10
        assert!((acc.strength_flat - 10.0).abs() < 0.01);
        assert!((acc.intelligence_flat - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_base_stats_priority() {
        let source = BaseStatsSource::new(1);
        assert_eq!(source.priority(), -100);
    }
}
