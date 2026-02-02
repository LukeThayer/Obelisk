//! DamagePacket - The output of damage calculation

use loot_core::types::{DamageType, StatusEffect};
use serde::{Deserialize, Serialize};

/// The result of a StatBlock generating damage with a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamagePacket {
    // === Source Info ===
    /// Who dealt this damage
    pub source_id: String,
    /// What skill was used
    pub skill_id: String,

    // === Damage Values (after all scaling) ===
    /// Damage per type
    pub damages: Vec<FinalDamage>,

    // === Crit Info ===
    /// Whether this hit was a critical strike
    pub is_critical: bool,
    /// Critical multiplier (already applied to damages if crit)
    pub crit_multiplier: f64,

    // === Penetration ===
    pub fire_pen: f64,
    pub cold_pen: f64,
    pub lightning_pen: f64,
    pub chaos_pen: f64,

    // === DoT Effects to Apply ===
    /// DoTs that should be applied from this hit
    pub dots_to_apply: Vec<PendingDoT>,

    // === Status Effects to Apply ===
    /// Status effects that should be applied from this hit
    pub status_effects_to_apply: Vec<PendingStatusEffect>,

    // === Accuracy ===
    /// Attacker's accuracy rating (used vs defender's evasion)
    pub accuracy: f64,

    // === Metadata ===
    /// For multi-hit tracking
    pub hit_count: u32,
    /// Whether this hit can trigger leech
    pub can_leech: bool,
    /// Whether this hit can trigger on-hit effects
    pub can_apply_on_hit: bool,
}

impl Default for DamagePacket {
    fn default() -> Self {
        DamagePacket {
            source_id: String::new(),
            skill_id: String::new(),
            damages: Vec::new(),
            is_critical: false,
            crit_multiplier: 1.5,
            fire_pen: 0.0,
            cold_pen: 0.0,
            lightning_pen: 0.0,
            chaos_pen: 0.0,
            dots_to_apply: Vec::new(),
            status_effects_to_apply: Vec::new(),
            accuracy: 1000.0, // Default accuracy
            hit_count: 1,
            can_leech: true,
            can_apply_on_hit: true,
        }
    }
}

impl DamagePacket {
    /// Create a new empty damage packet
    pub fn new(source_id: String, skill_id: String) -> Self {
        DamagePacket {
            source_id,
            skill_id,
            ..Default::default()
        }
    }

    /// Get total damage (sum of all types)
    pub fn total_damage(&self) -> f64 {
        self.damages.iter().map(|d| d.amount).sum()
    }

    /// Get damage for a specific type
    pub fn damage_of_type(&self, damage_type: DamageType) -> f64 {
        self.damages
            .iter()
            .filter(|d| d.damage_type == damage_type)
            .map(|d| d.amount)
            .sum()
    }

    /// Add damage of a type
    pub fn add_damage(&mut self, damage_type: DamageType, amount: f64) {
        if let Some(existing) = self.damages.iter_mut().find(|d| d.damage_type == damage_type) {
            existing.amount += amount;
        } else {
            self.damages.push(FinalDamage {
                damage_type,
                amount,
            });
        }
    }

    /// Get penetration for a damage type
    pub fn penetration(&self, damage_type: DamageType) -> f64 {
        match damage_type {
            DamageType::Physical => 0.0, // Physical doesn't penetrate
            DamageType::Fire => self.fire_pen,
            DamageType::Cold => self.cold_pen,
            DamageType::Lightning => self.lightning_pen,
            DamageType::Chaos => self.chaos_pen,
        }
    }

    /// Check if this packet has any damage
    pub fn has_damage(&self) -> bool {
        self.total_damage() > 0.0
    }

