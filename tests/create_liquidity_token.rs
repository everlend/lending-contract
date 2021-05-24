#![cfg(feature = "test-bpf")]

mod utils;

use solana_program_test::*;
use utils::*;

async fn setup() -> (ProgramTestContext, market::MarketInfo) {
    let mut context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    (context, market_info)
}

#[tokio::test]
async fn success() {}
