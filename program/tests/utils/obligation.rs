use super::{
    collateral::CollateralInfo, get_account, liquidity::LiquidityInfo, market::MarketInfo,
};
use everlend_lending::state::Obligation;
use everlend_lending::{find_obligation_authority, id, instruction};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

#[derive(Debug)]
pub struct ObligationInfo {
    pub obligation_pubkey: Pubkey,
    pub owner: Keypair,
}

impl ObligationInfo {
    pub fn new(
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
    ) -> Self {
        let owner = Keypair::new();
        let (obligation_authority, _) = find_obligation_authority(
            &everlend_lending::id(),
            &owner.pubkey(),
            &market_info.market.pubkey(),
            &liquidity_info.liquidity_pubkey,
            &collateral_info.collateral_pubkey,
        );

        Self {
            obligation_pubkey: Pubkey::create_with_seed(&obligation_authority, "obligation", &id())
                .unwrap(),
            owner,
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Obligation {
        let obligation_account = get_account(context, &self.obligation_pubkey).await;
        Obligation::unpack_unchecked(&obligation_account.data).unwrap()
    }

    pub async fn create(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[
                // Transfer a few lamports to cover fee for create account
                system_instruction::transfer(
                    &context.payer.pubkey(),
                    &self.owner.pubkey(),
                    999999999,
                ),
                instruction::create_obligation(
                    &id(),
                    &self.obligation_pubkey,
                    &liquidity_info.liquidity_pubkey,
                    &collateral_info.collateral_pubkey,
                    &market_info.market.pubkey(),
                    &self.owner.pubkey(),
                )
                .unwrap(),
            ],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.owner],
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
                &self.obligation_pubkey,
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
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
        amount: u64,
        destination: &Pubkey,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::obligation_collateral_withdraw(
                &id(),
                amount,
                &self.obligation_pubkey,
                &liquidity_info.liquidity_pubkey,
                &collateral_info.collateral_pubkey,
                destination,
                &collateral_info.token_account.pubkey(),
                &market_info.market.pubkey(),
                &self.owner.pubkey(),
                &liquidity_info.oracle,
                &collateral_info.oracle,
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn liquidity_borrow(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
        amount: u64,
        destination: &Pubkey,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::obligation_liquidity_borrow(
                &id(),
                amount,
                &self.obligation_pubkey,
                &liquidity_info.liquidity_pubkey,
                &collateral_info.collateral_pubkey,
                destination,
                &liquidity_info.token_account.pubkey(),
                &market_info.market.pubkey(),
                &self.owner.pubkey(),
                &liquidity_info.oracle,
                &collateral_info.oracle,
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, &self.owner],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }

    pub async fn liquidity_repay(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        amount: u64,
        source: &Pubkey,
    ) -> transport::Result<()> {
        let tx = Transaction::new_signed_with_payer(
            &[instruction::obligation_liquidity_repay(
                &id(),
                amount,
                &self.obligation_pubkey,
                &liquidity_info.liquidity_pubkey,
                source,
                &liquidity_info.token_account.pubkey(),
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

    pub async fn liquidate(
        &self,
        context: &mut ProgramTestContext,
        market_info: &MarketInfo,
        liquidity_info: &LiquidityInfo,
        collateral_info: &CollateralInfo,
        source: &Pubkey,
        destination: &Pubkey,
        liquidator: Option<&Keypair>,
    ) -> transport::Result<()> {
        let liquidator = liquidator.unwrap_or(&self.owner);

        let tx = Transaction::new_signed_with_payer(
            &[instruction::liquidate_obligation(
                &id(),
                &self.obligation_pubkey,
                source,
                destination,
                &liquidity_info.liquidity_pubkey,
                &collateral_info.collateral_pubkey,
                &liquidity_info.token_account.pubkey(),
                &collateral_info.token_account.pubkey(),
                &market_info.market.pubkey(),
                &liquidator.pubkey(),
                &liquidity_info.oracle,
                &collateral_info.oracle,
            )
            .unwrap()],
            Some(&context.payer.pubkey()),
            &[&context.payer, liquidator],
            context.last_blockhash,
        );

        context.banks_client.process_transaction(tx).await
    }
}
