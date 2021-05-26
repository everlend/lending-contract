//! Program state definitions
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

use super::*;

/// Liqudiity status
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum LiquidityStatus {
    /// Inactive and invisible
    InActive = 0,
    /// Active
    Active = 1,
    /// Inactive but visible
    InActiveAndVisible = 2,
}

impl Default for LiquidityStatus {
    fn default() -> Self {
        LiquidityStatus::InActive
    }
}

/// Lending Market
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Liquidity {
    /// State version
    pub version: u8,
    /// Token status
    pub status: LiquidityStatus,
    /// Market
    pub market: Pubkey,
    /// Supply token mint
    pub token_mint: Pubkey,
    /// Supply token account
    pub token_account: Pubkey,
    /// Token that lenders will receive
    pub pool_mint: Pubkey,
}

impl Liquidity {
    /// LEN
    pub const LEN: usize = 130;

    /// Initialize a market
    pub fn init(&mut self, params: InitLiquidityParams) {
        self.version = PROGRAM_VERSION;
        self.status = LiquidityStatus::InActive;
        self.market = params.market;
        self.token_mint = params.token_mint;
        self.token_account = params.token_account;
        self.pool_mint = params.pool_mint;
    }
}

/// Initialize a liquidity params
pub struct InitLiquidityParams {
    /// Market
    pub market: Pubkey,
    /// Supply token mint
    pub token_mint: Pubkey,
    /// Supply token account
    pub token_account: Pubkey,
    /// Token that lenders will receive
    pub pool_mint: Pubkey,
}

impl IsInitialized for Liquidity {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
