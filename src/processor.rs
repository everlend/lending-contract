//! Program state processor

use crate::{error::LendingError, instruction::LendingInstruction};
use crate::{find_program_address, state::*};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
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
    pub fn init_market(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let market_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        if !owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        assert_rent_exempt(rent, market_info)?;

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get market state
        let mut market = assert_uninitialized::<Market>(market_info)?;

        market.init(InitMarketParams {
            owner: *owner_info.key,
        });
        market.serialize(&mut *market_info.try_borrow_mut_data()?)?;

        Ok(())
    }

    /// Process CreateLiquidityToken instruction
    pub fn create_liquidity_token(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let _system_program_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;

        if !market_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get market state
        let mut market = Market::try_from_slice(&market_info.data.borrow())?;

        if market.owner != *market_owner_info.key {
            msg!("Market owner provided does not match owner in the market state");
            return Err(ProgramError::InvalidArgument.into());
        }

        // Create liquidity account
        let seed = format!("liquidity{:?}", market.liquidity_tokens);
        create_liquidity_account(
            program_id,
            market_info.key,
            market_owner_info.clone(),
            liquidity_info.clone(),
            market_authority_info.clone(),
            &seed,
            rent,
        )?;

        // Get liquidity state
        let mut liquidity = assert_uninitialized::<Liquidity>(liquidity_info)?;

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

        // Update liquidity state & increment liquidity tokens counter
        liquidity.init(InitLiquidityParams {
            market: *market_info.key,
            token_mint: *token_mint_info.key,
            token_account: *token_account_info.key,
            pool_mint: *pool_mint_info.key,
        });
        market.increment_liquidity_tokens();

        liquidity.serialize(&mut *liquidity_info.data.borrow_mut())?;
        market.serialize(&mut *market_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process UpdateLiquidityToken instruction
    pub fn update_liquidity_token(
        _program_id: &Pubkey,
        status: LiquidityStatus,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;

        if !market_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Get liquidity state
        let mut liquidity = Liquidity::try_from_slice(&liquidity_info.data.borrow())?;

        // Update liquidity state
        liquidity.status = status;

        liquidity.serialize(&mut *liquidity_info.data.borrow_mut())?;

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

            LendingInstruction::UpdateLiquidityToken { status } => {
                msg!("LendingInstruction: UpdateLiquidityToken");
                Self::update_liquidity_token(program_id, status, accounts)
            }
        }
    }
}

/// Create account with seed
pub fn create_liquidity_account<'a>(
    program_id: &Pubkey,
    market: &Pubkey,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    base: AccountInfo<'a>,
    seed: &str,
    rent: &Rent,
) -> ProgramResult {
    let (authority, bump_seed) = find_program_address(program_id, market);
    let signature = &[&market.to_bytes()[..32], &[bump_seed]];

    if authority != *base.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let generated_liquidity_pubkey = Pubkey::create_with_seed(&base.key, seed, program_id)?;
    if generated_liquidity_pubkey != *to.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let ix = system_instruction::create_account_with_seed(
        from.key,
        to.key,
        &base.key,
        seed,
        rent.minimum_balance(Liquidity::LEN),
        Liquidity::LEN as u64,
        program_id,
    );

    invoke_signed(&ix, &[from, to, base], &[signature])
}

/// Initialize SPL accont instruction.
pub fn spl_initialize_account<'a>(
    account: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    rent: AccountInfo<'a>,
) -> ProgramResult {
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
) -> ProgramResult {
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
        Err(ProgramError::AccountNotRentExempt.into())
    } else {
        Ok(())
    }
}

fn assert_uninitialized<T: BorshDeserialize + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::try_from_slice(&account_info.data.borrow())?;
    if account.is_initialized() {
        Err(ProgramError::AccountAlreadyInitialized.into())
    } else {
        Ok(account)
    }
}
