use super::get_account;
use crate::utils::{create_mint, liquidity};
use borsh::BorshDeserialize;
use everlend_lending::{
    find_program_address, id, instruction,
    state::{Liquidity, Market},
};
use solana_program::{borsh::get_packed_len, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

#[derive(Debug)]
pub struct MarketInfo {
    pub market: Keypair,
    pub owner: Keypair,
}

impl MarketInfo {
    pub fn new() -> Self {
        Self {
            market: Keypair::new(),
            owner: Keypair::new(),
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Market {
        let market_account = get_account(context, &self.market.pubkey()).await;
        Market::try_from_slice(&market_account.data).unwrap()
    }

    pub async fn init(&self, context: &mut ProgramTestContext) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();
        let tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.market.pubkey(),
                    rent.minimum_balance(get_packed_len::<Market>()),
                    get_packed_len::<Market>() as u64,
                    &id(),
                ),
                instruction::init_market(&id(), &self.market.pubkey(), &self.owner.pubkey())
                    .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.market, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn create_liquidity_token(
        &self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<liquidity::LiquidityInfo> {
        let liquidity_tokens = self.get_data(context).await.liquidity_tokens;

        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &self.market.pubkey());

        let seed = format!("liquidity{:?}", liquidity_tokens);
        let liquidity_info = liquidity::LiquidityInfo::new(&market_authority, &seed);

        println!("LEN: {:?}", get_packed_len::<Liquidity>());

        create_mint(context, &liquidity_info.token_mint, &self.owner.pubkey())
            .await
            .unwrap();

        liquidity_info
            .create(context, &self.market.pubkey(), &self.owner)
            .await
            .unwrap();

        Ok(liquidity_info)
    }
}
