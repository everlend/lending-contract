//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum LendingInstruction {
    /// Initializes new market
    ///
    /// Accounts:
    /// [W] new uninitialized market account
    /// [RS] market owner
    InitMarket,
}

/// Create `InitMarket` instruction
pub fn init_market(
    program_id: &Pubkey,
    market: &Pubkey,
    owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::InitMarket;
    let data = init_data.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
