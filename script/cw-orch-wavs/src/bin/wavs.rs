use anyhow::{Context, Result};
use btsg_account_scripts::deploy::btsg_wavs::BtsgWavsAuth;
use cosmic_wavs::smart_accounts::{
    setup_wavs_smart_account, CosmwasmAuthenticatorInitData, MsgAddAuthenticator,
};
use cosmwasm_std::{to_json_binary, Decimal};
use cw_orch::{
    daemon::{senders::CosmosSender, DaemonBase, DaemonBuilder, TxSender},
    environment::ChainKind,
    prelude::*,
};
use cw_orch_wavs::{
    networks::{BITSONG_MAINNET, BITSONG_TESTNET, LOCAL_NETWORK1},
    // tools::create_operator,
};
use secp256k1::All;
use std::{env, path::Path};
use tokio::runtime::{Handle, Runtime};

// use dotenv::dotenv;
// use tokio::runtime::{Handle, Runtime};
// use btsg_account_scripts::BtsgAccountSuite;
// use btsg_nft_scripts::framework::assert_wallet_balance;
// use clap::{Parser, Subcommand};
// use commonware_codec::extensions::DecodeExt;
// use commonware_cryptography::{Bls12381, Signer};
// use cw_infuser::msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns};
// use cw_infuser_scripts::CwInfuser;
// use cw_infusions::{
//     bundles::{Bundle, BundleType},
//     nfts::InfusedCollection,
//     state::{EligibleNFTCollection, Infusion, InfusionParamState},
// };

pub struct DeployInfusionDemo {
    pub cosmos: DaemonBase<CosmosSender<All>>,
    // pub bs_accounts: BtsgAccountSuite<DaemonBase<CosmosSender<All>>>,
    // pub infuser: CwInfuser<DaemonBase<CosmosSender<All>>>,
}

pub const WAVS_COMPONENT: &str = "cosmic-wavs-infusion.wasm";
pub const INFUSION_TRIGGER_EVENT: &str = "cw-infusion";

fn main() -> Result<()> {
    dotenv::from_path(Path::new(".env")).ok();
    env::vars().into_iter().for_each(|a| println!("Mnemonic: {},{}", a.0, a.1));
    let wavs_mnemonic = env::var("WAVS_CONTROLLER_MNEMONIC")?;
    let network = env::var("WAVS_NETWORK")?;

    let cosmos_chain = match network.as_str() {
        "main" => BITSONG_MAINNET.to_owned(),
        "testnet" => BITSONG_TESTNET.to_owned(),
        "local" => LOCAL_NETWORK1.to_owned(),
        _ => std::process::exit(1),
    }
    .to_owned();

    let rt = Runtime::new().expect("Failed to create tokio runtime");

    let cosmos =
        DaemonBuilder::new(cosmos_chain).handle(rt.handle()).mnemonic(wavs_mnemonic).build()?;

    let btsgwavs = BtsgWavsAuth::new("btsg-wavs-auth", cosmos.clone());
    // btsgwavs.upload_if_needed()?;

    // broadcast tx
    let res = cosmos.commit_any(
        vec![setup_wavs_smart_account(
            "bitsong",
            MsgAddAuthenticator {
                sender: cosmos.sender().pub_addr_str(),
                authenticator_type: "CosmwasmAuthenticatorV1".into(),
                data: to_json_binary(&CosmwasmAuthenticatorInitData {
                    contract: btsgwavs.address()?.into(),
                    params: vec![],
                })?
                .to_vec(),
            },
        )?],
        "Tuning account to the Cosmic Wavs Frequency...".into(),
    )?;
    // // handle response
    match res.code {
        0 => {}
        _ => {
            panic!("bad account registration")
        }
    }

    Ok(())
}

// Deploys any cosmwasm contract needed for this demo (using cw-orch & config files)
// async fn deploy_infusion_demo() -> Result<DeployInfusionDemo, anyhow::Error> {
// cw-orchestrator - bitsong account nft & cw-infuser suite
// let suite = btsg_account_scripts::BtsgAccountSuite::new(cosmos.clone());
// // let bs721base = suite.bs721base.clone();
// // let btsgwavs = suite.wavs.clone();

