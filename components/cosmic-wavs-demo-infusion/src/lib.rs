use alloy_primitives::hex::{self};
// Required imports
use alloy_sol_types::{sol, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_base64, to_json_binary};
use cw_infusions::wavs::WavsBundle;

use layer_climb::prelude::*;
use layer_climb::proto::{
    tx::{AuthInfo, BroadcastMode, Fee, TxBody},
    Any, MessageExt,
};

use commonware_codec::extensions::DecodeExt;
use commonware_cryptography::{Bls12381, Signer};
use sha2::{Digest, Sha256};

use wavs_wasi_chain::decode_event_log_data;
use wstd::runtime::block_on;

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::host::get_cosmos_chain_config;
use crate::bindings::wavs::worker::layer_types::{
    TriggerData, TriggerDataCosmosContractEvent, TriggerDataEthContractEvent,
};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    Cosmos,
    CliOutput,
}

// Define Solidity function signature for input format
sol! {
    function registerInfusionService(string escrow_address) external;
}

sol! {
    function checkBurnRequirements(string user_address) external;
}

// Define solidity module for trigger handling
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}
pub const CURRENT_CHAIN_COSMOS: &str = "layer-local";
pub const CURRENT_CHAIN_ETH: &str = "local";
pub const WAVS_CW_INFUSER: &str = "stars1...";
pub const WAVS_BLS_PRIVATE_KEY: &str = "";
pub const WAVS_SECP256k1_MNEMONIC: &str = "";
pub const WAVS_INFUSER_OPERATOR_ADDR: &str = "";

// Data structures for tracking infusion services and burn events
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BurnRequirement {
    collection_address: String,
    count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct InfusionService {
    id: String,
    minter_contract_address: String,
    collection_name: String,
    burn_requirements: Vec<BurnRequirement>,
    created_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BurnRecord {
    user_address: String,
    collection_address: String,
    token_id: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WavsBlsCosmosActionAuth {
    /// b2 point for operator public key
    pubkey_g2: String,
    /// base64 encoded sha256sum hash of msg being signed
    base64_msg_hash: String,
    /// msg that bls12 private key is signing
    msg: Vec<u8>,
    ///
    signature: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserBurnSummary {
    user_address: String,
    burns_by_collection: std::collections::HashMap<String, u64>,
    last_updated: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceResponse {
    message: String,
    success: bool,
    data: Option<WavsBlsCosmosActionAuth>,
}

/// TxExtension allows for additional authenticator-specific data in
/// transactions.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct TxExtension {
    pub selected_authenticators: Vec<u64>,
}
pub const TX_EXTENSION_TYPE: &str = "/bitsong.smartaccount.v1beta1.TxExtension";
// Component struct declaration
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Decode trigger event

        let (block_height, req, dest, event_type) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        let result = match event_type.as_deref() {
            Some("burn") => {
                // Burn event from NFT contract
                let contract_addr = String::from_utf8(vec![req[0]])
                    .map_err(|e| format!("Failed to deserialize nft-address burnt: {}", e))?;
                let token_id = String::from_utf8(vec![req[1]])
                    .map_err(|e| format!("Failed to deserialize token-id burnt: {}", e))?;
                let burner = String::from_utf8(vec![req[2]])
                    .map_err(|e| format!("Failed to deserialize bundle burner: {}", e))?;

                // a. retrieve registered cw-infuser contract stored in solidity contract
                // let cw_infuser = CW_INFUSER_ADDR;

                let cosm = get_cosmos_chain_config(CURRENT_CHAIN_COSMOS)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Failed to get Cosmos chain config for layer-local")
                    })
                    .map_err(|e| format!("failed: {}", e))?;

                // b. run nessesary queries & broadcasts to cosmos chain, responding with the result of the actions
                block_on(async {
                    process_burn_event(block_height, &contract_addr, &token_id, &burner, cosm)
                        .await
                        .map_err(|e| format!("Failed to process burn event: {}", e))
                })?
                // c. handle any unsuccessful transasctions in cache
            }

            _ => {
                // Unknown event type,default response
                ServiceResponse { message: "non-infusion".to_string(), success: true, data: None }
            }
        };

        // Serialize result
        let json_result = serde_json::to_vec(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        // Return based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(block_height, &json_result)),
            Destination::Cosmos => Some(json_result.clone()), // Would need proper Cosmos encoding
            Destination::CliOutput => Some(json_result),
        };

        Ok(output)
    }
}

