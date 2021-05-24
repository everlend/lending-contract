//! Program state definitions
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

use super::*;

/// Lending Market
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Market {
    /// State version
    pub version: u8,
    /// Market owner
    pub owner: Pubkey,
    /// Number of liquidity tokens in the market
    pub liquidity_tokens: u64,
    /// Number of collateral tokens in the market
    pub collateral_tokens: u64,
}

impl Market {
    /// Initialize a market
    pub fn init(&mut self, params: InitMarketParams) {
        self.version = PROGRAM_VERSION;
        self.liquidity_tokens = 0;
        self.collateral_tokens = 0;
        self.owner = params.owner;
    }

    /// Increment liquidity tokens
    pub fn increment_liquidity_tokens(&mut self) {
        self.liquidity_tokens += 1;
    }
}

/// Initialize a market params
pub struct InitMarketParams {
    /// Market owner
    pub owner: Pubkey,
}

impl IsInitialized for Market {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
