#![cfg(feature = "test-bpf")]

mod utils;

use borsh::BorshDeserialize;
use everlend_lending::state::*;
use everlend_lending::*;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::signature::Signer;
use utils::*;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "everlend_lending",
        id(),
        processor!(processor::Processor::process_instruction),
    )
}

#[tokio::test]
async fn test_init_market() {
    let mut test_context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();

    let transaction = market_info.init_transaction(&mut test_context).await;
    test_context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    let market_account = get_account(&mut test_context, &market_info.market.pubkey()).await;
    let market_account = Market::try_from_slice(&market_account.data).unwrap();
    assert_eq!(market_account.owner, market_info.owner.pubkey())
}
