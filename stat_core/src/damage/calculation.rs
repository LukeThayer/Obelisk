//! Damage calculation - turning a skill + stats into a DamagePacket

use super::{DamagePacket, DamagePacketGenerator, PendingStatusEffect, SkillStatusConversions};
use crate::stat_block::{StatusEffectData, StatusEffectStats, StatBlock};
use crate::types::Effect;
use loot_core::types::{DamageType, StatusEffect};
use rand::Rng;
use std::collections::HashMap;

/// Calculate damage from a skill and attacker's stats
pub fn calculate_damage(
    attacker: &StatBlock,
    skill: &DamagePacketGenerator,
    source_id: String,
    rng: &mut impl Rng,
) -> DamagePacket {
    let mut packet = DamagePacket::new(source_id, skill.id.clone());

    // Step 1: Gather base damage (pre-conversion, pre-scaling)
    let mut base_damages: HashMap<DamageType, f64> = HashMap::new();

    // Skill base damages
    for base_dmg in &skill.base_damages {
        let rolled = if base_dmg.min >= base_dmg.max {
            base_dmg.max
        } else {
            rng.gen_range(base_dmg.min..=base_dmg.max)
        };
        *base_damages.entry(base_dmg.damage_type).or_insert(0.0) += rolled;
    }

    // Weapon damage if this is an attack skill
    if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
        for damage_type in [
            DamageType::Physical,
            DamageType::Fire,
            DamageType::Cold,
            DamageType::Lightning,
            DamageType::Chaos,
        ] {
            let (min, max) = attacker.weapon_damage(damage_type);
            if max > 0.0 {
                let scaled_min = min * skill.weapon_effectiveness;
                let scaled_max = max * skill.weapon_effectiveness;
                let rolled = if scaled_min >= scaled_max {
                    scaled_max
                } else {
                    rng.gen_range(scaled_min..=scaled_max)
                };
                *base_damages.entry(damage_type).or_insert(0.0) += rolled;
            }
        }
    }

    // Step 2: Apply damage type conversions (before scaling)
    let converted_damages = if skill.damage_conversions.has_conversions() {
        skill.damage_conversions.apply(&base_damages)
    } else {
        base_damages
    };

    // Step 3: Apply damage scaling to each type
    for (damage_type, base_amount) in converted_damages {
        if base_amount <= 0.0 {
            continue;
        }

        let damage_stat = match damage_type {
            DamageType::Physical => &attacker.global_physical_damage,
            DamageType::Fire => &attacker.global_fire_damage,
            DamageType::Cold => &attacker.global_cold_damage,
            DamageType::Lightning => &attacker.global_lightning_damage,
            DamageType::Chaos => &attacker.global_chaos_damage,
        };

        let increased_mult = damage_stat.total_increased_multiplier();
        let more_mult = damage_stat.total_more_multiplier();
        let type_eff = skill.type_effectiveness.get(damage_type);

        let scaled_damage = base_amount * increased_mult * more_mult * skill.damage_effectiveness * type_eff;
        if scaled_damage > 0.0 {
            packet.add_damage(damage_type, scaled_damage);
        }
    }

    // Step 4: Calculate crit
    let crit_chance = calculate_crit_chance(attacker, skill);
    packet.is_critical = rng.gen::<f64>() < crit_chance / 100.0;

    if packet.is_critical {
        packet.crit_multiplier = attacker.computed_crit_multiplier() + skill.crit_multiplier_bonus;
        // Apply crit multiplier to all damages
        for damage in &mut packet.damages {
            damage.amount *= packet.crit_multiplier;
        }
    }

    // Step 4: Set penetration from attacker stats
    packet.fire_pen = attacker.fire_penetration.compute();
    packet.cold_pen = attacker.cold_penetration.compute();
    packet.lightning_pen = attacker.lightning_penetration.compute();
    packet.chaos_pen = attacker.chaos_penetration.compute();

    // Step 5: Set accuracy from attacker stats
    packet.accuracy = attacker.accuracy.compute();

    // Step 6: Calculate status effect applications
    // Status damage is converted from hit damage (combining skill + player conversions)
    // Status damage determines: chance to apply = status_damage / target_max_health
    // For damaging DoTs: DoT DPS = base_dot_percent * status_damage
    let damages_vec: Vec<(DamageType, f64)> = packet
        .damages
        .iter()
        .map(|d| (d.damage_type, d.amount))
        .collect();

    for status in [
        StatusEffect::Poison,
        StatusEffect::Bleed,
        StatusEffect::Burn,
        StatusEffect::Freeze,
        StatusEffect::Chill,
        StatusEffect::Static,
        StatusEffect::Fear,
        StatusEffect::Slow,
    ] {
        // Combine skill conversions + player stat conversions
        let status_damage = calculate_combined_status_damage(
            status,
            &damages_vec,
            &skill.status_conversions,
            &attacker.status_effect_stats,
        );

        if status_damage > 0.0 {
            let stats = attacker.status_effect_stats.get_stats(status);
            let base_duration = Effect::base_duration_for(status);
            let duration = base_duration * (1.0 + stats.duration_increased);
            let magnitude = 1.0 + stats.magnitude;

            // For damaging DoTs, calculate DoT DPS based on status damage
            let base_dot_percent = Effect::base_dot_percent_for(status);
            let dot_dps = calculate_status_dot_dps(base_dot_percent, status_damage, stats);

            packet.status_effects_to_apply.push(PendingStatusEffect::new_with_dot(
                status,
                status_damage,
                duration,
                magnitude,
                dot_dps,
            ));
        }
    }

    // Step 8: Set hit count for multi-hit skills
    packet.hit_count = skill.hits_per_attack;

    packet
}


