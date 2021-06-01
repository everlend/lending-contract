#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::state::{CollateralStatus, RATIO_POWER};
use solana_program_test::*;
use utils::*;

async fn setup() -> (ProgramTestContext, MarketInfo) {
    let mut context = program_test().start_with_context().await;

    let market_info = MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    (context, market_info)
}

#[tokio::test]
async fn success() {
    let (mut context, market_info) = setup().await;

    assert_eq!(
        market_info.get_data(&mut context).await.collateral_tokens,
        0
    );

    let collateral_info = market_info
        .create_collateral_token(&mut context)
        .await
        .unwrap();

    let collateral = collateral_info.get_data(&mut context).await;

    assert_eq!(collateral.status, CollateralStatus::InActive);
    assert_eq!(collateral.ratio_initial, collateral::RATIO_INITIAL);
    assert_eq!(
        market_info.get_data(&mut context).await.collateral_tokens,
        1
    );
}

#[tokio::test]
async fn two_tokens() {
    let (mut context, market_info) = setup().await;

    market_info
        .create_collateral_token(&mut context)
        .await
        .unwrap();

    market_info
        .create_collateral_token(&mut context)
        .await
        .unwrap();

    assert_eq!(
        market_info.get_data(&mut context).await.collateral_tokens,
        2
    );
}

#[tokio::test]
async fn update_token() {
    let (mut context, market_info) = setup().await;

    let collateral_info = market_info
        .create_collateral_token(&mut context)
        .await
        .unwrap();

    const NEW_RATIO_INITIAL: u64 = 35 * RATIO_POWER / 100;
    const NEW_RATIO_HEALTHY: u64 = 60 * RATIO_POWER / 100;

    collateral_info
        .update(
            &mut context,
            CollateralStatus::Active,
            NEW_RATIO_INITIAL,
            NEW_RATIO_HEALTHY,
            &market_info,
        )
        .await
        .unwrap();

    assert_eq!(
        collateral_info.get_data(&mut context).await.status,
        CollateralStatus::Active
    );
    assert_eq!(
        collateral_info.get_data(&mut context).await.ratio_initial,
        NEW_RATIO_INITIAL
    );
    assert_eq!(
        collateral_info.get_data(&mut context).await.ratio_healthy,
        NEW_RATIO_HEALTHY
    );
}

// TODO: need to add more fail tests
