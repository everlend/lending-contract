#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::{CollateralStatus, LiquidityStatus, PROGRAM_VERSION, RATIO_POWER};
use solana_program_test::*;
use solana_sdk::{signature::Keypair, signer::Signer};
use utils::*;

async fn setup() -> (
    ProgramTestContext,
    MarketInfo,
    LiquidityInfo,
    CollateralInfo,
) {
    let mut context = program_test().start_with_context().await;

    let market_info = MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let liquidity_info = market_info
        .create_liquidity_token(&mut context)
        .await
        .unwrap();

    let collateral_info = market_info
        .create_collateral_token(&mut context)
        .await
        .unwrap();

    liquidity_info
        .update(&mut context, LiquidityStatus::Active, &market_info)
        .await
        .unwrap();

    collateral_info
        .update(
            &mut context,
            CollateralStatus::Active,
            collateral::RATIO_INITIAL,
            collateral::RATIO_HEALTHY,
            &market_info,
        )
        .await
        .unwrap();

    (context, market_info, liquidity_info, collateral_info)
}

async fn prepare_borrower(
    context: &mut ProgramTestContext,
    market_info: &MarketInfo,
    liquidity_info: &LiquidityInfo,
    collateral_info: &CollateralInfo,
    mint_amount: u64,
) -> (ObligationInfo, Keypair, Keypair) {
    let obligation_info = ObligationInfo::new(market_info, liquidity_info, collateral_info);
    obligation_info
        .create(context, &market_info, &liquidity_info, &collateral_info)
        .await
        .unwrap();

    // Create accounts (collateral, liquidity) for borrower
    let borrower_collateral = Keypair::new();
    let borrower_liquidity = Keypair::new();

    create_token_account(
        context,
        &borrower_collateral,
        &collateral_info.token_mint.pubkey(),
        &obligation_info.owner.pubkey(),
    )
    .await
    .unwrap();

    create_token_account(
        context,
        &borrower_liquidity,
        &liquidity_info.token_mint.pubkey(),
        &obligation_info.owner.pubkey(),
    )
    .await
    .unwrap();

    mint_tokens(
        context,
        &collateral_info.token_mint.pubkey(),
        &borrower_collateral.pubkey(),
        &market_info.owner,
        mint_amount,
    )
    .await
    .unwrap();

    // Deposit liquidity from provider
    let provider_actor = ProviderActor::new();
    let (source, destination) = provider_actor
        .create_liquidity_accounts(context, &liquidity_info)
        .await
        .unwrap();
    provider_actor
        .deposit(
            context,
            &market_info,
            &liquidity_info,
            &source,
            &destination,
            999999,
        )
        .await;

    (obligation_info, borrower_collateral, borrower_liquidity)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;

    let obligation_info = ObligationInfo::new(&market_info, &liquidity_info, &collateral_info);
    obligation_info
        .create(
            &mut context,
            &market_info,
            &liquidity_info,
            &collateral_info,
        )
        .await
        .unwrap();

    let obligation = obligation_info.get_data(&mut context).await;

    assert_eq!(obligation.owner, obligation_info.owner.pubkey());
    assert_eq!(obligation.version, PROGRAM_VERSION);
}

#[tokio::test]
async fn collateral_deposit() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;
    let (obligation_info, borrower_collateral, _) = prepare_borrower(
        &mut context,
        &market_info,
        &liquidity_info,
        &collateral_info,
        99999,
    )
    .await;

    const DEPOSIT_AMOUNT: u64 = 10000;
    obligation_info
        .collateral_deposit(
            &mut context,
            &market_info,
            &collateral_info,
            DEPOSIT_AMOUNT,
            &borrower_collateral.pubkey(),
        )
        .await
        .unwrap();

    assert_eq!(
        obligation_info
            .get_data(&mut context)
            .await
            .amount_collateral_deposited,
        DEPOSIT_AMOUNT
    );
}

