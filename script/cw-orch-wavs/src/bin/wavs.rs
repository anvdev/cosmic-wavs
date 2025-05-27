use abstract_cw_multi_test::Contract;
use clap::Parser;
use cosmos_sdk_proto::Any;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Decimal};
use cw_infuser::msg::InstantiateMsg;
use cw_orch::{
    daemon::{DaemonBuilder, TxSender},
    prelude::*,
};
use cw_orch_wavs::networks::{BITSONG_MAINNET, BITSONG_TESTNET, LOCAL_NETWORK1};
use tokio::runtime::Runtime;
/// MsgAddAuthenticatorRequest defines the Msg/AddAuthenticator request type.

#[cw_serde]
pub struct CosmwasmAuthenticatorInitData {
    pub contract: String,
    pub params: Vec<u8>,
}

fn cw721_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw721_base::entry::execute,
        cw721_base::entry::instantiate,
        cw721_base::entry::query,
    );
    Box::new(contract)
}

#[cw_serde]
pub struct MsgAddAuthenticator {
    pub sender: String,
    pub authenticator_type: String,
    pub data: Vec<u8>,
}

pub const COMPONENT: &str = "cosmic-wavs-demo-infusion.wasm";

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

    // deploy local cosmos chain if enabled

    if args.network.as_str() == "local" {
        deploy_local_cosmos_node()?;
    }

    if let Err(ref err) = deploy_wavs(bitsong_chain.into()) {
        log::error!("{}", err);
        err.chain().skip(1).for_each(|cause| log::error!("because: {}", cause));

        ::std::process::exit(1);
    }
}

fn deploy_wavs(network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));

    let wavs_controller_mnemonic = "";
    let wavs_controller_bech32_address = "";

    let urls = network.grpc_urls.to_vec();
    // for url in urls {
    //     rt.block_on(ping_grpc(&url))?;
    // }

    let cosmos =
        DaemonBuilder::new(network.clone()).handle(rt.handle()).mnemonic(MNEMONIC).build()?;
    //  deploy & configure btsg-account contracts to cosmos network
    let bs_accounts =
        btsg_account_scripts::BtsgAccountSuite::deploy_on(cosmos.clone(), cosmos.sender_addr())?;

    //  deploy cw-infusion coontract to cosmos networkk
    let infuser = cw_infuser_scripts::CwInfuser::new(cosmos.clone());
    if let Some(res) = infuser.upload_if_needed()? {
        match res.code {
            _ => {}
        }
    };

    // register custom authenticator to account
    let register_smart_account = prost_types::Any {
        type_url: "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".into(),
        value: to_json_binary(&MsgAddAuthenticator {
            sender: cosmos.sender_addr().to_string(),
            authenticator_type: "CosmwasmAuthenticatorV1".into(),
            data: to_json_binary(&CosmwasmAuthenticatorInitData {
                contract: "suite.wavs.address()?.to_string()".into(),
                params: vec![],
            })?
            .to_vec(),
        })?
        .to_vec(),
    };

    // broadcast tx
    let res = cosmos.commit_any(
        vec![register_smart_account],
        "Tuning account to the Cosmic Wavs Frequency...".into(),
    )?;
    // handle response
    match res.code {
        _ => {}
    }

    // configure infusion with wavs support enabled
    infuser.instantiate(
        &InstantiateMsg {
            contract_owner: None,
            owner_fee: Decimal::zero(),
            min_creation_fee: Some(Coin::new(100u128, "ubtsg")),
            min_infusion_fee: Some(Coin::new(100u128, "ubtsg")),
            min_per_bundle: None,
            max_per_bundle: None,
            max_bundles: None,
            max_infusions: None,
            cw721_code_id: 1u64,
            wavs_public_key: None,
        },
        None,
        &[],
    )?;

    // deploy wavs service (eth chain, eigenlayer stuff)
    deploy_wavs_infra();

    // confirm wavs updated the contract state

    Ok(())
}

fn deploy_wavs_infra() -> Result<(), anyhow::Error> {
    // spin up local eth network or deploy on desired network

    //
    Ok(())
}
fn deploy_local_cosmos_node() -> Result<(), anyhow::Error> {
    // spin up local docker image with funded genesis state to use throughout the app.

    // spec out this part, as this is crucial

    Ok(())
}
