//! Program state processor

use crate::state::*;
use crate::{error::LendingError, instruction::LendingInstruction};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::get_packed_len,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::state::Mint;

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Process InitMarket instruction
    pub fn init_market(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let market_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;

        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let rent = &Rent::from_account_info(rent_info)?;

        assert_rent_exempt(rent, market_info)?;
        let mut market = assert_uninitialized::<Market>(market_info)?;

        if market_info.owner != _program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        market.init(InitMarketParams {
            owner: *owner_info.key,
        });
        market.serialize(&mut *market_info.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// Process CreateLiquidityToken instruction
    pub fn create_liquidity_token(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;

        let rent = &Rent::from_account_info(rent_info)?;

        let mut market = Market::try_from_slice(&market_info.data.borrow())?;
        if market_info.owner != _program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Create liquidity account
        let seed: &str = &vec!["liquidity", &market.liquidity_tokens.to_string()].join("");
        create_liquidity_account(
            market_info.clone(),
            liquidity_info.clone(),
            market_authority_info.clone(),
            seed,
            rent.minimum_balance(get_packed_len::<Liquidity>()),
            get_packed_len::<Liquidity>() as u64,
            _program_id,
        )?;

        assert_rent_exempt(rent, liquidity_info)?;
        let mut liquidity = assert_uninitialized::<Liquidity>(liquidity_info)?;
        if liquidity_info.owner != _program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        let token_mint = Mint::unpack(&token_mint_info.data.borrow())?;

        // Initialize token account for spl token
        spl_initialize_account(
            token_account_info.clone(),
            token_mint_info.clone(),
            market_authority_info.clone(),
            rent_info.clone(),
        )?;

        // Initialize mint (token) for pool
        spl_initialize_mint(
            pool_mint_info.clone(),
            market_authority_info.clone(),
            rent_info.clone(),
            token_mint.decimals,
        )?;

        liquidity.init(InitLiquidityParams {
            market: *market_info.key,
            token_mint: *token_mint_info.key,
            token_account: *token_account_info.key,
            pool_mint: *pool_mint_info.key,
        });
        market.increment_liquidity_tokens();

        liquidity.serialize(&mut *liquidity_info.try_borrow_mut_data()?)?;
        market.serialize(&mut *market_info.try_borrow_mut_data()?)?;

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
                Self::init_market(program_id, accounts)
            }

            LendingInstruction::CreateLiquidityToken => {
                msg!("LendingInstruction: CreateLiquidityToken");
                Self::create_liquidity_token(program_id, accounts)
            }
        }
    }
}

/// Create account with seed
fn create_liquidity_account<'a>(
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    base: AccountInfo<'a>,
    seed: &str,
    required_lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> ProgramResult {
    let generated_liquidity_pubkey = Pubkey::create_with_seed(&base.key, &seed, owner)?;
    if generated_liquidity_pubkey != *to.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let signers = &[&base.key.to_bytes()[..32]];

    let ix = system_instruction::create_account_with_seed(
        &from.key,
        &to.key,
        &base.key,
        &seed,
        required_lamports,
        space,
        owner,
    );

    invoke_signed(&ix, &[from.clone(), to.clone(), base.clone()], &[signers])
}

/// Initialize SPL accont instruction.
pub fn spl_initialize_account<'a>(
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::initialize_account(
        &spl_token::id(),
        account.key,
        mint.key,
        authority.key,
    )?;

    invoke(&ix, &[account, mint, authority, rent])
}

/// Initialize SPL mint instruction.
pub fn spl_initialize_mint<'a>(
    mint: AccountInfo<'a>,
    mint_authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
    decimals: u8,
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        mint.key,
        mint_authority.key,
        None,
        decimals,
    )?;

    invoke(&ix, &[mint, rent])
}

fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(LendingError::NotRentExempt.into())
    } else {
        Ok(())
    }
}

fn assert_uninitialized<T: BorshDeserialize + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::try_from_slice(&account_info.data.borrow())?;
    if account.is_initialized() {
        Err(LendingError::AlreadyInitialized.into())
    } else {
        Ok(account)
    }
}
