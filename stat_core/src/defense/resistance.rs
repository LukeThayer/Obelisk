//! Resistance - Elemental damage mitigation with penetration
//!
//! Resistance system with configurable cap (default 100% = immunity achievable).
//! Penetration has reduced effectiveness vs capped resistance.
//!
//! Formula:
//! - If resistance >= cap: effective_resist = cap - (penetration * pen_vs_capped)
//! - Otherwise: effective_resist = resistance - penetration
//! - damage_taken = damage * (1 - effective_resist / 100)

use crate::config::constants;

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
/// Penetration effectiveness vs capped resistance is configurable.
pub fn calculate_effective_resistance(resistance: f64, penetration: f64) -> f64 {
    let res_constants = &constants().resistances;
    let clamped_resist = resistance.clamp(res_constants.min_value, res_constants.max_cap);

    let effective = if clamped_resist >= res_constants.max_cap {
        // Capped: penetration is less effective
        res_constants.max_cap - (penetration * res_constants.penetration_vs_capped)
    } else {
        // Not capped: full penetration
        clamped_resist - penetration
    };

    effective.clamp(res_constants.min_value, res_constants.max_cap)
}

/// Calculate the resistance needed to achieve a target damage reduction
pub fn resistance_needed_for_reduction(target_reduction_percent: f64) -> f64 {
    let res_constants = &constants().resistances;
    target_reduction_percent.clamp(res_constants.min_value, res_constants.max_cap)
}

/// Calculate damage reduction percentage from resistance
pub fn resistance_reduction_percent(resistance: f64) -> f64 {
    let res_constants = &constants().resistances;
    resistance.clamp(res_constants.min_value, res_constants.max_cap)
}

/// Check if resistance is capped
pub fn is_resistance_capped(resistance: f64) -> bool {
    resistance >= constants().resistances.max_cap
}

/// Calculate how much penetration is needed to reduce effective resistance by a target amount
pub fn penetration_needed(current_resist: f64, target_resist: f64) -> f64 {
    let res_constants = &constants().resistances;
    if current_resist >= res_constants.max_cap {
        // Capped: need more penetration due to reduced effectiveness
        (res_constants.max_cap - target_resist) / res_constants.penetration_vs_capped
    } else {
        current_resist - target_resist
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ensure_constants_initialized;

    fn setup() {
        ensure_constants_initialized();
    }

    #[test]
    fn test_positive_resistance() {
        setup();
        // 50% fire resistance, no penetration
        let result = calculate_resistance_mitigation(100.0, 50.0, 0.0);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_negative_resistance() {
        setup();
        // -50% resistance = 50% extra damage
        let result = calculate_resistance_mitigation(100.0, -50.0, 0.0);
        assert!((result - 150.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_capped_resistance() {
        setup();
        // 100% resistance = immune
        let result = calculate_resistance_mitigation(100.0, 100.0, 0.0);
        assert!((result - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_basic_penetration() {
        setup();
        // 75% resistance, 25% penetration = 50% effective
        let result = calculate_resistance_mitigation(100.0, 75.0, 25.0);
        assert!((result - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_vs_capped() {
        setup();
        // 100% resistance (capped), 30% penetration
        // Effective penetration = 30% * 0.5 = 15%
        // Effective resistance = 100% - 15% = 85%
        // Damage = 100 * (1 - 0.85) = 15
        let result = calculate_resistance_mitigation(100.0, 100.0, 30.0);
        assert!((result - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_overcapped_resistance() {
        setup();
        // 120% resistance (overcapped to 100%), 30% penetration
        // Still treated as capped
        let effective = calculate_effective_resistance(120.0, 30.0);
        assert!((effective - 85.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_cannot_go_negative() {
        setup();
        // 100% resistance, 300% penetration
        // Even with massive pen, can't go below min_value
        let effective = calculate_effective_resistance(100.0, 300.0);
        assert!(effective >= constants().resistances.min_value);
    }

    #[test]
    fn test_is_capped() {
        setup();
        assert!(is_resistance_capped(100.0));
        assert!(is_resistance_capped(120.0));
        assert!(!is_resistance_capped(75.0));
        assert!(!is_resistance_capped(0.0));
        assert!(!is_resistance_capped(-50.0));
    }

    #[test]
    fn test_design_doc_example() {
        setup();
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
        setup();
        // Need to reduce 100% resist to 50% resist
        // At cap, need double pen: (100 - 50) / 0.5 = 100% pen
        let needed = penetration_needed(100.0, 50.0);
        assert!((needed - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penetration_needed_uncapped() {
        setup();
        // Need to reduce 75% resist to 50% resist
        // Uncapped: need 25% pen
        let needed = penetration_needed(75.0, 50.0);
        assert!((needed - 25.0).abs() < f64::EPSILON);
    }
}
