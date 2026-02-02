//! StatAccumulator - Collects stat modifications before applying to StatBlock

use crate::stat_block::StatBlock;
use loot_core::types::{DamageType, StatType, StatusEffect};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatusConversions {
    pub from_physical: f64,
    pub from_fire: f64,
    pub from_cold: f64,
    pub from_lightning: f64,
    pub from_chaos: f64,
}

impl StatusConversions {
    /// Get total conversion percentage (summed from all damage types)
    pub fn total(&self) -> f64 {
        self.from_physical + self.from_fire + self.from_cold + self.from_lightning + self.from_chaos
    }

    /// Get conversion from a specific damage type
    pub fn from_damage_type(&self, dt: DamageType) -> f64 {
        match dt {
            DamageType::Physical => self.from_physical,
            DamageType::Fire => self.from_fire,
            DamageType::Cold => self.from_cold,
            DamageType::Lightning => self.from_lightning,
            DamageType::Chaos => self.from_chaos,
        }
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

    // === Status Effect Stats ===
    // Poison
    pub poison_dot_increased: f64,
    pub poison_duration_increased: f64,
    pub poison_magnitude: f64,
    pub poison_max_stacks: i32,
    pub convert_physical_to_poison: f64,
    pub convert_fire_to_poison: f64,
    pub convert_cold_to_poison: f64,
    pub convert_lightning_to_poison: f64,
    pub convert_chaos_to_poison: f64,

    // Bleed
    pub bleed_dot_increased: f64,
    pub bleed_duration_increased: f64,
    pub bleed_magnitude: f64,
    pub bleed_max_stacks: i32,
    pub convert_physical_to_bleed: f64,
    pub convert_fire_to_bleed: f64,
    pub convert_cold_to_bleed: f64,
    pub convert_lightning_to_bleed: f64,
    pub convert_chaos_to_bleed: f64,

    // Burn
    pub burn_dot_increased: f64,
    pub burn_duration_increased: f64,
    pub burn_magnitude: f64,
    pub burn_max_stacks: i32,
    pub convert_physical_to_burn: f64,
    pub convert_fire_to_burn: f64,
    pub convert_cold_to_burn: f64,
    pub convert_lightning_to_burn: f64,
    pub convert_chaos_to_burn: f64,

    // Freeze
    pub freeze_duration_increased: f64,
    pub freeze_magnitude: f64,
    pub freeze_max_stacks: i32,
    pub convert_physical_to_freeze: f64,
    pub convert_fire_to_freeze: f64,
    pub convert_cold_to_freeze: f64,
    pub convert_lightning_to_freeze: f64,
    pub convert_chaos_to_freeze: f64,

    // Chill
    pub chill_duration_increased: f64,
    pub chill_magnitude: f64,
    pub chill_max_stacks: i32,
    pub convert_physical_to_chill: f64,
    pub convert_fire_to_chill: f64,
    pub convert_cold_to_chill: f64,
    pub convert_lightning_to_chill: f64,
    pub convert_chaos_to_chill: f64,

    // Static
    pub static_duration_increased: f64,
    pub static_magnitude: f64,
    pub static_max_stacks: i32,
    pub convert_physical_to_static: f64,
    pub convert_fire_to_static: f64,
    pub convert_cold_to_static: f64,
    pub convert_lightning_to_static: f64,
    pub convert_chaos_to_static: f64,

    // Fear
    pub fear_duration_increased: f64,
    pub fear_magnitude: f64,
    pub fear_max_stacks: i32,
    pub convert_physical_to_fear: f64,
    pub convert_fire_to_fear: f64,
    pub convert_cold_to_fear: f64,
    pub convert_lightning_to_fear: f64,
    pub convert_chaos_to_fear: f64,

    // Slow
    pub slow_duration_increased: f64,
    pub slow_magnitude: f64,
    pub slow_max_stacks: i32,
    pub convert_physical_to_slow: f64,
    pub convert_fire_to_slow: f64,
    pub convert_cold_to_slow: f64,
    pub convert_lightning_to_slow: f64,
    pub convert_chaos_to_slow: f64,
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

            // === Status Effect Stats ===
            // Poison
            StatType::PoisonDamageOverTime => self.poison_dot_increased += value / 100.0,
            StatType::IncreasedPoisonDuration => self.poison_duration_increased += value / 100.0,
            StatType::PoisonMagnitude => self.poison_magnitude += value / 100.0,
            StatType::PoisonMaxStacks => self.poison_max_stacks += value as i32,
            StatType::ConvertPhysicalToPoison => self.convert_physical_to_poison += value / 100.0,
            StatType::ConvertFireToPoison => self.convert_fire_to_poison += value / 100.0,
            StatType::ConvertColdToPoison => self.convert_cold_to_poison += value / 100.0,
            StatType::ConvertLightningToPoison => self.convert_lightning_to_poison += value / 100.0,
            StatType::ConvertChaosToPoison => self.convert_chaos_to_poison += value / 100.0,

            // Bleed
            StatType::BleedDamageOverTime => self.bleed_dot_increased += value / 100.0,
            StatType::IncreasedBleedDuration => self.bleed_duration_increased += value / 100.0,
            StatType::BleedMagnitude => self.bleed_magnitude += value / 100.0,
            StatType::BleedMaxStacks => self.bleed_max_stacks += value as i32,
            StatType::ConvertPhysicalToBleed => self.convert_physical_to_bleed += value / 100.0,
            StatType::ConvertFireToBleed => self.convert_fire_to_bleed += value / 100.0,
            StatType::ConvertColdToBleed => self.convert_cold_to_bleed += value / 100.0,
            StatType::ConvertLightningToBleed => self.convert_lightning_to_bleed += value / 100.0,
            StatType::ConvertChaosToBleed => self.convert_chaos_to_bleed += value / 100.0,

            // Burn
            StatType::BurnDamageOverTime => self.burn_dot_increased += value / 100.0,
            StatType::IncreasedBurnDuration => self.burn_duration_increased += value / 100.0,
            StatType::BurnMagnitude => self.burn_magnitude += value / 100.0,
            StatType::BurnMaxStacks => self.burn_max_stacks += value as i32,
            StatType::ConvertPhysicalToBurn => self.convert_physical_to_burn += value / 100.0,
            StatType::ConvertFireToBurn => self.convert_fire_to_burn += value / 100.0,
            StatType::ConvertColdToBurn => self.convert_cold_to_burn += value / 100.0,
            StatType::ConvertLightningToBurn => self.convert_lightning_to_burn += value / 100.0,
            StatType::ConvertChaosToBurn => self.convert_chaos_to_burn += value / 100.0,

            // Freeze
            StatType::IncreasedFreezeDuration => self.freeze_duration_increased += value / 100.0,
            StatType::FreezeMagnitude => self.freeze_magnitude += value / 100.0,
            StatType::FreezeMaxStacks => self.freeze_max_stacks += value as i32,
            StatType::ConvertPhysicalToFreeze => self.convert_physical_to_freeze += value / 100.0,
            StatType::ConvertFireToFreeze => self.convert_fire_to_freeze += value / 100.0,
            StatType::ConvertColdToFreeze => self.convert_cold_to_freeze += value / 100.0,
            StatType::ConvertLightningToFreeze => self.convert_lightning_to_freeze += value / 100.0,
            StatType::ConvertChaosToFreeze => self.convert_chaos_to_freeze += value / 100.0,

            // Chill
            StatType::IncreasedChillDuration => self.chill_duration_increased += value / 100.0,
            StatType::ChillMagnitude => self.chill_magnitude += value / 100.0,
            StatType::ChillMaxStacks => self.chill_max_stacks += value as i32,
            StatType::ConvertPhysicalToChill => self.convert_physical_to_chill += value / 100.0,
            StatType::ConvertFireToChill => self.convert_fire_to_chill += value / 100.0,
            StatType::ConvertColdToChill => self.convert_cold_to_chill += value / 100.0,
            StatType::ConvertLightningToChill => self.convert_lightning_to_chill += value / 100.0,
            StatType::ConvertChaosToChill => self.convert_chaos_to_chill += value / 100.0,

            // Static
            StatType::IncreasedStaticDuration => self.static_duration_increased += value / 100.0,
            StatType::StaticMagnitude => self.static_magnitude += value / 100.0,
            StatType::StaticMaxStacks => self.static_max_stacks += value as i32,
            StatType::ConvertPhysicalToStatic => self.convert_physical_to_static += value / 100.0,
            StatType::ConvertFireToStatic => self.convert_fire_to_static += value / 100.0,
            StatType::ConvertColdToStatic => self.convert_cold_to_static += value / 100.0,
            StatType::ConvertLightningToStatic => self.convert_lightning_to_static += value / 100.0,
            StatType::ConvertChaosToStatic => self.convert_chaos_to_static += value / 100.0,

            // Fear
            StatType::IncreasedFearDuration => self.fear_duration_increased += value / 100.0,
            StatType::FearMagnitude => self.fear_magnitude += value / 100.0,
            StatType::FearMaxStacks => self.fear_max_stacks += value as i32,
            StatType::ConvertPhysicalToFear => self.convert_physical_to_fear += value / 100.0,
            StatType::ConvertFireToFear => self.convert_fire_to_fear += value / 100.0,
            StatType::ConvertColdToFear => self.convert_cold_to_fear += value / 100.0,
            StatType::ConvertLightningToFear => self.convert_lightning_to_fear += value / 100.0,
            StatType::ConvertChaosToFear => self.convert_chaos_to_fear += value / 100.0,

            // Slow
            StatType::IncreasedSlowDuration => self.slow_duration_increased += value / 100.0,
            StatType::SlowMagnitude => self.slow_magnitude += value / 100.0,
            StatType::SlowMaxStacks => self.slow_max_stacks += value as i32,
            StatType::ConvertPhysicalToSlow => self.convert_physical_to_slow += value / 100.0,
            StatType::ConvertFireToSlow => self.convert_fire_to_slow += value / 100.0,
            StatType::ConvertColdToSlow => self.convert_cold_to_slow += value / 100.0,
            StatType::ConvertLightningToSlow => self.convert_lightning_to_slow += value / 100.0,
            StatType::ConvertChaosToSlow => self.convert_chaos_to_slow += value / 100.0,
        }
    }

    /// Get conversion percentage for a damage type to a status effect
    pub fn get_conversion(&self, from: DamageType, to: StatusEffect) -> f64 {
        match (from, to) {
            // Poison conversions
            (DamageType::Physical, StatusEffect::Poison) => self.convert_physical_to_poison,
            (DamageType::Fire, StatusEffect::Poison) => self.convert_fire_to_poison,
            (DamageType::Cold, StatusEffect::Poison) => self.convert_cold_to_poison,
            (DamageType::Lightning, StatusEffect::Poison) => self.convert_lightning_to_poison,
            (DamageType::Chaos, StatusEffect::Poison) => self.convert_chaos_to_poison,

            // Bleed conversions
            (DamageType::Physical, StatusEffect::Bleed) => self.convert_physical_to_bleed,
            (DamageType::Fire, StatusEffect::Bleed) => self.convert_fire_to_bleed,
            (DamageType::Cold, StatusEffect::Bleed) => self.convert_cold_to_bleed,
            (DamageType::Lightning, StatusEffect::Bleed) => self.convert_lightning_to_bleed,
            (DamageType::Chaos, StatusEffect::Bleed) => self.convert_chaos_to_bleed,

            // Burn conversions
            (DamageType::Physical, StatusEffect::Burn) => self.convert_physical_to_burn,
            (DamageType::Fire, StatusEffect::Burn) => self.convert_fire_to_burn,
            (DamageType::Cold, StatusEffect::Burn) => self.convert_cold_to_burn,
            (DamageType::Lightning, StatusEffect::Burn) => self.convert_lightning_to_burn,
            (DamageType::Chaos, StatusEffect::Burn) => self.convert_chaos_to_burn,

            // Freeze conversions
            (DamageType::Physical, StatusEffect::Freeze) => self.convert_physical_to_freeze,
            (DamageType::Fire, StatusEffect::Freeze) => self.convert_fire_to_freeze,
            (DamageType::Cold, StatusEffect::Freeze) => self.convert_cold_to_freeze,
            (DamageType::Lightning, StatusEffect::Freeze) => self.convert_lightning_to_freeze,
            (DamageType::Chaos, StatusEffect::Freeze) => self.convert_chaos_to_freeze,

            // Chill conversions
            (DamageType::Physical, StatusEffect::Chill) => self.convert_physical_to_chill,
            (DamageType::Fire, StatusEffect::Chill) => self.convert_fire_to_chill,
            (DamageType::Cold, StatusEffect::Chill) => self.convert_cold_to_chill,
            (DamageType::Lightning, StatusEffect::Chill) => self.convert_lightning_to_chill,
            (DamageType::Chaos, StatusEffect::Chill) => self.convert_chaos_to_chill,

            // Static conversions
            (DamageType::Physical, StatusEffect::Static) => self.convert_physical_to_static,
            (DamageType::Fire, StatusEffect::Static) => self.convert_fire_to_static,
            (DamageType::Cold, StatusEffect::Static) => self.convert_cold_to_static,
            (DamageType::Lightning, StatusEffect::Static) => self.convert_lightning_to_static,
            (DamageType::Chaos, StatusEffect::Static) => self.convert_chaos_to_static,

            // Fear conversions
            (DamageType::Physical, StatusEffect::Fear) => self.convert_physical_to_fear,
            (DamageType::Fire, StatusEffect::Fear) => self.convert_fire_to_fear,
            (DamageType::Cold, StatusEffect::Fear) => self.convert_cold_to_fear,
            (DamageType::Lightning, StatusEffect::Fear) => self.convert_lightning_to_fear,
            (DamageType::Chaos, StatusEffect::Fear) => self.convert_chaos_to_fear,

            // Slow conversions
            (DamageType::Physical, StatusEffect::Slow) => self.convert_physical_to_slow,
            (DamageType::Fire, StatusEffect::Slow) => self.convert_fire_to_slow,
            (DamageType::Cold, StatusEffect::Slow) => self.convert_cold_to_slow,
            (DamageType::Lightning, StatusEffect::Slow) => self.convert_lightning_to_slow,
            (DamageType::Chaos, StatusEffect::Slow) => self.convert_chaos_to_slow,
        }
    }

    /// Get status effect stats for a given status type
    pub fn get_status_stats(&self, status: StatusEffect) -> StatusEffectStats {
        match status {
            StatusEffect::Poison => StatusEffectStats {
                dot_increased: self.poison_dot_increased,
                duration_increased: self.poison_duration_increased,
                magnitude: self.poison_magnitude,
                max_stacks: self.poison_max_stacks,
            },
            StatusEffect::Bleed => StatusEffectStats {
                dot_increased: self.bleed_dot_increased,
                duration_increased: self.bleed_duration_increased,
                magnitude: self.bleed_magnitude,
                max_stacks: self.bleed_max_stacks,
            },
            StatusEffect::Burn => StatusEffectStats {
                dot_increased: self.burn_dot_increased,
                duration_increased: self.burn_duration_increased,
                magnitude: self.burn_magnitude,
                max_stacks: self.burn_max_stacks,
            },
            StatusEffect::Freeze => StatusEffectStats {
                dot_increased: 0.0,
                duration_increased: self.freeze_duration_increased,
                magnitude: self.freeze_magnitude,
                max_stacks: self.freeze_max_stacks,
            },
            StatusEffect::Chill => StatusEffectStats {
                dot_increased: 0.0,
                duration_increased: self.chill_duration_increased,
                magnitude: self.chill_magnitude,
                max_stacks: self.chill_max_stacks,
            },
            StatusEffect::Static => StatusEffectStats {
                dot_increased: 0.0,
                duration_increased: self.static_duration_increased,
                magnitude: self.static_magnitude,
                max_stacks: self.static_max_stacks,
            },
            StatusEffect::Fear => StatusEffectStats {
                dot_increased: 0.0,
                duration_increased: self.fear_duration_increased,
                magnitude: self.fear_magnitude,
                max_stacks: self.fear_max_stacks,
            },
            StatusEffect::Slow => StatusEffectStats {
                dot_increased: 0.0,
                duration_increased: self.slow_duration_increased,
                magnitude: self.slow_magnitude,
                max_stacks: self.slow_max_stacks,
            },
        }
    }

    /// Get status conversions for a given status effect type
    pub fn get_status_conversions(&self, status: StatusEffect) -> StatusConversions {
        match status {
            StatusEffect::Poison => StatusConversions {
                from_physical: self.convert_physical_to_poison,
                from_fire: self.convert_fire_to_poison,
                from_cold: self.convert_cold_to_poison,
                from_lightning: self.convert_lightning_to_poison,
                from_chaos: self.convert_chaos_to_poison,
            },
            StatusEffect::Bleed => StatusConversions {
                from_physical: self.convert_physical_to_bleed,
                from_fire: self.convert_fire_to_bleed,
                from_cold: self.convert_cold_to_bleed,
                from_lightning: self.convert_lightning_to_bleed,
                from_chaos: self.convert_chaos_to_bleed,
            },
            StatusEffect::Burn => StatusConversions {
                from_physical: self.convert_physical_to_burn,
                from_fire: self.convert_fire_to_burn,
                from_cold: self.convert_cold_to_burn,
                from_lightning: self.convert_lightning_to_burn,
                from_chaos: self.convert_chaos_to_burn,
            },
            StatusEffect::Freeze => StatusConversions {
                from_physical: self.convert_physical_to_freeze,
                from_fire: self.convert_fire_to_freeze,
                from_cold: self.convert_cold_to_freeze,
                from_lightning: self.convert_lightning_to_freeze,
                from_chaos: self.convert_chaos_to_freeze,
            },
            StatusEffect::Chill => StatusConversions {
                from_physical: self.convert_physical_to_chill,
                from_fire: self.convert_fire_to_chill,
                from_cold: self.convert_cold_to_chill,
                from_lightning: self.convert_lightning_to_chill,
                from_chaos: self.convert_chaos_to_chill,
            },
            StatusEffect::Static => StatusConversions {
                from_physical: self.convert_physical_to_static,
                from_fire: self.convert_fire_to_static,
                from_cold: self.convert_cold_to_static,
                from_lightning: self.convert_lightning_to_static,
                from_chaos: self.convert_chaos_to_static,
            },
            StatusEffect::Fear => StatusConversions {
                from_physical: self.convert_physical_to_fear,
                from_fire: self.convert_fire_to_fear,
                from_cold: self.convert_cold_to_fear,
                from_lightning: self.convert_lightning_to_fear,
                from_chaos: self.convert_chaos_to_fear,
            },
            StatusEffect::Slow => StatusConversions {
                from_physical: self.convert_physical_to_slow,
                from_fire: self.convert_fire_to_slow,
                from_cold: self.convert_cold_to_slow,
                from_lightning: self.convert_lightning_to_slow,
                from_chaos: self.convert_chaos_to_slow,
            },
        }
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

        // Status effect stats
        block.status_effect_stats.poison = self.get_status_stats(StatusEffect::Poison);
        block.status_effect_stats.poison_conversions = self.get_status_conversions(StatusEffect::Poison);

        block.status_effect_stats.bleed = self.get_status_stats(StatusEffect::Bleed);
        block.status_effect_stats.bleed_conversions = self.get_status_conversions(StatusEffect::Bleed);

        block.status_effect_stats.burn = self.get_status_stats(StatusEffect::Burn);
        block.status_effect_stats.burn_conversions = self.get_status_conversions(StatusEffect::Burn);

        block.status_effect_stats.freeze = self.get_status_stats(StatusEffect::Freeze);
        block.status_effect_stats.freeze_conversions = self.get_status_conversions(StatusEffect::Freeze);

        block.status_effect_stats.chill = self.get_status_stats(StatusEffect::Chill);
        block.status_effect_stats.chill_conversions = self.get_status_conversions(StatusEffect::Chill);

        block.status_effect_stats.static_effect = self.get_status_stats(StatusEffect::Static);
        block.status_effect_stats.static_conversions = self.get_status_conversions(StatusEffect::Static);

        block.status_effect_stats.fear = self.get_status_stats(StatusEffect::Fear);
        block.status_effect_stats.fear_conversions = self.get_status_conversions(StatusEffect::Fear);

        block.status_effect_stats.slow = self.get_status_stats(StatusEffect::Slow);
        block.status_effect_stats.slow_conversions = self.get_status_conversions(StatusEffect::Slow);
    }
}
