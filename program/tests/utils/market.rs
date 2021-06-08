use super::{collateral::CollateralInfo, get_account, liquidity::LiquidityInfo};
use crate::utils::create_mint;
use everlend_lending::{id, instruction, state::Market};
use solana_program::{borsh::get_packed_len, program_pack::Pack, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
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
        Market::unpack_unchecked(&market_account.data).unwrap()
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
    ) -> transport::Result<LiquidityInfo> {
        let liquidity_tokens = self.get_data(context).await.liquidity_tokens;
        let seed = format!("liquidity{:?}", liquidity_tokens);
        let liquidity_info = LiquidityInfo::new(&seed, &self, None);

        create_mint(context, &liquidity_info.token_mint, &self.owner.pubkey())
            .await
            .unwrap();

        liquidity_info.create(context, self).await.unwrap();

        Ok(liquidity_info)
    }

    pub async fn create_collateral_token(
        &self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<CollateralInfo> {
        let collateral_tokens = self.get_data(context).await.collateral_tokens;
        let seed = format!("collateral{:?}", collateral_tokens);
        let collateral_info = CollateralInfo::new(&seed, self, None);

        create_mint(context, &collateral_info.token_mint, &self.owner.pubkey())
            .await
            .unwrap();

        collateral_info.create(context, self).await.unwrap();

        Ok(collateral_info)
    }
}
