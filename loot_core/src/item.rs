use crate::config::{AffixConfig, AffixTierConfig, BaseTypeConfig};
use crate::storage::Operation;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A fully realized item with all stats computed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    // === Storage fields (for serialization) ===
    /// RNG seed used to generate this item
    pub seed: u64,
    /// Operations applied to this item (for deterministic reconstruction)
    pub operations: Vec<Operation>,

    // === Computed fields ===
    /// Reference to the base type ID
    pub base_type_id: String,
    /// Display name (for rares, this is the generated name)
    pub name: String,
    /// Base type display name
    pub base_name: String,
    /// Item class
    pub class: ItemClass,
    /// Current rarity (rarity ID)
    pub rarity: String,
    /// Tags inherited from base type
    pub tags: Vec<Tag>,
    /// Requirements to equip
    pub requirements: Requirements,
    /// Implicit modifier (if any)
    pub implicit: Option<Modifier>,
    /// Rolled prefix modifiers
    pub prefixes: Vec<Modifier>,
    /// Rolled suffix modifiers
    pub suffixes: Vec<Modifier>,
    /// Base defenses (for armour)
    pub defenses: Defenses,
    /// Base damage (for weapons)
    pub damage: Option<WeaponDamage>,
    /// Skill IDs granted by this item's base type
    #[serde(default)]
    pub granted_skills: Vec<String>,
}

impl Item {
    /// Create a new normal (white) item from a base type with a seed
    pub(crate) fn new_normal(base: &BaseTypeConfig, seed: u64) -> Self {
        let defenses = if let Some(ref def) = base.defenses {
            Defenses {
                armour: def.armour.map(|r| r.min), // Will be rolled properly with seed
                evasion: def.evasion.map(|r| r.min),
                energy_shield: def.energy_shield.map(|r| r.min),
            }
        } else {
            Defenses::default()
        };

        let damage = base.damage.as_ref().map(|d| WeaponDamage {
            damages: d
                .damages
                .iter()
                .map(|e| DamageValue {
                    damage_type: e.damage_type,
                    min: e.min,
                    max: e.max,
                })
                .collect(),
            attack_speed: d.attack_speed,
            critical_chance: d.critical_chance,
            spell_efficiency: d.spell_efficiency,
        });

        Item {
            seed,
            operations: Vec::new(),
            base_type_id: base.id.clone(),
            name: base.name.clone(),
            base_name: base.name.clone(),
            class: base.class,
            rarity: "normal".to_string(),
            tags: base.tags.clone(),
            requirements: base.requirements.clone(),
            implicit: None, // Will be rolled with seed
            prefixes: Vec::new(),
            suffixes: Vec::new(),
            defenses,
            damage,
            granted_skills: base.granted_skills.clone(),
        }
    }

    /// Record that a currency was applied to this item
    pub(crate) fn record_currency(&mut self, currency_id: impl Into<String>) {
        self.operations
            .push(Operation::Currency(currency_id.into()));
    }

    /// Count total affixes
    pub fn affix_count(&self) -> usize {
        self.prefixes.len() + self.suffixes.len()
    }

    /// Get all skill IDs granted by this item (base type + affixes)
    pub fn all_skills(&self) -> Vec<&str> {
        let mut skills: Vec<&str> = self.granted_skills.iter().map(|s| s.as_str()).collect();
        for modifier in self.prefixes.iter().chain(self.suffixes.iter()) {
            for skill in &modifier.granted_skills {
                skills.push(skill.as_str());
            }
        }
        if let Some(ref imp) = self.implicit {
            for skill in &imp.granted_skills {
                skills.push(skill.as_str());
            }
        }
        skills
    }

    /// Export item to markdown format
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header with name
        md.push_str(&format!("## {}\n", self.name));
        md.push_str(&format!("**{}** ({})\n\n", self.base_name, self.rarity));

        // Defenses (for armour)
        if self.defenses.has_any() {
            md.push_str("### Defenses\n");
            if let Some(armour) = self.defenses.armour {
                md.push_str(&format!("- Armour: {}\n", armour));
            }
            if let Some(evasion) = self.defenses.evasion {
                md.push_str(&format!("- Evasion: {}\n", evasion));
            }
            if let Some(es) = self.defenses.energy_shield {
                md.push_str(&format!("- Energy Shield: {}\n", es));
            }
            md.push('\n');
        }

