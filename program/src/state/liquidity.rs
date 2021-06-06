//! Program state definitions

use crate::error::LendingError;

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
    Inactive = 0,
    /// Active
    Active = 1,
    /// Inactive but visible
    InactiveAndVisible = 2,
}

impl Default for LiquidityStatus {
    fn default() -> Self {
        LiquidityStatus::Inactive
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
        self.status = LiquidityStatus::Inactive;
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
    ) -> Result<u64, ProgramError> {
        let result = if pool_mint_supply == 0 || token_account_amount == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(pool_mint_supply as u128)
                .ok_or(LendingError::CalculationFailure)?
                .checked_div(token_account_amount as u128)
                .ok_or(LendingError::CalculationFailure)? as u64
        };

        Ok(result)
    }

    /// Withdraw exchange amount
    pub fn calc_withdraw_exchange_amount(
        &self,
        amount: u64,
        token_account_amount: u64,
        pool_mint_supply: u64,
    ) -> Result<u64, ProgramError> {
        let result = if pool_mint_supply == 0 || token_account_amount == 0 {
            amount
        } else {
            (amount as u128)
                .checked_mul(token_account_amount as u128)
                .ok_or(LendingError::CalculationFailure)?
                .checked_div(pool_mint_supply as u128)
                .ok_or(LendingError::CalculationFailure)? as u64
        };

        Ok(result)
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
