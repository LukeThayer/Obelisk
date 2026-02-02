//! DoT tick processing

use super::{ActiveDoT, DotConfig, DotStacking};
use loot_core::types::DamageType;

/// Result of processing DoT ticks
#[derive(Debug, Clone, Default)]
pub struct DotTickResult {
    /// Total damage dealt this tick (by damage type)
    pub damage_by_type: Vec<(DamageType, f64)>,
    /// DoTs that expired this tick
    pub expired_dots: Vec<String>,
    /// Total damage dealt
    pub total_damage: f64,
}

impl DotTickResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_damage(&mut self, damage_type: DamageType, amount: f64) {
        if let Some(entry) = self.damage_by_type.iter_mut().find(|(t, _)| *t == damage_type) {
            entry.1 += amount;
        } else {
            self.damage_by_type.push((damage_type, amount));
        }
        self.total_damage += amount;
    }
}

/// Process a single tick for all active DoTs
///
/// Returns the total damage dealt and updates the DoT list.
pub fn process_dot_tick(
    dots: &mut Vec<ActiveDoT>,
    delta_time: f64,
    is_moving: bool,
    configs: &std::collections::HashMap<String, DotConfig>,
) -> DotTickResult {
    let mut result = DotTickResult::new();

    for dot in dots.iter_mut() {
        // Update duration
        dot.duration_remaining -= delta_time;
        dot.time_until_tick -= delta_time;

        // Check if it's time for a tick
        while dot.time_until_tick <= 0.0 && dot.is_active() {
            // Calculate damage for this tick
            let mut tick_damage = dot.damage_per_tick * dot.effectiveness;

            // Apply moving multiplier if applicable
            if is_moving {
                if let Some(config) = configs.get(&dot.dot_type) {
                    tick_damage *= config.moving_multiplier;
                }
            }

            result.add_damage(dot.damage_type, tick_damage);

            // Reset tick timer
            dot.time_until_tick += dot.tick_rate;
        }
    }

    // Collect expired DoTs
    result.expired_dots = dots
        .iter()
        .filter(|d| !d.is_active())
        .map(|d| d.dot_type.clone())
        .collect();

    // Remove expired DoTs
    dots.retain(|d| d.is_active());

    result
}

/// Apply a new DoT to a list of active DoTs, respecting stacking rules
pub fn apply_dot(
    dots: &mut Vec<ActiveDoT>,
    new_dot: ActiveDoT,
    config: &DotConfig,
) {
    match &config.stacking {
        DotStacking::StrongestOnly => {
            // Find existing DoT of same type
            if let Some(existing) = dots.iter_mut().find(|d| d.dot_type == new_dot.dot_type) {
                // Refresh if new one is stronger
                if new_dot.damage_per_tick >= existing.damage_per_tick {
                    existing.refresh(new_dot.total_duration, new_dot.damage_per_tick);
                }
            } else {
                dots.push(new_dot);
            }
        }
        DotStacking::Unlimited => {
            // Just add the new DoT
            dots.push(new_dot);
        }
        DotStacking::Limited { max_stacks, stack_effectiveness } => {
            // Count existing stacks of this type
            let existing_count = dots.iter().filter(|d| d.dot_type == new_dot.dot_type).count();

            if existing_count < *max_stacks as usize {
                // Add with appropriate effectiveness
                let mut dot_to_add = new_dot;
                if existing_count > 0 {
                    dot_to_add.effectiveness = *stack_effectiveness;
                    dot_to_add.is_strongest = false;
                }
                dots.push(dot_to_add);
            } else {
                // At max stacks - refresh the weakest or oldest
                // For simplicity, refresh the oldest (first found)
                if let Some(oldest) = dots.iter_mut().find(|d| d.dot_type == new_dot.dot_type && !d.is_strongest) {
                    oldest.refresh(new_dot.total_duration, new_dot.damage_per_tick);
                }
            }
        }
    }

    // Recalculate which is strongest for limited stacking
    recalculate_strongest(dots);
}

/// Recalculate which DoT is the "strongest" for each type
fn recalculate_strongest(dots: &mut [ActiveDoT]) {
    // Group by type
    let mut types: std::collections::HashSet<String> = std::collections::HashSet::new();
    for dot in dots.iter() {
        types.insert(dot.dot_type.clone());
    }

    for dot_type in types {
        // Find the strongest
        let max_damage = dots
            .iter()
            .filter(|d| d.dot_type == dot_type)
            .map(|d| d.damage_per_tick)
            .fold(0.0f64, f64::max);

        // Mark the strongest
        let mut found_strongest = false;
        for dot in dots.iter_mut().filter(|d| d.dot_type == dot_type) {
            if !found_strongest && (dot.damage_per_tick - max_damage).abs() < 0.01 {
                dot.is_strongest = true;
                found_strongest = true;
            } else {
                dot.is_strongest = false;
            }
        }
    }
}

