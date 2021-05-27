//! Program state definitions

use super::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

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

/// Liquidity
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
    /// Initialize a collateral
    pub fn init(&mut self, params: InitLiquidityParams) {
        self.version = PROGRAM_VERSION;
        self.status = LiquidityStatus::InActive;
        self.market = params.market;
        self.token_mint = params.token_mint;
        self.token_account = params.token_account;
        self.pool_mint = params.pool_mint;
    }

    /// Deposit exchange amount
    pub fn calc_deposit_exchange_amount(
        &self,
        amount: u64,
        token_account_amount: u64,
        pool_mint_supply: u64,
    ) -> u64 {
        if pool_mint_supply == 0 || token_account_amount == 0 {
            amount
        } else {
            amount * pool_mint_supply / token_account_amount
        }
    }

    /// Withdraw exchange amount
    pub fn calc_withdraw_exchange_amount(
        &self,
        amount: u64,
        token_account_amount: u64,
        pool_mint_supply: u64,
    ) -> u64 {
        if pool_mint_supply == 0 || token_account_amount == 0 {
            amount
        } else {
            amount * token_account_amount / pool_mint_supply
        }
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

impl Sealed for Liquidity {}
impl Pack for Liquidity {
    // 1 + 1 + 32 + 32 + 32 + 32
    const LEN: usize = 130;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        Self::try_from_slice(src).map_err(|_| {
            msg!("Failed to deserialize");
            ProgramError::InvalidAccountData
        })
    }
}

impl IsInitialized for Liquidity {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
