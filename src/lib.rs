#![deny(missing_docs)]

//! Everlend Lending Contract

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
use solana_program::pubkey::Pubkey;

solana_program::declare_id!("Fh6wJURoiAGhPDYJVKcW9EsHDBYdMfysaqk2jZfQWC1r");

/// Generates seed bump for authorities
pub fn find_program_address(program_id: &Pubkey, pubkey: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[&pubkey.to_bytes()[..32]], program_id)
}
