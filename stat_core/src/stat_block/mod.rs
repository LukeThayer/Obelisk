//! StatBlock - Aggregated character stats from all sources

mod aggregator;
mod computed;
mod stat_value;

pub use aggregator::{StatAccumulator, StatusConversions, StatusEffectStats};
pub use stat_value::StatValue;

use crate::combat::CombatResult;
use crate::damage::{calculate_damage, DamagePacket, DamagePacketGenerator};
use crate::dot::ActiveDoT;
use crate::combat::resolve_damage;
use crate::source::{BuffSource, GearSource, StatSource};
use crate::types::{ActiveBuff, ActiveStatusEffect, AilmentStacking, Effect, EffectType, EquipmentSlot, TickResult};
use loot_core::types::{DamageType, StatusEffect};
use loot_core::Item;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete stat state for an entity (player, monster, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatBlock {
    // === Identity ===
    /// Unique identifier for this entity
    pub id: String,

    // === Equipment ===
    /// Equipped items by slot
    #[serde(default)]
    equipped_items: HashMap<EquipmentSlot, Item>,

    // === Buff Sources ===
    /// Active buff sources for stat calculation
    #[serde(skip)]
    buff_sources: Vec<BuffSource>,

    // === Resources ===
    pub max_life: StatValue,
    pub current_life: f64,
    pub max_mana: StatValue,
    pub current_mana: f64,
    /// Maximum energy shield from warding spells (does NOT passively regenerate)
    pub max_energy_shield: f64,
    pub current_energy_shield: f64,

    // === Attributes ===
    pub strength: StatValue,
    pub dexterity: StatValue,
    pub intelligence: StatValue,
    pub constitution: StatValue,
    pub wisdom: StatValue,
    pub charisma: StatValue,

    // === Defenses ===
    pub armour: StatValue,
    /// Evasion also serves as one-shot protection threshold
    pub evasion: StatValue,
    pub fire_resistance: StatValue,
    pub cold_resistance: StatValue,
    pub lightning_resistance: StatValue,
    pub chaos_resistance: StatValue,

    // === Offense (Global) ===
    /// Accuracy rating - determines damage cap against evasion
    pub accuracy: StatValue,
    pub global_physical_damage: StatValue,
    pub global_fire_damage: StatValue,
    pub global_cold_damage: StatValue,
    pub global_lightning_damage: StatValue,
    pub global_chaos_damage: StatValue,
    pub attack_speed: StatValue,
    pub cast_speed: StatValue,
    pub critical_chance: StatValue,
    pub critical_multiplier: StatValue,

    // === Penetration ===
    pub fire_penetration: StatValue,
    pub cold_penetration: StatValue,
    pub lightning_penetration: StatValue,
    pub chaos_penetration: StatValue,

    // === Recovery ===
    pub life_regen: StatValue,
    pub mana_regen: StatValue,
    pub life_leech: StatValue,
    pub mana_leech: StatValue,

    // === Utility ===
    pub movement_speed_increased: f64,
    pub item_rarity_increased: f64,
    pub item_quantity_increased: f64,

    // === Active Effects (Unified) ===
    /// Unified effects list (replaces active_dots, active_buffs, active_status_effects)
    #[serde(default)]
    pub effects: Vec<Effect>,

    // === Legacy Active Effects (kept for backwards compatibility during transition) ===
    #[serde(default)]
    pub active_dots: Vec<ActiveDoT>,
    #[serde(default)]
    pub active_buffs: Vec<ActiveBuff>,
    #[serde(default)]
    pub active_status_effects: Vec<ActiveStatusEffect>,

    // === Weapon Stats (from equipped weapon) ===
    pub weapon_physical_min: f64,
    pub weapon_physical_max: f64,
    pub weapon_fire_min: f64,
    pub weapon_fire_max: f64,
    pub weapon_cold_min: f64,
    pub weapon_cold_max: f64,
    pub weapon_lightning_min: f64,
    pub weapon_lightning_max: f64,
    pub weapon_chaos_min: f64,
    pub weapon_chaos_max: f64,
    pub weapon_attack_speed: f64,
    pub weapon_crit_chance: f64,

    // === Status Effect Stats ===
    /// Stats for each status effect type (conversions, duration, magnitude, etc.)
    #[serde(default)]
    pub status_effect_stats: StatusEffectData,
}

/// Holds all status effect related stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusEffectData {
    // Poison
    pub poison: StatusEffectStats,
    pub poison_conversions: StatusConversions,
    // Bleed
    pub bleed: StatusEffectStats,
    pub bleed_conversions: StatusConversions,
    // Burn
    pub burn: StatusEffectStats,
    pub burn_conversions: StatusConversions,
    // Freeze
    pub freeze: StatusEffectStats,
    pub freeze_conversions: StatusConversions,
    // Chill
    pub chill: StatusEffectStats,
    pub chill_conversions: StatusConversions,
    // Static
    pub static_effect: StatusEffectStats,
    pub static_conversions: StatusConversions,
    // Fear
    pub fear: StatusEffectStats,
    pub fear_conversions: StatusConversions,
    // Slow
    pub slow: StatusEffectStats,
    pub slow_conversions: StatusConversions,
}