// Helper function to decode trigger event
pub fn decode_trigger_event(
    trigger_data: TriggerData,
) -> Result<(u64, Vec<u8>, Destination, Option<String>)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum, None))
        }
        TriggerData::CosmosContractEvent(TriggerDataCosmosContractEvent {
            contract_address,
            chain_name,
            event,
            block_height,
        }) => {
            // Extract event type and data from Cosmos event
            let event_type = Some(event.ty.clone());
            if let Some(et) = event_type.as_ref() {
                if et.as_str() == "wasm" {
                    // Look for burn action
                    if let Some(action_attr) = event.attributes.iter().find(|(k, _)| k == "action")
                    {
                        if action_attr.1 == "burn" {
                            let sender = event
                                .attributes
                                .iter()
                                .find(|(k, _)| k == "sender")
                                .map(|(_, v)| v.clone())
                                .ok_or(anyhow::anyhow!("Missing sender attribute"))?;
                            let token_id = event
                                .attributes
                                .iter()
                                .find(|(k, _)| k == "token_id")
                                .map(|(_, v)| v.clone())
                                .ok_or(anyhow::anyhow!("Missing token_id attribute"))?
                                .parse::<u64>()?;

                            // Return the burn event data
                            let data = vec![
                                contract_address.bech32_addr.as_bytes().to_vec(),
                                token_id.to_be_bytes().to_vec(),
                                sender.as_bytes().to_vec(),
                            ]
                            .into_iter()
                            .flatten()
                            .collect();

                            return Ok((block_height, data, Destination::Cosmos, event_type));
                        } else if action_attr.1 == "register_infusion" {
                            // ...
                        }
                    }
                }
            }

            // Default case for non-burn events
            Ok((0, vec![], Destination::Cosmos, event_type))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput, None)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

// Process registration event from escrow contract
async fn process_registration_event(escrow_address: &str) -> Result<ServiceResponse> {
    Ok(ServiceResponse { message: format!("Infusion"), success: true, data: None })
}

