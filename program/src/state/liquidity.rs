//! Program state definitions

use crate::error::LendingError;

use super::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
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
    /// Amount borrowed from the liquidity pool
    pub amount_borrowed: u64,
    /// Oracle price account pubkey
    pub oracle: Pubkey,
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
        self.amount_borrowed = 0;
        self.oracle = params.oracle;
    }

    /// Borrow funds
    pub fn borrow(&mut self, amount: u64) -> ProgramResult {
        self.amount_borrowed = self
            .amount_borrowed
            .checked_add(amount)
            .ok_or(LendingError::CalculationFailure)?;
        Ok(())
    }

    /// Repay funds
    pub fn repay(&mut self, amount: u64) -> ProgramResult {
        self.amount_borrowed = self
            .amount_borrowed
            .checked_sub(amount)
            .ok_or(LendingError::CalculationFailure)?;
        Ok(())
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
            let total_amount = token_account_amount
                .checked_add(self.amount_borrowed)
                .ok_or(LendingError::CalculationFailure)?;
            (amount as u128)
                .checked_mul(pool_mint_supply as u128)
                .ok_or(LendingError::CalculationFailure)?
                .checked_div(total_amount as u128)
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
            let total_amount = token_account_amount
                .checked_add(self.amount_borrowed)
                .ok_or(LendingError::CalculationFailure)?;
            (amount as u128)
                .checked_mul(total_amount as u128)
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
    /// Oracle price account pubkey
    pub oracle: Pubkey,
}

impl Sealed for Liquidity {}
impl Pack for Liquidity {
    // 1 + 1 + 32 + 32 + 32 + 32 + 8 + 32
    const LEN: usize = 170;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut slice = dst;
        self.serialize(&mut slice).unwrap()
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, solana_program::program_error::ProgramError> {
        let mut src_mut = src;
        Self::deserialize(&mut src_mut).map_err(|err| {
            msg!("{:?}", err);
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
