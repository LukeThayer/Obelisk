//! StatAccumulator - Collects stat modifications before applying to StatBlock

use crate::stat_block::StatBlock;
use loot_core::types::{DamageType, StatType, StatusEffect};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Stats for a specific status effect type
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatusEffectStats {
    /// Increased DoT damage for DoT-based status effects (poison, bleed, burn)
    pub dot_increased: f64,
    /// Increased duration for the status effect
    pub duration_increased: f64,
    /// Increased magnitude (affects status damage calculation)
    pub magnitude: f64,
    /// Additional max stacks beyond base
    pub max_stacks: i32,
}

/// Conversion stats from damage types to a status effect
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusConversions {
    /// Conversion percentages from each damage type
    conversions: HashMap<DamageType, f64>,
}

impl StatusConversions {
    /// Get total conversion percentage (summed from all damage types)
    pub fn total(&self) -> f64 {
        self.conversions.values().sum()
    }

    /// Get conversion from a specific damage type
    pub fn from_damage_type(&self, dt: DamageType) -> f64 {
        self.conversions.get(&dt).copied().unwrap_or(0.0)
    }

    /// Add conversion from a damage type
    pub fn add_conversion(&mut self, dt: DamageType, value: f64) {
        *self.conversions.entry(dt).or_insert(0.0) += value;
    }
}

/// Accumulates stat modifications from various sources
///
/// This is used during stat rebuilding to collect all modifications
/// before applying them to a StatBlock.
#[derive(Debug, Clone, Default)]
pub struct StatAccumulator {
    // === Resources ===
    pub life_flat: f64,
    pub life_increased: f64,
    pub life_more: Vec<f64>,
    pub mana_flat: f64,
    pub mana_increased: f64,
    pub mana_more: Vec<f64>,

    // === Attributes ===
    pub strength_flat: f64,
    pub dexterity_flat: f64,
    pub intelligence_flat: f64,
    pub constitution_flat: f64,
    pub wisdom_flat: f64,
    pub charisma_flat: f64,
    pub all_attributes_flat: f64,

    // === Defenses ===
    pub armour_flat: f64,
    pub armour_increased: f64,
    pub evasion_flat: f64,
    pub evasion_increased: f64,
    pub energy_shield_flat: f64,
    pub energy_shield_increased: f64,
    pub fire_resistance: f64,
    pub cold_resistance: f64,
    pub lightning_resistance: f64,
    pub chaos_resistance: f64,
    pub all_resistances: f64,

    // === Offense ===
    pub physical_damage_flat: f64,
    pub physical_damage_increased: f64,
    pub physical_damage_more: Vec<f64>,
    pub fire_damage_flat: f64,
    pub fire_damage_increased: f64,
    pub fire_damage_more: Vec<f64>,
    pub cold_damage_flat: f64,
    pub cold_damage_increased: f64,
    pub cold_damage_more: Vec<f64>,
    pub lightning_damage_flat: f64,
    pub lightning_damage_increased: f64,
    pub lightning_damage_more: Vec<f64>,
    pub chaos_damage_flat: f64,
    pub chaos_damage_increased: f64,
    pub chaos_damage_more: Vec<f64>,
    pub elemental_damage_increased: f64,
    pub attack_speed_increased: f64,
    pub cast_speed_increased: f64,
    pub critical_chance_flat: f64,
    pub critical_chance_increased: f64,
    pub critical_multiplier_flat: f64,

    // === Penetration ===
    pub fire_penetration: f64,
    pub cold_penetration: f64,
    pub lightning_penetration: f64,
    pub chaos_penetration: f64,

    // === Recovery ===
    pub life_regen_flat: f64,
    pub mana_regen_flat: f64,
    pub life_leech_percent: f64,
    pub mana_leech_percent: f64,
    pub life_on_hit: f64,

    // === Accuracy ===
    pub accuracy_flat: f64,
    pub accuracy_increased: f64,

    // === Utility ===
    pub movement_speed_increased: f64,
    pub item_rarity_increased: f64,
    pub item_quantity_increased: f64,

