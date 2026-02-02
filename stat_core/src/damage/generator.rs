//! DamagePacketGenerator - Skill/ability damage configuration

use crate::types::SkillTag;
use loot_core::types::{DamageType, StatusEffect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Describes how a skill calculates its damage
/// Loaded from TOML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamagePacketGenerator {
    /// Unique skill identifier
    pub id: String,
    /// Display name
    pub name: String,

    // === Base Damage ===
    /// Base damage values by type
    #[serde(default)]
    pub base_damages: Vec<BaseDamage>,

    // === Scaling ===
    /// How much weapon damage to use (0.0 = pure spell, 1.0 = full attack)
    #[serde(default)]
    pub weapon_effectiveness: f64,
    /// How much added damage applies (1.0 = 100%)
    #[serde(default = "default_damage_effectiveness")]
    pub damage_effectiveness: f64,
    /// Multiplier to attack/cast speed
    #[serde(default = "default_speed_modifier")]
    pub attack_speed_modifier: f64,

    // === Crit ===
    /// Skill's base crit chance (adds to weapon crit)
    #[serde(default)]
    pub base_crit_chance: f64,
    /// Added to base crit multiplier
    #[serde(default)]
    pub crit_multiplier_bonus: f64,

    // === Tags ===
    /// Skill tags for categorization and scaling
    #[serde(default)]
    pub tags: Vec<SkillTag>,

    // === Status Effect Conversions ===
    /// Skill-specific conversions from damage types to status effects
    /// These add to the player's stat-based conversions
    #[serde(default)]
    pub status_conversions: SkillStatusConversions,

    // === Damage Type Conversions ===
    /// Convert damage from one type to another (e.g., 50% physical to fire)
    /// Applied before damage scaling
    #[serde(default)]
    pub damage_conversions: DamageConversions,

    // === Per-Type Effectiveness ===
    /// Damage effectiveness multiplier for each damage type
    /// Defaults to 1.0 (100%) for all types if not specified
    #[serde(default)]
    pub type_effectiveness: DamageTypeEffectiveness,

    // === Special Mechanics ===
    /// Number of hits per attack (for multi-hit skills)
    #[serde(default = "default_hits")]
    pub hits_per_attack: u32,
    /// Whether the skill can chain to other targets
    #[serde(default)]
    pub can_chain: bool,
    /// Number of chains
    #[serde(default)]
    pub chain_count: u32,
    /// Chance to pierce targets (0.0 to 1.0)
    #[serde(default)]
    pub pierce_chance: f64,
}

/// Skill-specific status effect conversions
/// Values are percentages (0.0 to 1.0) of damage converted to status damage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStatusConversions {
    // Poison conversions
    #[serde(default)]
    pub physical_to_poison: f64,
    #[serde(default)]
    pub chaos_to_poison: f64,

    // Bleed conversions
    #[serde(default)]
    pub physical_to_bleed: f64,

    // Burn conversions
    #[serde(default)]
    pub fire_to_burn: f64,

    // Freeze conversions
    #[serde(default)]
    pub cold_to_freeze: f64,

    // Chill conversions
    #[serde(default)]
    pub cold_to_chill: f64,

    // Static conversions
    #[serde(default)]
    pub lightning_to_static: f64,

    // Fear conversions
    #[serde(default)]
    pub chaos_to_fear: f64,

    // Slow conversions
    #[serde(default)]
    pub physical_to_slow: f64,
    #[serde(default)]
    pub cold_to_slow: f64,
}

impl SkillStatusConversions {
    /// Get conversion percentage from a damage type to a status effect
    pub fn get_conversion(&self, from: DamageType, to: StatusEffect) -> f64 {
        use loot_core::types::StatusEffect::*;
        match (from, to) {
            // Poison - Physical and Chaos can poison
            (DamageType::Physical, Poison) => self.physical_to_poison,
            (DamageType::Chaos, Poison) => self.chaos_to_poison,

            // Bleed - Physical causes bleeding
            (DamageType::Physical, Bleed) => self.physical_to_bleed,

            // Burn - Fire causes burning
            (DamageType::Fire, Burn) => self.fire_to_burn,

            // Freeze - Cold causes freezing
            (DamageType::Cold, Freeze) => self.cold_to_freeze,

            // Chill - Cold causes chill
            (DamageType::Cold, Chill) => self.cold_to_chill,

            // Static - Lightning causes static
            (DamageType::Lightning, Static) => self.lightning_to_static,

            // Fear - Chaos causes fear
            (DamageType::Chaos, Fear) => self.chaos_to_fear,

            // Slow - Physical and Cold can slow
            (DamageType::Physical, Slow) => self.physical_to_slow,
            (DamageType::Cold, Slow) => self.cold_to_slow,

            // All other combinations have 0 conversion
            _ => 0.0,
        }
    }
}

