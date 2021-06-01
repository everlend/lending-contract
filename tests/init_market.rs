#![cfg(feature = "test-bpf")]

mod utils;

use everlend_lending::{id, instruction, state::*};
use solana_program::instruction::InstructionError;
use solana_program_test::*;
use solana_sdk::{
    signature::Signer,
    transaction::{Transaction, TransactionError},
};
use utils::*;

#[tokio::test]
async fn success() {
    let mut context = program_test().start_with_context().await;

    let market_info = MarketInfo::new();
    market_info.init(&mut context).await.unwrap();

    let market = market_info.get_data(&mut context).await;

    assert_eq!(market.owner, market_info.owner.pubkey());
    assert_eq!(market.version, PROGRAM_VERSION);
}

#[tokio::test]
async fn fail_already_initialized() {
    let mut context = program_test().start_with_context().await;

    let market_info = MarketInfo::new();
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
