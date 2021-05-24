use everlend_lending::{id, instruction, state::Market};
use solana_program::{borsh::get_packed_len, system_instruction};
use solana_program_test::ProgramTestContext;
use solana_sdk::transaction::Transaction;
use solana_sdk::{
    signature::{Keypair, Signer},
    transport,
};

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
}
