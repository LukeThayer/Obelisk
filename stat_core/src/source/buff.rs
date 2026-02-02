//! BuffSource - Temporary buffs and debuffs

use crate::source::StatSource;
use crate::stat_block::StatAccumulator;
use loot_core::types::StatType;

/// Temporary buff/debuff source
#[derive(Debug, Clone)]
pub struct BuffSource {
    /// Buff identifier
    pub buff_id: String,
    /// Display name
    pub name: String,
    /// Duration remaining in seconds
    pub duration_remaining: f64,
    /// Current stack count
    pub stacks: u32,
    /// Whether this is a debuff
    pub is_debuff: bool,
    /// Stat modifiers per stack
    modifiers: Vec<BuffModifier>,
}

/// A stat modifier from a buff
#[derive(Debug, Clone)]
pub struct BuffModifier {
    pub stat: StatType,
    /// Value per stack
    pub value_per_stack: f64,
    /// Whether this is a "more" multiplier
    pub is_more: bool,
}

impl BuffSource {
    /// Create a new buff source
    pub fn new(buff_id: String, name: String, duration: f64, is_debuff: bool) -> Self {
        BuffSource {
            buff_id,
            name,
            duration_remaining: duration,
            stacks: 1,
            is_debuff,
            modifiers: Vec::new(),
        }
    }

    /// Add a modifier to this buff
    pub fn with_modifier(mut self, stat: StatType, value_per_stack: f64, is_more: bool) -> Self {
        self.modifiers.push(BuffModifier {
            stat,
            value_per_stack,
            is_more,
        });
        self
    }

    /// Set the number of stacks
    pub fn with_stacks(mut self, stacks: u32) -> Self {
        self.stacks = stacks;
        self
    }

    /// Add a stack
    pub fn add_stack(&mut self) {
        self.stacks += 1;
    }

    /// Remove a stack
    pub fn remove_stack(&mut self) {
        self.stacks = self.stacks.saturating_sub(1);
    }

    /// Refresh duration
    pub fn refresh(&mut self, duration: f64) {
        self.duration_remaining = duration;
    }

    /// Tick the buff duration
    /// Returns true if the buff is still active
    pub fn tick(&mut self, delta: f64) -> bool {
        self.duration_remaining -= delta;
        self.duration_remaining > 0.0 && self.stacks > 0
    }

    /// Check if the buff is active
    pub fn is_active(&self) -> bool {
        self.duration_remaining > 0.0 && self.stacks > 0
    }
}

impl StatSource for BuffSource {
    fn id(&self) -> &str {
        &self.buff_id
    }

    fn priority(&self) -> i32 {
        200 // Buffs apply after skill tree
    }

    fn apply(&self, stats: &mut StatAccumulator) {
        if !self.is_active() {
            return;
        }

        let stack_mult = self.stacks as f64;

        for modifier in &self.modifiers {
            let total_value = modifier.value_per_stack * stack_mult;

            if modifier.is_more {
                // "More" multipliers
                match modifier.stat {
                    StatType::IncreasedPhysicalDamage => {
                        stats.physical_damage_more.push(total_value / 100.0);
                    }
                    StatType::IncreasedAttackSpeed => {
                        // Attack speed more would need special handling
                        stats.attack_speed_increased += total_value / 100.0;
                    }
                    _ => {
                        stats.apply_stat_type(modifier.stat, total_value);
                    }
                }
            } else {
                stats.apply_stat_type(modifier.stat, total_value);
            }
        }
    }
}

/// Common buff presets
pub struct BuffPresets;

impl BuffPresets {
    /// Create a generic damage buff
    pub fn damage_buff(name: &str, percent: f64, duration: f64) -> BuffSource {
        BuffSource::new(
            format!("buff_{}", name.to_lowercase().replace(' ', "_")),
            name.to_string(),
            duration,
            false,
        )
        .with_modifier(StatType::IncreasedPhysicalDamage, percent, false)
    }

    /// Create an attack speed buff
    pub fn haste(duration: f64) -> BuffSource {
        BuffSource::new("buff_haste".to_string(), "Haste".to_string(), duration, false)
            .with_modifier(StatType::IncreasedAttackSpeed, 20.0, false)
    }

    /// Create a generic debuff (reduces damage)
    pub fn weakness(duration: f64) -> BuffSource {
        BuffSource::new(
            "debuff_weakness".to_string(),
            "Weakness".to_string(),
            duration,
            true,
        )
        .with_modifier(StatType::IncreasedPhysicalDamage, -20.0, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buff_tick() {
        let mut buff = BuffSource::new("test".to_string(), "Test".to_string(), 5.0, false);
        assert!(buff.is_active());

        assert!(buff.tick(2.0));
        assert!((buff.duration_remaining - 3.0).abs() < 0.01);

        assert!(!buff.tick(4.0));
        assert!(!buff.is_active());
    }

    #[test]
    fn test_buff_stacks() {
        let mut buff = BuffSource::new("test".to_string(), "Test".to_string(), 5.0, false)
            .with_modifier(StatType::IncreasedPhysicalDamage, 10.0, false)
            .with_stacks(3);

        let mut acc = StatAccumulator::new();
        buff.apply(&mut acc);

        // 10% per stack * 3 stacks = 30%
        assert!((acc.physical_damage_increased - 0.30).abs() < 0.01);
    }

    #[test]
    fn test_buff_refresh() {
        let mut buff = BuffSource::new("test".to_string(), "Test".to_string(), 5.0, false);
        buff.tick(4.0);
        assert!((buff.duration_remaining - 1.0).abs() < 0.01);

        buff.refresh(5.0);
        assert!((buff.duration_remaining - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_buff_priority() {
        let buff = BuffSource::new("test".to_string(), "Test".to_string(), 5.0, false);
        assert_eq!(buff.priority(), 200);
    }
}
