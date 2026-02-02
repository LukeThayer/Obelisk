//! Evasion - One-shot protection (damage cap per hit)
//!
//! Evasion interacts with the attacker's accuracy to determine a damage cap.
//! Higher accuracy = higher cap, higher evasion = lower cap.
//!
//! Formula: damage_cap = accuracy / (1 + evasion / SCALE_FACTOR)
//!
//! Examples (with SCALE_FACTOR = 1000):
//! - 2000 accuracy vs 0 evasion: cap = 2000 (no reduction)
//! - 2000 accuracy vs 1000 evasion: cap = 1000 (50% reduction)
//! - 2000 accuracy vs 3000 evasion: cap = 500 (75% reduction)
//! - 5000 accuracy vs 1000 evasion: cap = 2500
//!
//! This creates meaningful interactions:
//! - Accuracy is an offensive stat that counters evasion
//! - Evasion provides diminishing-returns protection against big hits
//! - High evasion protects against one-shots from low-accuracy attackers

use super::constants::EVASION_SCALE_FACTOR;

/// Calculate the damage cap based on accuracy vs evasion
///
/// Higher accuracy = higher cap, higher evasion = lower cap
pub fn calculate_damage_cap(accuracy: f64, evasion: f64) -> f64 {
    if accuracy <= 0.0 {
        return 0.0; // No accuracy = no damage can land
    }
    if evasion <= 0.0 {
        return accuracy; // No evasion = cap equals accuracy
    }

    accuracy / (1.0 + evasion / EVASION_SCALE_FACTOR)
}

/// Apply evasion cap to incoming damage
///
/// Returns a tuple of (damage_taken, damage_evaded)
pub fn apply_evasion_cap(accuracy: f64, evasion: f64, damage: f64) -> (f64, f64) {
    if damage <= 0.0 {
        return (0.0, 0.0);
    }

    let cap = calculate_damage_cap(accuracy, evasion);

    if damage <= cap {
        // Hit is below threshold - full damage taken
        (damage, 0.0)
    } else {
        // Hit is above threshold - cap the damage
        let evaded = damage - cap;
        (cap, evaded)
    }
}

/// Check if a hit triggered the evasion cap
pub fn triggered_evasion_cap(accuracy: f64, evasion: f64, damage: f64) -> bool {
    let cap = calculate_damage_cap(accuracy, evasion);
    damage > cap
}

/// Calculate the evasion rating needed to achieve a target damage cap given accuracy
pub fn evasion_needed_for_cap(accuracy: f64, target_cap: f64) -> f64 {
    if target_cap >= accuracy || target_cap <= 0.0 {
        return 0.0;
    }

    // Solve: target = accuracy / (1 + evasion / SCALE)
    // target * (1 + evasion / SCALE) = accuracy
    // 1 + evasion / SCALE = accuracy / target
    // evasion / SCALE = accuracy / target - 1
    // evasion = SCALE * (accuracy / target - 1)
    EVASION_SCALE_FACTOR * (accuracy / target_cap - 1.0)
}

