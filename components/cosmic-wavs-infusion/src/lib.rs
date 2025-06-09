// Required imports
use alloy_sol_types::{sol, SolValue};
use anyhow::Result;
use cosmic_wavs::{
    common::{handle_tx_response, parse_string_attribute, parse_u64_attribute},
    wavs::{
        form_smart_acccount_tx_body, form_wavs_tx, get_wavs_smart_account,
        get_wavs_smart_acount_signer_info, WavsBlsCosmosActionAuth,
    },
};

use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_base64, to_json_binary};
use cw_infusions::wavs::WavsBundle;

use layer_climb::prelude::*;
use layer_climb::proto::{tx::BroadcastMode, wasm::MsgExecuteContract, Any, MessageExt};

use commonware_cryptography::Signer;
use sha2::{Digest, Sha256};

use wavs_wasi_utils::decode_event_log_data;
use wstd::runtime::block_on;

pub mod bindings; // Never edit bindings.rs!

use crate::bindings::{
    export,
    host::get_cosmos_chain_config,
    wavs::worker::layer_types::{
        TriggerData, TriggerDataCosmosContractEvent, TriggerDataEthContractEvent,
    },
    Guest, TriggerAction,
};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceResponse {
    message: String,
    success: bool,
    data: Option<WavsBlsCosmosActionAuth>,
}

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
                let chain = String::from_utf8(vec![req[3]])
                    .map_err(|e| format!("Failed to deserialize bundle burner: {}", e))?;

                // a. retrieve registered cw-infuser contract stored in solidity contract
                // let cw_infuser = CW_INFUSER_ADDR;

                let cosm = get_cosmos_chain_config(&chain)
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
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo)?;
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
                            let sender = parse_string_attribute(&event.attributes, "sender")?;
                            let token_id = parse_u64_attribute(&event.attributes, "token_id")?;

                            // Return the burn event data
                            let data = vec![
                                contract_address.bech32_addr.as_bytes().to_vec(),
                                token_id.to_be_bytes().to_vec(),
                                sender.as_bytes().to_vec(),
                                chain_name.as_bytes().to_vec(),
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
    //
    // // get operator signing key
    let mnemonic =
        std::env::var("WAVS_SECP256k1_MNEMONIC").expect("Missing 'WAVS_SECP256k1_MNEMONIC'");
    let wavs_bls_sk = std::env::var("WAVS_BLS_PK").expect("Missing 'WAVS_BLS_PK'");

    let op_secp256k1_signing_key = KeySigner::new_mnemonic_str(&mnemonic, None).unwrap();
    let secp256k1pubkey = op_secp256k1_signing_key.public_key().await?;

    let wavs_smart_acc_env =
        std::env::var("WAVS_SMART_ACC_PK").expect("Missing 'WAVS_SMART_ACC_PK'");
    let wavs_smart_acc_pk =
        PublicKey::from_raw_secp256k1(wavs_smart_acc_env.as_ref()).expect("should always exists");
    let wavs_smart_acccount_addr = chain_config.address_from_pub_key(&wavs_smart_acc_pk)?;

    // create signing client: TODO: make use of bls12 pubkeys for signing implementation
    let cosm_signing_client: SigningClient =
        SigningClient::new(chain_config.clone(), op_secp256k1_signing_key, None).await?;
    let cosm_guery = cosm_signing_client.querier.clone();
    let tx_builder = cosm_signing_client.tx_builder();

    let cw_infuser_addr = std::env::var("WAVS_CW_INFUSER").expect("Missing 'WAVS_CW_INFUSER'");

    //  query contract the check if operators need to update assigned cw-infuser state
    let on_chain_conditional: Vec<cw_infusions::wavs::WavsRecordResponse> = cosm_guery
        .contract_smart(
            &Address::new_cosmos_string(&cw_infuser_addr, None)?,
            &cw_infuser::msg::QueryMsg::WavsRecord {
                nfts: vec![nft_addr.to_string()],
                burner: None,
            },
        )
        .await?;

    // form msgs for operators to sign
    let mut infusions = vec![];
    for record in on_chain_conditional {
        if let Some(_c) = record.count {
            infusions.push(WavsBundle {
                infuser: burner.to_string(),
                nft_addr: record.addr,
                infused_ids: vec![token_id.to_string()],
            });
        }
    }

    // 4. perform workflow to sign msg the operator set is authorizing to perform
    if infusions.len() > 0 {
        let wavs_any_msg = Any::from_msg(&MsgExecuteContract {
            sender: wavs_smart_acccount_addr.to_string(), // wavs secp256k1 key address registered to x/accounts with bls12 authenticator
            contract: cw_infuser_addr.to_string(),
            msg: to_json_binary(&cw_infuser::msg::ExecuteMsg::WavsEntryPoint { infusions })?
                .to_vec(),
            funds: vec![],
        })?;

        cosmic_wavs_actions.push(wavs_any_msg);

        let mut imported_signer = get_wavs_smart_account(wavs_bls_sk)?;
        // gete account info for our smart-account
        let smart_account = cosm_guery
            .base_account(&Address::Cosmos {
                bech32_addr: chain_config
                    .address_kind
                    .address_from_pub_key(&wavs_smart_acc_pk)?
                    .to_string(),
                prefix_len: 7usize,
            })
            .await?;

        // - create sha256sum bytes that are being signed by operators for aggregated approval.
        // Current implementation signs binary formaated array of Any msgs being authorized.
        let msg_digest: [u8; 32] = Sha256::digest(to_json_binary(&cosmic_wavs_actions)?.as_ref())
            .to_vec()
            .try_into()
            .unwrap();

        // let namespace = Some(&b"additional_namespace. Commonware library already generates hash with standard dst"[..]);
        // push signature to array of operator bls signatures
        let signature = imported_signer.sign(None, &msg_digest).to_vec();
        signatures.push(signature.clone());

        // todo: if gas simulated is to be more that current x/smart-account params defined,
        // we need split messages into smaller batches to be verified.
        let signer_info = get_wavs_smart_acount_signer_info(&imported_signer.public_key());
        let wavs_tx_body =
            form_smart_acccount_tx_body(block_height, cosmic_wavs_actions, vec![1]).await?;
        let gas = tx_builder
            .simulate_gas(signer_info.clone(), smart_account.account_number, &wavs_tx_body)
            .await?;
        signer_infos.push(signer_info);

        // 5.handle transaction response (out of gas,edge case error)
        let cosm_res = tx_builder
            .querier
            .broadcast_tx_bytes(
                form_wavs_tx(wavs_tx_body, gas.gas_used, signer_infos, signatures)
                    .await?
                    .to_bytes()?,
                BroadcastMode::Sync,
            )
            .await?;
        handle_tx_response(cosm_res.code(), cosm_res.raw_log())?;

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
    }

    Ok(ServiceResponse { message: "Burn recorded".to_string(), success: true, data: None })
}

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}
