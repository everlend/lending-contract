#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::PROGRAM_VERSION;
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

    (context, market_info, liquidity_info, collateral_info)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info, liquidity_info, collateral_info) = setup().await;

    let obligation_info = ObligationInfo::new();
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

    let obligation_info = ObligationInfo::new();
    obligation_info
        .create(
            &mut context,
            &market_info,
            &liquidity_info,
            &collateral_info,
        )
        .await
        .unwrap();

    // Create source borrower
    let source = Keypair::new();
    create_token_account(
        &mut context,
        &source,
        &collateral_info.token_mint.pubkey(),
        &obligation_info.owner.pubkey(),
    )
    .await
    .unwrap();

    mint_tokens(
        &mut context,
        &collateral_info.token_mint.pubkey(),
        &source.pubkey(),
        &market_info.owner,
        99999,
    )
    .await
    .unwrap();

    const DEPOSIT_AMOUNT: u64 = 10000;
    obligation_info
        .collateral_deposit(
            &mut context,
            &market_info,
            &collateral_info,
            DEPOSIT_AMOUNT,
            &source.pubkey(),
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

// TODO: need to add more fail tests
