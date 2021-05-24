//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum LendingInstruction {
    /// Initializes new market
    ///
    /// Accounts:
    /// [W] new uninitialized market account
    /// [RS] market owner
    /// [R] rent sysvar
    InitMarket,

    /// Create liquidity token
    ///
    /// Accounts:
    /// [W] new uninitialized liquidity account
    /// [R] token mint account
    /// [W] token account
    /// [W] pool mint account
    /// [R] market account
    /// [R] market authority
    /// [R] rent sysvar
    /// [R] token program id
    CreateLiquidityToken,
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
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `CreateLiquidityToken` instruction
pub fn create_liquidity_token(
    program_id: &Pubkey,
    liquidity: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    pool_mint: &Pubkey,
    market: &Pubkey,
    market_authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::InitMarket;
    let data = init_data.try_to_vec()?;
    let accounts = vec![
        AccountMeta::new(*liquidity, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*market_authority, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
