//! Program state definitions
use crate::error::LendingError;

use super::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    clock::Slot,
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
    /// Interest amount
    pub interest_amount: u64,
    /// Interest slot
    pub interest_slot: Slot,
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
        self.interest_amount = 0;
        self.interest_slot = params.interest_slot;
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

    /// Calc pending interest amount
    /// borrowed * (current_slot - interest_slot) * interest
    pub fn calc_pending_interest_amount(
        &self,
        slot: Slot,
        interest: u64,
    ) -> Result<u64, ProgramError> {
        let slot_offset = slot
            .checked_sub(self.interest_slot)
            .ok_or(LendingError::CalculationFailure)?;

        let pending = (self.amount_liquidity_borrowed as u128)
            .checked_mul(slot_offset as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_mul(interest as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_div(INTEREST_POWER as u128)
            .ok_or(LendingError::CalculationFailure)? as u64;

        Ok(pending)
    }

    /// Calc effective interest amount
    /// interest_amount + borrowed * (current_slot - interest_slot) * interest
    pub fn calc_effective_interest_amount(
        &self,
        slot: Slot,
        interest: u64,
    ) -> Result<u64, ProgramError> {
        let amount = self
            .interest_amount
            .checked_add(self.calc_pending_interest_amount(slot, interest)?)
            .ok_or(LendingError::CalculationFailure)?;

        Ok(amount)
    }

    /// Update intereset per each borrow
    pub fn update_interest_amount(&mut self, amount: u64) {
        self.interest_amount = amount;
    }

    /// Update slot to last
    pub fn update_slot(&mut self, slot: Slot) {
        self.interest_slot = slot;
    }

    /// Calculate obligation ratio
    pub fn calc_ratio(
        &self,
        liquidity_market_price: u64,
        collateral_market_price: u64,
    ) -> Result<u64, ProgramError> {
        // TODO: Add oracle interface here to calculate collateral and borrowed liquidity value.
        // For now we assume that collateral and liquidity tokens have 1:1 value ratio
        let result = if self.amount_liquidity_borrowed == 0 && self.amount_collateral_deposited == 0
        {
            0
        } else {
            let liquidity_value = (self.amount_liquidity_borrowed as u128)
                .checked_mul(liquidity_market_price as u128)
                .ok_or(LendingError::CalculationFailure)?;
            let collateral_value = (self.amount_collateral_deposited as u128)
                .checked_mul(collateral_market_price as u128)
                .ok_or(LendingError::CalculationFailure)?;

            liquidity_value
                .checked_mul(RATIO_POWER as u128)
                .ok_or(LendingError::CalculationFailure)?
                .checked_div(collateral_value)
                .ok_or(LendingError::CollateralRatioCheckFailed)? as u64
        };

        Ok(result)
    }

    /// Calculation of available funds for withdrawal
    pub fn calc_withdrawal_limit(
        &self,
        ratio_initial: u64,
        liquidity_market_price: u64,
        collateral_market_price: u64,
    ) -> Result<u64, ProgramError> {
        let liquidity_value = (self.amount_liquidity_borrowed as u128)
            .checked_mul(liquidity_market_price as u128)
            .ok_or(LendingError::CalculationFailure)?;

        // deposited - borrowed / ratio_initial
        let result = (self.amount_collateral_deposited as u128)
            .checked_sub(
                liquidity_value
                    .checked_mul(RATIO_POWER as u128)
                    .ok_or(LendingError::CalculationFailure)?
                    .checked_div(ratio_initial as u128)
                    .ok_or(LendingError::CalculationFailure)?
                    .checked_div(collateral_market_price as u128)
                    .ok_or(LendingError::CalculationFailure)?,
            )
            .ok_or(LendingError::CalculationFailure)? as u64;

        Ok(result)
    }

    /// Calculation of available funds for borrowing
    pub fn calc_borrowing_limit(
        &self,
        ratio_initial: u64,
        liquidity_market_price: u64,
        collateral_market_price: u64,
    ) -> Result<u64, ProgramError> {
        let collateral_value = (self.amount_collateral_deposited as u128)
            .checked_mul(collateral_market_price as u128)
            .ok_or(LendingError::CalculationFailure)?;

        // deposited * ratio_initial - borrowed
        let result = collateral_value
            .checked_mul(ratio_initial as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_div(RATIO_POWER as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_div(liquidity_market_price as u128)
            .ok_or(LendingError::CalculationFailure)?
            .checked_sub(self.amount_liquidity_borrowed as u128)
            .ok_or(LendingError::CalculationFailure)? as u64;

        Ok(result)
    }
}

impl Sealed for Obligation {}
impl Pack for Obligation {
    // 1 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8
    const LEN: usize = 161;

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
    /// Interest slot
    pub interest_slot: Slot,
}

impl IsInitialized for Obligation {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