/// Damage type conversion configuration
/// Values are percentages (0.0 to 1.0) of damage converted from one type to another
/// Conversion order: Physical -> Lightning -> Cold -> Fire (like PoE)
/// Chaos cannot be converted to or from other types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DamageConversions {
    // Physical conversions
    #[serde(default)]
    pub physical_to_fire: f64,
    #[serde(default)]
    pub physical_to_cold: f64,
    #[serde(default)]
    pub physical_to_lightning: f64,
    #[serde(default)]
    pub physical_to_chaos: f64,

    // Lightning conversions
    #[serde(default)]
    pub lightning_to_fire: f64,
    #[serde(default)]
    pub lightning_to_cold: f64,

    // Cold conversions
    #[serde(default)]
    pub cold_to_fire: f64,

    // Fire conversions (fire is last in conversion order, can only go to chaos)
    #[serde(default)]
    pub fire_to_chaos: f64,
}

impl DamageConversions {
    /// Get conversion percentage from one damage type to another
    pub fn get_conversion(&self, from: DamageType, to: DamageType) -> f64 {
        match (from, to) {
            (DamageType::Physical, DamageType::Fire) => self.physical_to_fire,
            (DamageType::Physical, DamageType::Cold) => self.physical_to_cold,
            (DamageType::Physical, DamageType::Lightning) => self.physical_to_lightning,
            (DamageType::Physical, DamageType::Chaos) => self.physical_to_chaos,
            (DamageType::Lightning, DamageType::Fire) => self.lightning_to_fire,
            (DamageType::Lightning, DamageType::Cold) => self.lightning_to_cold,
            (DamageType::Cold, DamageType::Fire) => self.cold_to_fire,
            (DamageType::Fire, DamageType::Chaos) => self.fire_to_chaos,
            _ => 0.0,
        }
    }

    /// Check if there are any conversions defined
    pub fn has_conversions(&self) -> bool {
        self.physical_to_fire > 0.0
            || self.physical_to_cold > 0.0
            || self.physical_to_lightning > 0.0
            || self.physical_to_chaos > 0.0
            || self.lightning_to_fire > 0.0
            || self.lightning_to_cold > 0.0
            || self.cold_to_fire > 0.0
            || self.fire_to_chaos > 0.0
    }

    /// Apply conversions to a damage map, returning new damage values
    /// Conversion order: Physical -> Lightning -> Cold -> Fire
    pub fn apply(&self, damages: &HashMap<DamageType, f64>) -> HashMap<DamageType, f64> {
        let mut result: HashMap<DamageType, f64> = HashMap::new();

        // Start with original values
        for (dt, amt) in damages {
            *result.entry(*dt).or_insert(0.0) += amt;
        }

        // Convert Physical (first in order)
        if let Some(&phys) = result.get(&DamageType::Physical) {
            let to_fire = phys * self.physical_to_fire;
            let to_cold = phys * self.physical_to_cold;
            let to_lightning = phys * self.physical_to_lightning;
            let to_chaos = phys * self.physical_to_chaos;
            let total_converted = (to_fire + to_cold + to_lightning + to_chaos).min(phys);

            if total_converted > 0.0 {
                *result.entry(DamageType::Physical).or_insert(0.0) -= total_converted;
                *result.entry(DamageType::Fire).or_insert(0.0) += to_fire;
                *result.entry(DamageType::Cold).or_insert(0.0) += to_cold;
                *result.entry(DamageType::Lightning).or_insert(0.0) += to_lightning;
                *result.entry(DamageType::Chaos).or_insert(0.0) += to_chaos;
            }
        }

        // Convert Lightning (second in order)
        if let Some(&lightning) = result.get(&DamageType::Lightning) {
            let to_fire = lightning * self.lightning_to_fire;
            let to_cold = lightning * self.lightning_to_cold;
            let total_converted = (to_fire + to_cold).min(lightning);

            if total_converted > 0.0 {
                *result.entry(DamageType::Lightning).or_insert(0.0) -= total_converted;
                *result.entry(DamageType::Fire).or_insert(0.0) += to_fire;
                *result.entry(DamageType::Cold).or_insert(0.0) += to_cold;
            }
        }

        // Convert Cold (third in order)
        if let Some(&cold) = result.get(&DamageType::Cold) {
            let to_fire = cold * self.cold_to_fire;

            if to_fire > 0.0 {
                *result.entry(DamageType::Cold).or_insert(0.0) -= to_fire;
                *result.entry(DamageType::Fire).or_insert(0.0) += to_fire;
            }
        }

        // Convert Fire (last in order, can only go to chaos)
        if let Some(&fire) = result.get(&DamageType::Fire) {
            let to_chaos = fire * self.fire_to_chaos;

            if to_chaos > 0.0 {
                *result.entry(DamageType::Fire).or_insert(0.0) -= to_chaos;
                *result.entry(DamageType::Chaos).or_insert(0.0) += to_chaos;
            }
        }

        // Remove zero/negative entries
        result.retain(|_, v| *v > 0.0);
        result
    }
}

