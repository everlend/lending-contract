//! Instruction types

use crate::{
    find_obligation_authority, find_program_address,
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
    /// [W] Liquidity account to create - uninitialized
    /// [R] Token mint account
    /// [W] Token account - uninitialized
    /// [W] Pool mint account - uninitialized
    /// [W] Market account
    /// [RS] Market owner
    /// [R] Market authority
    /// [R] Rent sysvar
    /// [R] Sytem program
    /// [R] Token program id
    /// [R] Oracle state account pubkey - optional
    CreateLiquidityToken,

    /// Update liquidity token
    ///
    /// Accounts:
    /// [W] Liquidity account
    /// [W] Market account
    /// [RS] Market owner
    UpdateLiquidityToken {
        /// New status for liquidity token
        status: LiquidityStatus,
    },

    /// Create collateral token
    ///
    /// Accounts:
    /// [W] Collateral account to create - uninitialized
    /// [R] Token mint account
    /// [W] Token account - uninitialized
    /// [W] Market account
    /// [RS] Market owner
    /// [R] Market authority
    /// [R] Rent sysvar
    /// [R] Sytem program
    /// [R] Token program id
    /// [R] Oracle state account pubkey - optional
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
    /// [W] Market account
    /// [RS] Market owner
    UpdateCollateralToken {
        /// New status for collateral token
        status: CollateralStatus,
        /// Fractional initial collateralization ratio (multiplied by 10e9)
        ratio_initial: u64,
        /// Fractional limit for the healthy collateralization ratio (multiplied by 10e9)
        ratio_healthy: u64,
    },

    /// Deposit liquidity
    ///
    /// Accounts:
    /// [R] Liquidity account
    /// [W] Source provider account (for token mint)
    /// [W] Destination provider account (for pool mint)
    /// [W] Token account
    /// [W] Pool mint account
    /// [R] Market account
    /// [R] Market authority
    /// [RS] User transfer authority
    /// [R] Token program id
    LiquidityDeposit {
        /// Amount of liquidity to deposit
        amount: u64,
    },

    /// Withdraw liquidity
    ///
    /// Accounts:
    /// [R] Liquidity account
    /// [W] Source provider account (for pool mint)
    /// [W] Destination provider account (for token mint)
    /// [W] Token account
    /// [W] Pool mint account
    /// [R] Market account
    /// [R] Market authority
    /// [RS] User transfer authority
    /// [R] Token program id
    LiquidityWithdraw {
        /// Amount of liquidity to withdraw
        amount: u64,
    },

    /// Create obligation token
    ///
    /// Accounts:
    /// [W] Obligation account to create - uninitialized
    /// [R] Liquidity account
    /// [R] Collateral account
    /// [R] Market account
    /// [R] Obligation authority (owner/market/liquidity/collateral combination)
    /// [RS] Obligation owner
    /// [R] Rent sysvar
    /// [R] Sytem program
    /// [R] Token program id
    CreateObligation,

    /// Deposit collateral token to obligation
    ///
    /// Accounts:
    /// [W] Obligation account
    /// [R] Collateral account
    /// [W] Source account (for collateral token mint)
    /// [W] Collateral token account
    /// [R] Market account
    /// [RS] User transfer authority
    /// [R] Token program id
    ObligationCollateralDeposit {
        /// Amount of collateral to deposit
        amount: u64,
    },

    /// Withdraw collateral token from obligation
    ///
    /// Accounts:
    /// [W] Obligation account
    /// [R] Liquidity account
    /// [R] Collateral account
    /// [W] Destination account (for collateral token mint)
    /// [W] Collateral token account
    /// [R] Market account
    /// [RS] Obligation owner
    /// [R] Market authority
    /// [R] Token program id
    /// [R] Liquidity oracle state account pubkey - optional
    /// [R] Collateral oracle state account pubkey - optional
    ObligationCollateralWithdraw {
        /// Amount of collateral to withdraw
        amount: u64,
    },

    /// Borrow liquidity token from obligation
    ///
    /// Accounts:
    /// [W] Obligation account
    /// [R] Liquidity account
    /// [R] Collateral account
    /// [W] Destination account (for liquidity token mint)
    /// [W] Liquidity token account
    /// [R] Market account
    /// [RS] Obligation owner
    /// [R] Market authority
    /// [R] Token program id
    /// [R] Liquidity oracle state account pubkey
    /// [R] Collateral oracle state account pubkey
    ObligationLiquidityBorrow {
        /// Amount of liquidity to borrow
        amount: u64,
    },

    /// Repay liquidity token to obligation
    ///
    /// Accounts:
    /// [W] Obligation account
    /// [R] Liquidity account
    /// [W] Source account (for liquidity token mint)
    /// [W] Liquidity token account
    /// [R] Market account
    /// [RS] User transfer authority
    /// [R] Token program id
    ObligationLiquidityRepay {
        /// Amount of liquidity to repay
        amount: u64,
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
    liquidity_oracle: &Option<Pubkey>,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::CreateLiquidityToken;
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let mut accounts = vec![
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
    if let Some(liquidity_oracle) = liquidity_oracle {
        accounts.push(AccountMeta::new_readonly(*liquidity_oracle, false));
    }

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
    market: &Pubkey,
    market_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::UpdateLiquidityToken { status };
    let data = init_data.try_to_vec()?;

    let accounts = vec![
        AccountMeta::new(*liquidity, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*market_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `CreateCollateralToken` instruction
#[allow(clippy::too_many_arguments)]
pub fn create_collateral_token(
    program_id: &Pubkey,
    ratio_initial: u64,
    ratio_healthy: u64,
    collateral: &Pubkey,
    token_mint: &Pubkey,
    token_account: &Pubkey,
    market: &Pubkey,
    market_owner: &Pubkey,
    collateral_oracle: &Option<Pubkey>,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::CreateCollateralToken {
        ratio_initial,
        ratio_healthy,
    };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let mut accounts = vec![
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
    if let Some(collateral_oracle) = collateral_oracle {
        accounts.push(AccountMeta::new_readonly(*collateral_oracle, false));
    }

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
    market: &Pubkey,
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
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*market_owner, true),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `LiquidityDeposit` instruction
#[allow(clippy::too_many_arguments)]
pub fn liquidity_deposit(
    program_id: &Pubkey,
    amount: u64,
    liquidity: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    token_account: &Pubkey,
    pool_mint: &Pubkey,
    market: &Pubkey,
    user_transfer_authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::LiquidityDeposit { amount };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let accounts = vec![
        AccountMeta::new_readonly(*liquidity, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(*user_transfer_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `LiquidityWithdraw` instruction
#[allow(clippy::too_many_arguments)]
pub fn liquidity_withdraw(
    program_id: &Pubkey,
    amount: u64,
    liquidity: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    token_account: &Pubkey,
    pool_mint: &Pubkey,
    market: &Pubkey,
    user_transfer_authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::LiquidityWithdraw { amount };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let accounts = vec![
        AccountMeta::new_readonly(*liquidity, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new(*token_account, false),
        AccountMeta::new(*pool_mint, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(*user_transfer_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `CreateObligation` instruction
pub fn create_obligation(
    program_id: &Pubkey,
    obligation: &Pubkey,
    liquidity: &Pubkey,
    collateral: &Pubkey,
    market: &Pubkey,
    owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::CreateObligation;
    let data = init_data.try_to_vec()?;
    let (obligation_authority, _) =
        find_obligation_authority(program_id, owner, market, liquidity, collateral);

    let accounts = vec![
        AccountMeta::new(*obligation, false),
        AccountMeta::new_readonly(*liquidity, false),
        AccountMeta::new_readonly(*collateral, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(obligation_authority, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `ObligationCollateralDeposit` instruction
#[allow(clippy::too_many_arguments)]
pub fn obligation_collateral_deposit(
    program_id: &Pubkey,
    amount: u64,
    obligation: &Pubkey,
    collateral: &Pubkey,
    source: &Pubkey,
    collateral_token_account: &Pubkey,
    market: &Pubkey,
    user_transfer_authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::ObligationCollateralDeposit { amount };
    let data = init_data.try_to_vec()?;

    let accounts = vec![
        AccountMeta::new(*obligation, false),
        AccountMeta::new_readonly(*collateral, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*collateral_token_account, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*user_transfer_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `ObligationCollateralWithdraw` instruction
#[allow(clippy::too_many_arguments)]
pub fn obligation_collateral_withdraw(
    program_id: &Pubkey,
    amount: u64,
    obligation: &Pubkey,
    liquidity: &Pubkey,
    collateral: &Pubkey,
    destination: &Pubkey,
    collateral_token_account: &Pubkey,
    market: &Pubkey,
    obligation_owner: &Pubkey,
    liquidity_oracle: &Option<Pubkey>,
    collateral_oracle: &Option<Pubkey>,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::ObligationCollateralWithdraw { amount };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let mut accounts = vec![
        AccountMeta::new(*obligation, false),
        AccountMeta::new_readonly(*liquidity, false),
        AccountMeta::new_readonly(*collateral, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new(*collateral_token_account, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*obligation_owner, true),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    match (liquidity_oracle, collateral_oracle) {
        (Some(liquidity_oracle), Some(collateral_oracle)) => {
            accounts.push(AccountMeta::new_readonly(*liquidity_oracle, false));
            accounts.push(AccountMeta::new_readonly(*collateral_oracle, false));
        }
        _ => (),
    }

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `ObligationLiquidityBorrow` instruction
#[allow(clippy::too_many_arguments)]
pub fn obligation_liquidity_borrow(
    program_id: &Pubkey,
    amount: u64,
    obligation: &Pubkey,
    liquidity: &Pubkey,
    collateral: &Pubkey,
    destination: &Pubkey,
    liquidity_token_account: &Pubkey,
    market: &Pubkey,
    obligation_owner: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::ObligationLiquidityBorrow { amount };
    let data = init_data.try_to_vec()?;
    let (market_authority, _) = find_program_address(program_id, market);

    let accounts = vec![
        AccountMeta::new(*obligation, false),
        AccountMeta::new(*liquidity, false),
        AccountMeta::new_readonly(*collateral, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new(*liquidity_token_account, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*obligation_owner, true),
        AccountMeta::new_readonly(market_authority, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}

/// Create `ObligationLiquidityRepay` instruction
#[allow(clippy::too_many_arguments)]
pub fn obligation_liquidity_repay(
    program_id: &Pubkey,
    amount: u64,
    obligation: &Pubkey,
    liquidity: &Pubkey,
    source: &Pubkey,
    liquidity_token_account: &Pubkey,
    market: &Pubkey,
    user_transfer_authority: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let init_data = LendingInstruction::ObligationLiquidityRepay { amount };
    let data = init_data.try_to_vec()?;

    let accounts = vec![
        AccountMeta::new(*obligation, false),
        AccountMeta::new(*liquidity, false),
        AccountMeta::new(*source, false),
        AccountMeta::new(*liquidity_token_account, false),
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*user_transfer_authority, true),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Ok(Instruction {
        program_id: *program_id,
        accounts,
        data,
    })
}