impl StatusEffectData {
    /// Get stats for a given status effect
    pub fn get_stats(&self, effect: StatusEffect) -> &StatusEffectStats {
        match effect {
            StatusEffect::Poison => &self.poison,
            StatusEffect::Bleed => &self.bleed,
            StatusEffect::Burn => &self.burn,
            StatusEffect::Freeze => &self.freeze,
            StatusEffect::Chill => &self.chill,
            StatusEffect::Static => &self.static_effect,
            StatusEffect::Fear => &self.fear,
            StatusEffect::Slow => &self.slow,
        }
    }

    /// Get conversions for a given status effect
    pub fn get_conversions(&self, effect: StatusEffect) -> &StatusConversions {
        match effect {
            StatusEffect::Poison => &self.poison_conversions,
            StatusEffect::Bleed => &self.bleed_conversions,
            StatusEffect::Burn => &self.burn_conversions,
            StatusEffect::Freeze => &self.freeze_conversions,
            StatusEffect::Chill => &self.chill_conversions,
            StatusEffect::Static => &self.static_conversions,
            StatusEffect::Fear => &self.fear_conversions,
            StatusEffect::Slow => &self.slow_conversions,
        }
    }

    /// Calculate status damage for a given effect based on hit damages
    /// Returns the total status damage that would be converted from hit damage
    pub fn calculate_status_damage(
        &self,
        effect: StatusEffect,
        damages: &[(DamageType, f64)],
    ) -> f64 {
        let conversions = self.get_conversions(effect);
        let stats = self.get_stats(effect);

        let mut status_damage = 0.0;
        for (damage_type, amount) in damages {
            let conversion = conversions.from_damage_type(*damage_type);
            status_damage += amount * conversion;
        }

        // Apply magnitude modifier
        status_damage * (1.0 + stats.magnitude)
    }

    /// Calculate chance to apply status effect based on status damage vs target max health
    /// Returns a value between 0.0 and 1.0
    pub fn calculate_apply_chance(
        &self,
        effect: StatusEffect,
        damages: &[(DamageType, f64)],
        target_max_health: f64,
    ) -> f64 {
        if target_max_health <= 0.0 {
            return 0.0;
        }

        let status_damage = self.calculate_status_damage(effect, damages);
        (status_damage / target_max_health).clamp(0.0, 1.0)
    }
}

impl Default for StatBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl StatBlock {
    /// Create a new empty StatBlock with base values and default ID
    pub fn new() -> Self {
        Self::with_id("entity")
    }

    /// Create a new StatBlock with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        StatBlock {
            // Identity
            id: id.into(),

            // Equipment
            equipped_items: HashMap::new(),

            // Buff sources
            buff_sources: Vec::new(),

            // Resources
            max_life: StatValue::with_base(50.0),
            current_life: 50.0,
            max_mana: StatValue::with_base(40.0),
            current_mana: 40.0,
            max_energy_shield: 0.0,
            current_energy_shield: 0.0,

            // Attributes
            strength: StatValue::with_base(10.0),
            dexterity: StatValue::with_base(10.0),
            intelligence: StatValue::with_base(10.0),
            constitution: StatValue::with_base(10.0),
            wisdom: StatValue::with_base(10.0),
            charisma: StatValue::with_base(10.0),