/// Calculate total DPS from all active DoTs
pub fn total_dot_dps(dots: &[ActiveDoT]) -> f64 {
    dots.iter().map(|d| d.dps()).sum()
}

/// Calculate total DPS by damage type
pub fn dot_dps_by_type(dots: &[ActiveDoT]) -> Vec<(DamageType, f64)> {
    let mut result: Vec<(DamageType, f64)> = Vec::new();

    for dot in dots {
        let dps = dot.dps();
        if let Some(entry) = result.iter_mut().find(|(t, _)| *t == dot.damage_type) {
            entry.1 += dps;
        } else {
            result.push((dot.damage_type, dps));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_ignite_config() -> DotConfig {
        DotConfig {
            id: "ignite".to_string(),
            name: "Ignite".to_string(),
            damage_type: DamageType::Fire,
            stacking: DotStacking::StrongestOnly,
            base_duration: 4.0,
            tick_rate: 0.5,
            base_damage_percent: 0.25,
            max_stacks: 1,
            stack_effectiveness: 1.0,
            moving_multiplier: 1.0,
        }
    }

    fn make_bleed_config() -> DotConfig {
        DotConfig {
            id: "bleed".to_string(),
            name: "Bleed".to_string(),
            damage_type: DamageType::Physical,
            stacking: DotStacking::Limited {
                max_stacks: 8,
                stack_effectiveness: 0.5,
            },
            base_duration: 5.0,
            tick_rate: 1.0,
            base_damage_percent: 0.20,
            max_stacks: 8,
            stack_effectiveness: 0.5,
            moving_multiplier: 2.0,
        }
    }

    #[test]
    fn test_apply_dot_strongest_only() {
        let mut dots = Vec::new();
        let config = make_ignite_config();

        // Apply first ignite
        let dot1 = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            50.0,
            0.5,
            4.0,
        );
        apply_dot(&mut dots, dot1, &config);
        assert_eq!(dots.len(), 1);

        // Apply weaker ignite - should not replace
        let dot2 = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            30.0,
            0.5,
            4.0,
        );
        apply_dot(&mut dots, dot2, &config);
        assert_eq!(dots.len(), 1);
        assert!((dots[0].damage_per_tick - 50.0).abs() < 0.01);

        // Apply stronger ignite - should replace
        let dot3 = ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            70.0,
            0.5,
            4.0,
        );
        apply_dot(&mut dots, dot3, &config);
        assert_eq!(dots.len(), 1);
        assert!((dots[0].damage_per_tick - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_apply_dot_limited_stacking() {
        let mut dots = Vec::new();
        let config = make_bleed_config();

        // Apply 3 bleeds
        for i in 0..3 {
            let dot = ActiveDoT::new(
                "bleed".to_string(),
                "player".to_string(),
                DamageType::Physical,
                100.0 + i as f64 * 10.0,
                1.0,
                5.0,
            );
            apply_dot(&mut dots, dot, &config);
        }

        assert_eq!(dots.len(), 3);

        // First should be strongest (full effectiveness)
        // Others should have reduced effectiveness
        let strongest_count = dots.iter().filter(|d| d.is_strongest).count();
        assert_eq!(strongest_count, 1);
    }

    #[test]
    fn test_process_dot_tick() {
        let mut dots = vec![ActiveDoT::new(
            "ignite".to_string(),
            "player".to_string(),
            DamageType::Fire,
            50.0,
            0.5,
            4.0,
        )];

        let configs = HashMap::new();
        let result = process_dot_tick(&mut dots, 0.5, false, &configs);

        // Should have dealt 50 damage
        assert!((result.total_damage - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_bleed_moving_multiplier() {
        let mut dots = vec![ActiveDoT::new(
            "bleed".to_string(),
            "player".to_string(),
            DamageType::Physical,
            100.0,
            1.0,
            5.0,
        )];

        let mut configs = HashMap::new();
        configs.insert("bleed".to_string(), make_bleed_config());

        // Not moving
        let result1 = process_dot_tick(&mut dots.clone(), 1.0, false, &configs);
        assert!((result1.total_damage - 100.0).abs() < 0.01);

        // Moving - should deal double
        let result2 = process_dot_tick(&mut dots, 1.0, true, &configs);
        assert!((result2.total_damage - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_total_dot_dps() {
        let dots = vec![
            ActiveDoT::new(
                "ignite".to_string(),
                "player".to_string(),
                DamageType::Fire,
                50.0,
                0.5,
                4.0,
            ),
            ActiveDoT::new(
                "poison".to_string(),
                "player".to_string(),
                DamageType::Chaos,
                30.0,
                0.33,
                2.0,
            ),
        ];

        let total_dps = total_dot_dps(&dots);
        // Ignite: 50/0.5 = 100, Poison: 30/0.33 ≈ 90.9
        assert!((total_dps - (100.0 + 30.0 / 0.33)).abs() < 1.0);
    }
}