/// Calculate what percentage of incoming damage was evaded
pub fn evasion_effectiveness(accuracy: f64, evasion: f64, damage: f64) -> f64 {
    if damage <= 0.0 {
        return 0.0;
    }

    let (_, evaded) = apply_evasion_cap(accuracy, evasion, damage);
    (evaded / damage * 100.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_cap_no_evasion() {
        // No evasion = cap equals accuracy
        let cap = calculate_damage_cap(2000.0, 0.0);
        assert!((cap - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_cap_with_evasion() {
        // 2000 accuracy vs 1000 evasion: 2000 / (1 + 1000/1000) = 2000 / 2 = 1000
        let cap = calculate_damage_cap(2000.0, 1000.0);
        assert!((cap - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_cap_high_evasion() {
        // 2000 accuracy vs 3000 evasion: 2000 / (1 + 3000/1000) = 2000 / 4 = 500
        let cap = calculate_damage_cap(2000.0, 3000.0);
        assert!((cap - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_cap_high_accuracy() {
        // 5000 accuracy vs 1000 evasion: 5000 / (1 + 1000/1000) = 5000 / 2 = 2500
        let cap = calculate_damage_cap(5000.0, 1000.0);
        assert!((cap - 2500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_below_cap() {
        // 2000 accuracy vs 1000 evasion = 1000 cap, hit for 800
        let (taken, evaded) = apply_evasion_cap(2000.0, 1000.0, 800.0);
        assert!((taken - 800.0).abs() < f64::EPSILON);
        assert!((evaded - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_above_cap() {
        // 2000 accuracy vs 1000 evasion = 1000 cap, hit for 1500
        let (taken, evaded) = apply_evasion_cap(2000.0, 1000.0, 1500.0);
        assert!((taken - 1000.0).abs() < f64::EPSILON);
        assert!((evaded - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_at_cap() {
        // 2000 accuracy vs 1000 evasion = 1000 cap, hit for exactly 1000
        let (taken, evaded) = apply_evasion_cap(2000.0, 1000.0, 1000.0);
        assert!((taken - 1000.0).abs() < f64::EPSILON);
        assert!((evaded - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_evasion() {
        // 2000 accuracy vs 0 evasion = 2000 cap
        let (taken, evaded) = apply_evasion_cap(2000.0, 0.0, 1500.0);
        assert!((taken - 1500.0).abs() < f64::EPSILON);
        assert!((evaded - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_no_accuracy() {
        // 0 accuracy = 0 cap = no damage
        let (taken, evaded) = apply_evasion_cap(0.0, 1000.0, 1500.0);
        assert!((taken - 0.0).abs() < f64::EPSILON);
        assert!((evaded - 1500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_triggered_cap() {
        // 2000 accuracy vs 1000 evasion = 1000 cap
        assert!(triggered_evasion_cap(2000.0, 1000.0, 1200.0));  // above cap
        assert!(!triggered_evasion_cap(2000.0, 1000.0, 800.0));  // below cap
        assert!(!triggered_evasion_cap(2000.0, 1000.0, 1000.0)); // at cap
    }

    #[test]
    fn test_evasion_effectiveness() {
        // 2000 accuracy vs 1000 evasion = 1000 cap, 2000 damage = 1000 evaded = 50%
        let effectiveness = evasion_effectiveness(2000.0, 1000.0, 2000.0);
        assert!((effectiveness - 50.0).abs() < f64::EPSILON);

        // Below cap = 0% evaded
        let effectiveness = evasion_effectiveness(2000.0, 1000.0, 500.0);
        assert!((effectiveness - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_higher_accuracy_counters_evasion() {
        // More accuracy = higher cap (less protection for defender)
        let low_acc_cap = calculate_damage_cap(1000.0, 1000.0);  // 500
        let high_acc_cap = calculate_damage_cap(4000.0, 1000.0); // 2000

        assert!(high_acc_cap > low_acc_cap);
    }

    #[test]
    fn test_higher_evasion_is_better() {
        // More evasion = lower cap (more protection for defender)
        let low_eva_cap = calculate_damage_cap(2000.0, 500.0);   // 1333
        let high_eva_cap = calculate_damage_cap(2000.0, 2000.0); // 667

        assert!(high_eva_cap < low_eva_cap);
    }

    #[test]
    fn test_evasion_needed() {
        // With 2000 accuracy, want a 1000 cap -> need 1000 evasion
        let needed = evasion_needed_for_cap(2000.0, 1000.0);
        assert!((needed - 1000.0).abs() < f64::EPSILON);

        // With 2000 accuracy, want a 500 cap -> need 3000 evasion
        let needed = evasion_needed_for_cap(2000.0, 500.0);
        assert!((needed - 3000.0).abs() < f64::EPSILON);
    }
}