        // Damage (for weapons)
        if let Some(ref dmg) = self.damage {
            md.push_str("### Damage\n");
            for entry in &dmg.damages {
                md.push_str(&format!(
                    "- {:?}: {}-{}\n",
                    entry.damage_type, entry.min, entry.max
                ));
            }
            if dmg.attack_speed > 0.0 {
                md.push_str(&format!("- Attack Speed: {:.2}\n", dmg.attack_speed));
            }
            if dmg.critical_chance > 0.0 {
                md.push_str(&format!("- Critical Chance: {:.1}%\n", dmg.critical_chance));
            }
            if dmg.spell_efficiency > 0.0 {
                md.push_str(&format!(
                    "- Spell Efficiency: {:.0}%\n",
                    dmg.spell_efficiency
                ));
            }
            md.push('\n');
        }

        // Implicit
        if let Some(ref imp) = self.implicit {
            md.push_str("### Implicit\n");
            md.push_str(&format!("- {}\n\n", imp.display()));
        }

        // Explicit mods
        if !self.prefixes.is_empty() || !self.suffixes.is_empty() {
            md.push_str("### Modifiers\n");
            for prefix in &self.prefixes {
                md.push_str(&format!("- {} (P)\n", prefix.display()));
            }
            for suffix in &self.suffixes {
                md.push_str(&format!("- {} (S)\n", suffix.display()));
            }
            md.push('\n');
        }

        // Requirements
        if self.requirements.level > 0
            || self.requirements.strength > 0
            || self.requirements.dexterity > 0
            || self.requirements.intelligence > 0
        {
            let mut reqs = Vec::new();
            if self.requirements.level > 0 {
                reqs.push(format!("Level {}", self.requirements.level));
            }
            if self.requirements.strength > 0 {
                reqs.push(format!("{} Str", self.requirements.strength));
            }
            if self.requirements.dexterity > 0 {
                reqs.push(format!("{} Dex", self.requirements.dexterity));
            }
            if self.requirements.intelligence > 0 {
                reqs.push(format!("{} Int", self.requirements.intelligence));
            }
            md.push_str(&format!("*Requires: {}*\n", reqs.join(", ")));
        }

        md
    }
}

/// Defense values on an armour piece
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Defenses {
    pub armour: Option<i32>,
    pub evasion: Option<i32>,
    pub energy_shield: Option<i32>,
}

impl Defenses {
    pub fn has_any(&self) -> bool {
        self.armour.is_some() || self.evasion.is_some() || self.energy_shield.is_some()
    }
}

/// Individual damage entry with type and range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageValue {
    pub damage_type: DamageType,
    pub min: i32,
    pub max: i32,
}

/// Weapon damage values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDamage {
    pub damages: Vec<DamageValue>,
    pub attack_speed: f32,
    pub critical_chance: f32,
    pub spell_efficiency: f32,
}

/// Attribute scaling on a modifier — effective value = rolled_value * min(attribute / per, max_stacks)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModifierScaling {
    pub attribute: Attribute,
    pub per: f64,
    /// Maximum number of times the scaling can apply (caps attribute / per)
    pub max_stacks: Option<u32>,
}

/// A rolled modifier instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modifier {
    /// Reference to the affix ID
    pub affix_id: String,
    /// Display name of the affix
    pub name: String,
    /// The stat this modifies
    pub stat: StatType,
    /// Whether this modifier applies locally to the item or globally to the character
    #[serde(default)]
    pub scope: AffixScope,
    /// The rolled tier
    pub tier: u32,
    /// The rolled value within the tier's range (or min value for damage ranges)
    pub value: i32,
    /// For damage range stats: the rolled max value (e.g., the "10" in "Adds 5-10 Fire Damage")
    #[serde(default)]
    pub value_max: Option<i32>,
    /// Minimum value for this tier
    pub tier_min: i32,
    /// Maximum value for this tier
    pub tier_max: i32,
    /// For damage range stats: the tier range for the max value
    #[serde(default)]
    pub tier_max_value: Option<(i32, i32)>,
    /// Skill IDs granted by this modifier
    #[serde(default)]
    pub granted_skills: Vec<String>,
    /// Optional attribute scaling — when present, effective value = rolled_value * (attribute / per)
    #[serde(default)]
    pub scaling: Option<ModifierScaling>,
}

