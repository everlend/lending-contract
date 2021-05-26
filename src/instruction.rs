//! Instruction types

use crate::{
    find_program_address,
    state::{CollateralStatus, LiquidityStatus},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program, sysvar,
};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum LendingInstruction {
    /// Initializes new market
    ///
    /// Accounts:
    /// [W] Market account - uninitialized.
    /// [RS] Market owner
    /// [R] Rent sysvar
    InitMarket,

    /// Create liquidity token
    ///
    /// Accounts:
    /// [W] Liquidity account to create - uninitialized.
    /// [R] Token mint account
    /// [W] Token account - uninitialized.
    /// [W] Pool mint account - uninitialized.
    /// [W] Market account
    /// [RS] Market owner
    /// [R] Market authority
    /// [R] Rent sysvar
    /// [R] Token program id
    CreateLiquidityToken,

    /// Update liquidity token
    ///
    /// Accounts:
    /// [W] Liquidity account
    /// [RS] Market owner
    UpdateLiquidityToken {
        /// New status for liquidity token
        status: LiquidityStatus,
    },

    /// Create collateral token
    ///
    /// Accounts:
    /// [W] Collateral account to create - uninitialized.
    /// [R] Token mint account
    /// [W] Token account - uninitialized.
    /// [W] Market account
    /// [RS] Market owner
    /// [R] Market authority
    /// [R] Rent sysvar
    /// [R] Token program id
    CreateCollateralToken {
        /// Fractional initial collateralization ratio (multiplied by 10e9)
        ratio_initial: u64,
        /// Fractional limit for the healthy collateralization ratio (multiplied by 10e9)
        ratio_healthy: u64,
    },

    /// Update collateral token
    ///
    /// Accounts:
    /// [W] Collateral account
    /// [RS] Market owner
    UpdateCollateralToken {
        /// New status for collateral token
        status: CollateralStatus,
        /// Fractional initial collateralization ratio (multiplied by 10e9)
        ratio_initial: u64,
        /// Fractional limit for the healthy collateralization ratio (multiplied by 10e9)
        ratio_healthy: u64,
    },
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
    market_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::CreateLiquidityToken;
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let accounts = vec![
        AccountMeta::new(*liquidity, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*market_owner, true),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `UpdateLiquidityToken` instruction
pub fn update_liquidity_token(
    program_id: &Pubkey,
    status: LiquidityStatus,
    liquidity: &Pubkey,
    market_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::UpdateLiquidityToken { status };
    let data = init_data.try_to_vec()?;

    let accounts = vec![
        AccountMeta::new(*liquidity, false),
        AccountMeta::new_readonly(*market_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `CreateCollateralToken` instruction
pub fn create_collateral_token(
    program_id: &Pubkey,
    ratio_initial: u64,
    ratio_healthy: u64,
    collateral: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    market: &Pubkey,
    market_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::CreateCollateralToken {
        ratio_initial,
        ratio_healthy,
    };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let accounts = vec![
        AccountMeta::new(*collateral, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*market_owner, true),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `UpdateCollateralToken` instruction
pub fn update_collateral_token(
    program_id: &Pubkey,
    status: CollateralStatus,
    ratio_initial: u64,
    ratio_healthy: u64,
    collateral: &Pubkey,
    market_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::UpdateCollateralToken {
        status,
        ratio_initial,
        ratio_healthy,
    };
    let data = init_data.try_to_vec()?;

    let accounts = vec![
        AccountMeta::new(*collateral, false),
        AccountMeta::new_readonly(*market_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
