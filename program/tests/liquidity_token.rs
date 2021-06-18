#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::LiquidityStatus;
use solana_program_test::*;
use utils::*;

async fn setup() -> (ProgramTestContext, MarketInfo, TestOracle) {
    let mut test = program_test();
    let sol_oracle = add_sol_oracle(&mut test);
    let mut context = test.start_with_context().await;

    let market_info = MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    (context, market_info, sol_oracle)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info, oracle) = setup().await;

    assert_eq!(market_info.get_data(&mut context).await.liquidity_tokens, 0);

    let liquidity_info = market_info
        .create_liquidity_token(&mut context, &oracle)
        .await
        .unwrap();

    let liquidity = liquidity_info.get_data(&mut context).await;

    assert_eq!(liquidity.status, LiquidityStatus::Inactive);
    assert_eq!(market_info.get_data(&mut context).await.liquidity_tokens, 1);
}

#[tokio::test]
async fn success_two_tokens() {
    let (mut context, market_info, oracle) = setup().await;

    market_info
        .create_liquidity_token(&mut context, &oracle)
        .await
        .unwrap();

    market_info
        .create_liquidity_token(&mut context, &oracle)
        .await
        .unwrap();

    assert_eq!(market_info.get_data(&mut context).await.liquidity_tokens, 2);
}

#[tokio::test]
async fn success_update_token() {
    let (mut context, market_info, oracle) = setup().await;

    let liquidity_info = market_info
        .create_liquidity_token(&mut context, &oracle)
        .await
        .unwrap();

    liquidity_info
        .update(&mut context, LiquidityStatus::Active, &market_info)
        .await
        .unwrap();

    assert_eq!(
        liquidity_info.get_data(&mut context).await.status,
        LiquidityStatus::Active
    );
}

// TODO: need to add more fail tests
