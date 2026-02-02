//! Armour - Physical damage reduction with diminishing returns

use super::constants::ARMOUR_CONSTANT;

/// Calculate physical damage reduction from armour
///
/// Uses POE's diminishing returns formula:
/// `Reduction = Armour / (Armour + CONSTANT * Damage)`
///
/// This makes armour more effective against many small hits
/// and less effective against large hits.
///
/// # Arguments
/// * `armour` - The defender's armour value
/// * `damage` - The incoming physical damage
///
/// # Returns
/// The damage after armour reduction
pub fn calculate_armour_reduction(armour: f64, damage: f64) -> f64 {
    if damage <= 0.0 {
        return 0.0;
    }
    if armour <= 0.0 {
        return damage;
    }

    let reduction_percent = armour / (armour + ARMOUR_CONSTANT * damage);
    let reduced = damage * (1.0 - reduction_percent);

    reduced.max(0.0)
}

/// Calculate the damage reduction percentage for a given armour and damage value
pub fn armour_reduction_percent(armour: f64, damage: f64) -> f64 {
    if damage <= 0.0 || armour <= 0.0 {
        return 0.0;
    }

    (armour / (armour + ARMOUR_CONSTANT * damage) * 100.0).clamp(0.0, 100.0)
}

/// Calculate how much armour is needed to reduce damage by a target percentage
pub fn armour_needed_for_reduction(damage: f64, target_reduction_percent: f64) -> f64 {
    if target_reduction_percent <= 0.0 {
        return 0.0;
    }
    if target_reduction_percent >= 100.0 {
        return f64::INFINITY;
    }

    // Solving: reduction = armour / (armour + C * damage)
    // reduction * (armour + C * damage) = armour
    // reduction * armour + reduction * C * damage = armour
    // reduction * C * damage = armour - reduction * armour
    // reduction * C * damage = armour * (1 - reduction)
    // armour = (reduction * C * damage) / (1 - reduction)

    let reduction = target_reduction_percent / 100.0;
    (reduction * ARMOUR_CONSTANT * damage) / (1.0 - reduction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_armour() {
        let result = calculate_armour_reduction(0.0, 100.0);
        assert!((result - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_damage() {
        let result = calculate_armour_reduction(1000.0, 0.0);
        assert!((result - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_armour_vs_small_hit() {
        // 1000 armour vs 100 damage
        // Reduction = 1000 / (1000 + 5 * 100) = 1000 / 1500 = 66.67%
        // Damage taken = 100 * (1 - 0.6667) = 33.33
        let result = calculate_armour_reduction(1000.0, 100.0);
        assert!((result - 33.33).abs() < 0.1);
    }

    #[test]
    fn test_armour_vs_large_hit() {
        // 1000 armour vs 1000 damage
        // Reduction = 1000 / (1000 + 5 * 1000) = 1000 / 6000 = 16.67%
        // Damage taken = 1000 * (1 - 0.1667) = 833.33
        let result = calculate_armour_reduction(1000.0, 1000.0);
        assert!((result - 833.33).abs() < 0.1);
    }

    #[test]
    fn test_diminishing_returns() {
        let armour = 1000.0;

        // Small hit should have higher reduction %
        let small_reduction = armour_reduction_percent(armour, 100.0);
        let large_reduction = armour_reduction_percent(armour, 1000.0);

        assert!(small_reduction > large_reduction);
    }

    #[test]
    fn test_armour_needed() {
        // How much armour to reduce 1000 damage by 50%?
        let needed = armour_needed_for_reduction(1000.0, 50.0);
        // Should be 5000 armour
        assert!((needed - 5000.0).abs() < 0.1);

        // Verify it works
        let reduction = armour_reduction_percent(needed, 1000.0);
        assert!((reduction - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_high_armour() {
        // Very high armour vs small hit
        let result = calculate_armour_reduction(10000.0, 10.0);
        // 10000 / (10000 + 50) = 99.5% reduction
        // Damage = 10 * 0.005 = 0.05
        assert!(result < 1.0);
    }
}