/// Calculate combined status damage from skill conversions + player stat conversions
fn calculate_combined_status_damage(
    status: StatusEffect,
    damages: &[(DamageType, f64)],
    skill_conversions: &SkillStatusConversions,
    player_stats: &StatusEffectData,
) -> f64 {
    let player_conversions = player_stats.get_conversions(status);
    let mut total = 0.0;

    for (damage_type, amount) in damages {
        // Get skill conversion for this damage type -> status
        let skill_conv = skill_conversions.get_conversion(*damage_type, status);
        // Get player conversion from stats/gear
        let player_conv = player_conversions.from_damage_type(*damage_type);
        // Combine them (additive)
        let total_conv = skill_conv + player_conv;

        total += amount * total_conv;
    }

    total
}

/// Calculate DoT DPS for damaging status effects (Poison, Bleed, Burn)
/// DoT DPS = base_dot_percent * status_damage * (1 + dot_increased)
fn calculate_status_dot_dps(base_dot_percent: f64, status_damage: f64, stats: &StatusEffectStats) -> f64 {
    if base_dot_percent == 0.0 {
        return 0.0;
    }

    // Apply DoT increased modifier
    base_dot_percent * status_damage * (1.0 + stats.dot_increased)
}

/// Calculate critical strike chance
fn calculate_crit_chance(attacker: &StatBlock, skill: &DamagePacketGenerator) -> f64 {
    // Base crit = skill base + weapon base (for attacks)
    let base_crit = if skill.is_attack() {
        skill.base_crit_chance + attacker.weapon_crit_chance
    } else {
        skill.base_crit_chance
    };

    // Add flat crit chance from stats
    let flat_crit = base_crit + attacker.critical_chance.flat;

    // Apply increased crit chance
    let increased_mult = attacker.critical_chance.total_increased_multiplier();
    let more_mult = attacker.critical_chance.total_more_multiplier();

    (flat_crit * increased_mult * more_mult).clamp(0.0, 100.0)
}

/// Calculate effective DPS for a skill
pub fn calculate_skill_dps(
    attacker: &StatBlock,
    skill: &DamagePacketGenerator,
) -> f64 {
    // Use average damage instead of random
    let avg_damages = calculate_average_damage_by_type(attacker, skill);
    let total_avg_damage: f64 = avg_damages.iter().map(|(_, amt)| amt).sum();

    // Calculate crit contribution
    let crit_chance = calculate_crit_chance(attacker, skill) / 100.0;
    let crit_mult = attacker.computed_crit_multiplier() + skill.crit_multiplier_bonus;
    let crit_dps_mult = 1.0 + (crit_mult - 1.0) * crit_chance;

    // Get attack/cast speed
    let speed = if skill.is_attack() {
        attacker.computed_attack_speed() * skill.attack_speed_modifier
    } else {
        attacker.computed_cast_speed() * skill.attack_speed_modifier
    };

    // Calculate hit DPS (before crit scaling on avg damages)
    let hit_dps = total_avg_damage * crit_dps_mult * speed * skill.hits_per_attack as f64;

    // Calculate status DoT DPS contribution from damaging statuses (Poison, Bleed, Burn)
    let mut dot_dps = 0.0;
    for status in [StatusEffect::Poison, StatusEffect::Bleed, StatusEffect::Burn] {
        let status_damage = calculate_combined_status_damage(
            status,
            &avg_damages,
            &skill.status_conversions,
            &attacker.status_effect_stats,
        );

        if status_damage > 0.0 {
            let stats = attacker.status_effect_stats.get_stats(status);
            let base_dot_percent = Effect::base_dot_percent_for(status);
            let status_dot_dps = calculate_status_dot_dps(base_dot_percent, status_damage, stats);
            // Scale by attack speed (more hits = more DoT applications)
            dot_dps += status_dot_dps * speed;
        }
    }

    hit_dps + dot_dps
}

