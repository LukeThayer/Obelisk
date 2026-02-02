//! Core types specific to stat_manager

use loot_core::types::StatusEffect;
use serde::{Deserialize, Serialize};

// ============================================================================
// Unified Effect System
// ============================================================================

/// A unified effect that can represent buffs, debuffs, and ailments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    /// Unique identifier for this effect instance
    pub id: String,
    /// Display name
    pub name: String,
    /// The type of effect (stat modifier or ailment)
    pub effect_type: EffectType,
    /// Time remaining in seconds
    pub duration_remaining: f64,
    /// Total duration (for percentage calculations)
    pub total_duration: f64,
    /// Current stack count
    pub stacks: u32,
    /// Maximum allowed stacks
    pub max_stacks: u32,
    /// Source entity ID that applied this effect
    pub source_id: String,
}

/// The type of effect - either stat modifiers or ailments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectType {
    /// Stat modifier effect (buff or debuff)
    StatModifier {
        /// List of stat modifications
        modifiers: Vec<StatMod>,
        /// Whether this is a debuff (negative effect)
        is_debuff: bool,
    },
    /// Ailment effect (status effect like poison, bleed, etc.)
    Ailment {
        /// The status effect type
        status: StatusEffect,
        /// Effect magnitude (e.g., slow percentage)
        magnitude: f64,
        /// Damage per second for DoT ailments
        dot_dps: f64,
        /// Time between damage ticks
        tick_rate: f64,
        /// Time until next tick
        time_until_tick: f64,
        /// Stacking behavior
        stacking: AilmentStacking,
        /// Effectiveness multiplier (for stacking)
        effectiveness: f64,
    },
}

/// A stat modifier from an effect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatMod {
    /// The stat being modified
    pub stat: loot_core::types::StatType,
    /// Value per stack
    pub value_per_stack: f64,
    /// Whether this is a "more" multiplier
    pub is_more: bool,
}

/// How ailments stack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AilmentStacking {
    /// Only the strongest instance applies
    StrongestOnly,
    /// Unlimited stacking
    Unlimited,
    /// Limited stacking with effectiveness reduction
    Limited {
        /// Effectiveness of stacked instances
        stack_effectiveness: f64,
    },
}

impl Default for AilmentStacking {
    fn default() -> Self {
        AilmentStacking::StrongestOnly
    }
}

/// Result of processing effect ticks
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    /// Total DoT damage dealt this tick
    pub dot_damage: f64,
    /// IDs of effects that expired
    pub expired_effects: Vec<String>,
    /// Whether any stat modifier effects expired (requiring stat rebuild)
    pub stat_effects_expired: bool,
    /// Life remaining after DoT damage
    pub life_remaining: f64,
    /// Whether the entity died from DoT damage
    pub is_dead: bool,
}

impl Effect {
    /// Create a new stat modifier effect (buff or debuff)
    pub fn new_stat_modifier(
        id: impl Into<String>,
        name: impl Into<String>,
        duration: f64,
        is_debuff: bool,
        modifiers: Vec<StatMod>,
        source_id: impl Into<String>,
    ) -> Self {
        Effect {
            id: id.into(),
            name: name.into(),
            effect_type: EffectType::StatModifier { modifiers, is_debuff },
            duration_remaining: duration,
            total_duration: duration,
            stacks: 1,
            max_stacks: 1,
            source_id: source_id.into(),
        }
    }

    /// Create a new ailment effect
    pub fn new_ailment(
        id: impl Into<String>,
        name: impl Into<String>,
        status: StatusEffect,
        duration: f64,
        magnitude: f64,
        dot_dps: f64,
        tick_rate: f64,
        stacking: AilmentStacking,
        source_id: impl Into<String>,
    ) -> Self {
        Effect {
            id: id.into(),
            name: name.into(),
            effect_type: EffectType::Ailment {
                status,
                magnitude,
                dot_dps,
                tick_rate,
                time_until_tick: tick_rate,
                stacking,
                effectiveness: 1.0,
            },
            duration_remaining: duration,
            total_duration: duration,
            stacks: 1,
            max_stacks: 999,
            source_id: source_id.into(),
        }
    }

