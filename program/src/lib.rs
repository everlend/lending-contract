#![deny(missing_docs)]

//! Everlend Lending Contract

pub mod error;
pub mod instruction;
pub mod processor;
pub mod pyth;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("69LK6qziCCnqgmUPYpuiJ2y8JavKVRrCZ4pDekSyDZTn");

/// Generates seed bump for authorities
pub fn find_program_address(program_id: &Pubkey, pubkey: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&pubkey.to_bytes()[..32]], program_id)
}

/// Generates obligation authority & bump seed
pub fn find_obligation_authority(
    program_id: &Pubkey,
    owner: &Pubkey,
    market: &Pubkey,
    liquidity: &Pubkey,
    collateral: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &owner.to_bytes()[..32],
            &market.to_bytes()[..32],
            &liquidity.to_bytes()[..32],
            &collateral.to_bytes()[..32],
        ],
        program_id,
    )
}
