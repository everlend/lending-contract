use clap::{
    arg_enum, crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg,
    SubCommand,
};
use everlend_lending::{
    find_program_address, instruction,
    state::{ui_ratio_to_ratio, Collateral, CollateralStatus, Liquidity, LiquidityStatus, Market},
};
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{keypair_of, pubkey_of, value_of},
    input_validators::{
        is_amount, is_keypair, is_keypair_or_ask_keyword, is_pubkey, is_url_or_moniker,
    },
    keypair::signer_from_path,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    native_token::lamports_to_sol, program_pack::Pack, pubkey::Pubkey, system_instruction,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use std::{env, process::exit};

#[allow(dead_code)]
struct Config {
    rpc_client: RpcClient,
    verbose: bool,
    owner: Box<dyn Signer>,
    fee_payer: Box<dyn Signer>,
}

type Error = Box<dyn std::error::Error>;
type CommandResult = Result<Option<Transaction>, Error>;

arg_enum! {
    #[derive(Debug)]
    pub enum ArgTokenStatus {
        Inactive = 0,
        Active = 1,
        InactiveAndVisible = 2,
    }
}

impl From<ArgTokenStatus> for LiquidityStatus {
    fn from(other: ArgTokenStatus) -> LiquidityStatus {
        match other {
            ArgTokenStatus::Inactive => LiquidityStatus::Inactive,
            ArgTokenStatus::Active => LiquidityStatus::Active,
            ArgTokenStatus::InactiveAndVisible => LiquidityStatus::InactiveAndVisible,
        }
    }
}

impl From<ArgTokenStatus> for CollateralStatus {
    fn from(other: ArgTokenStatus) -> CollateralStatus {
        match other {
            ArgTokenStatus::Inactive => CollateralStatus::Inactive,
            ArgTokenStatus::Active => CollateralStatus::Active,
            ArgTokenStatus::InactiveAndVisible => CollateralStatus::InactiveAndVisible,
        }
    }
}

macro_rules! unique_signers {
    ($vec:ident) => {
        $vec.sort_by_key(|l| l.pubkey());
        $vec.dedup();
    };
}

fn check_fee_payer_balance(config: &Config, required_balance: u64) -> Result<(), Error> {
    let balance = config.rpc_client.get_balance(&config.fee_payer.pubkey())?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            config.fee_payer.pubkey(),
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}