    // === Ailment Constructors ===

    /// Create a Poison effect (Chaos DoT, unlimited stacking)
    /// - Duration: 2.0s
    /// - Tick rate: 0.33s
    /// - Stacking: Unlimited
    pub fn poison(dot_dps: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "poison", "Poison", StatusEffect::Poison,
            2.0, 0.0, dot_dps, 0.33,
            AilmentStacking::Unlimited,
            source_id,
        )
    }

    /// Create a Bleed effect (Physical DoT, limited stacking)
    /// - Duration: 5.0s
    /// - Tick rate: 1.0s
    /// - Stacking: Limited (50% effectiveness for additional stacks)
    /// - Note: Deals 2x damage while target is moving (handled externally)
    pub fn bleed(dot_dps: f64, source_id: impl Into<String>) -> Self {
        let mut effect = Self::new_ailment(
            "bleed", "Bleed", StatusEffect::Bleed,
            5.0, 0.0, dot_dps, 1.0,
            AilmentStacking::Limited { stack_effectiveness: 0.5 },
            source_id,
        );
        effect.max_stacks = 8;
        effect
    }

    /// Create a Burn effect (Fire DoT, strongest only)
    /// - Duration: 4.0s
    /// - Tick rate: 0.5s
    /// - Stacking: Strongest only
    pub fn burn(dot_dps: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "burn", "Burn", StatusEffect::Burn,
            4.0, 0.0, dot_dps, 0.5,
            AilmentStacking::StrongestOnly,
            source_id,
        )
    }

    /// Create a Freeze effect (Cold, no damage, strongest only)
    /// - Duration: 0.5s (short immobilization)
    /// - Stacking: Strongest only
    pub fn freeze(magnitude: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "freeze", "Freeze", StatusEffect::Freeze,
            0.5, magnitude, 0.0, 0.1,
            AilmentStacking::StrongestOnly,
            source_id,
        )
    }

    /// Create a Chill effect (Cold, no damage, strongest only)
    /// - Duration: 2.0s
    /// - Magnitude: slow percentage
    /// - Stacking: Strongest only
    pub fn chill(magnitude: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "chill", "Chill", StatusEffect::Chill,
            2.0, magnitude, 0.0, 0.5,
            AilmentStacking::StrongestOnly,
            source_id,
        )
    }

    /// Create a Static/Shock effect (Lightning, no damage, limited stacking)
    /// - Duration: 1.0s
    /// - Magnitude: increased damage taken
    /// - Stacking: Limited (up to 3 stacks)
    pub fn shock(magnitude: f64, source_id: impl Into<String>) -> Self {
        let mut effect = Self::new_ailment(
            "static", "Static", StatusEffect::Static,
            1.0, magnitude, 0.0, 0.25,
            AilmentStacking::Limited { stack_effectiveness: 1.0 },
            source_id,
        );
        effect.max_stacks = 3;
        effect
    }

    /// Create a Fear effect (Chaos, no damage, strongest only)
    /// - Duration: 1.5s
    /// - Stacking: Strongest only
    pub fn fear(magnitude: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "fear", "Fear", StatusEffect::Fear,
            1.5, magnitude, 0.0, 0.5,
            AilmentStacking::StrongestOnly,
            source_id,
        )
    }

    /// Create a Slow effect (Physical/Cold, no damage, strongest only)
    /// - Duration: 3.0s
    /// - Magnitude: slow percentage
    /// - Stacking: Strongest only
    pub fn slow(magnitude: f64, source_id: impl Into<String>) -> Self {
        Self::new_ailment(
            "slow", "Slow", StatusEffect::Slow,
            3.0, magnitude, 0.0, 0.5,
            AilmentStacking::StrongestOnly,
            source_id,
        )
    }

    /// Get the base duration for a status effect type
    pub fn base_duration_for(status: StatusEffect) -> f64 {
        match status {
            StatusEffect::Poison => 2.0,
            StatusEffect::Bleed => 5.0,
            StatusEffect::Burn => 4.0,
            StatusEffect::Freeze => 0.5,
            StatusEffect::Chill => 2.0,
            StatusEffect::Static => 1.0,
            StatusEffect::Fear => 1.5,
            StatusEffect::Slow => 3.0,
        }
    }

    /// Get the base DoT damage percentage for a status effect type
    /// Returns the percentage of status damage that becomes DPS
    pub fn base_dot_percent_for(status: StatusEffect) -> f64 {
        match status {
            StatusEffect::Poison => 0.20,  // 20%
            StatusEffect::Bleed => 0.20,   // 20%
            StatusEffect::Burn => 0.25,    // 25%
            _ => 0.0,  // Non-damaging
        }
    }

    /// Check if the effect is still active
    pub fn is_active(&self) -> bool {
        self.duration_remaining > 0.0 && self.stacks > 0
    }

    /// Check if this is a stat modifier effect
    pub fn is_stat_modifier(&self) -> bool {
        matches!(self.effect_type, EffectType::StatModifier { .. })
    }

    /// Check if this is an ailment effect
    pub fn is_ailment(&self) -> bool {
        matches!(self.effect_type, EffectType::Ailment { .. })
    }

    /// Check if this ailment deals DoT damage
    pub fn is_damaging(&self) -> bool {
        match &self.effect_type {
            EffectType::Ailment { dot_dps, .. } => *dot_dps > 0.0,
            _ => false,
        }
    }

    /// Get the status effect type if this is an ailment
    pub fn status(&self) -> Option<StatusEffect> {
        match &self.effect_type {
            EffectType::Ailment { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Get DPS for this effect (0 if not a damaging ailment)
    pub fn dps(&self) -> f64 {
        match &self.effect_type {
            EffectType::Ailment { dot_dps, effectiveness, .. } => {
                dot_dps * self.stacks as f64 * effectiveness
            }
            _ => 0.0,
        }
    }

    /// Calculate damage for a tick (returns 0 if not a damaging ailment)
    pub fn tick_damage(&self, delta: f64) -> f64 {
        match &self.effect_type {
            EffectType::Ailment { dot_dps, effectiveness, .. } => {
                dot_dps * delta * self.stacks as f64 * effectiveness
            }
            _ => 0.0,
        }
    }

    /// Get percentage of duration remaining
    pub fn duration_percent(&self) -> f64 {
        if self.total_duration <= 0.0 {
            return 0.0;
        }
        (self.duration_remaining / self.total_duration * 100.0).clamp(0.0, 100.0)
    }

    /// Add a stack (capped at max_stacks)
    pub fn add_stack(&mut self) {
        if self.stacks < self.max_stacks {
            self.stacks += 1;
        }
    }

    /// Refresh duration and optionally update values
    pub fn refresh(&mut self, new_duration: f64) {
        self.duration_remaining = new_duration;
        self.total_duration = new_duration;
    }

    /// Tick the effect by delta time, returning damage dealt (for ailments)
    /// Returns the damage dealt this tick
    pub fn tick(&mut self, delta: f64) -> f64 {
        let mut damage_dealt = 0.0;

        match &mut self.effect_type {
            EffectType::Ailment { time_until_tick, tick_rate, dot_dps, effectiveness, .. } => {
                if *dot_dps > 0.0 {
                    *time_until_tick -= delta;
                    while *time_until_tick <= 0.0 && self.duration_remaining > 0.0 {
                        damage_dealt += *dot_dps * *tick_rate * self.stacks as f64 * *effectiveness;
                        *time_until_tick += *tick_rate;
                    }
                }
            }
            _ => {}
        }

        self.duration_remaining -= delta;
        damage_dealt
    }
}

/// Equipment slot for gear
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Helmet,
    BodyArmour,
    Gloves,
    Boots,
    Ring1,
    Ring2,
    Amulet,
    Belt,
}

impl EquipmentSlot {
    /// Get all equipment slots
    pub fn all() -> &'static [EquipmentSlot] {
        &[
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Helmet,
            EquipmentSlot::BodyArmour,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Ring1,
            EquipmentSlot::Ring2,
            EquipmentSlot::Amulet,
            EquipmentSlot::Belt,
        ]
    }
}

