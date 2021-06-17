//! Program state processor

use std::convert::TryInto;

use crate::{
    error::LendingError,
    find_obligation_authority, find_program_address,
    instruction::LendingInstruction,
    pyth::{self, Price, PriceType, Product},
    state::*,
};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
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
use spl_token::state::{Account, Mint};

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
        let mut market = Market::unpack_unchecked(&market_info.data.borrow())?;
        assert_uninitialized(&market)?;

        market.init(InitMarketParams {
            owner: *owner_info.key,
        });

        Market::pack(market, *market_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process CreateLiquidityToken instruction
    pub fn create_liquidity_token(
        program_id: &Pubkey,
        interest: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let oracle_product_info = next_account_info(account_info_iter)?;
        let oracle_price_info = next_account_info(account_info_iter)?;
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
        let mut market = Market::unpack(&market_info.data.borrow())?;

        if market.owner != *market_owner_info.key {
            msg!("Market owner provided does not match owner in the market state");
            return Err(ProgramError::InvalidArgument);
        }

        // Create liquidity account
        let seed = format!("liquidity{:?}", market.liquidity_tokens);
        let (authority, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        create_account_with_seed::<Liquidity>(
            program_id,
            market_owner_info.clone(),
            liquidity_info.clone(),
            market_authority_info.clone(),
            &seed,
            &authority,
            &[signers_seeds],
            rent,
        )?;

        // Get liquidity state
        let mut liquidity = Liquidity::unpack_unchecked(&liquidity_info.data.borrow())?;
        assert_uninitialized(&liquidity)?;

        let token_mint = Mint::unpack(&token_mint_info.data.borrow())?;

        let oracle_product_data = oracle_product_info.try_borrow_data()?;
        let oracle_product = pyth::load::<Product>(&oracle_product_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if oracle_product.magic != pyth::MAGIC {
            msg!("Pyth product account provided is not a valid Pyth account");
            return Err(LendingError::InvalidOracleConfig.into());
        }
        if oracle_product.ver != pyth::VERSION_1 {
            msg!("Pyth product account provided has a different version than expected");
            return Err(LendingError::InvalidOracleConfig.into());
        }
        if oracle_product.atype != pyth::AccountType::Product as u32 {
            msg!("Pyth product account provided is not a valid Pyth product account");
            return Err(LendingError::InvalidOracleConfig.into());
        }

        let oracle_price_pubkey_bytes: &[u8; 32] = oracle_price_info
            .key
            .as_ref()
            .try_into()
            .map_err(|_| ProgramError::InvalidArgument)?;

        if &oracle_product.px_acc.val != oracle_price_pubkey_bytes {
            msg!("Pyth product price account does not match the Pyth price provided");
            return Err(LendingError::InvalidOracleConfig.into());
        }

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

        // Update liquidity state & increase liquidity tokens counter
        liquidity.init(InitLiquidityParams {
            market: *market_info.key,
            token_mint: *token_mint_info.key,
            token_account: *token_account_info.key,
            pool_mint: *pool_mint_info.key,
            oracle: *oracle_price_info.key,
            interest,
        });
        market.increase_liquidity_tokens();

        Liquidity::pack(liquidity, *liquidity_info.data.borrow_mut())?;
        Market::pack(market, *market_info.data.borrow_mut())?;

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
        let market_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;

        if !market_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Get market state
        let market = Market::unpack(&market_info.data.borrow())?;

        if market.owner != *market_owner_info.key {
            msg!("Market owner provided does not match owner in the market state");
            return Err(ProgramError::InvalidArgument);
        }

        // Get liquidity state
        let mut liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.market != *market_info.key {
            msg!("Liquidity market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Update liquidity state
        liquidity.status = status;

        Liquidity::pack(liquidity, *liquidity_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process CreateCollateralToken instruction
    pub fn create_collateral_token(
        program_id: &Pubkey,
        ratio_initial: u64,
        ratio_healthy: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let collateral_info = next_account_info(account_info_iter)?;
        let token_mint_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let oracle_product_info = next_account_info(account_info_iter)?;
        let oracle_price_info = next_account_info(account_info_iter)?;
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
        let mut market = Market::unpack(&market_info.data.borrow())?;

        if market.owner != *market_owner_info.key {
            msg!("Market owner provided does not match owner in the market state");
            return Err(ProgramError::InvalidArgument);
        }

        // Create collateral account
        let seed = format!("collateral{:?}", market.collateral_tokens);
        let (authority, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        create_account_with_seed::<Collateral>(
            program_id,
            market_owner_info.clone(),
            collateral_info.clone(),
            market_authority_info.clone(),
            &seed,
            &authority,
            &[signers_seeds],
            rent,
        )?;

        // Get collateral state
        let mut collateral = Collateral::unpack_unchecked(&collateral_info.data.borrow())?;
        assert_uninitialized(&collateral)?;

        let oracle_product_data = oracle_product_info.try_borrow_data()?;
        let oracle_product = pyth::load::<Product>(&oracle_product_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        if oracle_product.magic != pyth::MAGIC {
            msg!("Pyth product account provided is not a valid Pyth account");
            return Err(LendingError::InvalidOracleConfig.into());
        }
        if oracle_product.ver != pyth::VERSION_1 {
            msg!("Pyth product account provided has a different version than expected");
            return Err(LendingError::InvalidOracleConfig.into());
        }
        if oracle_product.atype != pyth::AccountType::Product as u32 {
            msg!("Pyth product account provided is not a valid Pyth product account");
            return Err(LendingError::InvalidOracleConfig.into());
        }

        let oracle_price_pubkey_bytes: &[u8; 32] = oracle_price_info
            .key
            .as_ref()
            .try_into()
            .map_err(|_| ProgramError::InvalidArgument)?;

        if &oracle_product.px_acc.val != oracle_price_pubkey_bytes {
            msg!("Pyth product price account does not match the Pyth price provided");
            return Err(LendingError::InvalidOracleConfig.into());
        }

        // Initialize token account for spl token
        spl_initialize_account(
            token_account_info.clone(),
            token_mint_info.clone(),
            market_authority_info.clone(),
            rent_info.clone(),
        )?;

        // Update collateral state & increase collateral tokens counter
        collateral.init(InitCollateralParams {
            market: *market_info.key,
            token_mint: *token_mint_info.key,
            token_account: *token_account_info.key,
            ratio_initial,
            ratio_healthy,
            oracle: *oracle_price_info.key,
        });
        market.increase_collateral_tokens();

        Collateral::pack(collateral, *collateral_info.data.borrow_mut())?;
        Market::pack(market, *market_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process UpdateCollateralToken instruction
    pub fn update_collateral_token(
        _program_id: &Pubkey,
        status: CollateralStatus,
        ratio_initial: u64,
        ratio_healthy: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let collateral_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_owner_info = next_account_info(account_info_iter)?;

        if !market_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Get market state
        let market = Market::unpack(&market_info.data.borrow())?;

        if market.owner != *market_owner_info.key {
            msg!("Market owner provided does not match owner in the market state");
            return Err(ProgramError::InvalidArgument);
        }

        // Get collateral state
        let mut collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        if collateral.market != *market_info.key {
            msg!("Collateral market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Update collateral state
        collateral.status = status;
        collateral.ratio_initial = ratio_initial;
        collateral.ratio_healthy = ratio_healthy;

        Collateral::pack(collateral, *collateral_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process LiquidityDeposit instruction
    pub fn liquidity_deposit(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get liquidity state
        let liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.market != *market_info.key {
            msg!("Liquidity market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        if liquidity.token_account != *token_account_info.key {
            msg!("Liquidity token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        if liquidity.pool_mint != *pool_mint_info.key {
            msg!("Liquidity pool mint does not match the pool mint provided");
            return Err(ProgramError::InvalidArgument);
        }

        // TODO: We can store total values in the liquidity state
        let token_account_amount =
            Account::unpack_unchecked(&token_account_info.data.borrow())?.amount;
        let pool_mint_supply = Mint::unpack_unchecked(&pool_mint_info.data.borrow())?.supply;

        // Transfer liquidity from source provider to token account
        spl_token_transfer(
            source_info.clone(),
            token_account_info.clone(),
            user_transfer_authority_info.clone(),
            amount,
            &[],
        )?;

        let (_, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        // Mint to destination provider pool token
        spl_token_mint_to(
            pool_mint_info.clone(),
            destination_info.clone(),
            market_authority_info.clone(),
            liquidity.calc_deposit_exchange_amount(
                amount,
                token_account_amount,
                pool_mint_supply,
            )?,
            &[signers_seeds],
        )?;

        Ok(())
    }

    /// Process LiquidityWithdraw instruction
    pub fn liquidity_withdraw(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let liquidity_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let token_account_info = next_account_info(account_info_iter)?;
        let pool_mint_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get liquidity state
        let liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.market != *market_info.key {
            msg!("Liquidity market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        if liquidity.token_account != *token_account_info.key {
            msg!("Liquidity token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        if liquidity.pool_mint != *pool_mint_info.key {
            msg!("Liquidity pool mint does not match the pool mint provided");
            return Err(ProgramError::InvalidArgument);
        }

        let token_account_amount =
            Account::unpack_unchecked(&token_account_info.data.borrow())?.amount;
        let pool_mint_supply = Mint::unpack_unchecked(&pool_mint_info.data.borrow())?.supply;

        // Burn from soruce provider pool token
        spl_token_burn(
            pool_mint_info.clone(),
            source_info.clone(),
            user_transfer_authority_info.clone(),
            amount,
            &[],
        )?;

        let (_, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        // Transfer liquidity from token account to destination provider
        spl_token_transfer(
            token_account_info.clone(),
            destination_info.clone(),
            market_authority_info.clone(),
            liquidity.calc_withdraw_exchange_amount(
                amount,
                token_account_amount,
                pool_mint_supply,
            )?,
            &[signers_seeds],
        )?;

        Ok(())
    }

    /// Process CreateObligation instruction
    pub fn create_obligation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let liquidity_info = next_account_info(account_info_iter)?;
        let collateral_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let obligation_authority_info = next_account_info(account_info_iter)?;
        let obligation_owner_info = next_account_info(account_info_iter)?;
        let rent_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _system_program_info = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(rent_info)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if !obligation_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if liquidity_info.owner != program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if collateral_info.owner != program_id {
            msg!("Collateral provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get liquidity state
        let liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.market != *market_info.key {
            msg!("Liquidity market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        if liquidity.status != LiquidityStatus::Active {
            msg!("Liquidity does not active");
            return Err(ProgramError::InvalidAccountData);
        }

        // Get collateral state
        let collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        if collateral.market != *market_info.key {
            msg!("Collateral market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        if collateral.status != CollateralStatus::Active {
            msg!("Collateral does not active");
            return Err(ProgramError::InvalidAccountData);
        }

        let (obligation_authority, bump_seed) = find_obligation_authority(
            program_id,
            obligation_owner_info.key,
            market_info.key,
            liquidity_info.key,
            collateral_info.key,
        );
        // TODO: refactor in the future
        let signers_seeds = &[
            &obligation_owner_info.key.to_bytes()[..32],
            &market_info.key.to_bytes()[..32],
            &liquidity_info.key.to_bytes()[..32],
            &collateral_info.key.to_bytes()[..32],
            &[bump_seed],
        ];

        // Create obligation account
        create_account_with_seed::<Obligation>(
            program_id,
            obligation_owner_info.clone(),
            obligation_info.clone(),
            obligation_authority_info.clone(),
            "obligation",
            &obligation_authority,
            &[signers_seeds],
            rent,
        )?;

        // Get obligation state
        let mut obligation = Obligation::unpack_unchecked(&obligation_info.data.borrow())?;
        assert_uninitialized(&obligation)?;

        // Init obligation state
        obligation.init(InitObligationParams {
            market: *market_info.key,
            owner: *obligation_owner_info.key,
            liquidity: *liquidity_info.key,
            collateral: *collateral_info.key,
            interest_slot: clock.slot,
        });

        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process ObligationCollateralDeposit instruction
    pub fn obligation_collateral_deposit(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let collateral_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let collateral_token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if collateral_info.owner != program_id {
            msg!("Collateral provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if obligation_info.owner != program_id {
            msg!("Obligation provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get obligation state
        let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;

        if obligation.collateral != *collateral_info.key {
            msg!("Obligation collateral does not match the collateral provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.market != *market_info.key {
            msg!("Obligation market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get collateral state
        let collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        if collateral.token_account != *collateral_token_account_info.key {
            msg!("Collateral token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        obligation.collateral_deposit(amount)?;
        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;

        // Transfer collateral from source borrower to token account
        spl_token_transfer(
            source_info.clone(),
            collateral_token_account_info.clone(),
            user_transfer_authority_info.clone(),
            amount,
            &[],
        )?;

        Ok(())
    }

    /// Process ObligationCollateralWithdraw instruction
    pub fn obligation_collateral_withdraw(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let liquidity_info = next_account_info(account_info_iter)?;
        let collateral_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let collateral_token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let obligation_owner_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let liquidity_oracle_info = next_account_info(account_info_iter)?;
        let collateral_oracle_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !obligation_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if liquidity_info.owner != program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if collateral_info.owner != program_id {
            msg!("Collateral provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if obligation_info.owner != program_id {
            msg!("Obligation provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get obligation state
        let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;

        if obligation.owner != *obligation_owner_info.key {
            msg!("Obligation owner does not match the owner provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.liquidity != *liquidity_info.key {
            msg!("Obligation liquidity does not match the liquidity provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.collateral != *collateral_info.key {
            msg!("Obligation collateral does not match the collateral provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.market != *market_info.key {
            msg!("Obligation market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get liquidity state
        let liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        // Get collateral state
        let collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        if collateral.token_account != *collateral_token_account_info.key {
            msg!("Collateral token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        let clock = &Clock::from_account_info(clock_info)?;

        let (liquidity_market_price, collateral_market_price) = get_prices_from_oracles(
            &liquidity.oracle,
            &collateral.oracle,
            liquidity_oracle_info,
            collateral_oracle_info,
            clock,
        )?;

        obligation.collateral_withdraw(amount)?;

        // Check obligation ratio
        collateral
            .check_ratio(obligation.calc_ratio(liquidity_market_price, collateral_market_price)?)?;

        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;

        let (_, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        // Transfer collateral from token account to destination borrower
        spl_token_transfer(
            collateral_token_account_info.clone(),
            destination_info.clone(),
            market_authority_info.clone(),
            amount,
            &[signers_seeds],
        )?;

        Ok(())
    }

    /// Process ObligationLiquidityBorrow instruction
    pub fn obligation_liquidity_borrow(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let liquidity_info = next_account_info(account_info_iter)?;
        let collateral_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let liquidity_token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let obligation_owner_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let liquidity_oracle_info = next_account_info(account_info_iter)?;
        let collateral_oracle_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if !obligation_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if liquidity_info.owner != program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if collateral_info.owner != program_id {
            msg!("Collateral provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if obligation_info.owner != program_id {
            msg!("Obligation provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get obligation state
        let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;

        if obligation.owner != *obligation_owner_info.key {
            msg!("Obligation owner does not match the owner provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.liquidity != *liquidity_info.key {
            msg!("Obligation liquidity does not match the liquidity provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.collateral != *collateral_info.key {
            msg!("Obligation collateral does not match the collateral provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.market != *market_info.key {
            msg!("Obligation market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get collateral state
        let collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        // Get liquidity state
        let mut liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.token_account != *liquidity_token_account_info.key {
            msg!("Liquidity token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        let (liquidity_market_price, collateral_market_price) = get_prices_from_oracles(
            &liquidity.oracle,
            &collateral.oracle,
            liquidity_oracle_info,
            collateral_oracle_info,
            clock,
        )?;

        obligation.update_interest_amount(clock.slot, liquidity.interest)?;
        obligation.update_slot(clock.slot);

        obligation.liquidity_borrow(amount)?;
        liquidity.borrow(amount)?;
        collateral
            .check_ratio(obligation.calc_ratio(liquidity_market_price, collateral_market_price)?)?;

        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;
        Liquidity::pack(liquidity, *liquidity_info.data.borrow_mut())?;

        let (_, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        // Transfer liquidity from token account to destination borrower
        spl_token_transfer(
            liquidity_token_account_info.clone(),
            destination_info.clone(),
            market_authority_info.clone(),
            amount,
            &[signers_seeds],
        )?;

        Ok(())
    }

    /// Process ObligationLiquidityRepay instruction
    pub fn obligation_liquidity_repay(
        program_id: &Pubkey,
        amount: u64,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let liquidity_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let liquidity_token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(clock_info)?;

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if liquidity_info.owner != program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if obligation_info.owner != program_id {
            msg!("Obligation provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get obligation state
        let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;

        if obligation.liquidity != *liquidity_info.key {
            msg!("Obligation liquidity does not match the liquidity provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.market != *market_info.key {
            msg!("Obligation market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get liquidity state
        let mut liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.token_account != *liquidity_token_account_info.key {
            msg!("Collateral token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        let repay_limit = obligation.amount_liquidity_borrowed;
        if amount > repay_limit {
            msg!("Repay limit exceeded");
            return Err(ProgramError::InvalidArgument);
        }

        obligation.update_interest_amount(clock.slot, liquidity.interest)?;
        obligation.update_slot(clock.slot);

        obligation.liquidity_repay(amount)?;
        liquidity.repay(amount)?;

        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;
        Liquidity::pack(liquidity, *liquidity_info.data.borrow_mut())?;

        // Transfer liquidity from source borrower to token account
        spl_token_transfer(
            source_info.clone(),
            liquidity_token_account_info.clone(),
            user_transfer_authority_info.clone(),
            amount,
            &[],
        )?;

        Ok(())
    }

    /// Process LiquidateObligation instruction
    pub fn liquidate_obligation(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let obligation_info = next_account_info(account_info_iter)?;
        let source_info = next_account_info(account_info_iter)?;
        let destination_info = next_account_info(account_info_iter)?;
        let liquidity_info = next_account_info(account_info_iter)?;
        let collateral_info = next_account_info(account_info_iter)?;
        let liquidity_token_account_info = next_account_info(account_info_iter)?;
        let collateral_token_account_info = next_account_info(account_info_iter)?;
        let market_info = next_account_info(account_info_iter)?;
        let user_transfer_authority_info = next_account_info(account_info_iter)?;
        let market_authority_info = next_account_info(account_info_iter)?;
        let liquidity_oracle_info = next_account_info(account_info_iter)?;
        let collateral_oracle_info = next_account_info(account_info_iter)?;
        let clock_info = next_account_info(account_info_iter)?;
        let _token_program_info = next_account_info(account_info_iter)?;

        if !user_transfer_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if market_info.owner != program_id {
            msg!("Market provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if liquidity_info.owner != program_id {
            msg!("Liquidity provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if collateral_info.owner != program_id {
            msg!("Collateral provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        if obligation_info.owner != program_id {
            msg!("Obligation provided is not owned by the market program");
            return Err(LendingError::InvalidAccountOwner.into());
        }

        // Get obligation state
        let mut obligation = Obligation::unpack(&obligation_info.data.borrow())?;

        if obligation.liquidity != *liquidity_info.key {
            msg!("Obligation liquidity does not match the liquidity provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.collateral != *collateral_info.key {
            msg!("Obligation collateral does not match the collateral provided");
            return Err(ProgramError::InvalidArgument);
        }

        if obligation.market != *market_info.key {
            msg!("Obligation market does not match the market provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get liquidity state
        let mut liquidity = Liquidity::unpack(&liquidity_info.data.borrow())?;

        if liquidity.token_account != *liquidity_token_account_info.key {
            msg!("Liquidity token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        // Get collateral state
        let collateral = Collateral::unpack(&collateral_info.data.borrow())?;

        if collateral.token_account != *collateral_token_account_info.key {
            msg!("Collateral token account does not match the token account provided");
            return Err(ProgramError::InvalidArgument);
        }

        let clock = &Clock::from_account_info(clock_info)?;

        let (liquidity_market_price, collateral_market_price) = get_prices_from_oracles(
            &liquidity.oracle,
            &collateral.oracle,
            liquidity_oracle_info,
            collateral_oracle_info,
            clock,
        )?;

        // 0. Check that we can liquidate
        collateral.check_healthy(
            obligation.calc_ratio(liquidity_market_price, collateral_market_price)?,
        )?;

        // 1. Repay
        let repay_amount = obligation.amount_liquidity_borrowed;
        obligation.liquidity_repay(repay_amount)?;
        liquidity.repay(repay_amount)?;

        Liquidity::pack(liquidity, *liquidity_info.data.borrow_mut())?;

        // Transfer liquidity from source liquidator to token account
        spl_token_transfer(
            source_info.clone(),
            liquidity_token_account_info.clone(),
            user_transfer_authority_info.clone(),
            repay_amount,
            &[],
        )?;

        // 2. Withdraw
        let withdraw_amount = obligation.amount_collateral_deposited;
        obligation.collateral_withdraw(withdraw_amount)?;

        Obligation::pack(obligation, *obligation_info.data.borrow_mut())?;

        let (_, bump_seed) = find_program_address(program_id, market_info.key);
        let signers_seeds = &[&market_info.key.to_bytes()[..32], &[bump_seed]];

        // Transfer collateral from token account to destination borrower
        spl_token_transfer(
            collateral_token_account_info.clone(),
            destination_info.clone(),
            market_authority_info.clone(),
            withdraw_amount,
            &[signers_seeds],
        )?;

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

            LendingInstruction::CreateLiquidityToken { interest } => {
                msg!("LendingInstruction: CreateLiquidityToken");
                Self::create_liquidity_token(program_id, interest, accounts)
            }

            LendingInstruction::UpdateLiquidityToken { status } => {
                msg!("LendingInstruction: UpdateLiquidityToken");
                Self::update_liquidity_token(program_id, status, accounts)
            }

            LendingInstruction::CreateCollateralToken {
                ratio_initial,
                ratio_healthy,
            } => {
                msg!("LendingInstruction: CreateCollateralToken");
                Self::create_collateral_token(program_id, ratio_initial, ratio_healthy, accounts)
            }

            LendingInstruction::UpdateCollateralToken {
                status,
                ratio_initial,
                ratio_healthy,
            } => {
                msg!("LendingInstruction: UpdateCollateralToken");
                Self::update_collateral_token(
                    program_id,
                    status,
                    ratio_initial,
                    ratio_healthy,
                    accounts,
                )
            }

            LendingInstruction::LiquidityDeposit { amount } => {
                msg!("LendingInstruction: LiquidityDeposit");
                Self::liquidity_deposit(program_id, amount, accounts)
            }

            LendingInstruction::LiquidityWithdraw { amount } => {
                msg!("LendingInstruction: LiquidityWithdraw");
                Self::liquidity_withdraw(program_id, amount, accounts)
            }

            LendingInstruction::CreateObligation => {
                msg!("LendingInstruction: CreateObligation");
                Self::create_obligation(program_id, accounts)
            }

            LendingInstruction::ObligationCollateralDeposit { amount } => {
                msg!("LendingInstruction: ObligationCollateralDeposit");
                Self::obligation_collateral_deposit(program_id, amount, accounts)
            }

            LendingInstruction::ObligationCollateralWithdraw { amount } => {
                msg!("LendingInstruction: ObligationCollateralWithdraw");
                Self::obligation_collateral_withdraw(program_id, amount, accounts)
            }

            LendingInstruction::ObligationLiquidityBorrow { amount } => {
                msg!("LendingInstruction: ObligationLiquidityBorrow");
                Self::obligation_liquidity_borrow(program_id, amount, accounts)
            }

            LendingInstruction::ObligationLiquidityRepay { amount } => {
                msg!("LendingInstruction: ObligationLiquidityRepay");
                Self::obligation_liquidity_repay(program_id, amount, accounts)
            }

            LendingInstruction::LiquidateObligation => {
                msg!("LendingInstruction: LiquidateObligation");
                Self::liquidate_obligation(program_id, accounts)
            }
        }
    }
}

/// Create account with seed
#[allow(clippy::too_many_arguments)]
pub fn create_account_with_seed<'a, S: Pack>(
    program_id: &Pubkey,
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    base: AccountInfo<'a>,
    seed: &str,
    authority: &Pubkey,
    signers_seeds: &[&[&[u8]]],
    rent: &Rent,
) -> ProgramResult {
    if *authority != *base.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let generated_pubkey = Pubkey::create_with_seed(&base.key, seed, program_id)?;
    if generated_pubkey != *to.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let ix = system_instruction::create_account_with_seed(
        from.key,
        to.key,
        &base.key,
        seed,
        rent.minimum_balance(S::LEN),
        S::LEN as u64,
        program_id,
    );

    invoke_signed(&ix, &[from, to, base], signers_seeds)
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

/// SPL transfer instruction.
pub fn spl_token_transfer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        &spl_token::id(),
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[source, destination, authority], signers_seeds)
}

/// SPL mint instruction.
pub fn spl_token_mint_to<'a>(
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, destination, authority], signers_seeds)
}

/// SPL burn instruction.
pub fn spl_token_burn<'a>(
    mint: AccountInfo<'a>,
    account: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::burn(
        &spl_token::id(),
        account.key,
        mint.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, account, authority], signers_seeds)
}

/// Fetch prices from oracle accounts
pub fn get_prices_from_oracles(
    liquidity_oracle: &Pubkey,
    collateral_oracle: &Pubkey,
    liquidity_oracle_info: &AccountInfo,
    collateral_oracle_info: &AccountInfo,
    clock: &Clock,
) -> Result<(u64, u64), ProgramError> {
    if liquidity_oracle != liquidity_oracle_info.key {
        return Err(LendingError::InvalidOracle.into());
    }

    if collateral_oracle != collateral_oracle_info.key {
        return Err(LendingError::InvalidOracle.into());
    }

    let liquidity_market_price = get_pyth_price(liquidity_oracle_info, clock)?;
    let collateral_market_price = get_pyth_price(collateral_oracle_info, clock)?;

    msg!(
        "Market prices: {} {}",
        liquidity_market_price,
        collateral_market_price,
    );

    Ok((liquidity_market_price, collateral_market_price))
}

fn get_pyth_price(pyth_price_info: &AccountInfo, clock: &Clock) -> Result<u64, ProgramError> {
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 5;

    let pyth_price_data = pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth::load::<Price>(&pyth_price_data).unwrap();

    if pyth_price.ptype != PriceType::Price {
        msg!("Oracle price type is invalid");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let slots_elapsed = clock
        .slot
        .checked_sub(pyth_price.valid_slot)
        .ok_or(LendingError::MathOverflow)?;
    if slots_elapsed >= STALE_AFTER_SLOTS_ELAPSED {
        msg!("Oracle price is stale");
        return Err(LendingError::InvalidOracleConfig.into());
    }

    let price: u64 = pyth_price.agg.price.try_into().map_err(|_| {
        msg!("Oracle price cannot be negative");
        LendingError::InvalidOracleConfig
    })?;

    Ok(price)
}

fn assert_rent_exempt(rent: &Rent, account_info: &AccountInfo) -> ProgramResult {
    if !rent.is_exempt(account_info.lamports(), account_info.data_len()) {
        msg!(&rent.minimum_balance(account_info.data_len()).to_string());
        Err(ProgramError::AccountNotRentExempt)
    } else {
        Ok(())
    }
}

fn assert_uninitialized<T: IsInitialized>(account: &T) -> ProgramResult {
    if account.is_initialized() {
        Err(ProgramError::AccountAlreadyInitialized)
    } else {
        Ok(())
    }
}
