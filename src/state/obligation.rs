//! Program state definitions
use super::*;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
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
    /// Amount of liquidity
    pub amount_liquidity: u64,
    /// Amount of collateral
    pub amount_collateral: u64,
}

impl Obligation {
    /// Initialize a obligation
    pub fn init(&mut self, params: InitObligationParams) {
        self.version = PROGRAM_VERSION;
        self.market = params.market;
        self.owner = params.owner;
        self.liquidity = params.liquidity;
        self.collateral = params.collateral;
        self.amount_liquidity = 0;
        self.amount_collateral = 0;
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