/// Skill tags for damage scaling and categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillTag {
    // Damage source
    Attack,
    Spell,
    // Damage types
    Physical,
    Fire,
    Cold,
    Lightning,
    Chaos,
    Elemental,
    // Delivery
    Melee,
    Ranged,
    Projectile,
    // Area
    Aoe,
}

/// Identifier for a skill tree node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillNodeId(pub String);

impl From<&str> for SkillNodeId {
    fn from(s: &str) -> Self {
        SkillNodeId(s.to_string())
    }
}

impl From<String> for SkillNodeId {
    fn from(s: String) -> Self {
        SkillNodeId(s)
    }
}

/// Active buff/debuff on an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveBuff {
    /// Buff identifier
    pub buff_id: String,
    /// Display name
    pub name: String,
    /// Time remaining in seconds
    pub duration_remaining: f64,
    /// Current stack count
    pub stacks: u32,
    /// Whether this is a debuff (negative effect)
    pub is_debuff: bool,
}

/// Active status effect on an entity (freeze, chill, burn, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatusEffect {
    /// The type of status effect
    pub effect_type: loot_core::types::StatusEffect,
    /// Time remaining in seconds
    pub duration_remaining: f64,
    /// Current stack count
    pub stacks: u32,
    /// Effect magnitude (e.g., slow percentage)
    pub magnitude: f64,
    /// Damage per second for damaging statuses (Poison, Bleed, Burn)
    pub dot_dps: f64,
    /// Source entity ID that applied this effect
    pub source_id: String,
}

