#![allow(dead_code)]

use everlend_lending::{id, processor};
use solana_program_test::ProgramTestContext;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

pub mod market;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "everlend_lending",
        id(),
        processor!(processor::Processor::process_instruction),
    )
}

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}
