//! Defense system - Armour, Evasion, Resistances

mod armour;
mod evasion;
mod resistance;

pub use armour::calculate_armour_reduction;
pub use evasion::{apply_evasion_cap, calculate_damage_cap};
pub use resistance::{calculate_effective_resistance, calculate_resistance_mitigation};