/// Calculate average damage by type (non-random)
/// Returns Vec of (DamageType, scaled_amount) after conversions and scaling
pub fn calculate_average_damage_by_type(attacker: &StatBlock, skill: &DamagePacketGenerator) -> Vec<(DamageType, f64)> {
    // Step 1: Gather base damage averages (pre-conversion, pre-scaling)
    let mut base_damages: HashMap<DamageType, f64> = HashMap::new();

    // Skill base damages
    for base_dmg in &skill.base_damages {
        let avg = (base_dmg.min + base_dmg.max) / 2.0;
        *base_damages.entry(base_dmg.damage_type).or_insert(0.0) += avg;
    }

    // Weapon damages for attacks
    if skill.is_attack() && skill.weapon_effectiveness > 0.0 {
        for damage_type in [
            DamageType::Physical,
            DamageType::Fire,
            DamageType::Cold,
            DamageType::Lightning,
            DamageType::Chaos,
        ] {
            let (min, max) = attacker.weapon_damage(damage_type);
            if max > 0.0 {
                let avg = (min + max) / 2.0 * skill.weapon_effectiveness;
                *base_damages.entry(damage_type).or_insert(0.0) += avg;
            }
        }
    }

    // Step 2: Apply damage type conversions
    let converted_damages = if skill.damage_conversions.has_conversions() {
        skill.damage_conversions.apply(&base_damages)
    } else {
        base_damages
    };

    // Step 3: Apply damage scaling to each type
    let mut result: Vec<(DamageType, f64)> = Vec::new();

    for (damage_type, base_amount) in converted_damages {
        if base_amount <= 0.0 {
            continue;
        }

        let damage_stat = match damage_type {
            DamageType::Physical => &attacker.global_physical_damage,
            DamageType::Fire => &attacker.global_fire_damage,
            DamageType::Cold => &attacker.global_cold_damage,
            DamageType::Lightning => &attacker.global_lightning_damage,
            DamageType::Chaos => &attacker.global_chaos_damage,
        };

        let increased_mult = damage_stat.total_increased_multiplier();
        let more_mult = damage_stat.total_more_multiplier();
        let type_eff = skill.type_effectiveness.get(damage_type);

        let scaled = base_amount * increased_mult * more_mult * skill.damage_effectiveness * type_eff;
        if scaled > 0.0 {
            result.push((damage_type, scaled));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::BaseDamage;
    use crate::types::SkillTag;
    use rand::SeedableRng;

    fn make_test_rng() -> rand::rngs::StdRng {
        rand::rngs::StdRng::seed_from_u64(12345)
    }

    #[test]
    fn test_basic_damage_calculation() {
        let attacker = StatBlock::new();
        let skill = DamagePacketGenerator {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_damages: vec![BaseDamage::new(DamageType::Physical, 100.0, 100.0)],
            weapon_effectiveness: 0.0,
            ..Default::default()
        };

        let mut rng = make_test_rng();
        let packet = calculate_damage(&attacker, &skill, "player".to_string(), &mut rng);

        // With no scaling, should deal base damage
        assert!((packet.total_damage() - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_damage_scaling() {
        let mut attacker = StatBlock::new();
        attacker.global_physical_damage.add_increased(0.50); // 50% increased

        let skill = DamagePacketGenerator {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_damages: vec![BaseDamage::new(DamageType::Physical, 100.0, 100.0)],
            weapon_effectiveness: 0.0,
            ..Default::default()
        };

        let mut rng = make_test_rng();
        let packet = calculate_damage(&attacker, &skill, "player".to_string(), &mut rng);

        // 100 * 1.5 = 150
        assert!((packet.total_damage() - 150.0).abs() < 1.0);
    }

    #[test]
    fn test_weapon_damage() {
        let mut attacker = StatBlock::new();
        attacker.weapon_physical_min = 50.0;
        attacker.weapon_physical_max = 50.0;

        let skill = DamagePacketGenerator {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.0,
            tags: vec![SkillTag::Attack],
            ..Default::default()
        };

        let mut rng = make_test_rng();
        let packet = calculate_damage(&attacker, &skill, "player".to_string(), &mut rng);

        // Should deal weapon damage
        assert!((packet.damage_of_type(DamageType::Physical) - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_crit_multiplier() {
        let mut attacker = StatBlock::new();
        // Force crit by setting high crit chance via flat (added from gear etc)
        attacker.critical_chance.flat = 100.0;

        let skill = DamagePacketGenerator {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_damages: vec![BaseDamage::new(DamageType::Physical, 100.0, 100.0)],
            weapon_effectiveness: 0.0,
            base_crit_chance: 0.0,
            ..Default::default()
        };

        let mut rng = make_test_rng();
        let packet = calculate_damage(&attacker, &skill, "player".to_string(), &mut rng);

        assert!(packet.is_critical);
        // 100 * 1.5 (base crit multi) = 150
        assert!((packet.total_damage() - 150.0).abs() < 1.0);
    }

    #[test]
    fn test_skill_dps() {
        let mut attacker = StatBlock::new();
        attacker.weapon_physical_min = 100.0;
        attacker.weapon_physical_max = 100.0;
        attacker.weapon_attack_speed = 1.0;

        let skill = DamagePacketGenerator {
            id: "attack".to_string(),
            name: "Attack".to_string(),
            base_damages: vec![],
            weapon_effectiveness: 1.0,
            tags: vec![SkillTag::Attack],
            base_crit_chance: 5.0,
            ..Default::default()
        };

        let dps = calculate_skill_dps(&attacker, &skill);

        // Base DPS: 100 damage * 1.0 speed = 100
        // With 5% crit at 1.5x: 100 * (1 + 0.05 * 0.5) = 102.5
        assert!(dps > 100.0);
        assert!(dps < 110.0);
    }
}