// let infuser = cw_infuser_scripts::CwInfuser::new(cosmos.clone());

// // Create nft ccollection to mint and register to trigger AVS.
// // We can use the bs-acount collection to assert that filtering actions is implemented properly by the AVS
// // bs721base.instantiate(
// //     &bs721_base::msg::InstantiateMsg {
// //         name: "cosmic-wavs".into(),
// //         symbol: "COSMIC_WAVS".into(),
// //         uri: None,
// //         minter: cosmos.sender().address().to_string(),
// //     },
// //     None,
// //     &[],
// // )?;

// let pubkey = hex::encode(
//     <Bls12381 as Signer>::from(<Bls12381 as Signer>::PrivateKey::decode(
//         env::var("WAVS_BLS12_PRIVKEY").unwrap().as_bytes(),
//     )?)
//     .expect("broken private key")
//     .public_key()
//     .to_string(),
// );

// btsgwavs.instantiate(
//     &btsg_wavs::msg::InstantiateMsg {
//         owner: Some(cosmos.sender_addr()),
//         wavs_operator_pubkeys: vec![pubkey.as_bytes().into()],
//         wavs_pubkey_type: "bls12-381".into(),
//     },
//     None,
//     &[],
// )?;

// if let Some(res) = infuser.upload_if_needed()? {
//     // todo: handle response
//     match res.code {
//         0 => {}
//         _ => {
//             panic!("non-0 response")
//         }
//     }
// };

// // configure infusion with wavs support enabled
// infuser.instantiate(
//     &InstantiateMsg {
//         contract_owner: None,
//         owner_fee: Decimal::zero(),
//         min_creation_fee: Some(Coin::new(100u128, "ubtsg")),
//         min_infusion_fee: Some(Coin::new(100u128, "ubtsg")),
//         min_per_bundle: None,
//         max_per_bundle: None,
//         max_bundles: None,
//         max_infusions: None,
//         cw721_code_id: 1u64,
//         wavs_public_key: None,
//     },
//     None,
//     &[],
// )?;

// // ccreate nft collection eligilbe to burn
// // create
// let infusion = Infusion {
//     description: None,
//     owner: Some(cosmos.sender_addr()),
//     collections: vec![EligibleNFTCollection {
//         addr: bs721base.address()?,
//         min_req: 3,
//         max_req: None,
//         payment_substitute: Some(Coin::new(100u128, "ubtsg")),
//     }],
//     infused_collection: InfusedCollection {
//         sg: false,
//         admin: None,
//         name: "eretskeret".into(),
//         description: "check123".into(),
//         symbol: "CHECK".into(),
//         base_uri: "btsg://".into(),
//         image: "btsg://".into(),
//         num_tokens: 2,
//         royalty_info: None,
//         start_trading_time: None,
//         explicit_content: None,
//         external_link: None,
//         addr: None,
//     },
//     infusion_params: InfusionParamState {
//         bundle_type: BundleType::AllOf {},
//         mint_fee: None,
//         params: None,
//         wavs_enabled: true,
//     },
//     payment_recipient: None,
// };

// // create infusion with wavs enabled
// infuser.create_infusion(vec![infusion])?;

// Ok(DeployInfusionDemo { cosmos })
// }

// /// Runs the logic that we expect the wavs service to be triggered by.
// async fn run_infusion_demo(suite: DeployInfusionDemo) -> Result<(), anyhow::Error> {
//     // mint & burn nft, triggering wavs service
//     suite
//         .bs_accounts
//         .bs721base
//         .execute(&bs721_base::msg::ExecuteMsg::<Empty>::Burn { token_id: "1".into() }, &[])?;
//     // query that wavs record has been added to wavs service
//     assert_eq!(
//         suite.infuser.wavs_record(
//             vec![suite.bs_accounts.bs721base.addr_str()?],
//             Some(suite.cosmos.sender_addr())
//         )?[0]
//             .count,
//         Some(1)
//     );

//     // burn via contract call to infuser
//     suite.infuser.infuse(
//         vec![Bundle {
//             nfts: vec![cw_infusions::nfts::NFT {
//                 addr: suite.bs_accounts.bs721base.address()?,
//                 token_id: 2,
//             }],
//         }],
//         1,
//     )?;
//     Ok(())
// }
