//! Damage resolution - Apply DamagePacket to StatBlock

use super::result::{CombatResult, DamageTaken};
use crate::config::dot_registry;
use crate::damage::DamagePacket;
use crate::defense::{
    apply_evasion_cap, calculate_armour_reduction, calculate_resistance_mitigation,
};
use crate::stat_block::StatBlock;
use crate::types::Effect;
use loot_core::types::{DamageType, StatusEffect};
use rand::Rng;

/// Resolve a damage packet against a defending stat block (immutable API)
///
/// Returns the new defender state and combat result. This is the main combat
/// resolution function that:
/// 1. Applies resistances to each damage type
/// 2. Applies armour to physical damage
/// 3. Applies evasion one-shot protection
/// 4. Applies damage to ES then life
/// 5. Processes status effect applications (chance = status_damage / max_health)
pub fn resolve_damage(defender: &StatBlock, packet: &DamagePacket) -> (StatBlock, CombatResult) {
    let mut rng = rand::thread_rng();
    resolve_damage_with_rng(defender, packet, &mut rng)
}

/// Resolve damage with a provided RNG (for deterministic testing)
pub fn resolve_damage_with_rng(
    defender: &StatBlock,
    packet: &DamagePacket,
    rng: &mut impl Rng,
) -> (StatBlock, CombatResult) {
    let mut new_defender = defender.clone();
    let mut result = CombatResult::new();

    // Store initial state
    result.es_before = new_defender.current_energy_shield;
    result.life_before = new_defender.current_life;

    // Step 0: Spell dodge check
    if packet.is_spell {
        let dodge_chance = new_defender.computed_spell_dodge_chance() / 100.0;
        if dodge_chance > 0.0 && rng.gen::<f64>() < dodge_chance {
            result.was_dodged = true;
            result.es_after = new_defender.current_energy_shield;
            result.life_after = new_defender.current_life;
            return (new_defender, result);
        }
    }

    // Step 1: Calculate mitigated damage for each type
    for final_damage in &packet.damages {
        let raw = final_damage.amount;
        let pen = packet.penetration(final_damage.damage_type);
        let resist = new_defender.resistance(final_damage.damage_type);

        let after_resist = if final_damage.damage_type == DamageType::Physical {
            // Physical uses armour instead of resistance
            raw
        } else {
            calculate_resistance_mitigation(raw, resist, pen)
        };

        let mitigated = raw - after_resist;
        if mitigated > 0.0 {
            result.damage_reduced_by_resists += mitigated;
        }

        result.damage_taken.push(DamageTaken::new(
            final_damage.damage_type,
            raw,
            mitigated.max(0.0),
            after_resist,
        ));
    }

    // Step 2: Apply armour to physical damage
    let physical_damage = result
        .damage_taken
        .iter_mut()
        .find(|d| d.damage_type == DamageType::Physical);

    if let Some(phys) = physical_damage {
        if phys.final_amount > 0.0 {
            let armour = new_defender.armour.compute();
            let after_armour = calculate_armour_reduction(armour, phys.final_amount);
            let armour_reduced = phys.final_amount - after_armour;

            result.damage_reduced_by_armour = armour_reduced;
            phys.mitigated_amount += armour_reduced;
            phys.final_amount = after_armour;
        }
    }

    // Step 2b: Apply physical damage reduction (% reduction, separate from armour)
    let phys_dr = new_defender.physical_damage_reduction.clamp(0.0, 90.0) / 100.0;
    if phys_dr > 0.0 {
        if let Some(phys) = result
            .damage_taken
            .iter_mut()
            .find(|d| d.damage_type == DamageType::Physical)
        {
            if phys.final_amount > 0.0 {
                let reduced = phys.final_amount * phys_dr;
                result.damage_reduced_by_physical_dr = reduced;
                phys.mitigated_amount += reduced;
                phys.final_amount -= reduced;
            }
        }
    }

    // Recalculate total after armour + physical DR
    let total_before_evasion: f64 = result.damage_taken.iter().map(|d| d.final_amount).sum();

    // Step 3: Apply evasion one-shot protection (accuracy vs evasion)
    let evasion = new_defender.evasion.compute();
    let accuracy = packet.accuracy;
    let (damage_after_evasion, evaded) = apply_evasion_cap(accuracy, evasion, total_before_evasion);

    if evaded > 0.0 {
        result.triggered_evasion_cap = true;
        result.damage_prevented_by_evasion = evaded;

        // Proportionally reduce each damage type
        if total_before_evasion > 0.0 {
            let ratio = damage_after_evasion / total_before_evasion;
            for damage in &mut result.damage_taken {
                let evaded_portion = damage.final_amount * (1.0 - ratio);
                damage.mitigated_amount += evaded_portion;
                damage.final_amount *= ratio;
            }
        }
    }

    // Step 3b: Block check
    let block_chance = new_defender.computed_block_chance() / 100.0;
    if block_chance > 0.0 && rng.gen::<f64>() < block_chance {
        let block_amount = new_defender.computed_block_amount();
        result.was_blocked = true;
        result.damage_blocked = block_amount;

        // Subtract block amount proportionally from each damage type
        let total_pre_block: f64 = result.damage_taken.iter().map(|d| d.final_amount).sum();
        if total_pre_block > 0.0 && block_amount > 0.0 {
            let block_ratio = (block_amount / total_pre_block).min(1.0);
            for damage in &mut result.damage_taken {
                let blocked = damage.final_amount * block_ratio;
                damage.mitigated_amount += blocked;
                damage.final_amount -= blocked;
            }
        }
    }

    // Step 3c: Reduced damage taken (final global multiplier)
    let dr = new_defender.reduced_damage_taken.clamp(0.0, 90.0) / 100.0;
    if dr > 0.0 {
        let total_pre_dr: f64 = result.damage_taken.iter().map(|d| d.final_amount).sum();
        for damage in &mut result.damage_taken {
            let reduced = damage.final_amount * dr;
            damage.mitigated_amount += reduced;
            damage.final_amount -= reduced;
        }
        let total_post_dr: f64 = result.damage_taken.iter().map(|d| d.final_amount).sum();
        result.damage_reduced_by_dr = total_pre_dr - total_post_dr;
    }

    // Calculate final total damage
    result.total_damage = result.damage_taken.iter().map(|d| d.final_amount).sum();

    // Step 4: Apply damage to ES then life
    let mut remaining_damage = result.total_damage;

    // ES absorbs damage first
    if new_defender.current_energy_shield > 0.0 && remaining_damage > 0.0 {
        let es_absorbed = remaining_damage.min(new_defender.current_energy_shield);
        new_defender.current_energy_shield -= es_absorbed;
        remaining_damage -= es_absorbed;
        result.damage_blocked_by_es = es_absorbed;
    }

    // Remaining damage goes to life
    if remaining_damage > 0.0 {
        new_defender.current_life -= remaining_damage;
    }

    // Check for death
    if new_defender.current_life <= 0.0 {
        result.is_killing_blow = true;
        new_defender.current_life = 0.0;
    }

    // Step 4b: Culling strike — if defender is below threshold, kill them
    if !result.is_killing_blow && packet.culling_strike > 0.0 {
        let life_percent = new_defender.life_percent();
        if life_percent > 0.0 && life_percent <= packet.culling_strike {
            result.is_killing_blow = true;
            result.culled = true;
            new_defender.current_life = 0.0;
        }
    }

    // Step 4c: Life/Mana on kill
    if result.is_killing_blow {
        result.life_gained_on_kill = packet.life_on_kill;
        result.mana_gained_on_kill = packet.mana_on_kill;
    }

    // Store final state
    result.es_after = new_defender.current_energy_shield;
    result.life_after = new_defender.current_life;

    // Step 5: Process status effect applications using unified Effect system
    let target_max_health = new_defender.computed_max_life();
    for pending_status in &packet.status_effects_to_apply {
        let config_id = status_to_config_id(pending_status.effect_type);
        let registry = dot_registry();
        let config = registry.get(config_id);

        let should_apply = match config.map(|c| &c.application) {
            Some(crate::dot::StatusApplication::Buildup { threshold }) => {
                // Buildup-based: accumulate status damage until threshold
                let buildup = new_defender
                    .status_buildup
                    .entry(pending_status.effect_type)
                    .or_insert(0.0);
                *buildup += pending_status.status_damage;
                if *buildup >= *threshold {
                    *buildup -= threshold;
                    true
                } else {
                    false
                }
            }
            _ => {
                // Default: chance-based (status_damage / target_max_health)
                let apply_chance = pending_status.calculate_apply_chance(target_max_health);
                rng.gen::<f64>() < apply_chance
            }
        };

        if should_apply {
            // Create unified Effect based on status type
            let effect = create_effect_from_status(
                pending_status.effect_type,
                pending_status.duration,
                pending_status.magnitude,
                pending_status.dot_dps,
                &packet.source_id,
            );

            // Add to unified effects (handles stacking internally)
            new_defender.add_effect(effect.clone());
            result.effects_applied.push(effect);
        }
    }

    (new_defender, result)
}

