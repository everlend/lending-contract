use everlend_lending::pyth::{load_mut, Price};
use solana_program::pubkey::Pubkey;
use solana_program_test::{find_file, read_file, ProgramTest};
use solana_sdk::{account::Account, signature::Keypair, signer::Signer};

const ORACLE_SECRET: &[u8] = &[
    34, 165, 56, 243, 236, 153, 203, 167, 72, 136, 131, 212, 217, 45, 33, 184, 51, 188, 81, 218,
    245, 67, 252, 172, 250, 244, 94, 239, 138, 13, 166, 47, 132, 99, 44, 63, 242, 187, 236, 116,
    168, 172, 11, 28, 66, 228, 151, 55, 166, 71, 44, 51, 64, 111, 49, 62, 187, 222, 61, 97, 138,
    87, 87, 216,
];

#[derive(Debug, Clone, Copy)]
pub struct TestOracle {
    pub product_pubkey: Pubkey,
    pub price_pubkey: Pubkey,
    pub price: i64,
}

impl TestOracle {
    pub fn new(product_pubkey: &Pubkey, price_pubkey: &Pubkey, price: i64) -> Self {
        Self {
            product_pubkey: *product_pubkey,
            price_pubkey: *price_pubkey,
            price,
        }
    }

    pub fn init(&self, test: &mut ProgramTest) {
        let oracle_program = Keypair::from_bytes(ORACLE_SECRET).unwrap();

        // Add Pyth product account
        test.add_account_with_file_data(
            self.product_pubkey,
            u32::MAX as u64,
            oracle_program.pubkey(),
            &format!("{}.bin", self.product_pubkey.to_string()),
        );

        // Add Pyth price account after setting the price
        let filename = &format!("{}.bin", self.price_pubkey.to_string());
        let mut pyth_price_data = read_file(find_file(filename).unwrap_or_else(|| {
            panic!("Unable to locate {}", filename);
        }));

        let mut pyth_price = load_mut::<Price>(pyth_price_data.as_mut_slice()).unwrap();

        println!("Price expo: {}", pyth_price.expo);

        pyth_price.valid_slot = 0;
        pyth_price.agg.price = self.price;

        test.add_account(
            self.price_pubkey,
            Account {
                lamports: u32::MAX as u64,
                data: pyth_price_data,
                owner: oracle_program.pubkey(),
                executable: false,
                rent_epoch: 0,
            },
        );
    }
}
