//! Computed/derived stat calculations for StatBlock

use crate::stat_block::StatBlock;
use loot_core::types::DamageType;

impl StatBlock {
    /// Get the damage scaling multiplier for a specific damage type
    pub fn damage_multiplier(&self, damage_type: DamageType) -> f64 {
        match damage_type {
            DamageType::Physical => self.global_physical_damage.compute().max(1.0),
            DamageType::Fire => self.global_fire_damage.compute().max(1.0),
            DamageType::Cold => self.global_cold_damage.compute().max(1.0),
            DamageType::Lightning => self.global_lightning_damage.compute().max(1.0),
            DamageType::Chaos => self.global_chaos_damage.compute().max(1.0),
        }
    }

    /// Get the resistance value for a damage type (uncapped)
    pub fn resistance(&self, damage_type: DamageType) -> f64 {
        match damage_type {
            DamageType::Physical => 0.0, // Physical uses armour, not resistance
            DamageType::Fire => self.fire_resistance.compute(),
            DamageType::Cold => self.cold_resistance.compute(),
            DamageType::Lightning => self.lightning_resistance.compute(),
            DamageType::Chaos => self.chaos_resistance.compute(),
        }
    }

    /// Get the penetration value for a damage type
    pub fn penetration(&self, damage_type: DamageType) -> f64 {
        match damage_type {
            DamageType::Physical => 0.0, // Physical doesn't have penetration
            DamageType::Fire => self.fire_penetration.compute(),
            DamageType::Cold => self.cold_penetration.compute(),
            DamageType::Lightning => self.lightning_penetration.compute(),
            DamageType::Chaos => self.chaos_penetration.compute(),
        }
    }

    /// Get computed attack speed
    pub fn computed_attack_speed(&self) -> f64 {
        self.attack_speed.compute() * self.weapon_attack_speed
    }

    /// Get computed cast speed
    pub fn computed_cast_speed(&self) -> f64 {
        self.cast_speed.compute()
    }

    /// Get computed critical strike chance for attacks
    pub fn computed_attack_crit_chance(&self) -> f64 {
        // Base crit from weapon + modifiers
        let base_crit = self.weapon_crit_chance + self.critical_chance.flat;
        base_crit * self.critical_chance.total_increased_multiplier()
            * self.critical_chance.total_more_multiplier()
    }

    /// Get computed critical strike multiplier
    pub fn computed_crit_multiplier(&self) -> f64 {
        self.critical_multiplier.compute()
    }

    /// Get weapon damage range for a damage type
    pub fn weapon_damage(&self, damage_type: DamageType) -> (f64, f64) {
        match damage_type {
            DamageType::Physical => (self.weapon_physical_min, self.weapon_physical_max),
            DamageType::Fire => (self.weapon_fire_min, self.weapon_fire_max),
            DamageType::Cold => (self.weapon_cold_min, self.weapon_cold_max),
            DamageType::Lightning => (self.weapon_lightning_min, self.weapon_lightning_max),
            DamageType::Chaos => (self.weapon_chaos_min, self.weapon_chaos_max),
        }
    }

    /// Get total weapon DPS (all damage types)
    pub fn weapon_dps(&self) -> f64 {
        let phys_avg = (self.weapon_physical_min + self.weapon_physical_max) / 2.0;
        let fire_avg = (self.weapon_fire_min + self.weapon_fire_max) / 2.0;
        let cold_avg = (self.weapon_cold_min + self.weapon_cold_max) / 2.0;
        let light_avg = (self.weapon_lightning_min + self.weapon_lightning_max) / 2.0;
        let chaos_avg = (self.weapon_chaos_min + self.weapon_chaos_max) / 2.0;

        let total_avg = phys_avg + fire_avg + cold_avg + light_avg + chaos_avg;
        total_avg * self.weapon_attack_speed
    }

    /// Calculate life percentage remaining
    pub fn life_percent(&self) -> f64 {
        let max = self.computed_max_life();
        if max <= 0.0 {
            return 0.0;
        }
        (self.current_life / max * 100.0).clamp(0.0, 100.0)
    }

    /// Calculate mana percentage remaining
    pub fn mana_percent(&self) -> f64 {
        let max = self.computed_max_mana();
        if max <= 0.0 {
            return 0.0;
        }
        (self.current_mana / max * 100.0).clamp(0.0, 100.0)
    }

    /// Calculate ES percentage remaining
    pub fn energy_shield_percent(&self) -> f64 {
        if self.max_energy_shield <= 0.0 {
            return 0.0;
        }
        (self.current_energy_shield / self.max_energy_shield * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_multiplier_default() {
        let block = StatBlock::new();
        // Default should be 1.0 (no bonus)
        assert!((block.damage_multiplier(DamageType::Physical) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_life_percent() {
        let mut block = StatBlock::new();
        block.current_life = 25.0;
        // Base life is 50, so 25/50 = 50%
        assert!((block.life_percent() - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_weapon_dps() {
        let mut block = StatBlock::new();
        block.weapon_physical_min = 10.0;
        block.weapon_physical_max = 20.0;
        block.weapon_attack_speed = 1.5;
        // Average: 15, DPS: 15 * 1.5 = 22.5
        assert!((block.weapon_dps() - 22.5).abs() < 0.01);
    }
}
