//! GearSource - Stats from equipped items

use crate::source::StatSource;
use crate::stat_block::StatAccumulator;
use crate::types::EquipmentSlot;
use loot_core::item::Modifier;
use loot_core::types::{AffixScope, DamageType, StatType};
use loot_core::Item;

/// Stats from an equipped item
pub struct GearSource {
    /// Which slot this item is in
    pub slot: EquipmentSlot,
    /// The equipped item
    pub item: Item,
}

impl GearSource {
    /// Create a new gear source
    pub fn new(slot: EquipmentSlot, item: Item) -> Self {
        GearSource { slot, item }
    }

    /// Apply a modifier, handling local scope for weapons
    fn apply_modifier(&self, stats: &mut StatAccumulator, modifier: &Modifier, is_weapon: bool) {
        // Local scope on weapons: add to weapon damage
        if is_weapon && modifier.scope == AffixScope::Local {
            match modifier.stat {
                StatType::AddedPhysicalDamage => {
                    let min = modifier.value as f64;
                    let max = modifier.value_max.unwrap_or(modifier.value) as f64;
                    stats.weapon_physical_min += min;
                    stats.weapon_physical_max += max;
                }
                StatType::AddedFireDamage => {
                    let min = modifier.value as f64;
                    let max = modifier.value_max.unwrap_or(modifier.value) as f64;
                    stats.weapon_elemental_damages.push((DamageType::Fire, min, max));
                }
                StatType::AddedColdDamage => {
                    let min = modifier.value as f64;
                    let max = modifier.value_max.unwrap_or(modifier.value) as f64;
                    stats.weapon_elemental_damages.push((DamageType::Cold, min, max));
                }
                StatType::AddedLightningDamage => {
                    let min = modifier.value as f64;
                    let max = modifier.value_max.unwrap_or(modifier.value) as f64;
                    stats.weapon_elemental_damages.push((DamageType::Lightning, min, max));
                }
                StatType::AddedChaosDamage => {
                    let min = modifier.value as f64;
                    let max = modifier.value_max.unwrap_or(modifier.value) as f64;
                    stats.weapon_elemental_damages.push((DamageType::Chaos, min, max));
                }
                StatType::IncreasedPhysicalDamage => {
                    stats.weapon_physical_increased += modifier.value as f64 / 100.0;
                }
                // Other local stats fall through to global handling
                _ => {
                    stats.apply_stat_type(modifier.stat, modifier.value as f64);
                }
            }
        } else {
            // Global scope or non-weapon: apply as character stat
            stats.apply_stat_type(modifier.stat, modifier.value as f64);
        }
    }
}

impl StatSource for GearSource {
    fn id(&self) -> &str {
        &self.item.base_type_id
    }

    fn priority(&self) -> i32 {
        0 // Gear applies at default priority
    }

    fn apply(&self, stats: &mut StatAccumulator) {
        let is_weapon = self.item.damage.is_some() && matches!(self.slot, EquipmentSlot::MainHand);

        // Apply implicit modifier
        if let Some(ref implicit) = self.item.implicit {
            self.apply_modifier(stats, implicit, is_weapon);
        }

        // Apply prefix modifiers
        for prefix in &self.item.prefixes {
            self.apply_modifier(stats, prefix, is_weapon);
        }

        // Apply suffix modifiers
        for suffix in &self.item.suffixes {
            self.apply_modifier(stats, suffix, is_weapon);
        }

        // Apply base defenses
        if let Some(armour) = self.item.defenses.armour {
            stats.armour_flat += armour as f64;
        }
        if let Some(evasion) = self.item.defenses.evasion {
            stats.evasion_flat += evasion as f64;
        }
        if let Some(es) = self.item.defenses.energy_shield {
            stats.energy_shield_flat += es as f64;
        }

        // Apply weapon damage (if weapon)
        if let Some(ref damage) = self.item.damage {
            // Only apply if this is the main hand weapon
            if matches!(self.slot, EquipmentSlot::MainHand) {
                for entry in &damage.damages {
                    match entry.damage_type {
                        DamageType::Physical => {
                            stats.weapon_physical_min = entry.min as f64;
                            stats.weapon_physical_max = entry.max as f64;
                        }
                        _ => {
                            stats.weapon_elemental_damages.push((
                                entry.damage_type,
                                entry.min as f64,
                                entry.max as f64,
                            ));
                        }
                    }
                }
                stats.weapon_attack_speed = damage.attack_speed as f64;
                stats.weapon_crit_chance = damage.critical_chance as f64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gear_source_id() {
        // Create a minimal item for testing
        let item = Item {
            seed: 12345,
            operations: vec![],
            base_type_id: "test_sword".to_string(),
            name: "Test Sword".to_string(),
            base_name: "Sword".to_string(),
            class: loot_core::types::ItemClass::OneHandSword,
            rarity: loot_core::types::Rarity::Normal,
            tags: vec![],
            requirements: loot_core::types::Requirements::default(),
            implicit: None,
            prefixes: vec![],
            suffixes: vec![],
            defenses: loot_core::item::Defenses::default(),
            damage: None,
        };

        let source = GearSource::new(EquipmentSlot::MainHand, item);
        assert_eq!(source.id(), "test_sword");
    }
}
