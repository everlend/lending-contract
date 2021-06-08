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

/// Obligation
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Obligation {
    /// State version
    pub version: u8,
    /// Market
    pub market: Pubkey,
    /// Obligation owner
    pub owner: Pubkey,
    /// Liquidity
    pub liquidity: Pubkey,
    /// Collateral
    pub collateral: Pubkey,
    /// Amount of borrowed liquidity
    pub amount_liquidity_borrowed: u64,
    /// Amount of deposited collateral
    pub amount_collateral_deposited: u64,
}

impl Obligation {
    /// Initialize a obligation
    pub fn init(&mut self, params: InitObligationParams) {
        self.version = PROGRAM_VERSION;
        self.market = params.market;
        self.owner = params.owner;
        self.liquidity = params.liquidity;
        self.collateral = params.collateral;
        self.amount_liquidity_borrowed = 0;
        self.amount_collateral_deposited = 0;
    }

    /// Increase amount of deposited collateral
    pub fn collateral_deposit(&mut self, amount: u64) -> ProgramResult {
        self.amount_collateral_deposited = self
            .amount_collateral_deposited
            .checked_add(amount)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(())
    }

    /// Decrease amount of deposited collateral
    pub fn collateral_withdraw(&mut self, amount: u64) -> ProgramResult {
        self.amount_collateral_deposited = self
            .amount_collateral_deposited
            .checked_sub(amount)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(())
    }

    /// Increase amount of borrowed liquidity
    pub fn liquidity_borrow(&mut self, amount: u64) -> ProgramResult {
        self.amount_liquidity_borrowed = self
            .amount_liquidity_borrowed
            .checked_add(amount)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(())
    }

    /// Decrease amount of borrowed liquidity
    pub fn liquidity_repay(&mut self, amount: u64) -> ProgramResult {
        self.amount_liquidity_borrowed = self
            .amount_liquidity_borrowed
            .checked_sub(amount)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(())
    }

    /// Calculate obligation health ratio
    pub fn calc_health(&self) -> Result<u64, ProgramError> {
        // TODO: Add oracle interface here to calculate collateral and borrowed liquidity value.
        // For now we assume that collateral and liquidity tokens have 1:1 value ratio
        let result = (self.amount_liquidity_borrowed as u128)
            .checked_mul(RATIO_POWER as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_div(self.amount_collateral_deposited as u128)
            .ok_or(LendingError::CollateralHealthCheckFailed)? as u64;

        Ok(result)
    }

    /// Calculation of available funds for withdrawal
    pub fn calc_withdrawal_limit(&self, ratio_initial: u64) -> Result<u64, ProgramError> {
        // deposited - borrowed / ratio_initial
        let result = self
            .amount_collateral_deposited
            .checked_sub(
                self.amount_liquidity_borrowed
                    .checked_mul(RATIO_POWER)
                    .ok_or(LendingError::CalculationFailure)?
                    .checked_div(ratio_initial)
                    .ok_or(LendingError::CalculationFailure)?,
            )
            .ok_or(LendingError::CalculationFailure)?;

        Ok(result)
    }

    /// Calculation of available funds for borrowing
    pub fn calc_borrowing_limit(&self, ratio_initial: u64) -> Result<u64, ProgramError> {
        // deposited * ratio_initial - borrowed
        let result = self
            .amount_collateral_deposited
            .checked_mul(ratio_initial)
            .ok_or(LendingError::CalculationFailure)?
            .checked_div(RATIO_POWER)
            .ok_or(LendingError::CalculationFailure)?
            .checked_sub(self.amount_liquidity_borrowed)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(result)
    }
}

impl Sealed for Obligation {}
impl Pack for Obligation {
    // 1 + 32 + 32 + 32 + 32 + 8 + 8
    const LEN: usize = 145;

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

/// Initialize a obligation params
pub struct InitObligationParams {
    /// Market
    pub market: Pubkey,
    /// Obligation owner
    pub owner: Pubkey,
    /// Liquidity
    pub liquidity: Pubkey,
    /// Collateral
    pub collateral: Pubkey,
}

impl IsInitialized for Obligation {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