// Process burn event and check if requirements are met
async fn process_burn_event(
    block_height: u64,
    nft_addr: &String,
    token_id: &String,
    burner: &String,
    bindings::wavs::worker::layer_types::CosmosChainConfig {
        chain_id,
        rpc_endpoint,
        grpc_endpoint,
        grpc_web_endpoint,
        gas_price,
        gas_denom,
        bech32_prefix,
    }: bindings::wavs::worker::layer_types::CosmosChainConfig,
) -> Result<ServiceResponse> {
    let mut signer_infos = vec![];
    let mut signatures = vec![];
    let mut cosmic_wavs_actions = vec![];

    // Get cosmos chain configuration
    let chain_config = ChainConfig {
        address_kind: AddrKind::Cosmos { prefix: bech32_prefix },
        chain_id: ChainId::new(chain_id),
        rpc_endpoint,
        grpc_endpoint,
        grpc_web_endpoint,
        gas_price,
        gas_denom,
    };

    // get operator signing key
    let mnemonic = std::env::var(WAVS_SECP256k1_MNEMONIC)
        .expect("Missing 'WAVS_SECP256k1_MNEMONIC' in environment.");
    let op_secp256k1_signing_key = KeySigner::new_mnemonic_str(&mnemonic, None).unwrap();
    let secp256k1pubkey = op_secp256k1_signing_key.public_key().await?;

    // create signing client: TODO: make use of bls12 pubkeys for signing implementation
    let cosm_signing_client: SigningClient =
        SigningClient::new(chain_config.clone(), op_secp256k1_signing_key, None).await?;
    let cosm_guery = cosm_signing_client.querier.clone();

    // TODO: get cw-infuser contracts & params registered when creating the service
    // let eth = get_eth_chain_config(&CURRENT_CHAIN_ETH)
    //     .ok_or_else(|| anyhow::anyhow!("Failed to get Eth chain config for local"))?;
    let cw_infuser_addr = WAVS_CW_INFUSER;

    // 2.query contract the check if operators need to update assigned cw-infuser state
    let res: Vec<cw_infusions::wavs::WavsRecordResponse> = cosm_guery
        .contract_smart(
            &Address::new_cosmos_string(&cw_infuser_addr, None)?,
            &cw_infuser::msg::QueryMsg::WavsRecord {
                nfts: vec![nft_addr.to_string()],
                burner: None,
            },
        )
        .await?;

    // 3. form msgs for operators to sign
    let mut infusions = vec![];
    for record in res {
        if let Some(count) = record.count {
            infusions.push(WavsBundle {
                infuser: burner.to_string(),
                nft_addr: record.addr,
                infused_ids: vec![token_id.to_string()],
            });
        }
    }
    // - sign msg the operator set is authorizing to perform
    let wavs_any_msg = Any {
        type_url: "/cosmwasm.wasm.v1.MsgExecuteContract".into(),
        value: cosmos_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract {
            sender: WAVS_INFUSER_OPERATOR_ADDR.into(), // wavs secp256k1 key address registered to x/accounts with bls12 authenticator
            contract: cw_infuser_addr.to_string(),
            msg: to_json_binary(&cw_infuser::msg::ExecuteMsg::WavsEntryPoint { infusions })?
                .to_vec(),
            funds: vec![],
        }
        .to_bytes()?,
    };
    cosmic_wavs_actions.push(wavs_any_msg);

    // Import the bls12-381 private key
    let bls_key_pair = match <Bls12381 as commonware_cryptography::Signer>::PrivateKey::decode(
        hex::decode(WAVS_BLS_PRIVATE_KEY.as_bytes())?.as_ref(),
    ) {
        Ok(key) => key,
        Err(e) => {
            return Err(e.into());
        }
    };

    // Create a signer from the imported key
    let mut imported_signer = <Bls12381 as commonware_cryptography::Signer>::from(bls_key_pair)
        .expect("broken private key");

    // - create sha256sum bytes that are being signed by operators for aggregated approval.
    // Current implementation signs single msgs for authorization,
    let msg_digest: [u8; 32] =
        Sha256::digest(to_json_binary(&cosmic_wavs_actions)?.as_ref()).to_vec().try_into().unwrap();

    // let namespace = Some(&b"demo"[..]);
    let signature = imported_signer.sign(None, &msg_digest).to_vec();
    signatures.push(signature.clone());

    // push signature
    // generate message to broadcast with use of the x/smart-account function
    let wavs_broadcast_msg: TxBody = TxBody {
        messages: cosmic_wavs_actions,
        memo: "Cosmic Wavs Account Action".into(),
        timeout_height: 100u64,
        extension_options: vec![],
        non_critical_extension_options: vec![Any {
            type_url: TX_EXTENSION_TYPE.into(),
            value: to_json_binary(&TxExtension { selected_authenticators: vec![1] })?.to_vec(),
        }]
        .to_vec(),
    };

    // gete account info for our smart-account
    let smart_account = cosm_guery
        .base_account(&Address::Cosmos {
            bech32_addr: chain_config
                .address_kind
                .address_from_pub_key(&secp256k1pubkey)?
                .to_string(),
            prefix_len: 7usize,
        })
        .await?;

    // signer info. This demo implements the signing info for single wav operator bls12 keys
    let signer_info = cosmos_sdk_proto::cosmos::tx::v1beta1::SignerInfo {
        public_key: Some(Any {
            type_url: "/cosmos.crypto.bls12_381.PubKey".into(),
            value: imported_signer.public_key().to_vec(),
        }),
        mode_info: None,
        sequence: 0,
    };

    let gas = cosm_signing_client
        .clone()
        .tx_builder()
        .simulate_gas(signer_info.clone(), smart_account.account_number, &wavs_broadcast_msg)
        .await?;

    let fee = Fee {
        amount: vec![Coin { denom: "ubtsg".into(), amount: 100u64.to_string() }],
        gas_limit: gas.gas_used * 2,
        payer: "".to_string(), // wavs operated account
        granter: "".to_string(),
    };

    signer_infos.push(signer_info);

    //  SIGN_MODE_DIRECT
    let wavs_request = cosmos_sdk_proto::cosmos::tx::v1beta1::Tx {
        body: Some(wavs_broadcast_msg),
        auth_info: Some(AuthInfo { signer_infos, fee: Some(fee), tip: None }),
        signatures, // added array of bls signatures
    };

    // 5.handle transaction response (out of gas,edge case error)
    let cosm_res = cosm_signing_client
        .tx_builder()
        .querier
        .broadcast_tx_bytes(wavs_request.to_bytes()?, BroadcastMode::Sync)
        .await?;

    match cosm_res.code() {
        0 => {
            // successful response
        }
        _ => {
            // errors
        }
    }

    // form object to use with  other operators
    let service_res = WavsBlsCosmosActionAuth {
        base64_msg_hash: to_base64(msg_digest),
        msg: vec![],
        signature: hex::encode(signature),
        pubkey_g2: imported_signer.public_key().to_string(),
    };

    if cosm_res.code() != 0 {
        return Ok(ServiceResponse {
            message: "Infusion record failuter".to_string(),
            success: false,
            data: Some(service_res),
        });
    }

    Ok(ServiceResponse {
        message: "Burn recorded".to_string(),
        success: true,
        data: Some(service_res),
    })
}

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}
