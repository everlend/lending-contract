use super::{create_token_account, liquidity::LiquidityInfo};
use solana_program_test::ProgramTestContext;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

#[derive(Debug)]
pub struct ProviderActor {
    pub owner: Keypair,
}

impl ProviderActor {
    pub fn new() -> Self {
        Self {
            owner: Keypair::new(),
        }
    }

    pub async fn create_liquidity_accounts(
        &self,
        context: &mut ProgramTestContext,
        liquidity_info: &LiquidityInfo,
    ) -> transport::Result<(Keypair, Keypair)> {
        let source = Keypair::new();
        let destination = Keypair::new();

        create_token_account(
            context,
            &source,
            &liquidity_info.token_mint.pubkey(),
            &self.owner.pubkey(),
        )
        .await
        .unwrap();

        create_token_account(
            context,
            &destination,
            &liquidity_info.pool_mint.pubkey(),
            &self.owner.pubkey(),
        )
        .await
        .unwrap();

        Ok((source, destination))
    }
}
