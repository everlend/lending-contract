use solana_program_test::ProgramTestContext;
use solana_sdk::account::Account;
use solana_sdk::pubkey::Pubkey;

pub mod market;

pub async fn get_account(context: &mut ProgramTestContext, pubkey: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .expect("account not found")
        .expect("account empty")
}
