use super::{get_account, market::MarketInfo, oracle::TestOracle};
use everlend_lending::{
    find_program_address, id, instruction,
    state::{Collateral, CollateralStatus, RATIO_POWER},
};
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport,
};

pub const RATIO_INITIAL: u64 = 50 * RATIO_POWER / 100; // 0.5 * 10^9
pub const RATIO_HEALTHY: u64 = 75 * RATIO_POWER / 100; // 0.75 * 10^9

#[derive(Debug)]
pub struct CollateralInfo {
    pub collateral_pubkey: Pubkey,
    pub token_mint: Keypair,
    pub token_account: Keypair,
    pub oracle: Pubkey,
}

impl CollateralInfo {
    pub fn new(seed: &str, market_info: &MarketInfo, oracle: &TestOracle) -> Self {
        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &market_info.market.pubkey());

        Self {
            collateral_pubkey: Pubkey::create_with_seed(&market_authority, seed, &id()).unwrap(),
            token_mint: Keypair::new(),
            token_account: Keypair::new(),
            oracle: oracle.price_pubkey,
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Collateral {
        let collateral_account = get_account(context, &self.collateral_pubkey).await;
        Collateral::unpack_unchecked(&collateral_account.data).unwrap()
    }

    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        oracle: &TestOracle,
    ) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[
                // Transfer a few lamports to cover fee for create account
                system_instruction::transfer(
                    &context.payer.pubkey(),
                    &market_info.owner.pubkey(),
                    999999999,
                ),
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.token_account.pubkey(),
                    rent.minimum_balance(spl_token::state::Account::LEN),
                    spl_token::state::Account::LEN as u64,
                    &spl_token::id(),
                ),
                instruction::create_collateral_token(
                    &id(),
                    RATIO_INITIAL,
                    RATIO_HEALTHY,
                    &self.collateral_pubkey,
                    &self.token_mint.pubkey(),
                    &self.token_account.pubkey(),
                    &market_info.market.pubkey(),
                    &market_info.owner.pubkey(),
                    &oracle.product_pubkey,
                    &oracle.price_pubkey,
                )
                .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.token_account, &market_info.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update(
        &self,
        context: &mut ProgramTestContext,
        status: CollateralStatus,
        ratio_initial: u64,
        ratio_healthy: u64,
        market_info: &MarketInfo,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_collateral_token(
                &id(),
                status,
                ratio_initial,
                ratio_healthy,
                &self.collateral_pubkey,
                &market_info.market.pubkey(),
                &market_info.owner.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &market_info.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
