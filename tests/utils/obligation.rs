use super::{
    collateral::CollateralInfo, get_account, liquidity::LiquidityInfo, market::MarketInfo,
};
use everlend_lending::state::Obligation;
use everlend_lending::{id, instruction};
use solana_program::{program_pack::Pack, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

#[derive(Debug)]
pub struct ObligationInfo<'a> {
    pub obligation: Keypair,
    pub owner: Keypair,
    pub market_info: &'a MarketInfo,
    pub liquidity_info: &'a LiquidityInfo,
    pub collateral_info: &'a CollateralInfo,
}

impl<'a> ObligationInfo<'a> {
    pub fn new(
        market_info: &'a MarketInfo,
        liquidity_info: &'a LiquidityInfo,
        collateral_info: &'a CollateralInfo,
    ) -> Self {
        Self {
            obligation: Keypair::new(),
            owner: Keypair::new(),
            market_info,
            liquidity_info,
            collateral_info,
        }
    }

    pub async fn get_data(&self, context: &mut ProgramTestContext) -> Obligation {
        let obligation_account = get_account(context, &self.obligation.pubkey()).await;
        Obligation::unpack_unchecked(&obligation_account.data).unwrap()
    }

    pub async fn create(&self, context: &mut ProgramTestContext) -> transport::Result<()> {
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
                    &self.liquidity_info.liquidity_pubkey,
                    &self.collateral_info.collateral_pubkey,
                    &self.market_info.market.pubkey(),
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
}
