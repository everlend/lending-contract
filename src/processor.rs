//! Program state processor

use crate::instruction::LendingInstruction;
use crate::state::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Process InitMarket instruction
    pub fn init_market(
        _program_id: &Pubkey,
        market: &AccountInfo,
        owner: &AccountInfo,
    ) -> ProgramResult {
        let mut market_data = Market::try_from_slice(&market.data.borrow())?;

        if !owner.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        market_data.version.uninitialized()?;

        market_data.owner = *owner.key;
        market_data.version = StateVersion::V1;

        market_data.serialize(&mut *market.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// Instruction processing router
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = LendingInstruction::try_from_slice(input)?;

        match instruction {
            LendingInstruction::InitMarket => {
                msg!("LendingInstruction: InitMarket");
                match accounts {
                    [market, owner, ..] => Self::init_market(program_id, market, owner),
                    _ => Err(ProgramError::NotEnoughAccountKeys),
                }
            }
        }
    }
}