            // Defenses
            armour: StatValue::default(),
            evasion: StatValue::default(),
            fire_resistance: StatValue::default(),
            cold_resistance: StatValue::default(),
            lightning_resistance: StatValue::default(),
            chaos_resistance: StatValue::default(),

            // Offense
            accuracy: StatValue::with_base(1000.0), // Base accuracy
            global_physical_damage: StatValue::default(),
            global_fire_damage: StatValue::default(),
            global_cold_damage: StatValue::default(),
            global_lightning_damage: StatValue::default(),
            global_chaos_damage: StatValue::default(),
            attack_speed: StatValue::with_base(1.0),
            cast_speed: StatValue::with_base(1.0),
            critical_chance: StatValue::default(),
            critical_multiplier: StatValue::with_base(1.5), // 150% base crit multiplier

            // Penetration
            fire_penetration: StatValue::default(),
            cold_penetration: StatValue::default(),
            lightning_penetration: StatValue::default(),
            chaos_penetration: StatValue::default(),

            // Recovery
            life_regen: StatValue::default(),
            mana_regen: StatValue::default(),
            life_leech: StatValue::default(),
            mana_leech: StatValue::default(),

            // Utility
            movement_speed_increased: 0.0,
            item_rarity_increased: 0.0,
            item_quantity_increased: 0.0,

            // Active effects (unified)
            effects: Vec::new(),

            // Legacy active effects
            active_dots: Vec::new(),
            active_buffs: Vec::new(),
            active_status_effects: Vec::new(),

            // Weapon stats
            weapon_physical_min: 0.0,
            weapon_physical_max: 0.0,
            weapon_fire_min: 0.0,
            weapon_fire_max: 0.0,
            weapon_cold_min: 0.0,
            weapon_cold_max: 0.0,
            weapon_lightning_min: 0.0,
            weapon_lightning_max: 0.0,
            weapon_chaos_min: 0.0,
            weapon_chaos_max: 0.0,
            weapon_attack_speed: 1.0,
            weapon_crit_chance: 5.0,

