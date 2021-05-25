#![cfg(feature = "test-bpf")]

mod utils;

use borsh::BorshDeserialize;
use everlend_lending::state::{Liquidity, LiquidityStatus};
use solana_program_test::*;
use utils::*;

async fn setup() -> (ProgramTestContext, market::MarketInfo) {
    let mut context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    (context, market_info)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info) = setup().await;

    let liquidity_tokens = market_info.get_liquidity_tokens(&mut context).await;
    assert_eq!(liquidity_tokens, 0);

    let liquidity_info = market_info
        .create_liquidity_token(&mut context)
        .await
        .unwrap();

    let liquidity_account = get_account(&mut context, &liquidity_info.liquidity_pubkey).await;
    let liquidity = Liquidity::try_from_slice(&liquidity_account.data).unwrap();

    assert_eq!(liquidity.status, LiquidityStatus::InActive);

    let liquidity_tokens = market_info.get_liquidity_tokens(&mut context).await;
    assert_eq!(liquidity_tokens, 1);
}

#[tokio::test]
async fn two_tokens() {
    let (mut context, market_info) = setup().await;

    market_info
        .create_liquidity_token(&mut context)
        .await
        .unwrap();

    market_info
        .create_liquidity_token(&mut context)
        .await
        .unwrap();

    let liquidity_tokens = market_info.get_liquidity_tokens(&mut context).await;
    assert_eq!(liquidity_tokens, 2);
}

// TODO: need to add more fail tests
