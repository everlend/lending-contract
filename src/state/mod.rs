//! State types

mod collateral;
mod liquidity;
mod market;

pub use collateral::*;
pub use liquidity::*;
pub use market::*;

/// Accounts are created with data zeroed out, so uninitialized state instances
/// will have the version set to 0.
pub const UNINITIALIZED_VERSION: u8 = 0;

/// Current version of the program and all new accounts created
pub const PROGRAM_VERSION: u8 = 1;
