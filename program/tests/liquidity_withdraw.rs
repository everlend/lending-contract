#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::LiquidityStatus;
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::TransactionError};
use spl_token::error::TokenError;
use utils::*;

async fn setup() -> (ProgramTestContext, MarketInfo, LiquidityInfo) {
    let mut context = program_test().start_with_context().await;

    let market_info = MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let liquidity_info = market_info
        .create_liquidity_token(&mut context, None)
        .await
        .unwrap();

    liquidity_info
        .update(&mut context, LiquidityStatus::Active, &market_info)
        .await
        .unwrap();

    (context, market_info, liquidity_info)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info, liquidity_info) = setup().await;
    let provider_actor = ProviderActor::new();

    let (source, destination) = provider_actor
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

    liquidity_info
        .deposit(
            &mut context,
            &market_info,
            &source.pubkey(),
            &destination.pubkey(),
            10000,
            &provider_actor.owner,
        )
        .await
        .unwrap();

    liquidity_info
        .withdraw(
            &mut context,
            &market_info,
            &destination.pubkey(),
            &source.pubkey(),
            7000,
            &provider_actor.owner,
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
    let provider_actor = ProviderActor::new();

    let (source, destination) = provider_actor
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

    liquidity_info
        .deposit(
            &mut context,
            &market_info,
            &source.pubkey(),
            &destination.pubkey(),
            500,
            &provider_actor.owner,
        )
        .await
        .unwrap();

    assert_eq!(
        liquidity_info
            .withdraw(
                &mut context,
                &market_info,
                &destination.pubkey(),
                &source.pubkey(),
                1000,
                &provider_actor.owner,
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
