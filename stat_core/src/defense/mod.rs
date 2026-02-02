//! Defense system - Armour, Evasion, Resistances

mod armour;
mod evasion;
mod resistance;

pub use armour::calculate_armour_reduction;
pub use evasion::{apply_evasion_cap, calculate_damage_cap};
pub use resistance::calculate_resistance_mitigation;

/// Defense calculation constants
pub mod constants {
    /// Maximum resistance cap (100% = immunity)
    pub const MAX_RESISTANCE: f64 = 100.0;

    /// Minimum resistance (can go negative)
    pub const MIN_RESISTANCE: f64 = -200.0;

    /// Penetration effectiveness vs capped resistance
    pub const PENETRATION_VS_CAPPED: f64 = 0.5;

    /// Armour formula constant (higher = armour less effective vs big hits)
    pub const ARMOUR_CONSTANT: f64 = 5.0;

    /// Evasion scaling factor (controls diminishing returns)
    /// Formula: damage_cap = accuracy / (1 + evasion / SCALE_FACTOR)
    pub const EVASION_SCALE_FACTOR: f64 = 1000.0;
}
