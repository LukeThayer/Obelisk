//! StatValue - The triple modifier container (Flat → Increased → More)

use serde::{Deserialize, Serialize};

/// Represents a stat that follows the Flat → Increased → More model
///
/// Final value is calculated as:
/// `(base + flat) × (1 + increased) × Π(1 + more)`
///
/// - `base`: The base value (from character/skill)
/// - `flat`: Sum of all flat additions
/// - `increased`: Sum of all increased% (as decimal, e.g., 0.40 = 40%)
/// - `more`: List of more% multipliers (as decimal, each multiplies the result)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatValue {
    /// Base value (from character/skill)
    pub base: f64,
    /// Sum of all flat additions
    pub flat: f64,
    /// Sum of all increased% (as decimal, e.g., 0.40 = 40%)
    pub increased: f64,
    /// List of more% multipliers (as decimal)
    pub more: Vec<f64>,
}

impl StatValue {
    /// Create a new StatValue with the given base
    pub fn with_base(base: f64) -> Self {
        StatValue {
            base,
            flat: 0.0,
            increased: 0.0,
            more: Vec::new(),
        }
    }

    /// Calculate final value: (base + flat) × (1 + increased) × Π(1 + more)
    pub fn compute(&self) -> f64 {
        let base_total = self.base + self.flat;
        let increased_mult = 1.0 + self.increased;
        let more_mult: f64 = self.more.iter().map(|m| 1.0 + m).product();
        base_total * increased_mult * more_mult
    }

    /// Add a flat bonus
    pub fn add_flat(&mut self, value: f64) {
        self.flat += value;
    }

    /// Add an increased% bonus (as decimal, e.g., 0.40 for 40%)
    pub fn add_increased(&mut self, value: f64) {
        self.increased += value;
    }

    /// Add a more% multiplier (as decimal, e.g., 0.20 for 20% more)
    pub fn add_more(&mut self, value: f64) {
        self.more.push(value);
    }

    /// Reset to just the base value
    pub fn reset_to_base(&mut self) {
        self.flat = 0.0;
        self.increased = 0.0;
        self.more.clear();
    }

    /// Get the total flat value (base + flat additions)
    pub fn total_flat(&self) -> f64 {
        self.base + self.flat
    }

    /// Get the total increased multiplier (1 + sum of increased%)
    pub fn total_increased_multiplier(&self) -> f64 {
        1.0 + self.increased
    }

    /// Get the total more multiplier (product of all more multipliers)
    pub fn total_more_multiplier(&self) -> f64 {
        self.more.iter().map(|m| 1.0 + m).product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_basic() {
        let stat = StatValue::with_base(100.0);
        assert!((stat.compute() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_with_flat() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_flat(50.0);
        assert!((stat.compute() - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_with_increased() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_increased(0.40); // 40%
        assert!((stat.compute() - 140.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_with_more() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_more(0.20); // 20% more
        assert!((stat.compute() - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_full_formula() {
        // Example from design doc:
        // 100 base damage + 50 flat + (40% + 30% increased) + (20% more × 15% more)
        // = (100 + 50) × (1 + 0.70) × (1.20 × 1.15)
        // = 150 × 1.70 × 1.38
        // = 351.9 damage
        let mut stat = StatValue::with_base(100.0);
        stat.add_flat(50.0);
        stat.add_increased(0.40);
        stat.add_increased(0.30);
        stat.add_more(0.20);
        stat.add_more(0.15);

        let expected = 150.0 * 1.70 * (1.20 * 1.15);
        assert!((stat.compute() - expected).abs() < 0.01);
    }

    #[test]
    fn test_multiple_increased_stack_additively() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_increased(0.20);
        stat.add_increased(0.30);
        // Should be 100 * (1 + 0.50) = 150, not 100 * 1.2 * 1.3 = 156
        assert!((stat.compute() - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_more_stack_multiplicatively() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_more(0.20);
        stat.add_more(0.30);
        // Should be 100 * 1.2 * 1.3 = 156, not 100 * 1.5 = 150
        assert!((stat.compute() - 156.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reset_to_base() {
        let mut stat = StatValue::with_base(100.0);
        stat.add_flat(50.0);
        stat.add_increased(0.40);
        stat.add_more(0.20);
        stat.reset_to_base();
        assert!((stat.compute() - 100.0).abs() < f64::EPSILON);
    }
}
