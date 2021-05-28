#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::PROGRAM_VERSION;
use solana_program_test::*;
use solana_sdk::signer::Signer;
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

    let obligation_info = ObligationInfo::new(&&market_info, &liquidity_info, &collateral_info);
    obligation_info.create(&mut context).await.unwrap();

    let obligation = obligation_info.get_data(&mut context).await;

    assert_eq!(obligation.owner, obligation_info.owner.pubkey());
    assert_eq!(obligation.version, PROGRAM_VERSION);
}

// TODO: need to add more fail tests