/// Map StatusEffect enum to config ID
fn status_to_config_id(status: StatusEffect) -> &'static str {
    match status {
        StatusEffect::Poison => "poison",
        StatusEffect::Bleed => "bleed",
        StatusEffect::Burn => "burn",
        StatusEffect::Freeze => "freeze",
        StatusEffect::Chill => "chill",
        StatusEffect::Static => "static",
        StatusEffect::Fear => "fear",
        StatusEffect::Slow => "slow",
    }
}

/// Create an Effect from a pending status effect using config
fn create_effect_from_status(
    status: StatusEffect,
    duration: f64,
    magnitude: f64,
    dot_dps: f64,
    source_id: &str,
) -> Effect {
    let config_id = status_to_config_id(status);
    let registry = dot_registry();

    if let Some(config) = registry.get(config_id) {
        Effect::from_config(config, status, duration, magnitude, dot_dps, source_id)
    } else {
        // Fallback if config not found (shouldn't happen with proper initialization)
        Effect::new_ailment(
            config_id,
            config_id,
            status,
            duration,
            magnitude,
            dot_dps,
            0.5, // default tick rate
            crate::types::AilmentStacking::StrongestOnly,
            source_id,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ensure_constants_initialized, ensure_dot_registry_initialized};
    use crate::damage::FinalDamage;

    fn setup() {
        ensure_constants_initialized();
        ensure_dot_registry_initialized();
    }

    fn make_test_packet(damages: Vec<(DamageType, f64)>) -> DamagePacket {
        let mut packet = DamagePacket::new("attacker".to_string(), "test_skill".to_string());
        for (dtype, amount) in damages {
            packet.damages.push(FinalDamage::new(dtype, amount));
        }
        packet
    }

    #[test]
    fn test_basic_damage() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 100.0;

        let packet = make_test_packet(vec![(DamageType::Physical, 50.0)]);

        let (new_defender, result) = resolve_damage(&defender, &packet);

        // Should take some damage (reduced by armour if any)
        assert!(result.total_damage > 0.0);
        assert!(new_defender.current_life < 100.0);
    }

    #[test]
    fn test_resistance_mitigation() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 100.0;
        defender.fire_resistance.base = 50.0; // 50% fire resist

        let packet = make_test_packet(vec![(DamageType::Fire, 100.0)]);

        let (_, result) = resolve_damage(&defender, &packet);

        // Should take 50 damage after 50% resist
        assert!((result.total_damage - 50.0).abs() < 1.0);
        assert!((result.damage_reduced_by_resists - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_armour_reduction() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 200.0;
        defender.armour.base = 1000.0;

        let packet = make_test_packet(vec![(DamageType::Physical, 100.0)]);

        let (_, result) = resolve_damage(&defender, &packet);

        // Armour should reduce physical damage
        assert!(result.damage_reduced_by_armour > 0.0);
        assert!(result.total_damage < 100.0);
    }

    #[test]
    fn test_evasion_cap() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 10000.0;
        // 1000 evasion vs 2000 accuracy = cap of 1000 (2000 / (1 + 1000/1000) = 1000)
        defender.evasion.base = 1000.0;

        // Hit for 1500 fire damage with 2000 accuracy
        let mut packet = make_test_packet(vec![(DamageType::Fire, 1500.0)]);
        packet.accuracy = 2000.0;

        let (_, result) = resolve_damage(&defender, &packet);

        // Should cap at 1000 (1500 - 1000 = 500 evaded)
        assert!(result.triggered_evasion_cap);
        assert!((result.damage_prevented_by_evasion - 500.0).abs() < 1.0);
        assert!((result.total_damage - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_es_absorbs_first() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 100.0;
        defender.current_energy_shield = 50.0;
        defender.max_energy_shield = 50.0;

        let packet = make_test_packet(vec![(DamageType::Fire, 75.0)]);

        let (new_defender, result) = resolve_damage(&defender, &packet);

        // ES should absorb first 50, life takes remaining 25
        assert!((result.damage_blocked_by_es - 50.0).abs() < 1.0);
        assert!((new_defender.current_energy_shield - 0.0).abs() < 0.1);
        assert!((new_defender.current_life - 75.0).abs() < 1.0);
    }

    #[test]
    fn test_killing_blow() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 50.0;

        let packet = make_test_packet(vec![(DamageType::Fire, 1000.0)]);

        let (new_defender, result) = resolve_damage(&defender, &packet);

        assert!(result.is_killing_blow);
        assert!(!new_defender.is_alive());
        assert!(new_defender.current_life <= 0.0);
    }

    #[test]
    fn test_penetration() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 200.0;
        defender.fire_resistance.base = 75.0;

        let mut packet = make_test_packet(vec![(DamageType::Fire, 100.0)]);
        packet.fire_pen = 25.0; // 25% penetration

        let (_, result) = resolve_damage(&defender, &packet);

        // 75% resist - 25% pen = 50% effective resist
        // 100 * (1 - 0.5) = 50 damage
        assert!((result.total_damage - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_multiple_damage_types() {
        setup();
        let mut defender = StatBlock::new();
        defender.current_life = 200.0;
        defender.fire_resistance.base = 50.0;
        defender.cold_resistance.base = 25.0;

        let packet = make_test_packet(vec![
            (DamageType::Fire, 100.0), // 50 after resist
            (DamageType::Cold, 100.0), // 75 after resist
        ]);

        let (_, result) = resolve_damage(&defender, &packet);

        // Total: 50 + 75 = 125
        assert!((result.total_damage - 125.0).abs() < 1.0);
    }
}