    // === Weapon Stats ===
    pub weapon_physical_min: f64,
    pub weapon_physical_max: f64,
    pub weapon_physical_increased: f64, // Local increased physical damage on weapon
    pub weapon_elemental_damages: Vec<(DamageType, f64, f64)>,
    pub weapon_attack_speed: f64,
    pub weapon_crit_chance: f64,

    // === Status Effect Stats (HashMap-based for extensibility) ===
    /// Stats for each status effect type (dot_increased, duration, magnitude, max_stacks)
    pub status_stats: HashMap<StatusEffect, StatusEffectStats>,
    /// Damage type to status effect conversion percentages
    pub status_conversions: HashMap<StatusEffect, StatusConversions>,
}

impl StatAccumulator {
    /// Create a new empty accumulator
    pub fn new() -> Self {
        StatAccumulator::default()
    }

    /// Apply a loot_core StatType modifier to this accumulator
    pub fn apply_stat_type(&mut self, stat: StatType, value: f64) {
        match stat {
            // Flat damage additions
            StatType::AddedPhysicalDamage => self.physical_damage_flat += value,
            StatType::AddedFireDamage => self.fire_damage_flat += value,
            StatType::AddedColdDamage => self.cold_damage_flat += value,
            StatType::AddedLightningDamage => self.lightning_damage_flat += value,
            StatType::AddedChaosDamage => self.chaos_damage_flat += value,

            // Percentage increases (convert from percentage to decimal)
            StatType::IncreasedPhysicalDamage => self.physical_damage_increased += value / 100.0,
            StatType::IncreasedFireDamage => self.fire_damage_increased += value / 100.0,
            StatType::IncreasedColdDamage => self.cold_damage_increased += value / 100.0,
            StatType::IncreasedLightningDamage => self.lightning_damage_increased += value / 100.0,
            StatType::IncreasedElementalDamage => self.elemental_damage_increased += value / 100.0,
            StatType::IncreasedChaosDamage => self.chaos_damage_increased += value / 100.0,
            StatType::IncreasedAttackSpeed => self.attack_speed_increased += value / 100.0,
            StatType::IncreasedCriticalChance => self.critical_chance_increased += value / 100.0,
            StatType::IncreasedCriticalDamage => self.critical_multiplier_flat += value / 100.0,

            // Defenses
            StatType::AddedArmour => self.armour_flat += value,
            StatType::AddedEvasion => self.evasion_flat += value,
            StatType::AddedEnergyShield => self.energy_shield_flat += value,
            StatType::IncreasedArmour => self.armour_increased += value / 100.0,
            StatType::IncreasedEvasion => self.evasion_increased += value / 100.0,
            StatType::IncreasedEnergyShield => self.energy_shield_increased += value / 100.0,

            // Attributes
            StatType::AddedStrength => self.strength_flat += value,
            StatType::AddedDexterity => self.dexterity_flat += value,
            StatType::AddedConstitution => self.constitution_flat += value,
            StatType::AddedIntelligence => self.intelligence_flat += value,
            StatType::AddedWisdom => self.wisdom_flat += value,
            StatType::AddedCharisma => self.charisma_flat += value,
            StatType::AddedAllAttributes => self.all_attributes_flat += value,

            // Life and resources
            StatType::AddedLife => self.life_flat += value,
            StatType::AddedMana => self.mana_flat += value,
            StatType::IncreasedLife => self.life_increased += value / 100.0,
            StatType::IncreasedMana => self.mana_increased += value / 100.0,
            StatType::LifeRegeneration => self.life_regen_flat += value,
            StatType::ManaRegeneration => self.mana_regen_flat += value,
            StatType::LifeOnHit => self.life_on_hit += value,
            StatType::LifeLeech => self.life_leech_percent += value / 100.0,
            StatType::ManaLeech => self.mana_leech_percent += value / 100.0,

            // Resistances
            StatType::FireResistance => self.fire_resistance += value,
            StatType::ColdResistance => self.cold_resistance += value,
            StatType::LightningResistance => self.lightning_resistance += value,
            StatType::ChaosResistance => self.chaos_resistance += value,
            StatType::AllResistances => self.all_resistances += value,

            // Accuracy
            StatType::AddedAccuracy => self.accuracy_flat += value,
            StatType::IncreasedAccuracy => self.accuracy_increased += value / 100.0,

            // Utility
            StatType::IncreasedMovementSpeed => self.movement_speed_increased += value / 100.0,
            StatType::IncreasedItemRarity => self.item_rarity_increased += value / 100.0,
            StatType::IncreasedItemQuantity => self.item_quantity_increased += value / 100.0,

            // === Status Effect Stats (using helper methods for HashMap access) ===
            // Poison
            StatType::PoisonDamageOverTime => self.add_status_dot(StatusEffect::Poison, value / 100.0),
            StatType::IncreasedPoisonDuration => self.add_status_duration(StatusEffect::Poison, value / 100.0),
            StatType::PoisonMagnitude => self.add_status_magnitude(StatusEffect::Poison, value / 100.0),
            StatType::PoisonMaxStacks => self.add_status_max_stacks(StatusEffect::Poison, value as i32),
            StatType::ConvertPhysicalToPoison => self.add_conversion(DamageType::Physical, StatusEffect::Poison, value / 100.0),
            StatType::ConvertFireToPoison => self.add_conversion(DamageType::Fire, StatusEffect::Poison, value / 100.0),
            StatType::ConvertColdToPoison => self.add_conversion(DamageType::Cold, StatusEffect::Poison, value / 100.0),
            StatType::ConvertLightningToPoison => self.add_conversion(DamageType::Lightning, StatusEffect::Poison, value / 100.0),
            StatType::ConvertChaosToPoison => self.add_conversion(DamageType::Chaos, StatusEffect::Poison, value / 100.0),

            // Bleed
            StatType::BleedDamageOverTime => self.add_status_dot(StatusEffect::Bleed, value / 100.0),
            StatType::IncreasedBleedDuration => self.add_status_duration(StatusEffect::Bleed, value / 100.0),
            StatType::BleedMagnitude => self.add_status_magnitude(StatusEffect::Bleed, value / 100.0),
            StatType::BleedMaxStacks => self.add_status_max_stacks(StatusEffect::Bleed, value as i32),
            StatType::ConvertPhysicalToBleed => self.add_conversion(DamageType::Physical, StatusEffect::Bleed, value / 100.0),
            StatType::ConvertFireToBleed => self.add_conversion(DamageType::Fire, StatusEffect::Bleed, value / 100.0),
            StatType::ConvertColdToBleed => self.add_conversion(DamageType::Cold, StatusEffect::Bleed, value / 100.0),
            StatType::ConvertLightningToBleed => self.add_conversion(DamageType::Lightning, StatusEffect::Bleed, value / 100.0),
            StatType::ConvertChaosToBleed => self.add_conversion(DamageType::Chaos, StatusEffect::Bleed, value / 100.0),

            // Burn
            StatType::BurnDamageOverTime => self.add_status_dot(StatusEffect::Burn, value / 100.0),
            StatType::IncreasedBurnDuration => self.add_status_duration(StatusEffect::Burn, value / 100.0),
            StatType::BurnMagnitude => self.add_status_magnitude(StatusEffect::Burn, value / 100.0),
            StatType::BurnMaxStacks => self.add_status_max_stacks(StatusEffect::Burn, value as i32),
            StatType::ConvertPhysicalToBurn => self.add_conversion(DamageType::Physical, StatusEffect::Burn, value / 100.0),
            StatType::ConvertFireToBurn => self.add_conversion(DamageType::Fire, StatusEffect::Burn, value / 100.0),
            StatType::ConvertColdToBurn => self.add_conversion(DamageType::Cold, StatusEffect::Burn, value / 100.0),
            StatType::ConvertLightningToBurn => self.add_conversion(DamageType::Lightning, StatusEffect::Burn, value / 100.0),
            StatType::ConvertChaosToBurn => self.add_conversion(DamageType::Chaos, StatusEffect::Burn, value / 100.0),

            // Freeze
            StatType::IncreasedFreezeDuration => self.add_status_duration(StatusEffect::Freeze, value / 100.0),
            StatType::FreezeMagnitude => self.add_status_magnitude(StatusEffect::Freeze, value / 100.0),
            StatType::FreezeMaxStacks => self.add_status_max_stacks(StatusEffect::Freeze, value as i32),
            StatType::ConvertPhysicalToFreeze => self.add_conversion(DamageType::Physical, StatusEffect::Freeze, value / 100.0),
            StatType::ConvertFireToFreeze => self.add_conversion(DamageType::Fire, StatusEffect::Freeze, value / 100.0),
            StatType::ConvertColdToFreeze => self.add_conversion(DamageType::Cold, StatusEffect::Freeze, value / 100.0),
            StatType::ConvertLightningToFreeze => self.add_conversion(DamageType::Lightning, StatusEffect::Freeze, value / 100.0),
            StatType::ConvertChaosToFreeze => self.add_conversion(DamageType::Chaos, StatusEffect::Freeze, value / 100.0),

            // Chill
            StatType::IncreasedChillDuration => self.add_status_duration(StatusEffect::Chill, value / 100.0),
            StatType::ChillMagnitude => self.add_status_magnitude(StatusEffect::Chill, value / 100.0),
            StatType::ChillMaxStacks => self.add_status_max_stacks(StatusEffect::Chill, value as i32),
            StatType::ConvertPhysicalToChill => self.add_conversion(DamageType::Physical, StatusEffect::Chill, value / 100.0),
            StatType::ConvertFireToChill => self.add_conversion(DamageType::Fire, StatusEffect::Chill, value / 100.0),
            StatType::ConvertColdToChill => self.add_conversion(DamageType::Cold, StatusEffect::Chill, value / 100.0),
            StatType::ConvertLightningToChill => self.add_conversion(DamageType::Lightning, StatusEffect::Chill, value / 100.0),
            StatType::ConvertChaosToChill => self.add_conversion(DamageType::Chaos, StatusEffect::Chill, value / 100.0),

            // Static
            StatType::IncreasedStaticDuration => self.add_status_duration(StatusEffect::Static, value / 100.0),
            StatType::StaticMagnitude => self.add_status_magnitude(StatusEffect::Static, value / 100.0),
            StatType::StaticMaxStacks => self.add_status_max_stacks(StatusEffect::Static, value as i32),
            StatType::ConvertPhysicalToStatic => self.add_conversion(DamageType::Physical, StatusEffect::Static, value / 100.0),
            StatType::ConvertFireToStatic => self.add_conversion(DamageType::Fire, StatusEffect::Static, value / 100.0),
            StatType::ConvertColdToStatic => self.add_conversion(DamageType::Cold, StatusEffect::Static, value / 100.0),
            StatType::ConvertLightningToStatic => self.add_conversion(DamageType::Lightning, StatusEffect::Static, value / 100.0),
            StatType::ConvertChaosToStatic => self.add_conversion(DamageType::Chaos, StatusEffect::Static, value / 100.0),

            // Fear
            StatType::IncreasedFearDuration => self.add_status_duration(StatusEffect::Fear, value / 100.0),
            StatType::FearMagnitude => self.add_status_magnitude(StatusEffect::Fear, value / 100.0),
            StatType::FearMaxStacks => self.add_status_max_stacks(StatusEffect::Fear, value as i32),
            StatType::ConvertPhysicalToFear => self.add_conversion(DamageType::Physical, StatusEffect::Fear, value / 100.0),
            StatType::ConvertFireToFear => self.add_conversion(DamageType::Fire, StatusEffect::Fear, value / 100.0),
            StatType::ConvertColdToFear => self.add_conversion(DamageType::Cold, StatusEffect::Fear, value / 100.0),
            StatType::ConvertLightningToFear => self.add_conversion(DamageType::Lightning, StatusEffect::Fear, value / 100.0),
            StatType::ConvertChaosToFear => self.add_conversion(DamageType::Chaos, StatusEffect::Fear, value / 100.0),

            // Slow
            StatType::IncreasedSlowDuration => self.add_status_duration(StatusEffect::Slow, value / 100.0),
            StatType::SlowMagnitude => self.add_status_magnitude(StatusEffect::Slow, value / 100.0),
            StatType::SlowMaxStacks => self.add_status_max_stacks(StatusEffect::Slow, value as i32),
            StatType::ConvertPhysicalToSlow => self.add_conversion(DamageType::Physical, StatusEffect::Slow, value / 100.0),
            StatType::ConvertFireToSlow => self.add_conversion(DamageType::Fire, StatusEffect::Slow, value / 100.0),
            StatType::ConvertColdToSlow => self.add_conversion(DamageType::Cold, StatusEffect::Slow, value / 100.0),
            StatType::ConvertLightningToSlow => self.add_conversion(DamageType::Lightning, StatusEffect::Slow, value / 100.0),
            StatType::ConvertChaosToSlow => self.add_conversion(DamageType::Chaos, StatusEffect::Slow, value / 100.0),
        }
    }