            // Status effect stats
            status_effect_stats: StatusEffectData::default(),
        }
    }

    /// Rebuild stats from all sources (external API for custom sources)
    pub fn rebuild_from_sources(&mut self, sources: &[Box<dyn StatSource>]) {
        // Preserve identity and equipment
        let id = std::mem::take(&mut self.id);
        let equipped_items = std::mem::take(&mut self.equipped_items);
        let buff_sources = std::mem::take(&mut self.buff_sources);

        // Reset to base values
        *self = StatBlock::with_id(id);
        self.equipped_items = equipped_items;
        self.buff_sources = buff_sources;

        // Create accumulator and apply all sources
        let mut accumulator = StatAccumulator::new();

        // Sort sources by priority
        let mut sorted_sources: Vec<_> = sources.iter().collect();
        sorted_sources.sort_by_key(|s| s.priority());

        for source in sorted_sources {
            source.apply(&mut accumulator);
        }

        // Apply accumulated stats to self
        accumulator.apply_to(self);

        // Update current values to max if they exceed
        self.current_life = self.current_life.min(self.max_life.compute());
        self.current_mana = self.current_mana.min(self.max_mana.compute());
        self.current_energy_shield = self.current_energy_shield.min(self.max_energy_shield);
    }

    /// Rebuild stats from internal equipment and buffs
    fn rebuild(&mut self) {
        // Preserve identity and internal state
        let id = std::mem::take(&mut self.id);
        let equipped_items = std::mem::take(&mut self.equipped_items);
        let buff_sources = std::mem::take(&mut self.buff_sources);

        // Reset to base values
        *self = StatBlock::with_id(id);
        self.equipped_items = equipped_items;
        self.buff_sources = buff_sources;

        // Create accumulator
        let mut accumulator = StatAccumulator::new();

        // Apply gear sources
        for (slot, item) in &self.equipped_items {
            let gear_source = GearSource::new(*slot, item.clone());
            gear_source.apply(&mut accumulator);
        }

        // Apply buff sources
        for buff in &self.buff_sources {
            buff.apply(&mut accumulator);
        }

        // Apply accumulated stats to self
        accumulator.apply_to(self);

        // Update current values to max if they exceed
        self.current_life = self.current_life.min(self.max_life.compute());
        self.current_mana = self.current_mana.min(self.max_mana.compute());
        self.current_energy_shield = self.current_energy_shield.min(self.max_energy_shield);
    }

    /// Check if the entity is alive
    pub fn is_alive(&self) -> bool {
        self.current_life > 0.0
    }

    /// Get computed max life
    pub fn computed_max_life(&self) -> f64 {
        self.max_life.compute()
    }

    /// Get computed max mana
    pub fn computed_max_mana(&self) -> f64 {
        self.max_mana.compute()
    }

    /// Heal life by amount, capped at max
    pub fn heal(&mut self, amount: f64) {
        let max = self.computed_max_life();
        self.current_life = (self.current_life + amount).min(max);
    }

    /// Restore mana by amount, capped at max
    pub fn restore_mana(&mut self, amount: f64) {
        let max = self.computed_max_mana();
        self.current_mana = (self.current_mana + amount).min(max);
    }

    /// Apply energy shield (from warding spells)
    pub fn apply_energy_shield(&mut self, amount: f64) {
        self.current_energy_shield = (self.current_energy_shield + amount).min(self.max_energy_shield);
    }

    /// Set maximum energy shield capacity
    pub fn set_max_energy_shield(&mut self, amount: f64) {
        self.max_energy_shield = amount;
        self.current_energy_shield = self.current_energy_shield.min(amount);
    }

    // === Equipment Methods ===

    /// Equip an item to a slot, automatically rebuilding stats
    pub fn equip(&mut self, slot: EquipmentSlot, item: Item) {
        self.equipped_items.insert(slot, item);
        self.rebuild();
    }

    /// Unequip an item from a slot, returning it if present
    pub fn unequip(&mut self, slot: EquipmentSlot) -> Option<Item> {
        let item = self.equipped_items.remove(&slot);
        if item.is_some() {
            self.rebuild();
        }
        item
    }

    /// Get a reference to the item equipped in a slot
    pub fn equipped(&self, slot: EquipmentSlot) -> Option<&Item> {
        self.equipped_items.get(&slot)
    }

    /// Get all equipped items
    pub fn all_equipped(&self) -> impl Iterator<Item = (&EquipmentSlot, &Item)> {
        self.equipped_items.iter()
    }

    // === Buff Methods ===

    /// Apply a buff, automatically rebuilding stats
    pub fn apply_buff(&mut self, buff: BuffSource) {
        // Check if buff already exists and refresh/stack instead
        if let Some(existing) = self.buff_sources.iter_mut().find(|b| b.buff_id == buff.buff_id) {
            existing.refresh(buff.duration_remaining);
            existing.add_stack();
        } else {
            self.buff_sources.push(buff);
        }
        self.rebuild();
    }

    /// Remove a buff by ID
    pub fn remove_buff(&mut self, buff_id: &str) {
        let had_buff = self.buff_sources.iter().any(|b| b.buff_id == buff_id);
        self.buff_sources.retain(|b| b.buff_id != buff_id);
        if had_buff {
            self.rebuild();
        }
    }

    /// Tick all buffs by delta time, removing expired ones
    pub fn tick_buffs(&mut self, delta: f64) {
        let count_before = self.buff_sources.len();
        self.buff_sources.retain_mut(|buff| buff.tick(delta));
        if self.buff_sources.len() != count_before {
            self.rebuild();
        }
    }

    /// Get all active buffs
    pub fn active_buff_sources(&self) -> &[BuffSource] {
        &self.buff_sources
    }

    // === Combat Methods ===

    /// Generate a damage packet for a skill attack (RNG handled internally)
    pub fn attack(&self, skill: &DamagePacketGenerator) -> DamagePacket {
        let mut rng = rand::thread_rng();
        calculate_damage(self, skill, self.id.clone(), &mut rng)
    }

    /// Receive damage from a damage packet (immutable API)
    /// Returns new state and combat result
    pub fn receive_damage(&self, packet: &DamagePacket) -> (StatBlock, CombatResult) {
        resolve_damage(self, packet)
    }

    // === Unified Effect System Methods ===

    /// Add an effect to this entity (immutable pattern)
    pub fn with_effect(&self, effect: Effect) -> StatBlock {
        let mut new_block = self.clone();
        new_block.add_effect(effect);
        new_block
    }

    /// Add an effect to this entity (mutable)
    pub fn add_effect(&mut self, effect: Effect) {
        // Handle stacking logic for ailments
        if let EffectType::Ailment { status, stacking, .. } = &effect.effect_type {
            let existing = self.effects.iter_mut().find(|e| {
                if let EffectType::Ailment { status: existing_status, .. } = &e.effect_type {
                    existing_status == status
                } else {
                    false
                }
            });

            if let Some(existing_effect) = existing {
                match stacking {
                    AilmentStacking::StrongestOnly => {
                        // Only keep if new is stronger, otherwise just refresh
                        if effect.dps() >= existing_effect.dps() {
                            existing_effect.refresh(effect.duration_remaining);
                            // Update dps if higher
                            if let EffectType::Ailment { dot_dps: existing_dps, .. } = &mut existing_effect.effect_type {
                                if let EffectType::Ailment { dot_dps: new_dps, .. } = &effect.effect_type {
                                    if *new_dps > *existing_dps {
                                        *existing_dps = *new_dps;
                                    }
                                }
                            }
                        }
                        return; // Don't add new effect
                    }
                    AilmentStacking::Limited { .. } => {
                        // Add stack to existing, refresh duration
                        existing_effect.add_stack();
                        existing_effect.refresh(effect.duration_remaining);
                        return; // Don't add new effect
                    }
                    AilmentStacking::Unlimited => {
                        // Just add as new effect (fall through)
                    }
                }
            }
        }

        // Check for stat modifier with same ID
        if let EffectType::StatModifier { .. } = &effect.effect_type {
            let existing = self.effects.iter_mut().find(|e| e.id == effect.id);
            if let Some(existing_effect) = existing {
                existing_effect.add_stack();
                existing_effect.refresh(effect.duration_remaining);
                return;
            }
        }

        self.effects.push(effect);
    }

    /// Tick all effects by delta time (immutable pattern)
    /// Returns a new StatBlock and the tick result
    pub fn tick_effects(&self, delta: f64) -> (StatBlock, TickResult) {
        let mut new_block = self.clone();
        let mut result = TickResult::default();

        // Process all effects
        for effect in &mut new_block.effects {
            let damage = effect.tick(delta);
            if damage > 0.0 {
                result.dot_damage += damage;
            }
        }

        // Apply DoT damage
        if result.dot_damage > 0.0 {
            new_block.current_life -= result.dot_damage;
            if new_block.current_life <= 0.0 {
                new_block.current_life = 0.0;
                result.is_dead = true;
            }
        }
        result.life_remaining = new_block.current_life;

        // Collect expired effects
        for effect in &new_block.effects {
            if !effect.is_active() {
                result.expired_effects.push(effect.id.clone());
                if effect.is_stat_modifier() {
                    result.stat_effects_expired = true;
                }
            }
        }

        // Remove expired effects
        new_block.effects.retain(|e| e.is_active());

        // Rebuild stats if stat modifiers expired
        if result.stat_effects_expired {
            new_block.rebuild_from_effects();
        }

        (new_block, result)
    }

    /// Rebuild stats considering effects
    fn rebuild_from_effects(&mut self) {
        // For now, just call the standard rebuild
        // Stat modifier effects would be applied during stat accumulation
        self.rebuild();
    }

    /// Get all active effects
    pub fn active_effects(&self) -> &[Effect] {
        &self.effects
    }

    /// Get effects of a specific status type
    pub fn effects_of_status(&self, status: StatusEffect) -> Vec<&Effect> {
        self.effects.iter().filter(|e| e.status() == Some(status)).collect()
    }

    /// Get total DPS from all damaging effects
    pub fn total_effect_dps(&self) -> f64 {
        self.effects.iter().map(|e| e.dps()).sum()
    }

    /// Clear all effects
    pub fn clear_effects(&mut self) {
        self.effects.clear();
    }
}
