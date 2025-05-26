use clap::Parser;
use cosmos_sdk_proto::Any;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::to_json_binary;
use cw_orch::{
    daemon::{DaemonBuilder, TxSender},
    prelude::*,
};
use tokio::runtime::Runtime;

/// MsgAddAuthenticatorRequest defines the Msg/AddAuthenticator request type.

#[cw_serde]
pub struct CosmwasmAuthenticatorInitData {
    pub contract: String,
    pub params: Vec<u8>,
}

#[cw_serde]
pub struct MsgAddAuthenticator {
    pub sender: String,
    pub authenticator_type: String,
    pub data: Vec<u8>,
}

// todo: move to .env file
pub const MNEMONIC: &str =
    "garage dial step tourist hint select patient eternal lesson raccoon shaft palace flee purpose vivid spend place year file life cliff winter race fox";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network to deploy on: main, testnet, local
    #[clap(short, long)]
    network: String,
    // optional address to broadcast msg on behalf of. This address must have authorized the wallet calling these scripts
    // #[clap(short, long)]
    // authz: Option<String>,
}

fn main() {
    // parse cargo command arguments for network type
    let args = Args::parse();
    // logs any errors
    env_logger::init();

    println!("Deploying Bitsong Accounts Framework...");
    let bitsong_chain = match args.network.as_str() {
        "main" => BITSONG_MAINNET.to_owned(),
        "testnet" => BITSONG_TESTNET.to_owned(),
        "local" => LOCAL_NETWORK1.to_owned(),
        _ => panic!("Invalid network"),
    };

    if let Err(ref err) = deploy_wavs(bitsong_chain.into()) {
        log::error!("{}", err);
        err.chain().skip(1).for_each(|cause| log::error!("because: {}", cause));

        ::std::process::exit(1);
    }
}

fn deploy_wavs(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let urls = network.grpc_urls.to_vec();
    for url in urls {
        rt.block_on(ping_grpc(&url))?;
    }

    let chain =
        DaemonBuilder::new(network.clone()).handle(rt.handle()).mnemonic(MNEMONIC).build()?;

    let suite = BtsgAccountSuite::deploy_on(chain.clone(), chain.sender().address())?;
    // deploy cw-infuser

    // register custom authenticator to account
    let register_smart_account = prost_types::Any {
        type_url: "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".into(),
        value: to_json_binary(&MsgAddAuthenticator {
            sender: chain.sender_addr().to_string(),
            authenticator_type: "CosmwasmAuthenticatorV1".into(),
            data: to_json_binary(&CosmwasmAuthenticatorInitData {
                contract: suite.wavs.address()?.to_string(),
                params: vec![],
            })?
            .to_vec(),
        })?
        .to_vec(),
    };
    // broadcast tx
    let res = chain.commit_any(
        vec![register_smart_account],
        "Tuning account to the Cosmic Wavs Frequency...".into(),
    )?;

    // handle response

    // deploy wavs service (eth chain, eigenlayer stuff)

    // infuse nft bundle

    // confirm wavs updated the contract state

    Ok(())
}
