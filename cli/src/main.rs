use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, SubCommand,
};
use everlend_lending::{find_program_address, instruction, state::Market};
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{keypair_of, pubkey_of},
    input_validators::{is_keypair, is_keypair_or_ask_keyword, is_pubkey, is_url_or_moniker},
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
            SubCommand::with_name("create-liquidity-token")
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
        ("create-liquidity-token", Some(arg_matches)) => {
            let market_pubkey = pubkey_of(arg_matches, "market_pubkey").unwrap();
            let token_mint = pubkey_of(arg_matches, "token_mint").unwrap();
            command_create_liquidity_token(&config, &&market_pubkey, &token_mint)
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
