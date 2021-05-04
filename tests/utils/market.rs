use crate::state::*;
use everlend_lending::*;
use solana_program::system_instruction;
use solana_program_test::ProgramTestContext;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

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

    pub async fn init_transaction(&self, test_context: &mut ProgramTestContext) -> Transaction {
        let rent = &test_context.banks_client.get_rent().await.unwrap();

        let mut transaction = Transaction::new_with_payer(
            &[
                system_instruction::create_account(
                    &test_context.payer.pubkey(),
                    &self.market.pubkey(),
                    rent.minimum_balance(Market::LEN),
                    Market::LEN as u64,
                    &id(),
                ),
                instruction::init_market(&id(), &self.market.pubkey(), &self.owner.pubkey())
                    .unwrap(),
            ],
            Some(&test_context.payer.pubkey()),
        );

        transaction.sign(
            &[&test_context.payer, &self.market, &self.owner],
            test_context.last_blockhash,
        );
        transaction
    }
}
