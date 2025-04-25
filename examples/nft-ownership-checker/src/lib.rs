use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::ethereum::new_eth_provider;
use wstd::runtime::block_on;

pub mod bindings; // bindings are auto-generated during the build process
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define NFT ERC721 interface
sol! {
    interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
    }
}

// Destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Fixed NFT contract address on Ethereum mainnet
const NFT_CONTRACT_ADDRESS: &str = "0xbd3531da5cf5857e7cfaa92426877b022e612cf8";
const CONTRACT_NAME: &str = "BAYC";

// Response structure with Clone derivation to avoid ownership issues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NftOwnershipData {
    wallet: String,
    owns_nft: bool,
    balance: String,
    nft_contract: String,
    contract_name: String,
    timestamp: String,
}

pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let log_clone = log.clone();
            let event: solidity::NewTrigger = decode_event_log_data!(log_clone)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");

    // Define our Solidity input type
    sol! {
        function checkNftOwnership(string wallet) external;
    }
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the wallet address string using proper ABI decoding
        let wallet_address_str =
            if let Ok(decoded) = solidity::checkNftOwnershipCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.wallet
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Checking NFT ownership for wallet: {}", wallet_address_str);

        // Run the NFT ownership check and return the result
        let res = block_on(async move {
            let ownership_data = check_nft_ownership(&wallet_address_str).await?;
            println!("Ownership data: {:?}", ownership_data);
            serde_json::to_vec(&ownership_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

async fn check_nft_ownership(wallet_address_str: &str) -> Result<NftOwnershipData, String> {
    // Parse the wallet address
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address format '{}': {}", wallet_address_str, e))?;

    // Parse the NFT contract address
    let nft_address = Address::from_str(NFT_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid NFT contract address: {}", e))?;

    // Get Ethereum provider
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

    let provider: RootProvider<Ethereum> =
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    // Create balanceOf call to get the NFT balance
    let balance_call = IERC721::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(nft_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    // Execute call to get balance
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance: U256 = U256::from_be_slice(&result);

    // Determine if wallet owns at least one NFT
    let owns_nft = balance > U256::ZERO;

    // Get current timestamp
    let timestamp = get_current_timestamp();

    Ok(NftOwnershipData {
        wallet: wallet_address_str.to_string(),
        owns_nft,
        balance: balance.to_string(),
        nft_contract: NFT_CONTRACT_ADDRESS.to_string(),
        contract_name: CONTRACT_NAME.to_string(),
        timestamp,
    })
}

// Get current timestamp in ISO 8601 format
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    format!("{}", now)
}
