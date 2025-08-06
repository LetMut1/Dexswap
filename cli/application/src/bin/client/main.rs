mod command_processor;
mod utility;
use {
    clap::{
        Arg,
        Command,
    },
    command_processor::CommandProcessor,
    solana_rpc_client::rpc_client::RpcClient,
    solana_sdk::commitment_config::{
        CommitmentConfig,
        CommitmentLevel,
    },
    std::{
        error::Error,
        time::Duration,
    },
};
const LOGIC_ERROR: &str = "Logic error.";
fn main() -> Result<(), Box<dyn Error + 'static>> {
    Processor::process()
}
struct Processor;
impl Processor {
    fn process() -> Result<(), Box<dyn Error + 'static>> {
        const COMMAND_INITIALIZE: &str = "initialize";
        const COMMAND_DEPOSIT_FUNDS: &str = "deposit_funds";
        const COMMAND_WITHDRAW_FUNDS: &str = "withdraw_funds";
        const COMMAND_SWAP: &str = "swap";
        const ARGUMENT_INTERMEDIARY_MANAGER: &str = "intermediary_manager";
        const ARGUMENT_INTERMEDIARY_TRADER: &str = "intermediary_trader";
        const ARGUMENT_LAMPORTS_TO_TREASURY: &str = "lamports_to_treasury";
        const ARGUMENT_LAMPORTS_FROM_TREASURY: &str = "lamports_from_treasury";
        const ARGUMENT_INTERMEDIARY: &str = "intermediary";
        const ARGUMENT_AMOUNT_IN: &str = "amount_in";
        const ARGUMENT_MIN_AMOUNT_OUT: &str = "min_amount_out";
        const ARGUMENT_SOLANA_RPC_URL: &str = "solana_rpc_url";
        let command = clap::command!()
            .arg(Arg::new(ARGUMENT_SOLANA_RPC_URL).required(true).long(ARGUMENT_SOLANA_RPC_URL))
            .arg_required_else_help(true)
            .subcommand_required(true)
            .subcommand(
                Command::new(COMMAND_INITIALIZE)
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY_MANAGER).required(true).long(ARGUMENT_INTERMEDIARY_MANAGER).help("Fee payer keypair.json file path."))
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY_TRADER).required(true).long(ARGUMENT_INTERMEDIARY_TRADER).help("The keypair.json file path."))
                    .arg(Arg::new(ARGUMENT_LAMPORTS_TO_TREASURY).required(true).long(ARGUMENT_LAMPORTS_TO_TREASURY).help("Lamports to treasury.")),
            )
            .subcommand(
                Command::new(COMMAND_DEPOSIT_FUNDS)
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY).required(true).long(ARGUMENT_INTERMEDIARY).help("Intermediary pubkey."))
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY_MANAGER).required(true).long(ARGUMENT_INTERMEDIARY_MANAGER).help("Fee payer keypair.json file path."))
                    .arg(Arg::new(ARGUMENT_LAMPORTS_TO_TREASURY).required(true).long(ARGUMENT_LAMPORTS_TO_TREASURY).help("Lamports to treasury.")),
            )
            .subcommand(
                Command::new(COMMAND_WITHDRAW_FUNDS)
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY).required(true).long(ARGUMENT_INTERMEDIARY).help("Intermediary pubkey."))
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY_MANAGER).required(true).long(ARGUMENT_INTERMEDIARY_MANAGER).help("Fee payer keypair.json file path."))
                    .arg(Arg::new(ARGUMENT_LAMPORTS_FROM_TREASURY).required(true).long(ARGUMENT_LAMPORTS_FROM_TREASURY).help("Lamports from treasury.")),
            )
            .subcommand(
                Command::new(COMMAND_SWAP)
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY).required(true).long(ARGUMENT_INTERMEDIARY).help("Intermediary pubkey."))
                    .arg(Arg::new(ARGUMENT_INTERMEDIARY_TRADER).required(true).long(ARGUMENT_INTERMEDIARY_TRADER).help("Fee payer keypair.json file path."))
                    .arg(Arg::new(ARGUMENT_AMOUNT_IN).required(true).long(ARGUMENT_AMOUNT_IN).help("Amount in."))
                    .arg(Arg::new(ARGUMENT_MIN_AMOUNT_OUT).required(true).long(ARGUMENT_MIN_AMOUNT_OUT).help("Min amount out.")),
            );
        let arg_matches = command.get_matches();
        let solana_rpc_url = arg_matches.get_one::<String>(ARGUMENT_SOLANA_RPC_URL).unwrap();
        let rpc_client = RpcClient::new_with_timeout_and_commitment(
            solana_rpc_url.as_str(),
            Duration::from_secs(90),
            CommitmentConfig {
                commitment: CommitmentLevel::Finalized,
            },
        );
        match arg_matches.subcommand().unwrap() {
            (COMMAND_INITIALIZE, arg_matches_) => {
                CommandProcessor::initialize(
                    &rpc_client,
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY_MANAGER).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY_TRADER).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_LAMPORTS_TO_TREASURY).unwrap().parse::<u64>()?,
                )
            }
            (COMMAND_DEPOSIT_FUNDS, arg_matches_) => {
                CommandProcessor::deposit_funds(
                    &rpc_client,
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY_MANAGER).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_LAMPORTS_TO_TREASURY).unwrap().parse::<u64>()?,
                )
            }
            (COMMAND_WITHDRAW_FUNDS, arg_matches_) => {
                CommandProcessor::withdraw_funds(
                    &rpc_client,
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY_MANAGER).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_LAMPORTS_FROM_TREASURY).unwrap().parse::<u64>()?,
                )
            }
            (COMMAND_SWAP, arg_matches_) => {
                CommandProcessor::swap(
                    &rpc_client,
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_INTERMEDIARY_TRADER).unwrap().as_str(),
                    arg_matches_.get_one::<String>(ARGUMENT_AMOUNT_IN).unwrap().parse::<u64>()?,
                    arg_matches_.get_one::<String>(ARGUMENT_MIN_AMOUNT_OUT).unwrap().parse::<u64>()?,
                )
            }
            _ => Err(LOGIC_ERROR.into()),
        }
    }
}
