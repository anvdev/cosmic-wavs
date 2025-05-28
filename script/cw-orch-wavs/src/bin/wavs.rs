use anyhow::Context;
use clap::Parser;
use commonware_cryptography::{Bls12381, Signer};
use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Decimal};
use cw_infuser::msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns};
use cw_orch::{
    core::serde_json::json,
    daemon::{senders::CosmosSender, DaemonBase, DaemonBuilder, TxSender},
    prelude::*,
};
use cw_orch_wavs::networks::{BITSONG_MAINNET, BITSONG_TESTNET, LOCAL_NETWORK1};
use secp256k1::All;
use std::{
    env, fs,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
use tokio::{
    runtime::{Handle, Runtime},
    time::sleep,
};

#[cw_serde]
pub struct CosmwasmAuthenticatorInitData {
    pub contract: String,
    pub params: Vec<u8>,
}
#[cw_serde]
pub struct DeployInfusionDemo {
    pub cosmos: DaemonBase<CosmosSender<All>>,
    pub bs_accounts: BtsgAccountSuite<DaemonBase<CosmosSender<All>>>,
    pub infuser: CwInfuser<DaemonBase<CosmosSender<All>>>,
}

#[cw_serde]
pub struct MsgAddAuthenticator {
    pub sender: String,
    pub authenticator_type: String,
    pub data: Vec<u8>,
}

pub const WAVS_COMPONENT: &str = "cosmic-wavs-demo-infusion.wasm";
pub const INFUSION_TRIGGER_EVENT: &str = "cw-infusion";

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
    // rt.block_on(assert_wallet_balance(vec![network.clone()]));
    let wavs_bech32_addr = env::var("WAVS_CONTROLLER_ADDRESS").unwrap_or_else(|_| "".to_string());
    let service_config_file_path = env::var("SERVICE_CONFIG").unwrap_or_else(|_| "".to_string());
    let service_sub_addr = env::var("SERVICE_SUBMISSION_ADDR").unwrap_or_else(|_| "".to_string());
    let service_trigger_addr = env::var("SERVICE_TRIGGER_ADDR").unwrap_or_else(|_| "".to_string());

    setup_local_crypto_keys()?;
    // deploy networks
    rt.block_on(deploy_wavs_infra())?;
    // deploy cosmos smart contracct
    let DeployInfusionDemo { cosmos, bs_accounts, infuser } =
        match rt.block_on(deploy_infusion_demo(rt.handle(), network)) {
            Ok(value) => value,
            Err(e) => return Err(e.into()),
        }?;

    // deploy eth contracts (wavs service)
    rt.block_on(deploy_wavs_service(
        WAVS_COMPONENT,
        INFUSION_TRIGGER_EVENT,
        &service_trigger_addr,
        &service_sub_addr,
        &service_config_file_path,
    ))?;
    // run demo
    rt.block_on(run_infusion_demo())?;

    Ok(())
}

/// Creates cryptographic keys for integration tests
fn setup_local_crypto_keys() -> Result<(), anyhow::Error> {
    // Define output path for keys
    let home_dir = env::var("HOME").context("HOME environment variable not set")?;
    let config_dir = Path::new(&home_dir).join(".omnibus/config");

    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    let keys_path = config_dir.join("keys.json");

    // Generate secp256k1 keypair
    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    // Convert to strings (simplified; in practice, derive bech32 address)
    let secp256k1_private = hex::encode(secret_key.to_bytes());
    let secp256k1_public = hex::encode(public_key.serialize());

    // Generate BLS12-381 keypair (placeholder)
    let mut bls12 = Bls12381::new(&mut OsRng);
    let private_key = bls12.private_key();
    let public_key = bls12.public_key();

    // Create JSON structure
    let keys = json!({
        "secp256k1": {
            "private_key": secp256k1_private,
            "public_key": secp256k1_public,
            "address": env::var("WAVS_CONTROLLER_ADDRESS").unwrap_or_else(|_| "cosmos1...".to_string())
        },
        "bls12_381": {
            "private_key": private_key.to_string(),
            "public_key": public_key.to_string(),
        }
    });

    // Write to keys.json (expecccted  to be used in cosmos node docker entrypoint)
    let keys_file = File::create(&keys_path).context("Failed to create keys.json")?;
    serde_json::to_writer_pretty(keys_file, &keys).context("Failed to write keys.json")?;

    Ok(())
}