    // === Status Effect Helper Methods ===

    /// Add to a status effect's DoT increased stat
    fn add_status_dot(&mut self, status: StatusEffect, value: f64) {
        self.status_stats.entry(status).or_default().dot_increased += value;
    }

    /// Add to a status effect's duration increased stat
    fn add_status_duration(&mut self, status: StatusEffect, value: f64) {
        self.status_stats.entry(status).or_default().duration_increased += value;
    }

    /// Add to a status effect's magnitude stat
    fn add_status_magnitude(&mut self, status: StatusEffect, value: f64) {
        self.status_stats.entry(status).or_default().magnitude += value;
    }

    /// Add to a status effect's max stacks
    fn add_status_max_stacks(&mut self, status: StatusEffect, value: i32) {
        self.status_stats.entry(status).or_default().max_stacks += value;
    }

    /// Add a damage type to status effect conversion
    fn add_conversion(&mut self, from: DamageType, to: StatusEffect, value: f64) {
        self.status_conversions.entry(to).or_default().add_conversion(from, value);
    }

    /// Get conversion percentage for a damage type to a status effect
    pub fn get_conversion(&self, from: DamageType, to: StatusEffect) -> f64 {
        self.status_conversions
            .get(&to)
            .map(|conv| conv.from_damage_type(from))
            .unwrap_or(0.0)
    }

