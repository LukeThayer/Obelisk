//! Resistance - Elemental damage mitigation with penetration
//!
//! Resistance system with 100% cap (immunity is achievable).
//! Penetration has reduced effectiveness vs capped resistance.
//!
//! Formula:
//! - If resistance >= cap: effective_resist = cap - (penetration * 0.5)
//! - Otherwise: effective_resist = resistance - penetration
//! - damage_taken = damage * (1 - effective_resist / 100)

use super::constants::{MAX_RESISTANCE, MIN_RESISTANCE, PENETRATION_VS_CAPPED};

/// Calculate damage after resistance mitigation
///
/// # Arguments
/// * `damage` - The incoming elemental damage
/// * `resistance` - The defender's resistance (can be negative)
/// * `penetration` - The attacker's penetration for this element
///
/// # Returns
/// The damage after resistance mitigation
pub fn calculate_resistance_mitigation(damage: f64, resistance: f64, penetration: f64) -> f64 {
    if damage <= 0.0 {
        return 0.0;
    }

    let effective_resist = calculate_effective_resistance(resistance, penetration);
    let mitigation = effective_resist / 100.0;

    // Damage multiplier: 1.0 = full damage, 0.0 = no damage, >1.0 = extra damage
    let damage_mult = 1.0 - mitigation;

    (damage * damage_mult).max(0.0)
}

/// Calculate effective resistance after penetration
///
/// Penetration has 50% effectiveness vs capped resistance.
pub fn calculate_effective_resistance(resistance: f64, penetration: f64) -> f64 {
    let clamped_resist = resistance.clamp(MIN_RESISTANCE, MAX_RESISTANCE);

    let effective = if clamped_resist >= MAX_RESISTANCE {
        // Capped: penetration is half as effective
        MAX_RESISTANCE - (penetration * PENETRATION_VS_CAPPED)
    } else {
        // Not capped: full penetration
        clamped_resist - penetration
    };

    effective.clamp(MIN_RESISTANCE, MAX_RESISTANCE)
}

/// Calculate the resistance needed to achieve a target damage reduction
pub fn resistance_needed_for_reduction(target_reduction_percent: f64) -> f64 {
    target_reduction_percent.clamp(MIN_RESISTANCE, MAX_RESISTANCE)
}

/// Calculate damage reduction percentage from resistance
pub fn resistance_reduction_percent(resistance: f64) -> f64 {
    resistance.clamp(MIN_RESISTANCE, MAX_RESISTANCE)
}

/// Check if resistance is capped
pub fn is_resistance_capped(resistance: f64) -> bool {
    resistance >= MAX_RESISTANCE
}

/// Calculate how much penetration is needed to reduce effective resistance by a target amount
pub fn penetration_needed(current_resist: f64, target_resist: f64) -> f64 {
    if current_resist >= MAX_RESISTANCE {
        // Capped: need double the penetration
        (MAX_RESISTANCE - target_resist) / PENETRATION_VS_CAPPED
    } else {
        current_resist - target_resist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positive_resistance() {
        // 50% fire resistance, no penetration
        let result = calculate_resistance_mitigation(100.0, 50.0, 0.0);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_negative_resistance() {
        // -50% resistance = 50% extra damage
        let result = calculate_resistance_mitigation(100.0, -50.0, 0.0);
        assert!((result - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_capped_resistance() {
        // 100% resistance = immune
        let result = calculate_resistance_mitigation(100.0, 100.0, 0.0);
        assert!((result - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_basic_penetration() {
        // 75% resistance, 25% penetration = 50% effective
        let result = calculate_resistance_mitigation(100.0, 75.0, 25.0);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_vs_capped() {
        // 100% resistance (capped), 30% penetration
        // Effective penetration = 30% * 0.5 = 15%
        // Effective resistance = 100% - 15% = 85%
        // Damage = 100 * (1 - 0.85) = 15
        let result = calculate_resistance_mitigation(100.0, 100.0, 30.0);
        assert!((result - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_overcapped_resistance() {
        // 120% resistance (overcapped to 100%), 30% penetration
        // Still treated as capped
        let effective = calculate_effective_resistance(120.0, 30.0);
        assert!((effective - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_cannot_go_negative() {
        // 100% resistance, 300% penetration
        // Even with massive pen, can't go below MIN_RESISTANCE
        let effective = calculate_effective_resistance(100.0, 300.0);
        assert!(effective >= MIN_RESISTANCE);
    }

    #[test]
    fn test_is_capped() {
        assert!(is_resistance_capped(100.0));
        assert!(is_resistance_capped(120.0));
        assert!(!is_resistance_capped(75.0));
        assert!(!is_resistance_capped(0.0));
        assert!(!is_resistance_capped(-50.0));
    }

    #[test]
    fn test_design_doc_example() {
        // From design doc:
        // If enemy has 100% fire res and you have 30% fire pen:
        // Effective penetration = 30% × 0.5 = 15%
        // Enemy takes damage as if they had 85% fire res
        let effective = calculate_effective_resistance(100.0, 30.0);
        assert!((effective - 85.0).abs() < f64::EPSILON);

        // 100 damage * (1 - 0.85) = 15 damage
        let damage = calculate_resistance_mitigation(100.0, 100.0, 30.0);
        assert!((damage - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_penetration_needed_capped() {
        // Need to reduce 100% resist to 50% resist
        // At cap, need double pen: (100 - 50) / 0.5 = 100% pen
        let needed = penetration_needed(100.0, 50.0);
        assert!((needed - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_needed_uncapped() {
        // Need to reduce 75% resist to 50% resist
        // Uncapped: need 25% pen
        let needed = penetration_needed(75.0, 50.0);
        assert!((needed - 25.0).abs() < f64::EPSILON);
    }
}