/// Runs Anvil & Docker Compose to deploy networks & service
async fn deploy_wavs_infra() -> Result<(), anyhow::Error> {
    let docker_compose_path = env::var("DOCKER_COMPOSE_PATH")
        .unwrap_or_else(|e| "missing docker-compose.yml".to_string());
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
        .args(["compose", "-f", &docker_compose_path, "up", "-d"])
        .status()
        .context("Failed to run docker compose up")?;

    if !status.success() {
        // Clean up Anvil process on failure
        let _ = Command::new("kill").arg(anvil_process.id().to_string()).status();
        return Err(anyhow::anyhow!("Docker compose failed with status: {}", status));
    }

    Ok(())
}

// Deploys any cosmwasm contract needed for this demo ( using cw-orch & config files)
async fn deploy_infusion_demo(
    handle: &Handle,
    network: ChainInfoOwned,
) -> Result<DeployInfusionDemo, anyhow::Error> {
    let wavs_mnemonic = env::var("WAVS_CONTROLLER_MNEMONIC").unwrap_or_else(|_| "".to_string());

    let cosmos =
        DaemonBuilder::new(network.clone()).handle(handle).mnemonic(wavs_mnemonic).build()?;

    // cw-orchestrator - bitsong account nft suite
    let bs_accounts =
        btsg_account_scripts::BtsgAccountSuite::deploy_on(cosmos.clone(), cosmos.sender_addr())?;
    // cw-orchestrator - cw-infuser suite
    let infuser = cw_infuser_scripts::CwInfuser::new(cosmos.clone());
    if let Some(res) = infuser.upload_if_needed()? {
        match res.code {
            _ => {}
        }
    };

    let register_smart_account = setup_bitsong_smart_account(MsgAddAuthenticator {
        sender: cosmos.sender().pub_addr_str(),
        authenticator_type: "CosmwasmAuthenticatorV1".into(),
        data: to_json_binary(&CosmwasmAuthenticatorInitData {
            contract: bs_accounts.wavs.address()?.into(),
            params: vec![],
        })?
        .to_vec(),
    })
    .await?;

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

    // create infusion with wavs enabled
    infuser.create_infusion(vec![])?;

    Ok(DeployInfusionDemo { cosmos, bs_accounts, infuser })
}

async fn deploy_wavs_service(
    component_filename: &str,
    trigger_event: &str,
    service_trigger_addr: &str,
    service_submission_addr: &str,
    service_config_path: &str,
) -> Result<(), anyhow::Error> {
    let component_path = format!("/data/compiled/{}", component_filename);
    // Deploy the WAVS component service
    let wavs_cmd = "wavs";
    // WAVS_CMD ?= $(SUDO) docker run --rm --network host $$(test -f .env && echo "--env-file ./.env") -v $$(pwd):/data ghcr.io/lay3rlabs/wavs:0.3.0 wavs-cli

    let service_config = service_config_path;

    let status = Command::new(wavs_cmd)
        .args([
            "deploy-service",
            "--log-level=info",
            &format!("--data={}", "/data/.docker"),
            &format!("--home={}", "/data"),
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

async fn run_infusion_demo(suite: DeployInfusionDemo) -> Result<(), anyhow::Error> {
    // burn nft,triggering wavs service

    // query that wavs record has been added to wavs service
    assert_eq!(
        suite
            .infuser
            .wavs_record(vec![bs_accounts.wavs.addr_str()?], Some(cosmos.sender_addr()))?[0]
            .count,
        Some(1)
    );

    Ok(())
}

/// Register a given seckp256k1 key with a specific authenticator
async fn setup_bitsong_smart_account(
    authenticator: MsgAddAuthenticator,
) -> Result<prost_types::Any, anyhow::Error> {
    // register custom authenticator to account
    Ok(prost_types::Any {
        type_url: "/bitsong.smartaccount.v1beta1.MsgAddAuthenticator".into(),
        value: to_json_binary(&authenticator)?.to_vec(),
    })
}