    /// Get status effect stats for a given status type
    pub fn get_status_stats(&self, status: StatusEffect) -> StatusEffectStats {
        self.status_stats.get(&status).copied().unwrap_or_default()
    }

    /// Get status conversions for a given status effect type
    pub fn get_status_conversions(&self, status: StatusEffect) -> StatusConversions {
        self.status_conversions.get(&status).cloned().unwrap_or_default()
    }

    /// Apply accumulated stats to a StatBlock
    pub fn apply_to(&self, block: &mut StatBlock) {
        // Resources
        block.max_life.add_flat(self.life_flat);
        block.max_life.add_increased(self.life_increased);
        for more in &self.life_more {
            block.max_life.add_more(*more);
        }
        block.max_mana.add_flat(self.mana_flat);
        block.max_mana.add_increased(self.mana_increased);
        for more in &self.mana_more {
            block.max_mana.add_more(*more);
        }

        // Attributes (all_attributes applies to all)
        block.strength.add_flat(self.strength_flat + self.all_attributes_flat);
        block.dexterity.add_flat(self.dexterity_flat + self.all_attributes_flat);
        block.intelligence.add_flat(self.intelligence_flat + self.all_attributes_flat);
        block.constitution.add_flat(self.constitution_flat + self.all_attributes_flat);
        block.wisdom.add_flat(self.wisdom_flat + self.all_attributes_flat);
        block.charisma.add_flat(self.charisma_flat + self.all_attributes_flat);

        // Defenses
        block.armour.add_flat(self.armour_flat);
        block.armour.add_increased(self.armour_increased);
        block.evasion.add_flat(self.evasion_flat);
        block.evasion.add_increased(self.evasion_increased);

        // Resistances (all_resistances applies to elemental)
        block.fire_resistance.add_flat(self.fire_resistance + self.all_resistances);
        block.cold_resistance.add_flat(self.cold_resistance + self.all_resistances);
        block.lightning_resistance.add_flat(self.lightning_resistance + self.all_resistances);
        block.chaos_resistance.add_flat(self.chaos_resistance);

        // Damage - apply elemental increased to fire/cold/lightning
        block.global_physical_damage.add_flat(self.physical_damage_flat);
        block.global_physical_damage.add_increased(self.physical_damage_increased);
        for more in &self.physical_damage_more {
            block.global_physical_damage.add_more(*more);
        }

        block.global_fire_damage.add_flat(self.fire_damage_flat);
        block.global_fire_damage.add_increased(self.fire_damage_increased + self.elemental_damage_increased);
        for more in &self.fire_damage_more {
            block.global_fire_damage.add_more(*more);
        }

        block.global_cold_damage.add_flat(self.cold_damage_flat);
        block.global_cold_damage.add_increased(self.cold_damage_increased + self.elemental_damage_increased);
        for more in &self.cold_damage_more {
            block.global_cold_damage.add_more(*more);
        }

        block.global_lightning_damage.add_flat(self.lightning_damage_flat);
        block.global_lightning_damage.add_increased(self.lightning_damage_increased + self.elemental_damage_increased);
        for more in &self.lightning_damage_more {
            block.global_lightning_damage.add_more(*more);
        }

        block.global_chaos_damage.add_flat(self.chaos_damage_flat);
        block.global_chaos_damage.add_increased(self.chaos_damage_increased);
        for more in &self.chaos_damage_more {
            block.global_chaos_damage.add_more(*more);
        }

        // Attack/Cast speed
        block.attack_speed.add_increased(self.attack_speed_increased);
        block.cast_speed.add_increased(self.cast_speed_increased);

        // Crit
        block.critical_chance.add_flat(self.critical_chance_flat);
        block.critical_chance.add_increased(self.critical_chance_increased);
        block.critical_multiplier.add_flat(self.critical_multiplier_flat);

        // Penetration
        block.fire_penetration.add_flat(self.fire_penetration);
        block.cold_penetration.add_flat(self.cold_penetration);
        block.lightning_penetration.add_flat(self.lightning_penetration);
        block.chaos_penetration.add_flat(self.chaos_penetration);

        // Recovery
        block.life_regen.add_flat(self.life_regen_flat);
        block.mana_regen.add_flat(self.mana_regen_flat);
        block.life_leech.add_flat(self.life_leech_percent);
        block.mana_leech.add_flat(self.mana_leech_percent);

        // Weapon stats - apply local increased physical damage
        if self.weapon_physical_min > 0.0 || self.weapon_physical_max > 0.0 {
            let phys_mult = 1.0 + self.weapon_physical_increased;
            block.weapon_physical_min = self.weapon_physical_min * phys_mult;
            block.weapon_physical_max = self.weapon_physical_max * phys_mult;
        }
        if self.weapon_attack_speed > 0.0 {
            block.weapon_attack_speed = self.weapon_attack_speed;
        }
        if self.weapon_crit_chance > 0.0 {
            block.weapon_crit_chance = self.weapon_crit_chance;
        }

        // Apply weapon elemental damages
        for (dmg_type, min, max) in &self.weapon_elemental_damages {
            match dmg_type {
                DamageType::Fire => {
                    block.weapon_fire_min += min;
                    block.weapon_fire_max += max;
                }
                DamageType::Cold => {
                    block.weapon_cold_min += min;
                    block.weapon_cold_max += max;
                }
                DamageType::Lightning => {
                    block.weapon_lightning_min += min;
                    block.weapon_lightning_max += max;
                }
                DamageType::Chaos => {
                    block.weapon_chaos_min += min;
                    block.weapon_chaos_max += max;
                }
                DamageType::Physical => {
                    // Physical is handled separately
                }
            }
        }

        // Accuracy
        block.accuracy.add_flat(self.accuracy_flat);
        block.accuracy.add_increased(self.accuracy_increased);

        // Utility
        block.movement_speed_increased += self.movement_speed_increased;
        block.item_rarity_increased += self.item_rarity_increased;
        block.item_quantity_increased += self.item_quantity_increased;

        // Status effect stats - copy all accumulated stats and conversions
        for (status, stats) in &self.status_stats {
            block.status_effect_stats.set_stats(*status, *stats);
        }
        for (status, conversions) in &self.status_conversions {
            block.status_effect_stats.set_conversions(*status, conversions.clone());
        }
    }
}
