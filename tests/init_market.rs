#![cfg(feature = "test-bpf")]

mod utils;

use borsh::BorshDeserialize;
use everlend_lending::state::*;
use everlend_lending::*;
use solana_program::{instruction::InstructionError, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "everlend_lending",
        id(),
        processor!(processor::Processor::process_instruction),
    )
}

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let market_account = get_account(&mut context, &market_info.market.pubkey()).await;
    let market_account = Market::try_from_slice(&market_account.data).unwrap();
    assert_eq!(market_account.owner, market_info.owner.pubkey());
    assert_eq!(market_account.version, StateVersion::V1);
}

#[tokio::test]
async fn fail_already_initialized() {
    let mut context = program_test().start_with_context().await;

    let market_info = market::MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[instruction::init_market(
            &id(),
            &market_info.market.pubkey(),
            &market_info.owner.pubkey(),
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, &market_info.owner],
        context.last_blockhash,
    );

    assert_eq!(
        context
            .banks_client
            .process_transaction(tx)
            .await
            .unwrap_err()
            .unwrap(),
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}
