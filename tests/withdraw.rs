#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::LiquidityStatus;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError};
use spl_token::error::TokenError;
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
        10000,
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
        .withdraw(
            &mut context,
            &liquidity_info,
            &destination.pubkey(),
            &source.pubkey(),
            7000,
            &provider_info.owner,
        )
        .await
        .unwrap();

    assert_eq!(
        get_token_balance(&mut context, &source.pubkey()).await,
        7000
    );
    assert_eq!(
        get_token_balance(&mut context, &destination.pubkey()).await,
        3000
    );
}

#[tokio::test]
async fn fail_more_than_possible() {
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
        500,
    )
    .await
    .unwrap();

    market_info
        .deposit(
            &mut context,
            &liquidity_info,
            &source.pubkey(),
            &destination.pubkey(),
            500,
            &provider_info.owner,
        )
        .await
        .unwrap();

    assert_eq!(
        market_info
            .withdraw(
                &mut context,
                &liquidity_info,
                &destination.pubkey(),
                &source.pubkey(),
                1000,
                &provider_info.owner,
            )
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(TokenError::InsufficientFunds as u32)
        )
    );
}

// TODO: need to add more fail tests
