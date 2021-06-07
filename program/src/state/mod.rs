//! State types

mod collateral;
mod liquidity;
mod market;
mod obligation;

pub use collateral::*;
pub use liquidity::*;
pub use market::*;
pub use obligation::*;

/// Accounts are created with data zeroed out, so uninitialized state instances
/// will have the version set to 0.
pub const UNINITIALIZED_VERSION: u8 = 0;

/// Current version of the program and all new accounts created
pub const PROGRAM_VERSION: u8 = 1;

/// Ratio power
pub const RATIO_POWER: u64 = 1_000_000_000;

/// Convert the UI representation of a ratio (like 0.5) to the raw ratio
pub fn ui_ratio_to_ratio(ui_ratio: f64) -> u64 {
    (ui_ratio * RATIO_POWER as f64).round() as u64
}

/// Convert the raw ratio (like 500_000_000) to the UI representation
pub fn ratio_to_ui_ratio(ratio: u64) -> f64 {
    ratio as f64 / RATIO_POWER as f64
}
