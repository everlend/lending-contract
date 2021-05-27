use super::{collateral, get_account};
use crate::utils::{create_mint, liquidity};
use everlend_lending::{find_program_address, id, instruction, state::Market};
use solana_program::pubkey::Pubkey;
use solana_program::{borsh::get_packed_len, program_pack::Pack, system_instruction};
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
    ) -> transport::Result<liquidity::LiquidityInfo> {
        let liquidity_tokens = self.get_data(context).await.liquidity_tokens;

        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &self.market.pubkey());

        let seed = format!("liquidity{:?}", liquidity_tokens);
        let liquidity_info = liquidity::LiquidityInfo::new(&market_authority, &seed);

        create_mint(context, &liquidity_info.token_mint, &self.owner.pubkey())
            .await
            .unwrap();

        liquidity_info
            .create(context, &self.market.pubkey(), &self.owner)
            .await
            .unwrap();

        Ok(liquidity_info)
    }

    pub async fn create_collateral_token(
        &self,
        context: &mut ProgramTestContext,
    ) -> transport::Result<collateral::CollateralInfo> {
        let collateral_tokens = self.get_data(context).await.collateral_tokens;

        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &self.market.pubkey());

        let seed = format!("collateral{:?}", collateral_tokens);
        let collateral_info = collateral::CollateralInfo::new(&market_authority, &seed);

        create_mint(context, &collateral_info.token_mint, &self.owner.pubkey())
            .await
            .unwrap();

        collateral_info
            .create(context, &self.market.pubkey(), &self.owner)
            .await
            .unwrap();

        Ok(collateral_info)
    }

    pub async fn deposit(
        &self,
        context: &mut ProgramTestContext,
        liquidity_info: &liquidity::LiquidityInfo,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        provider: &Keypair,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::deposit(
                &id(),
                amount,
                &liquidity_info.liquidity_pubkey,
                source,
                destination,
                &liquidity_info.token_account.pubkey(),
                &liquidity_info.pool_mint.pubkey(),
                &self.market.pubkey(),
                &provider.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &provider],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn withdraw(
        &self,
        context: &mut ProgramTestContext,
        liquidity_info: &liquidity::LiquidityInfo,
        source: &Pubkey,
        destination: &Pubkey,
        amount: u64,
        provider: &Keypair,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::withdraw(
                &id(),
                amount,
                &liquidity_info.liquidity_pubkey,
                source,
                destination,
                &liquidity_info.token_account.pubkey(),
                &liquidity_info.pool_mint.pubkey(),
                &self.market.pubkey(),
                &provider.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &provider],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
