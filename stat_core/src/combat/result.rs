//! CombatResult - Outcome of damage resolution

use crate::types::Effect;
use loot_core::types::DamageType;
use serde::{Deserialize, Serialize};

/// Result of applying a DamagePacket to a StatBlock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatResult {
    // === Damage Breakdown ===
    /// Damage taken per type
    pub damage_taken: Vec<DamageTaken>,
    /// Total damage after all mitigation
    pub total_damage: f64,

    // === Mitigation Info ===
    /// Damage absorbed by energy shield
    pub damage_blocked_by_es: f64,
    /// Damage reduced by armour
    pub damage_reduced_by_armour: f64,
    /// Damage reduced by resistances
    pub damage_reduced_by_resists: f64,
    /// Damage prevented by evasion cap
    pub damage_prevented_by_evasion: f64,

    // === Effects Applied ===
    /// Effects that were applied (unified Effect system)
    pub effects_applied: Vec<Effect>,

    // === State Changes ===
    /// ES before damage
    pub es_before: f64,
    /// ES after damage
    pub es_after: f64,
    /// Life before damage
    pub life_before: f64,
    /// Life after damage
    pub life_after: f64,

    // === Flags ===
    /// Whether this was a killing blow
    pub is_killing_blow: bool,
    /// Whether the evasion cap was triggered
    pub triggered_evasion_cap: bool,
}

impl Default for CombatResult {
    fn default() -> Self {
        CombatResult {
            damage_taken: Vec::new(),
            total_damage: 0.0,
            damage_blocked_by_es: 0.0,
            damage_reduced_by_armour: 0.0,
            damage_reduced_by_resists: 0.0,
            damage_prevented_by_evasion: 0.0,
            effects_applied: Vec::new(),
            es_before: 0.0,
            es_after: 0.0,
            life_before: 0.0,
            life_after: 0.0,
            is_killing_blow: false,
            triggered_evasion_cap: false,
        }
    }
}

impl CombatResult {
    /// Create a new empty combat result
    pub fn new() -> Self {
        Self::default()
    }

    /// Get total raw damage (before mitigation)
    pub fn total_raw_damage(&self) -> f64 {
        self.damage_taken.iter().map(|d| d.raw_amount).sum()
    }

    /// Get total mitigated damage
    pub fn total_mitigated(&self) -> f64 {
        self.damage_taken.iter().map(|d| d.mitigated_amount).sum()
    }

    /// Get damage taken for a specific type
    pub fn damage_of_type(&self, damage_type: DamageType) -> Option<&DamageTaken> {
        self.damage_taken.iter().find(|d| d.damage_type == damage_type)
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.total_damage > 0.0 {
            parts.push(format!("{:.0} damage taken", self.total_damage));
        }

        if self.damage_blocked_by_es > 0.0 {
            parts.push(format!("{:.0} blocked by ES", self.damage_blocked_by_es));
        }

        if self.damage_reduced_by_armour > 0.0 {
            parts.push(format!("{:.0} reduced by armour", self.damage_reduced_by_armour));
        }

        if self.damage_reduced_by_resists > 0.0 {
            parts.push(format!("{:.0} reduced by resists", self.damage_reduced_by_resists));
        }

        if self.damage_prevented_by_evasion > 0.0 {
            parts.push(format!("{:.0} evaded", self.damage_prevented_by_evasion));
        }

        if self.is_killing_blow {
            parts.push("FATAL".to_string());
        }

        if parts.is_empty() {
            "No damage".to_string()
        } else {
            parts.join(", ")
        }
    }

    /// Get life change
    pub fn life_change(&self) -> f64 {
        self.life_after - self.life_before
    }

    /// Get ES change
    pub fn es_change(&self) -> f64 {
        self.es_after - self.es_before
    }
}

/// Damage breakdown for a single damage type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageTaken {
    /// The damage type
    pub damage_type: DamageType,
    /// Raw damage before mitigation
    pub raw_amount: f64,
    /// Amount reduced by defenses
    pub mitigated_amount: f64,
    /// Final damage after mitigation
    pub final_amount: f64,
}

impl DamageTaken {
    /// Create a new damage taken entry
    pub fn new(damage_type: DamageType, raw: f64, mitigated: f64, final_dmg: f64) -> Self {
        DamageTaken {
            damage_type,
            raw_amount: raw,
            mitigated_amount: mitigated,
            final_amount: final_dmg,
        }
    }

    /// Get mitigation percentage
    pub fn mitigation_percent(&self) -> f64 {
        if self.raw_amount <= 0.0 {
            return 0.0;
        }
        (self.mitigated_amount / self.raw_amount * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_result_totals() {
        let mut result = CombatResult::new();
        result.damage_taken.push(DamageTaken::new(
            DamageType::Physical,
            100.0,
            30.0,
            70.0,
        ));
        result.damage_taken.push(DamageTaken::new(
            DamageType::Fire,
            50.0,
            25.0,
            25.0,
        ));
        result.total_damage = 95.0;

        assert!((result.total_raw_damage() - 150.0).abs() < f64::EPSILON);
        assert!((result.total_mitigated() - 55.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_taken_mitigation_percent() {
        let damage = DamageTaken::new(DamageType::Physical, 100.0, 40.0, 60.0);
        assert!((damage.mitigation_percent() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_combat_result_summary() {
        let mut result = CombatResult::new();
        result.total_damage = 100.0;
        result.damage_reduced_by_armour = 25.0;

        let summary = result.summary();
        assert!(summary.contains("100 damage"));
        assert!(summary.contains("armour"));
    }

    #[test]
    fn test_killing_blow_summary() {
        let mut result = CombatResult::new();
        result.total_damage = 100.0;
        result.is_killing_blow = true;

        let summary = result.summary();
        assert!(summary.contains("FATAL"));
    }
}
