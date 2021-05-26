//! Program state definitions
use super::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    msg,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
};

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

    /// Increment collateral tokens
    pub fn increment_collateral_tokens(&mut self) {
        self.collateral_tokens += 1;
    }
}

/// Initialize a market params
pub struct InitMarketParams {
    /// Market owner
    pub owner: Pubkey,
}

impl Sealed for Market {}
impl Pack for Market {
    // 1 + 32 + 8 + 8
    const LEN: usize = 49;

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

impl IsInitialized for Market {
    fn is_initialized(&self) -> bool {
        self.version != UNINITIALIZED_VERSION
    }
}
