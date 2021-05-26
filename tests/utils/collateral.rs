use super::get_account;
use everlend_lending::state::Collateral;
use everlend_lending::{id, instruction};
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

pub const RATIO_INITIAL: u64 = 50 * u64::pow(10, 9);
pub const RATIO_HEALTHY: u64 = 75 * u64::pow(10, 9);

#[derive(Debug)]
pub struct CollateralInfo {
    pub collateral_pubkey: Pubkey,
    pub token_mint: Keypair,
    pub token_account: Keypair,
}

impl CollateralInfo {
    pub fn new(base: &Pubkey, seed: &str) -> Self {
        Self {
            collateral_pubkey: Pubkey::create_with_seed(base, seed, &id()).unwrap(),
            token_mint: Keypair::new(),
            token_account: Keypair::new(),
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Collateral {
        let collateral_account = get_account(context, &self.collateral_pubkey).await;
        Collateral::unpack_unchecked(&collateral_account.data).unwrap()
    }

    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        market_pubkey: &Pubkey,
        market_owner: &Keypair,
    ) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[
                // Transfer a few lamports to cover fee for create account
                system_instruction::transfer(
                    &context.payer.pubkey(),
                    &market_owner.pubkey(),
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
                    &market_pubkey,
                    &market_owner.pubkey(),
                )
                .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.token_account, &market_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
