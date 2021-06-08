//! Program state definitions
use super::*;
use crate::error::LendingError;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

/// Collateral status
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum CollateralStatus {
    /// Inactive and invisible
    Inactive = 0,
    /// Active
    Active = 1,
    /// Inactive but visible
    InactiveAndVisible = 2,
}

impl Default for CollateralStatus {
    fn default() -> Self {
        CollateralStatus::Inactive
    }
}

/// Collateral
#[repr(C)]
#[derive(Debug, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct Collateral {
    /// State version
    pub version: u8,
    /// Token status
    pub status: CollateralStatus,
    /// Market
    pub market: Pubkey,
    /// Supply token mint
    pub token_mint: Pubkey,
    /// Supply token account
    pub token_account: Pubkey,
    /// Fractional initial collateralization ratio (multiplied by 10e9)
    pub ratio_initial: u64,
    /// Fractional limit for the healthy collateralization ratio (multiplied by 10e9)
    pub ratio_healthy: u64,
    /// Oracle state account pubkey - optional
    pub oracle: Option<Pubkey>,
}

impl Collateral {
    /// Initialize a collateral
    pub fn init(&mut self, params: InitCollateralParams) {
        self.version = PROGRAM_VERSION;
        self.status = CollateralStatus::Inactive;
        self.market = params.market;
        self.token_mint = params.token_mint;
        self.token_account = params.token_account;
        self.ratio_initial = params.ratio_initial;
        self.ratio_healthy = params.ratio_healthy;
        self.oracle = params.oracle;
    }

    /// Check ratio to be within the collateral limits
    pub fn check_ratio(&self, ratio: u64) -> ProgramResult {
        if ratio > self.ratio_initial {
            Err(LendingError::CollateralRatioCheckFailed.into())
        } else {
            Ok(())
        }
    }
}

impl Sealed for Collateral {}
impl Pack for Collateral {
    // 1 + 1 + 32 + 32 + 32 + 8 + 8 + (1 + 32)
    const LEN: usize = 147;

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

/// Initialize a collateral params
pub struct InitCollateralParams {
    /// Market
    pub market: Pubkey,
    /// Supply token mint
    pub token_mint: Pubkey,
    /// Supply token account
    pub token_account: Pubkey,
    /// Fractional initial collateralization ratio (multiplied by 10e9)
    pub ratio_initial: u64,
    /// Fractional limit for the healthy collateralization ratio (multiplied by 10e9)
    pub ratio_healthy: u64,
    /// Oracle state account pubkey - optional
    pub oracle: Option<Pubkey>,
}

impl IsInitialized for Collateral {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
