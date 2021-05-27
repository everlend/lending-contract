#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::LiquidityStatus;
use solana_program_test::*;
use solana_sdk::signer::Signer;
use utils::*;

async fn setup() -> (
    ProgramTestContext,
    market::MarketInfo,
    liquidity::LiquidityInfo,
) {
    let mut context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let liquidity_info = market_info
        .create_liquidity_token(&mut context)
        .await
        .unwrap();

    liquidity_info
        .update(&mut context, LiquidityStatus::Active, &market_info.owner)
        .await
        .unwrap();

    (context, market_info, liquidity_info)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info, liquidity_info) = setup().await;
    let provider_info = provider::ProviderInfo::new();

    let (source, destination) = provider_info
        .create_liquidity_accounts(&mut context, &liquidity_info)
        .await
        .unwrap();

    mint_tokens(
        &mut context,
        &liquidity_info.token_mint.pubkey(),
        &source.pubkey(),
        &market_info.owner,
        9999999,
    )
    .await
    .unwrap();

    market_info
        .deposit(
            &mut context,
            &liquidity_info,
            &source.pubkey(),
            &destination.pubkey(),
            10000,
            &provider_info.owner,
        )
        .await
        .unwrap();

    assert_eq!(
        get_token_balance(&mut context, &destination.pubkey()).await,
        10000
    );
}

#[tokio::test]
async fn two_deposits() {
    let (mut context, market_info, liquidity_info) = setup().await;
    let provider_info = provider::ProviderInfo::new();

    let (source, destination) = provider_info
        .create_liquidity_accounts(&mut context, &liquidity_info)
        .await
        .unwrap();

    mint_tokens(
        &mut context,
        &liquidity_info.token_mint.pubkey(),
        &source.pubkey(),
        &market_info.owner,
        9999999,
    )
    .await
    .unwrap();

    market_info
        .deposit(
            &mut context,
            &liquidity_info,
            &source.pubkey(),
            &destination.pubkey(),
            10000,
            &provider_info.owner,
        )
        .await
        .unwrap();

    market_info
        .deposit(
            &mut context,
            &liquidity_info,
            &source.pubkey(),
            &destination.pubkey(),
            5000,
            &provider_info.owner,
        )
        .await
        .unwrap();

    assert_eq!(
        get_token_balance(&mut context, &destination.pubkey()).await,
        15000
    );
}

// TODO: need to add more fail tests