#[tokio::test]
async fn collateral_withdraw() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;
    let (obligation_info, borrower_collateral, _) = prepare_borrower(
        &mut context,
        &market_info,
        &liquidity_info,
        &collateral_info,
        10000,
    )
    .await;

    obligation_info
        .collateral_deposit(
            &mut context,
            &market_info,
            &collateral_info,
            10000,
            &borrower_collateral.pubkey(),
        )
        .await
        .unwrap();

    const WITHDRAW_AMOUNT: u64 = 10000;
    obligation_info
        .collateral_withdraw(
            &mut context,
            &market_info,
            &collateral_info,
            WITHDRAW_AMOUNT,
            &borrower_collateral.pubkey(),
        )
        .await
        .unwrap();

    assert_eq!(
        get_token_balance(&mut context, &borrower_collateral.pubkey()).await,
        WITHDRAW_AMOUNT
    );
}

#[tokio::test]
async fn liquidity_borrow() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;

    const DEPOSIT_AMOUNT: u64 = 10000;
    let (obligation_info, borrower_collateral, borrower_liquidity) = prepare_borrower(
        &mut context,
        &market_info,
        &liquidity_info,
        &collateral_info,
        DEPOSIT_AMOUNT,
    )
    .await;

    // Deposit collateral
    obligation_info
        .collateral_deposit(
            &mut context,
            &market_info,
            &collateral_info,
            DEPOSIT_AMOUNT,
            &borrower_collateral.pubkey(),
        )
        .await
        .unwrap();

    let borrow_ammount = DEPOSIT_AMOUNT * collateral::RATIO_INITIAL / (100 * RATIO_POWER);
    println!("Borrow amount: {}", borrow_ammount);
    obligation_info
        .liquidity_borrow(
            &mut context,
            &market_info,
            &liquidity_info,
            &collateral_info,
            borrow_ammount,
            &borrower_liquidity.pubkey(),
        )
        .await
        .unwrap();

    assert_eq!(
        obligation_info
            .get_data(&mut context)
            .await
            .amount_liquidity_borrowed,
        borrow_ammount
    );

    assert_eq!(
        get_token_balance(&mut context, &borrower_liquidity.pubkey()).await,
        borrow_ammount
    );
}

#[tokio::test]
async fn liquidity_repay() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;

    const DEPOSIT_AMOUNT: u64 = 10000;
    let (obligation_info, borrower_collateral, borrower_liquidity) = prepare_borrower(
        &mut context,
        &market_info,
        &liquidity_info,
        &collateral_info,
        DEPOSIT_AMOUNT,
    )
    .await;

    // Deposit collateral
    obligation_info
        .collateral_deposit(
            &mut context,
            &market_info,
            &collateral_info,
            DEPOSIT_AMOUNT,
            &borrower_collateral.pubkey(),
        )
        .await
        .unwrap();

    let borrow_ammount = DEPOSIT_AMOUNT * collateral::RATIO_INITIAL / (100 * RATIO_POWER);
    println!("Borrow amount: {}", borrow_ammount);
    obligation_info
        .liquidity_borrow(
            &mut context,
            &market_info,
            &liquidity_info,
            &collateral_info,
            borrow_ammount,
            &borrower_liquidity.pubkey(),
        )
        .await
        .unwrap();

    assert_eq!(
        get_token_balance(&mut context, &borrower_liquidity.pubkey()).await,
        borrow_ammount
    );

    obligation_info
        .liquidity_repay(
            &mut context,
            &market_info,
            &liquidity_info,
            borrow_ammount,
            &borrower_liquidity.pubkey(),
        )
        .await
        .unwrap();

    assert_eq!(
        obligation_info
            .get_data(&mut context)
            .await
            .amount_liquidity_borrowed,
        0
    );

    assert_eq!(
        get_token_balance(&mut context, &borrower_liquidity.pubkey()).await,
        0
    );
}

// TODO: need to add more fail tests
