use super::get_account;
use everlend_lending::{
    id, instruction,
    state::{Liquidity, LiquidityStatus},
};
use solana_program::{program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport,
};

#[derive(Debug)]
pub struct LiquidityInfo {
    pub liquidity_pubkey: Pubkey,
    pub token_mint: Keypair,
    pub token_account: Keypair,
    pub pool_mint: Keypair,
}

impl LiquidityInfo {
    pub fn new(base: &Pubkey, seed: &str) -> Self {
        Self {
            liquidity_pubkey: Pubkey::create_with_seed(base, seed, &id()).unwrap(),
            token_mint: Keypair::new(),
            token_account: Keypair::new(),
            pool_mint: Keypair::new(),
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Liquidity {
        let liquidity_account = get_account(context, &self.liquidity_pubkey).await;
        Liquidity::unpack_unchecked(&liquidity_account.data).unwrap()
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
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.pool_mint.pubkey(),
                    rent.minimum_balance(spl_token::state::Mint::LEN),
                    spl_token::state::Mint::LEN as u64,
                    &spl_token::id(),
                ),
                instruction::create_liquidity_token(
                    &id(),
                    &self.liquidity_pubkey,
                    &self.token_mint.pubkey(),
                    &self.token_account.pubkey(),
                    &self.pool_mint.pubkey(),
                    &market_pubkey,
                    &market_owner.pubkey(),
                )
                .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[
                &context.payer,
                &self.token_account,
                &self.pool_mint,
                &market_owner,
            ],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn update(
        &self,
        context: &mut ProgramTestContext,
        status: LiquidityStatus,
        market_owner: &Keypair,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::update_liquidity_token(
                &id(),
                status,
                &self.liquidity_pubkey,
                &market_owner.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &market_owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
