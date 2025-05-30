use anyhow::Context;
use btsg_account_scripts::BtsgAccountSuite;
use btsg_nft_scripts::framework::assert_wallet_balance;
use clap::{Parser, Subcommand};
use commonware_cryptography::{Bls12381, Signer};
use cosmrs::bip32::secp256k1::elliptic_curve::rand_core::OsRng;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Decimal};
use cw_infuser::msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns};
use cw_infuser_scripts::CwInfuser;
use cw_orch::{
    core::serde_json::json,
    daemon::{
        keys::private::PrivateKey, senders::CosmosSender, DaemonBase, DaemonBuilder, TxSender,
    },
    prelude::*,
};
use cw_orch_wavs::networks::{BITSONG_MAINNET, BITSONG_TESTNET, LOCAL_NETWORK1};
use secp256k1::{All, Secp256k1};
use std::{
    env,
    fs::{self, File},
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
    let wavs_bech32_addr = env::var("WAVS_CONTROLLER_ADDRESS").unwrap_or_else(|_| "".to_string());
    let service_sub_addr = env::var("SERVICE_SUBMISSI1ON_ADDR").unwrap_or_else(|_| "".to_string());
    let service_trigger_addr = env::var("SERVICE_TRIGGER_ADDR").unwrap_or_else(|_| "".to_string());
    let wavs_toml_path = env::var("WAVS_TOML_PATH").unwrap_or_else(|_| "".to_string());

    rt.block_on(assert_wallet_balance(vec![network.clone()]));
    // setup_local_crypto_keys()?;
    // deploy networks
    // rt.block_on(deploy_wavs_infra())?;
    // deploy cosmos smart contracct

    let infusion_demo: DeployInfusionDemo =
        match rt.block_on(deploy_infusion_demo(rt.handle(), network)) {
            Ok(value) => value,
            Err(e) => return Err(e.into()),
        };
    // // deploy eth contracts (wavs service)
    // rt.block_on(deploy_wavs_service(
    //     WAVS_COMPONENT,
    //     INFUSION_TRIGGER_EVENT,
    //     &service_trigger_addr,
    //     &service_sub_addr,
    //     &service_config_file_path,
    // ))?;
    // run demo
    rt.block_on(run_infusion_demo(infusion_demo))?;

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
    let private_key = PrivateKey::new(&secp, 118u32)?;
    let public_key = private_key.public_key(&secp);

    // Convert to strings (simplified; in practice, derive bech32 address)
    let secp256k1_private = hex::encode(private_key.raw_key());
    let secp256k1_public = hex::encode(&public_key.raw_pub_key.unwrap_or_default());

    // Generate BLS12-381 keypair (placeholder)
    let mut bls12 = Bls12381::new(&mut OsRng);
    let private_key = bls12.private_key();
    let public_key = bls12.public_key();

    // Create JSON structure
    //      Setup test keys
    //   todo: import and generate keys based on json file generated with this format
    //  {
    //      "members": [
    //          {
    //              "cosmos": {
    //                  "ed12259": {},
    //                  "secp256k1": {}
    //              },
    //              "eth": {
    //                  "bls12": {}
    //              }
    //          }
    //      ]
    //  }
    //  Create a validator key if it doesn't exist
    let keys = json!({
        "secp256k1": {
            "private_key": secp256k1_private,
            "public_key": secp256k1_public,
            "address": env::var("WAVS_CONTROLLER_ADDRESS").expect("NO ADDRESS PROVIDED")
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
// async fn deploy_wavs_infra() -> Result<(), anyhow::Error> {
//     let docker_compose_path = env::var("DOCKER_COMPOSE_PATH")
//         .unwrap_or_else(|e| "missing docker-compose.yml".to_string());
//     if Path::new(".docker").exists() {
//         fs::remove_dir_all(".docker").context("Failed to remove .docker directory")?;
//     }
//     fs::create_dir_all(".docker").context("Failed to create .docker directory")?;

//     // Copy .env.example to .env if it doesn't exist
//     if !Path::new(".env").exists() {
//         fs::copy(".env.example", ".env").context("Failed to copy .env.example to .env")?;
//     }

//     // Start Anvil in the background
//     let anvil_process = Command::new("anvil")
//         .stdout(Stdio::null())
//         .stderr(Stdio::null())
//         .spawn()
//         .context("Failed to start Anvil")?;

//     // Ensure Anvil has time to start
//     sleep(Duration::from_secs(2)).await;

//     // Start Docker Compose for WAVS
//     let status = Command::new("docker")
//         .args(["compose", "-f", &docker_compose_path, "up", "-d"])
//         .status()
//         .context("Failed to run docker compose up")?;

//     if !status.success() {
//         // Clean up Anvil process on failure
//         let _ = Command::new("kill").arg(anvil_process.id().to_string()).status();
//         return Err(anyhow::anyhow!("Docker compose failed with status: {}", status));
//     }

//     Ok(())
// }

// Deploys any cosmwasm contract needed for this demo (using cw-orch & config files)
async fn deploy_infusion_demo(
    handle: &Handle,
    network: ChainInfoOwned,
) -> Result<DeployInfusionDemo, anyhow::Error> {
    let wavs_mnemonic = env::var("WAVS_CONTROLLER_MNEMONIC").unwrap_or_else(|_| "".to_string());

    let cosmos =
        DaemonBuilder::new(network.clone()).handle(handle).mnemonic(wavs_mnemonic).build()?;

    // cw-orchestrator - bitsong account nft & cw-infuser suite
    let bs_accounts =
        btsg_account_scripts::BtsgAccountSuite::deploy_on(cosmos.clone(), cosmos.sender_addr())?;
    let infuser = cw_infuser_scripts::CwInfuser::new(cosmos.clone());

    // Create nft ccollection to mint and register to trigger AVS.
    // We can use the bs-acount collection to assert that filtering actions is implemented properly by the AVS
    bs_accounts.bs721base.instantiate(
        &bs721_base::msg::InstantiateMsg {
            name: "cosmic-wavs".into(),
            symbol: "COSMIC_WAVS".into(),
            uri: None,
            minter: cosmos.sender().address().to_string(),
        },
        None,
        &[],
    )?;

    if let Some(res) = infuser.upload_if_needed()? {
        // todo: handle response
        match res.code {
            _ => {}
        }
    };

    // register a secp256k1 key to make use of authorizations
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
        0 => {}

        _ => {
            panic!("bad account registration")
        }
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

// async fn deploy_wavs_service(
//     component_filename: &str,
//     trigger_event: &str,
//     service_trigger_addr: &str,
//     service_submission_addr: &str,
//     service_config_path: &str,
// ) -> Result<(), anyhow::Error> {
//     let component_path = format!("/data/compiled/{}", component_filename);
//     // Deploy the WAVS component service
//     let wavs_cmd = "wavs";
//     // WAVS_CMD ?= $(SUDO) docker run --rm --network host $$(test -f .env && echo "--env-file ./.env") -v $$(pwd):/data ghcr.io/lay3rlabs/wavs:0.3.0 wavs-cli

//     let service_config = service_config_path;

//     let status = Command::new(wavs_cmd)
//         .args([
//             "deploy-service",
//             "--log-level=info",
//             &format!("--data={}", "/data/.docker"),
//             &format!("--home={}", "/data"),
//             &format!("--component={}", component_path),
//             &format!("--trigger-event-name={}", trigger_event),
//             &format!("--trigger-address={}", service_trigger_addr),
//             &format!("--submit-address={}", service_submission_addr),
//             &format!("--service-config={}", service_config),
//         ])
//         .status()
//         .context("Failed to run WAVS deploy-service")?;

//     if !status.success() {
//         return Err(anyhow::anyhow!("WAVS deploy-service failed with status: {}", status));
//     }

//     Ok(())
// }

/// Runs the logic that we expect the wavs service to be triggered by.
async fn run_infusion_demo(suite: DeployInfusionDemo) -> Result<(), anyhow::Error> {
    // burn nft,triggering wavs service
    // query that wavs record has been added to wavs service
    assert_eq!(
        suite.infuser.wavs_record(
            vec![suite.bs_accounts.bs721base.addr_str()?],
            Some(suite.cosmos.sender_addr())
        )?[0]
            .count,
        Some(1)
    );

    Ok(())
}

fn deploy_eigenlayer_contracts(rpc_url: &str) -> Result<(), anyhow::Error> {
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