/// Per-damage-type effectiveness multipliers
/// Values are multipliers (1.0 = 100%, 1.5 = 150%, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageTypeEffectiveness {
    #[serde(default = "default_effectiveness")]
    pub physical: f64,
    #[serde(default = "default_effectiveness")]
    pub fire: f64,
    #[serde(default = "default_effectiveness")]
    pub cold: f64,
    #[serde(default = "default_effectiveness")]
    pub lightning: f64,
    #[serde(default = "default_effectiveness")]
    pub chaos: f64,
}

fn default_effectiveness() -> f64 {
    1.0
}

impl Default for DamageTypeEffectiveness {
    fn default() -> Self {
        DamageTypeEffectiveness {
            physical: 1.0,
            fire: 1.0,
            cold: 1.0,
            lightning: 1.0,
            chaos: 1.0,
        }
    }
}

impl DamageTypeEffectiveness {
    /// Get effectiveness multiplier for a damage type
    pub fn get(&self, damage_type: DamageType) -> f64 {
        match damage_type {
            DamageType::Physical => self.physical,
            DamageType::Fire => self.fire,
            DamageType::Cold => self.cold,
            DamageType::Lightning => self.lightning,
            DamageType::Chaos => self.chaos,
        }
    }

    /// Check if all effectiveness values are 1.0 (default)
    pub fn is_default(&self) -> bool {
        (self.physical - 1.0).abs() < f64::EPSILON
            && (self.fire - 1.0).abs() < f64::EPSILON
            && (self.cold - 1.0).abs() < f64::EPSILON
            && (self.lightning - 1.0).abs() < f64::EPSILON
            && (self.chaos - 1.0).abs() < f64::EPSILON
    }
}

fn default_damage_effectiveness() -> f64 {
    1.0
}

fn default_speed_modifier() -> f64 {
    1.0
}

fn default_hits() -> u32 {
    1
}

impl Default for DamagePacketGenerator {
    fn default() -> Self {
        DamagePacketGenerator {
            id: "default".to_string(),
            name: "Default Attack".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.0,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 1.0,
            base_crit_chance: 0.0,
            crit_multiplier_bonus: 0.0,
            tags: vec![SkillTag::Attack],
            status_conversions: SkillStatusConversions::default(),
            damage_conversions: DamageConversions::default(),
            type_effectiveness: DamageTypeEffectiveness::default(),
            hits_per_attack: 1,
            can_chain: false,
            chain_count: 0,
            pierce_chance: 0.0,
        }
    }
}

impl DamagePacketGenerator {
    /// Create a basic melee attack
    pub fn basic_attack() -> Self {
        DamagePacketGenerator {
            id: "basic_attack".to_string(),
            name: "Basic Attack".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.0,
            damage_effectiveness: 1.0,
            attack_speed_modifier: 1.0,
            base_crit_chance: 0.0,
            crit_multiplier_bonus: 0.0,
            tags: vec![SkillTag::Attack, SkillTag::Melee],
            status_conversions: SkillStatusConversions::default(),
            damage_conversions: DamageConversions::default(),
            type_effectiveness: DamageTypeEffectiveness::default(),
            hits_per_attack: 1,
            can_chain: false,
            chain_count: 0,
            pierce_chance: 0.0,
        }
    }

    /// Check if this skill is an attack (uses weapon)
    pub fn is_attack(&self) -> bool {
        self.tags.contains(&SkillTag::Attack)
    }

    /// Check if this skill is a spell
    pub fn is_spell(&self) -> bool {
        self.tags.contains(&SkillTag::Spell)
    }

    /// Check if this skill deals a specific damage type
    pub fn deals_damage_type(&self, damage_type: DamageType) -> bool {
        self.base_damages.iter().any(|d| d.damage_type == damage_type)
            || match damage_type {
                DamageType::Physical => self.tags.contains(&SkillTag::Physical),
                DamageType::Fire => self.tags.contains(&SkillTag::Fire),
                DamageType::Cold => self.tags.contains(&SkillTag::Cold),
                DamageType::Lightning => self.tags.contains(&SkillTag::Lightning),
                DamageType::Chaos => self.tags.contains(&SkillTag::Chaos),
            }
    }

    /// Get the effective attack speed for this skill
    pub fn effective_speed(&self, base_speed: f64) -> f64 {
        base_speed * self.attack_speed_modifier
    }
}

