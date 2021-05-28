use super::{
    collateral::CollateralInfo, get_account, liquidity::LiquidityInfo, market::MarketInfo,
};
use everlend_lending::state::Obligation;
use everlend_lending::{id, instruction};
use solana_program::pubkey::Pubkey;
use solana_program::{program_pack::Pack, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

#[derive(Debug)]
pub struct ObligationInfo {
    pub obligation: Keypair,
    pub owner: Keypair,
}

impl ObligationInfo {
    pub fn new() -> Self {
        Self {
            obligation: Keypair::new(),
            owner: Keypair::new(),
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Obligation {
        let obligation_account = get_account(context, &self.obligation.pubkey()).await;
        Obligation::unpack_unchecked(&obligation_account.data).unwrap()
    }

    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
    ) -> transport::Result<()> {
        let rent = context.banks_client.get_rent().await.unwrap();

        let tx = Transaction::new_signed_with_payer(
            &[
                system_instruction::create_account(
                    &context.payer.pubkey(),
                    &self.obligation.pubkey(),
                    rent.minimum_balance(Obligation::LEN),
                    Obligation::LEN as u64,
                    &id(),
                ),
                instruction::create_obligation(
                    &id(),
                    &self.obligation.pubkey(),
                    &liquidity_info.liquidity_pubkey,
                    &collateral_info.collateral_pubkey,
                    &market_info.market.pubkey(),
                    &self.owner.pubkey(),
                )
                .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.obligation, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn collateral_deposit(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        collateral_info: &CollateralInfo,
        amount: u64,
        source: &Pubkey,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::obligation_collateral_deposit(
                &id(),
                amount,
                &self.obligation.pubkey(),
                &collateral_info.collateral_pubkey,
                source,
                &collateral_info.token_account.pubkey(),
                &market_info.market.pubkey(),
                &self.owner.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn collateral_withdraw(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        collateral_info: &CollateralInfo,
        amount: u64,
        destination: &Pubkey,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::obligation_collateral_withdraw(
                &id(),
                amount,
                &self.obligation.pubkey(),
                &collateral_info.collateral_pubkey,
                destination,
                &collateral_info.token_account.pubkey(),
                &market_info.market.pubkey(),
                &self.owner.pubkey(),
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