fn command_create_market(config: &Config, market_keypair: Option<Keypair>) -> CommandResult {
    let market_keypair = market_keypair.unwrap_or_else(Keypair::new);

    println!("Creating market {}", market_keypair.pubkey());

    let market_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(Market::LEN)?;
    let total_rent_free_balances = market_balance;

    let mut tx = Transaction::new_with_payer(
        &[
            // Market account
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &market_keypair.pubkey(),
                market_balance,
                Market::LEN as u64,
                &everlend_lending::id(),
            ),
            // Initialize pool account
            instruction::init_market(
                &everlend_lending::id(),
                &market_keypair.pubkey(),
                &config.owner.pubkey(),
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        config,
        total_rent_free_balances + fee_calculator.calculate_fee(&tx.message()),
    )?;

    let mut signers = vec![
        config.fee_payer.as_ref(),
        config.owner.as_ref(),
        &market_keypair,
    ];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn command_market_info(config: &Config, market_pubkey: &Pubkey) -> CommandResult {
    let market_account = config.rpc_client.get_account(&market_pubkey)?;
    let market = Market::unpack(&market_account.data)?;
    let (market_authority, _) = find_program_address(&everlend_lending::id(), market_pubkey);

    println!("{:#?}", market);

    println!("Liquidity tokens:");
    for index in 0..market.liquidity_tokens {
        let liquidity_pubkey = Pubkey::create_with_seed(
            &market_authority,
            &format!("liquidity{:?}", index),
            &everlend_lending::id(),
        )?;
        let liquidity_account = config.rpc_client.get_account(&liquidity_pubkey)?;
        let liquidity = Liquidity::unpack(&liquidity_account.data)?;

        println!("{:#?}", liquidity);
    }

    println!("Collateral tokens:");
    for index in 0..market.collateral_tokens {
        let collateral_pubkey = Pubkey::create_with_seed(
            &market_authority,
            &format!("collateral{:?}", index),
            &everlend_lending::id(),
        )?;
        let collateral_account = config.rpc_client.get_account(&collateral_pubkey)?;
        let collateral = Collateral::unpack(&collateral_account.data)?;

        println!("{:#?}", collateral);
    }

    Ok(None)
}

fn command_create_liquidity_token(
    config: &Config,
    market_pubkey: &Pubkey,
    token_mint: &Pubkey,
) -> CommandResult {
    let market_account = config.rpc_client.get_account(&market_pubkey)?;
    let market = Market::unpack(&market_account.data)?;

    // Generate new accounts
    let token_account = Keypair::new();
    let pool_mint = Keypair::new();

    // Calculate liquidity pubkey
    let seed = format!("liquidity{:?}", market.liquidity_tokens);
    let (market_authority, _) = find_program_address(&everlend_lending::id(), market_pubkey);
    let liquidity_pubkey =
        Pubkey::create_with_seed(&market_authority, &seed, &everlend_lending::id())?;

    println!("Liquidity: {}", &liquidity_pubkey);
    println!("Token mint: {}", &token_mint);
    println!("Token account: {}", &token_account.pubkey());
    println!("Pool mint: {}", &pool_mint.pubkey());
    println!("Market: {}", &market_pubkey);

    let token_account_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?;
    let pool_mint_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;

    let total_rent_free_balances = token_account_balance + pool_mint_balance;

    let mut tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &token_account.pubkey(),
                token_account_balance,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &pool_mint.pubkey(),
                pool_mint_balance,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            instruction::create_liquidity_token(
                &everlend_lending::id(),
                &liquidity_pubkey,
                &token_mint,
                &token_account.pubkey(),
                &pool_mint.pubkey(),
                &market_pubkey,
                &config.owner.pubkey(),
                &None,
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        config,
        total_rent_free_balances + fee_calculator.calculate_fee(&tx.message()),
    )?;

    let mut signers = vec![
        config.fee_payer.as_ref(),
        config.owner.as_ref(),
        &token_account,
        &pool_mint,
    ];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn command_create_collateral_token(
    config: &Config,
    market_pubkey: &Pubkey,
    token_mint: &Pubkey,
    ui_ratio_initial: f64,
    ui_ratio_healthy: f64,
) -> CommandResult {
    let market_account = config.rpc_client.get_account(&market_pubkey)?;
    let market = Market::unpack(&market_account.data)?;

    // Generate new accounts
    let token_account = Keypair::new();
    let ratio_initial = ui_ratio_to_ratio(ui_ratio_initial);
    let ratio_healthy = ui_ratio_to_ratio(ui_ratio_healthy);

    // Calculate collateral pubkey
    let seed = format!("collateral{:?}", market.collateral_tokens);
    let (market_authority, _) = find_program_address(&everlend_lending::id(), market_pubkey);
    let collateral_pubkey =
        Pubkey::create_with_seed(&market_authority, &seed, &everlend_lending::id())?;

    println!("Collateral: {}", &collateral_pubkey);
    println!(
        "Ratio initial: {}, ratio healthy: {}",
        ui_ratio_initial, ui_ratio_healthy
    );
    println!("Token mint: {}", &token_mint);
    println!("Token account: {}", &token_account.pubkey());
    println!("Market: {}", &market_pubkey);

    let token_account_balance = config
        .rpc_client
        .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)?;

    let total_rent_free_balances = token_account_balance;

    let mut tx = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &config.fee_payer.pubkey(),
                &token_account.pubkey(),
                token_account_balance,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            instruction::create_collateral_token(
                &everlend_lending::id(),
                ratio_initial,
                ratio_healthy,
                &collateral_pubkey,
                &token_mint,
                &token_account.pubkey(),
                &market_pubkey,
                &config.owner.pubkey(),
                &None,
            )?,
        ],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(
        config,
        total_rent_free_balances + fee_calculator.calculate_fee(&tx.message()),
    )?;

    let mut signers = vec![
        config.fee_payer.as_ref(),
        config.owner.as_ref(),
        &token_account,
    ];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn command_update_liquidity_token(
    config: &Config,
    liquidity_pubkey: Option<Pubkey>,
    market_pubkey: Option<Pubkey>,
    liquidity_index: Option<u64>,
    status: LiquidityStatus,
) -> CommandResult {
    let liquidity_pubkey = liquidity_pubkey.unwrap_or_else(|| {
        let seed = format!("liquidity{:?}", liquidity_index.unwrap());
        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &market_pubkey.unwrap());

        Pubkey::create_with_seed(&market_authority, &seed, &everlend_lending::id()).unwrap()
    });

    let liquidity_account = config.rpc_client.get_account(&liquidity_pubkey)?;
    let liquidity = Liquidity::unpack(&liquidity_account.data)?;

    println!("Liquidity: {}", &liquidity_pubkey);
    println!("New status: {:?}", status);

    let mut tx = Transaction::new_with_payer(
        &[instruction::update_liquidity_token(
            &everlend_lending::id(),
            status,
            &liquidity_pubkey,
            &liquidity.market,
            &config.owner.pubkey(),
        )?],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&tx.message()))?;

    let mut signers = vec![config.fee_payer.as_ref(), config.owner.as_ref()];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn command_update_collateral_token(
    config: &Config,
    collateral_pubkey: Option<Pubkey>,
    market_pubkey: Option<Pubkey>,
    collateral_index: Option<u64>,
    status: CollateralStatus,
    ui_ratio_initial: Option<f64>,
    ui_ratio_healthy: Option<f64>,
) -> CommandResult {
    let collateral_pubkey = collateral_pubkey.unwrap_or_else(|| {
        let seed = format!("collateral{:?}", collateral_index.unwrap());
        let (market_authority, _) =
            find_program_address(&everlend_lending::id(), &market_pubkey.unwrap());

        Pubkey::create_with_seed(&market_authority, &seed, &everlend_lending::id()).unwrap()
    });

    let collateral_account = config.rpc_client.get_account(&collateral_pubkey)?;
    let collateral = Collateral::unpack(&collateral_account.data)?;

    println!("Liquidity: {}", &collateral_pubkey);
    println!("New status: {:?}", status);

    let ratio_initial = match ui_ratio_initial {
        Some(ui_ratio_initial) => {
            println!("New ratio initial: {:?}", ui_ratio_initial);
            ui_ratio_to_ratio(ui_ratio_initial)
        }
        _ => collateral.ratio_initial,
    };
    let ratio_healthy = match ui_ratio_healthy {
        Some(ui_ratio_healthy) => {
            println!("New ration healthy: {:?}", ui_ratio_healthy);
            ui_ratio_to_ratio(ui_ratio_healthy)
        }
        _ => collateral.ratio_healthy,
    };

    let mut tx = Transaction::new_with_payer(
        &[instruction::update_collateral_token(
            &everlend_lending::id(),
            status,
            ratio_initial,
            ratio_healthy,
            &collateral_pubkey,
            &collateral.market,
            &config.owner.pubkey(),
        )?],
        Some(&config.fee_payer.pubkey()),
    );

    let (recent_blockhash, fee_calculator) = config.rpc_client.get_recent_blockhash()?;
    check_fee_payer_balance(config, fee_calculator.calculate_fee(&tx.message()))?;

    let mut signers = vec![config.fee_payer.as_ref(), config.owner.as_ref()];

    unique_signers!(signers);
    tx.sign(&signers, recent_blockhash);

    Ok(Some(tx))
}

fn main() {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .short("u")
                .long("url")
                .value_name("URL_OR_MONIKER")
                .takes_value(true)
                .global(true)
                .validator(is_url_or_moniker)
                .help(
                    "URL for Solana's JSON RPC or moniker (or their first letter): \
                       [mainnet-beta, testnet, devnet, localhost] \
                    Default from the configuration file.",
                ),
        )
        .arg(
            Arg::with_name("owner")
                .long("owner")
                .value_name("KEYPAIR")
                .validator(is_keypair)
                .takes_value(true)
                .global(true)
                .help(
                    "Specify the token owner account. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .arg(fee_payer_arg().global(true))
        .subcommand(
            SubCommand::with_name("create-market")
                .about("Create a new market")
                .arg(
                    Arg::with_name("market_keypair")
                        .long("keypair")
                        .validator(is_keypair_or_ask_keyword)
                        .value_name("PATH")
                        .takes_value(true)
                        .help("Market keypair [default: new keypair]"),
                ),
        )
        .subcommand(
            SubCommand::with_name("market-info")
                .about("Print out market information and tokens")
                .arg(
                    Arg::with_name("market_pubkey")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .index(1)
                        .help("Market pubkey"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-liquidity")
                .about("Add a liquidity token")
                .arg(
                    Arg::with_name("market_pubkey")
                        .long("market")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Market pubkey"),
                )
                .arg(
                    Arg::with_name("token_mint")
                        .long("token")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Mint for the token to be added as liquidity"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create-collateral")
                .about("Add a collateral token")
                .arg(
                    Arg::with_name("market_pubkey")
                        .long("market")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Market pubkey"),
                )
                .arg(
                    Arg::with_name("token_mint")
                        .long("token")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required(true)
                        .help("Mint for the token to be added as liquidity"),
                )
                .arg(
                    Arg::with_name("ratio_initial")
                        .long("ratio-initial")
                        .validator(is_amount)
                        .value_name("RATIO")
                        .takes_value(true)
                        .default_value("0.5")
                        .help("Ratio initial"),
                )
                .arg(
                    Arg::with_name("ratio_healthy")
                        .long("ratio-healthy")
                        .validator(is_amount)
                        .value_name("RATIO")
                        .takes_value(true)
                        .default_value("0.75")
                        .help("Ratio healthy"),
                ),
        )
        .subcommand(
            SubCommand::with_name("update-liquidity")
                .about("Update a liquidity token")
                .arg(
                    Arg::with_name("liquidity_pubkey")
                        .long("pubkey")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required_unless_all(&["market_pubkey", "liquidity_index"])
                        .help("Liquidity pubkey"),
                )
                .arg(
                    Arg::with_name("market_pubkey")
                        .long("market")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required_unless("liquidity_pubkey")
                        .help("Market pubkey"),
                )
                .arg(
                    Arg::with_name("liquidity_index")
                        .long("index")
                        .value_name("NUMBER")
                        .takes_value(true)
                        .required_unless("liquidity_pubkey")
                        .requires("market_pubkey")
                        .help("Liquidity index"),
                )
                .arg(
                    Arg::with_name("status")
                        .value_name("NEW_STATUS")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&ArgTokenStatus::variants())
                        .index(1)
                        .help("New liquidity status."),
                ),
        )
        .subcommand(
            SubCommand::with_name("update-collateral")
                .about("Update a collateral token")
                .arg(
                    Arg::with_name("collateral_pubkey")
                        .long("pubkey")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required_unless_all(&["market_pubkey", "collateral_index"])
                        .help("Liquidity pubkey"),
                )
                .arg(
                    Arg::with_name("market_pubkey")
                        .long("market")
                        .validator(is_pubkey)
                        .value_name("ADDRESS")
                        .takes_value(true)
                        .required_unless("collateral_pubkey")
                        .help("Market pubkey"),
                )
                .arg(
                    Arg::with_name("collateral_index")
                        .long("index")
                        .value_name("NUMBER")
                        .takes_value(true)
                        .required_unless("collateral_pubkey")
                        .requires("market_pubkey")
                        .help("Liquidity index"),
                )
                .arg(
                    Arg::with_name("status")
                        .value_name("NEW_STATUS")
                        .takes_value(true)
                        .required(true)
                        .possible_values(&ArgTokenStatus::variants())
                        .index(1)
                        .help("New collateral status."),
                )
                .arg(
                    Arg::with_name("ratio_initial")
                        .long("ratio-initial")
                        .validator(is_amount)
                        .value_name("RATIO")
                        .takes_value(true)
                        .help("Ratio initial"),
                )
                .arg(
                    Arg::with_name("ratio_healthy")
                        .long("ratio-healthy")
                        .validator(is_amount)
                        .value_name("RATIO")
                        .takes_value(true)
                        .help("Ratio healthy"),
                ),
        )
        .get_matches();

    let mut wallet_manager = None;
    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };

        let json_rpc_url = value_t!(matches, "json_rpc_url", String)
            .unwrap_or_else(|_| cli_config.json_rpc_url.clone());

        let owner = signer_from_path(
            &matches,
            &cli_config.keypair_path,
            "owner",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

        let fee_payer = signer_from_path(
            &matches,
            &cli_config.keypair_path,
            "fee_payer",
            &mut wallet_manager,
        )
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });

        let verbose = matches.is_present("verbose");

        Config {
            rpc_client: RpcClient::new_with_commitment(json_rpc_url, CommitmentConfig::confirmed()),
            verbose,
            owner,
            fee_payer,
        }
    };

    solana_logger::setup_with_default("solana=info");

    let _ = match matches.subcommand() {
        ("create-market", Some(arg_matches)) => {
            let market_keypair = keypair_of(arg_matches, "market_keypair");
            command_create_market(&config, market_keypair)
        }
        ("market-info", Some(arg_matches)) => {
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey").unwrap();
            command_market_info(&config, &market_pubkey)
        }
        ("create-liquidity", Some(arg_matches)) => {
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey").unwrap();
            let token_mint = pubkey_of(arg_matches, "token_mint").unwrap();
            command_create_liquidity_token(&config, &market_pubkey, &token_mint)
        }
        ("create-collateral", Some(arg_matches)) => {
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey").unwrap();
            let token_mint = pubkey_of(arg_matches, "token_mint").unwrap();
            let ratio_initial = value_of::<f64>(arg_matches, "ratio_initial").unwrap();
            let ratio_healthy = value_of::<f64>(arg_matches, "ratio_healthy").unwrap();
            command_create_collateral_token(
                &config,
                &market_pubkey,
                &token_mint,
                ratio_initial,
                ratio_healthy,
            )
        }
        ("update-liquidity", Some(arg_matches)) => {
            let liquidity_pubkey = pubkey_of(arg_matches, "liquidity_pubkey");
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey");
            let liquidity_index = value_of::<u64>(arg_matches, "liquidity_index");
            let status = value_t!(arg_matches, "status", ArgTokenStatus).unwrap();
            command_update_liquidity_token(
                &config,
                liquidity_pubkey,
                market_pubkey,
                liquidity_index,
                LiquidityStatus::from(status),
            )
        }
        ("update-collateral", Some(arg_matches)) => {
            let collateral_pubkey = pubkey_of(arg_matches, "collateral_pubkey");
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey");
            let collateral_index = value_of::<u64>(arg_matches, "collateral_index");
            let status = value_t!(arg_matches, "status", ArgTokenStatus).unwrap();
            let ratio_initial = value_of::<f64>(arg_matches, "ratio_initial");
            let ratio_healthy = value_of::<f64>(arg_matches, "ratio_healthy");
            command_update_collateral_token(
                &config,
                collateral_pubkey,
                market_pubkey,
                collateral_index,
                CollateralStatus::from(status),
                ratio_initial,
                ratio_healthy,
            )
        }
        _ => unreachable!(),
    }
    .and_then(|tx| {
        if let Some(tx) = tx {
            let signature = config
                .rpc_client
                .send_and_confirm_transaction_with_spinner(&tx)?;
            println!("Signature: {}", signature);
        }
        Ok(())
    })
    .map_err(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}
