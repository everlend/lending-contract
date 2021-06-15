#![allow(dead_code)]

use std::str::FromStr;

use everlend_lending::{id, processor};
use solana_program::{program_pack::Pack, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_program_test::*;
use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
    transport,
};

pub mod collateral;
pub mod liquidity;
pub mod market;
pub mod obligation;
pub mod oracle;
pub mod provider;

pub use collateral::CollateralInfo;
pub use liquidity::LiquidityInfo;
pub use market::MarketInfo;
pub use obligation::ObligationInfo;
pub use oracle::TestOracle;
pub use provider::ProviderActor;

pub const SOL_PYTH_PRODUCT: &str = "8yrQMUyJRnCJ72NWwMiPV9dNGw465Z8bKUvnUC8P5L6F";
pub const SOL_PYTH_PRICE: &str = "BdgHsXrH1mXqhdosXavYxZgX6bGqTdj5mh2sxDhF8bJy";

pub const SRM_PYTH_PRODUCT: &str = "5agdsn3jogTt8F537GW3g8BuLaBGrg9Q2gPKUNqBV6Dh";
pub const SRM_PYTH_PRICE: &str = "2Mt2wcRXpCAbTRp2VjFqGa8SbJVzjJvyK4Tx7aqbRtBJ";

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

pub async fn get_token_balance(context: &mut ProgramTestContext, pubkey: &Pubkey) -> u64 {
    let account = get_account(context, pubkey).await;
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(account.data.as_slice()).unwrap();

    account_info.amount
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    mint: &Keypair,
    manager: &Pubkey,
) -> transport::Result<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(spl_token::state::Mint::LEN),
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &manager,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    account: &Keypair,
    mint: &Pubkey,
    manager: &Pubkey,
) -> transport::Result<()> {
    let rent = context.banks_client.get_rent().await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &context.payer.pubkey(),
                &account.pubkey(),
                rent.minimum_balance(spl_token::state::Account::LEN),
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                manager,
            )
            .unwrap(),
        ],
        Some(&context.payer.pubkey()),
        &[&context.payer, &account],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub async fn mint_tokens(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    account: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) -> transport::Result<()> {
    let tx = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );

    context.banks_client.process_transaction(tx).await
}

pub fn add_sol_oracle(test: &mut ProgramTest) -> TestOracle {
    let oracle = TestOracle::new(
        &Pubkey::from_str(SOL_PYTH_PRODUCT).unwrap(),
        &Pubkey::from_str(SOL_PYTH_PRICE).unwrap(),
        10000,
    );
    oracle.init(test);

    oracle
}

pub fn add_srm_oracle(test: &mut ProgramTest) -> TestOracle {
    let oracle = TestOracle::new(
        &Pubkey::from_str(SRM_PYTH_PRODUCT).unwrap(),
        &Pubkey::from_str(SRM_PYTH_PRICE).unwrap(),
        20000,
    );
    oracle.init(test);

    oracle
}
