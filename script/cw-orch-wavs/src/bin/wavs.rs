use abstract_cw_multi_test::Contract;
use anyhow::Context;
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
use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::{runtime::Runtime, time::sleep};

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

pub const COMPONENT: &str = "cosmic-wavs-demo-infusion.wasm";
pub const DOCKER_COMPOSE_PATH: &str = "cosmic-wavs-demo-infusion.wasm";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Network to deploy on: main, testnet, local
    #[clap(short, long)]
    network: String,
    /// PAth to dockercomposefile for deploying eth & cosmos network
    #[clap(short, long)]
    docker_compose: String,
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

    if let Err(ref err) = deploy_wavs(&args.network, bitsong_chain.into()) {
        log::error!("{}", err);
        err.chain().skip(1).for_each(|cause| log::error!("because: {}", cause));

        ::std::process::exit(1);
    }
}

fn deploy_wavs(chain: &str, network: ChainInfoOwned) -> anyhow::Result<()> {
    let rt = Runtime::new()?;
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));

    // deploy wavs service (eth chain, eigenlayer stuff)
    // simulating the existing logic in the makefile to:
    // -  deploy a local ethereum network
    // -  deploy the eigenlayer contract
    deploy_wavs_infra();
    // -  deploy a cosmos chain
    if chain == "local" {
        deploy_local_cosmos_node();
    }
    // -  deploy the nft & other contracts
    // -  register a new service
    // - simulate a trigger for the service to perform its designed logic
    deploy_eigenlayer_contracts();

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

    // confirm wavs updated the contract state

    Ok(())
}

async fn deploy_wavs_infra() -> Result<(), anyhow::Error> {
    // Clean up existing Docker data (equivalent to `clean-docker` and `rm .docker/*.json`)
    if Path::new(".docker").exists() {
        fs::remove_dir_all(".docker").context("Failed to remove .docker directory")?;
    }
    fs::create_dir_all(".docker").context("Failed to create .docker directory")?;

    // Copy .env.example to .env if it doesn't exist
    if !Path::new(".env").exists() {
        fs::copy(".env.example", ".env").context("Failed to copy .env.example to .env")?;
    }

    // Start Anvil in the background
    let anvil_process = Command::new("anvil")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start Anvil")?;

    // Ensure Anvil has time to start
    sleep(Duration::from_secs(2)).await;

    // Start Docker Compose for WAVS
    let status = Command::new("docker")
        .args(["compose", "-f", DOCKER_COMPOSE_PATH, "up", "-d"])
        .status()
        .context("Failed to run docker compose up")?;

    if !status.success() {
        // Clean up Anvil process on failure
        let _ = Command::new("kill").arg(anvil_process.id().to_string()).status();
        return Err(anyhow::anyhow!("Docker compose failed with status: {}", status));
    }

    Ok(())
}

// Deploys Eigenlayer contracts and returns the ServiceManager address
async fn deploy_eigenlayer_contracts(rpc_url: &str) -> Result<(), anyhow::Error> {
    Ok(())
}

// Placeholder for deploy_mock_service_manager (unchanged)
async fn deploy_mock_service_manager(_rpc_url: &str) -> Result<(), anyhow::Error> {
    // Implement actual contract deployment logic here
    unimplemented!("deploy_mock_service_manager not implemented")
}

async fn deploy_service(
    component_filename: &str,
    trigger_event: &str,
    service_trigger_addr: &str,
    service_submission_addr: &str,
    service_config: &str,
) -> Result<(), anyhow::Error> {
    // Deploy the WAVS component service
    let wavs_cmd = "wavs"; // Replace with actual WAVS command or binary path
    let data_dir = "/data/.docker";
    let home_dir = "/data";
    let component_path = format!("/data/compiled/{}", component_filename);

    let status = Command::new(wavs_cmd)
        .args([
            "deploy-service",
            "--log-level=info",
            &format!("--data={}", data_dir),
            &format!("--home={}", home_dir),
            &format!("--component={}", component_path),
            &format!("--trigger-event-name={}", trigger_event),
            &format!("--trigger-address={}", service_trigger_addr),
            &format!("--submit-address={}", service_submission_addr),
            &format!("--service-config={}", service_config),
        ])
        .status()
        .context("Failed to run WAVS deploy-service")?;

    if !status.success() {
        return Err(anyhow::anyhow!("WAVS deploy-service failed with status: {}", status));
    }

    Ok(())
}