impl ActiveStatusEffect {
    pub fn new(
        effect_type: loot_core::types::StatusEffect,
        duration: f64,
        magnitude: f64,
        source_id: String,
    ) -> Self {
        ActiveStatusEffect {
            effect_type,
            duration_remaining: duration,
            stacks: 1,
            magnitude,
            dot_dps: 0.0,
            source_id,
        }
    }

    /// Create a new status effect with DoT damage
    pub fn new_with_dot(
        effect_type: loot_core::types::StatusEffect,
        duration: f64,
        magnitude: f64,
        dot_dps: f64,
        source_id: String,
    ) -> Self {
        ActiveStatusEffect {
            effect_type,
            duration_remaining: duration,
            stacks: 1,
            magnitude,
            dot_dps,
            source_id,
        }
    }

    /// Check if the effect is still active
    pub fn is_active(&self) -> bool {
        self.duration_remaining > 0.0 && self.stacks > 0
    }

    /// Check if this is a damaging status effect
    pub fn is_damaging(&self) -> bool {
        use loot_core::types::StatusEffect;
        matches!(
            self.effect_type,
            StatusEffect::Poison | StatusEffect::Bleed | StatusEffect::Burn
        )
    }

    /// Get damage for a tick (damage = dot_dps * delta * stacks)
    pub fn tick_damage(&self, delta: f64) -> f64 {
        self.dot_dps * delta * self.stacks as f64
    }

    /// Tick the effect duration, returns damage dealt this tick
    pub fn tick(&mut self, delta: f64) -> f64 {
        let damage = if self.is_damaging() {
            self.tick_damage(delta)
        } else {
            0.0
        };
        self.duration_remaining -= delta;
        damage
    }

    /// Add a stack (also increases DoT damage proportionally)
    pub fn add_stack(&mut self, max_stacks: u32) {
        if self.stacks < max_stacks {
            self.stacks += 1;
        }
    }

    /// Refresh duration and update dot_dps if new is higher
    pub fn refresh(&mut self, duration: f64, dot_dps: f64) {
        self.duration_remaining = duration;
        if dot_dps > self.dot_dps {
            self.dot_dps = dot_dps;
        }
    }
}