impl Modifier {
    /// Create a modifier from an affix config and rolled values
    pub fn from_affix(
        affix: &AffixConfig,
        tier: &AffixTierConfig,
        value: i32,
        value_max: Option<i32>,
    ) -> Self {
        Modifier {
            affix_id: affix.id.clone(),
            name: affix.name.clone(),
            stat: affix.stat,
            scope: affix.scope,
            tier: tier.tier,
            value,
            value_max,
            tier_min: tier.min,
            tier_max: tier.max,
            tier_max_value: tier.max_value.map(|r| (r.min, r.max)),
            granted_skills: affix.granted_skills.clone(),
            scaling: affix.scaling.as_ref().map(|s| ModifierScaling {
                attribute: s.attribute,
                per: s.per,
                max_stacks: s.max_stacks,
            }),
        }
    }

    /// Display the modifier as a human-readable string
    pub fn display(&self) -> String {
        // Check if this is a flat damage stat with a range
        if let Some(max_val) = self.value_max {
            let damage_type = match self.stat {
                StatType::AddedPhysicalDamage => Some("Physical"),
                StatType::AddedFireDamage => Some("Fire"),
                StatType::AddedColdDamage => Some("Cold"),
                StatType::AddedLightningDamage => Some("Lightning"),
                StatType::AddedChaosDamage => Some("Chaos"),
                _ => None,
            };

            if let Some(dmg_type) = damage_type {
                return format!("Adds {} to {} {} Damage", self.value, max_val, dmg_type);
            }
        }

        let stat_name = format!("{:?}", self.stat)
            .chars()
            .fold(String::new(), |mut acc, c| {
                if c.is_uppercase() && !acc.is_empty() {
                    acc.push(' ');
                }
                acc.push(c);
                acc
            });

        // Determine if this is a percentage or flat value based on stat type
        let is_percent = matches!(
            self.stat,
            StatType::IncreasedPhysicalDamage
                | StatType::IncreasedElementalDamage
                | StatType::IncreasedChaosDamage
                | StatType::IncreasedAttackSpeed
                | StatType::IncreasedCriticalChance
                | StatType::IncreasedCriticalDamage
                | StatType::IncreasedArmour
                | StatType::IncreasedEvasion
                | StatType::IncreasedEnergyShield
                | StatType::IncreasedLife
                | StatType::IncreasedMana
                | StatType::IncreasedAccuracy
                | StatType::IncreasedStrength
                | StatType::IncreasedDexterity
                | StatType::IncreasedConstitution
                | StatType::IncreasedIntelligence
                | StatType::IncreasedWisdom
                | StatType::IncreasedCharisma
                | StatType::IncreasedAllAttributes
                | StatType::IncreasedMovementSpeed
                | StatType::IncreasedItemRarity
                | StatType::IncreasedItemQuantity
                | StatType::FireResistance
                | StatType::ColdResistance
                | StatType::LightningResistance
                | StatType::ChaosResistance
                | StatType::AllResistances
                | StatType::LifeLeech
                | StatType::ManaLeech
                // Status effect durations
                | StatType::IncreasedPoisonDuration
                | StatType::IncreasedBleedDuration
                | StatType::IncreasedBurnDuration
                | StatType::IncreasedFreezeDuration
                | StatType::IncreasedChillDuration
                | StatType::IncreasedStaticDuration
                | StatType::IncreasedFearDuration
                | StatType::IncreasedSlowDuration
                // Status effect magnitudes
                | StatType::PoisonMagnitude
                | StatType::BleedMagnitude
                | StatType::BurnMagnitude
                | StatType::FreezeMagnitude
                | StatType::ChillMagnitude
                | StatType::StaticMagnitude
                | StatType::FearMagnitude
                | StatType::SlowMagnitude
                // Damage conversions to status effects
                | StatType::ConvertPhysicalToPoison
                | StatType::ConvertFireToPoison
                | StatType::ConvertColdToPoison
                | StatType::ConvertLightningToPoison
                | StatType::ConvertChaosToPoison
                | StatType::ConvertPhysicalToBleed
                | StatType::ConvertFireToBleed
                | StatType::ConvertColdToBleed
                | StatType::ConvertLightningToBleed
                | StatType::ConvertChaosToBleed
                | StatType::ConvertPhysicalToBurn
                | StatType::ConvertFireToBurn
                | StatType::ConvertColdToBurn
                | StatType::ConvertLightningToBurn
                | StatType::ConvertChaosToBurn
                | StatType::ConvertPhysicalToFreeze
                | StatType::ConvertFireToFreeze
                | StatType::ConvertColdToFreeze
                | StatType::ConvertLightningToFreeze
                | StatType::ConvertChaosToFreeze
                | StatType::ConvertPhysicalToChill
                | StatType::ConvertFireToChill
                | StatType::ConvertColdToChill
                | StatType::ConvertLightningToChill
                | StatType::ConvertChaosToChill
                | StatType::ConvertPhysicalToStatic
                | StatType::ConvertFireToStatic
                | StatType::ConvertColdToStatic
                | StatType::ConvertLightningToStatic
                | StatType::ConvertChaosToStatic
                | StatType::ConvertPhysicalToFear
                | StatType::ConvertFireToFear
                | StatType::ConvertColdToFear
                | StatType::ConvertLightningToFear
                | StatType::ConvertChaosToFear
                | StatType::ConvertPhysicalToSlow
                | StatType::ConvertFireToSlow
                | StatType::ConvertColdToSlow
                | StatType::ConvertLightningToSlow
                | StatType::ConvertChaosToSlow
        );

        if let Some(ref scaling) = self.scaling {
            let attr_name = format!("{:?}", scaling.attribute);
            let per = if scaling.per.fract() == 0.0 {
                format!("{}", scaling.per as i64)
            } else {
                format!("{}", scaling.per)
            };
            let cap = match scaling.max_stacks {
                Some(max) => format!(" (max {})", max),
                None => String::new(),
            };
            return format!("+{} {} per {} {}{}", self.value, stat_name, per, attr_name, cap);
        }

        if is_percent {
            format!("+{}% {}", self.value, stat_name)
        } else {
            format!("+{} {}", self.value, stat_name)
        }
    }
}

impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Header
        writeln!(f, "══ {} ══", self.name)?;
        writeln!(f, "{} ({})", self.base_name, self.rarity)?;

        // Defenses
        if self.defenses.has_any() {
            writeln!(f)?;
            writeln!(f, "Defenses")?;
            if let Some(armour) = self.defenses.armour {
                writeln!(f, "  Armour: {}", armour)?;
            }
            if let Some(evasion) = self.defenses.evasion {
                writeln!(f, "  Evasion: {}", evasion)?;
            }
            if let Some(es) = self.defenses.energy_shield {
                writeln!(f, "  Energy Shield: {}", es)?;
            }
        }

        // Damage
        if let Some(ref dmg) = self.damage {
            writeln!(f)?;
            writeln!(f, "Damage")?;
            for entry in &dmg.damages {
                writeln!(f, "  {}: {}-{}", entry.damage_type, entry.min, entry.max)?;
            }
            if dmg.attack_speed > 0.0 {
                writeln!(f, "  Attack Speed: {:.2}", dmg.attack_speed)?;
            }
            if dmg.critical_chance > 0.0 {
                writeln!(f, "  Critical Chance: {:.1}%", dmg.critical_chance)?;
            }
            if dmg.spell_efficiency > 0.0 {
                writeln!(f, "  Spell Efficiency: {:.0}%", dmg.spell_efficiency)?;
            }
        }

        // Implicit
        if let Some(ref imp) = self.implicit {
            writeln!(f)?;
            writeln!(f, "Implicit")?;
            writeln!(f, "  {}", imp)?;
        }

        // Explicit mods
        if !self.prefixes.is_empty() || !self.suffixes.is_empty() {
            writeln!(f)?;
            writeln!(f, "Modifiers")?;
            for prefix in &self.prefixes {
                writeln!(f, "  {} (P)", prefix)?;
            }
            for suffix in &self.suffixes {
                writeln!(f, "  {} (S)", suffix)?;
            }
        }

        // Skills
        let all_skills = self.all_skills();
        if !all_skills.is_empty() {
            writeln!(f)?;
            writeln!(f, "Skills")?;
            for skill in &all_skills {
                writeln!(f, "  {}", skill)?;
            }
        }

        // Requirements
        let mut reqs = Vec::new();
        if self.requirements.level > 0 {
            reqs.push(format!("Level {}", self.requirements.level));
        }
        if self.requirements.strength > 0 {
            reqs.push(format!("{} Str", self.requirements.strength));
        }
        if self.requirements.dexterity > 0 {
            reqs.push(format!("{} Dex", self.requirements.dexterity));
        }
        if self.requirements.intelligence > 0 {
            reqs.push(format!("{} Int", self.requirements.intelligence));
        }
        if !reqs.is_empty() {
            writeln!(f)?;
            write!(f, "Requires: {}", reqs.join(", "))?;
        }

        Ok(())
    }
}