/// Base damage for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseDamage {
    /// Damage type
    #[serde(rename = "type")]
    pub damage_type: DamageType,
    /// Minimum damage
    pub min: f64,
    /// Maximum damage
    pub max: f64,
}

impl BaseDamage {
    /// Create a new base damage entry
    pub fn new(damage_type: DamageType, min: f64, max: f64) -> Self {
        BaseDamage {
            damage_type,
            min,
            max,
        }
    }

    /// Get average damage
    pub fn average(&self) -> f64 {
        (self.min + self.max) / 2.0
    }

    /// Roll a random damage value
    pub fn roll(&self, rng: &mut impl rand::Rng) -> f64 {
        rng.gen_range(self.min..=self.max)
    }
}

/// DoT application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotApplication {
    /// DoT type ID (references DoT config)
    pub dot_type: String,
    /// Chance to apply (1.0 = 100%)
    #[serde(default = "default_dot_chance")]
    pub chance: f64,
    /// Percentage of hit damage dealt as DoT
    pub damage_percent: f64,
}

fn default_dot_chance() -> f64 {
    1.0
}

impl DotApplication {
    /// Check if the DoT should be applied (rolls chance)
    pub fn should_apply(&self, rng: &mut impl rand::Rng) -> bool {
        rng.gen::<f64>() < self.chance
    }

    /// Calculate DoT damage from hit damage
    pub fn calculate_dot_damage(&self, hit_damage: f64) -> f64 {
        hit_damage * self.damage_percent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_attack() {
        let attack = DamagePacketGenerator::basic_attack();
        assert!(attack.is_attack());
        assert!(!attack.is_spell());
        assert!((attack.weapon_effectiveness - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_base_damage_average() {
        let damage = BaseDamage::new(DamageType::Physical, 10.0, 20.0);
        assert!((damage.average() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_skill_tags() {
        let skill = DamagePacketGenerator {
            tags: vec![SkillTag::Spell, SkillTag::Fire, SkillTag::Aoe],
            ..Default::default()
        };

        assert!(skill.is_spell());
        assert!(!skill.is_attack());
        assert!(skill.deals_damage_type(DamageType::Fire));
    }

    #[test]
    fn test_effective_speed() {
        let skill = DamagePacketGenerator {
            attack_speed_modifier: 0.85, // 15% slower
            ..Default::default()
        };

        let base_speed = 1.5;
        let effective = skill.effective_speed(base_speed);
        assert!((effective - 1.275).abs() < 0.001);
    }

    #[test]
    fn test_damage_conversion_physical_to_fire() {
        let conv = DamageConversions {
            physical_to_fire: 0.5, // 50% physical to fire
            ..Default::default()
        };

        let mut input = HashMap::new();
        input.insert(DamageType::Physical, 100.0);

        let result = conv.apply(&input);

        // Should have 50 physical remaining and 50 fire
        assert!((result.get(&DamageType::Physical).unwrap_or(&0.0) - 50.0).abs() < 0.001);
        assert!((result.get(&DamageType::Fire).unwrap_or(&0.0) - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_damage_conversion_full_chain() {
        // Test Physical -> Lightning -> Cold -> Fire chain
        let conv = DamageConversions {
            physical_to_lightning: 0.5, // 50% phys to lightning
            lightning_to_cold: 0.5,     // 50% lightning to cold (including converted)
            cold_to_fire: 0.5,          // 50% cold to fire
            ..Default::default()
        };

        let mut input = HashMap::new();
        input.insert(DamageType::Physical, 100.0);

        let result = conv.apply(&input);

        // Physical: 100 - 50 (converted) = 50
        // Lightning: 50 (from phys) - 25 (to cold) = 25
        // Cold: 25 (from lightning) - 12.5 (to fire) = 12.5
        // Fire: 12.5 (from cold)
        assert!((result.get(&DamageType::Physical).unwrap_or(&0.0) - 50.0).abs() < 0.001);
        assert!((result.get(&DamageType::Lightning).unwrap_or(&0.0) - 25.0).abs() < 0.001);
        assert!((result.get(&DamageType::Cold).unwrap_or(&0.0) - 12.5).abs() < 0.001);
        assert!((result.get(&DamageType::Fire).unwrap_or(&0.0) - 12.5).abs() < 0.001);
    }

    #[test]
    fn test_type_effectiveness() {
        let eff = DamageTypeEffectiveness {
            physical: 1.0,
            fire: 1.5,    // 150% fire effectiveness
            cold: 0.5,    // 50% cold effectiveness
            lightning: 1.0,
            chaos: 1.0,
        };

        assert!((eff.get(DamageType::Fire) - 1.5).abs() < f64::EPSILON);
        assert!((eff.get(DamageType::Cold) - 0.5).abs() < f64::EPSILON);
        assert!(!eff.is_default());
    }
}