    /// Get a damage breakdown string for display
    pub fn damage_breakdown(&self) -> String {
        self.damages
            .iter()
            .map(|d| format!("{:?}: {:.0}", d.damage_type, d.amount))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Final damage value for a single damage type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalDamage {
    pub damage_type: DamageType,
    pub amount: f64,
}

impl FinalDamage {
    pub fn new(damage_type: DamageType, amount: f64) -> Self {
        FinalDamage {
            damage_type,
            amount,
        }
    }
}

/// A DoT effect pending application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingDoT {
    /// DoT type ID
    pub dot_type: String,
    /// Damage per second
    pub damage_per_second: f64,
    /// Duration in seconds
    pub duration: f64,
}

impl PendingDoT {
    pub fn new(dot_type: String, dps: f64, duration: f64) -> Self {
        PendingDoT {
            dot_type,
            damage_per_second: dps,
            duration,
        }
    }

    /// Get total damage this DoT will deal
    pub fn total_damage(&self) -> f64 {
        self.damage_per_second * self.duration
    }
}

/// A status effect pending application
/// Status effects don't deal direct damage but have a chance to apply
/// based on status_damage / target_max_health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingStatusEffect {
    /// The type of status effect
    pub effect_type: StatusEffect,
    /// The "status damage" that determines application chance
    /// Chance to apply = status_damage / target_max_health
    pub status_damage: f64,
    /// Duration of the effect in seconds
    pub duration: f64,
    /// Magnitude of the effect (e.g., slow percentage)
    pub magnitude: f64,
    /// For damaging DoTs (Poison, Bleed, Burn): damage per second
    /// Based on base_dot_percent * status_damage
    pub dot_dps: f64,
}

impl PendingStatusEffect {
    pub fn new(effect_type: StatusEffect, status_damage: f64, duration: f64, magnitude: f64) -> Self {
        PendingStatusEffect {
            effect_type,
            status_damage,
            duration,
            magnitude,
            dot_dps: 0.0,
        }
    }

    /// Create a new status effect with DoT damage
    pub fn new_with_dot(
        effect_type: StatusEffect,
        status_damage: f64,
        duration: f64,
        magnitude: f64,
        dot_dps: f64,
    ) -> Self {
        PendingStatusEffect {
            effect_type,
            status_damage,
            duration,
            magnitude,
            dot_dps,
        }
    }

    /// Calculate the chance to apply this status effect
    /// Returns a value between 0.0 and 1.0
    pub fn calculate_apply_chance(&self, target_max_health: f64) -> f64 {
        if target_max_health <= 0.0 {
            return 0.0;
        }
        (self.status_damage / target_max_health).clamp(0.0, 1.0)
    }

    /// Check if this is a damaging status effect
    pub fn is_damaging(&self) -> bool {
        matches!(
            self.effect_type,
            StatusEffect::Poison | StatusEffect::Bleed | StatusEffect::Burn
        )
    }

    /// Get total DoT damage this effect will deal over its duration
    pub fn total_dot_damage(&self) -> f64 {
        self.dot_dps * self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_packet_total() {
        let mut packet = DamagePacket::new("player".to_string(), "fireball".to_string());
        packet.add_damage(DamageType::Fire, 100.0);
        packet.add_damage(DamageType::Physical, 20.0);

        assert!((packet.total_damage() - 120.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_packet_by_type() {
        let mut packet = DamagePacket::new("player".to_string(), "fireball".to_string());
        packet.add_damage(DamageType::Fire, 100.0);
        packet.add_damage(DamageType::Cold, 50.0);

        assert!((packet.damage_of_type(DamageType::Fire) - 100.0).abs() < f64::EPSILON);
        assert!((packet.damage_of_type(DamageType::Cold) - 50.0).abs() < f64::EPSILON);
        assert!((packet.damage_of_type(DamageType::Physical) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_damage_packet_add_same_type() {
        let mut packet = DamagePacket::new("player".to_string(), "fireball".to_string());
        packet.add_damage(DamageType::Fire, 100.0);
        packet.add_damage(DamageType::Fire, 50.0);

        assert!((packet.damage_of_type(DamageType::Fire) - 150.0).abs() < f64::EPSILON);
        assert_eq!(packet.damages.len(), 1);
    }

    #[test]
    fn test_pending_dot_total() {
        let dot = PendingDoT::new("ignite".to_string(), 25.0, 4.0);
        assert!((dot.total_damage() - 100.0).abs() < f64::EPSILON);
    }
}
