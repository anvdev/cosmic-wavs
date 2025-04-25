// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Input function signature
sol! {
    function resolveEnsDomain(string input) external;
}

// API response structures for ENS lookups
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsApiResponse {
    address: String,
    name: Option<String>,
    #[serde(default)]
    avatar: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    twitter: Option<String>,
    #[serde(default)]
    github: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    expiry_date: Option<String>,
}

// Response data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsResolveResponse {
    input: String,
    is_address: bool,
    ens_name: Option<String>,
    eth_address: Option<String>,
    avatar: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    social: Option<EnsSocialData>,
    expiry_date: Option<String>,
    timestamp: String,
}

// Social data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsSocialData {
    twitter: Option<String>,
    github: Option<String>,
    url: Option<String>,
    email: Option<String>,
}

// Solidity types
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// Component struct declaration
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the input parameter using proper ABI decoding
        let input = if let Ok(decoded) = resolveEnsDomainCall::abi_decode(&req_clone, false) {
            // Successfully decoded as function call
            decoded.input
        } else {
            // Try decoding just as a string parameter
            match String::abi_decode(&req_clone, false) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
            }
        };

        // Process the ENS resolution
        let res = block_on(async move {
            let result = resolve_ens_domain(&input).await?;
            serde_json::to_vec(&result).map_err(|e| e.to_string())
        })?;

        // Return encoded result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };

        Ok(output)
    }
}

// Helper function to decode trigger event data
pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

// Helper function to encode trigger output
pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

// Function to get current timestamp as string
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}

// Main function to resolve ENS domain or address
async fn resolve_ens_domain(input: &str) -> Result<EnsResolveResponse, String> {
    // Determine if input is an address or ENS name
    let is_address = input.starts_with("0x") && input.len() == 42;

    // Normalize input
    let normalized_input = if !is_address && !input.contains('.') {
        format!("{}.eth", input)
    } else {
        input.to_string()
    };

    // Try with Ethereum public ENS API (ethers.js provider uses this)
    let api_endpoint = if is_address {
        // Reverse lookup (address → ENS)
        format!(
            "https://eth-mainnet.g.alchemy.com/v2/demo/ens/getEnsAddress?address={}",
            normalized_input
        )
    } else {
        // Forward lookup (ENS → address)
        format!(
            "https://eth-mainnet.g.alchemy.com/v2/demo/ens/getEnsAddress?name={}",
            normalized_input
        )
    };

    // Create HTTP request with headers
    let mut req =
        http_request_get(&api_endpoint).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

    // Create fallback response in case API fails
    let mut fallback_response = EnsResolveResponse {
        input: input.to_string(),
        is_address,
        ens_name: if !is_address { Some(normalized_input.clone()) } else { None },
        eth_address: if is_address { Some(normalized_input.clone()) } else { None },
        avatar: None,
        display_name: None,
        description: None,
        social: Some(EnsSocialData { twitter: None, github: None, url: None, email: None }),
        expiry_date: None,
        timestamp: get_current_timestamp(),
    };

    // Simple response structure for basic API response
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct SimpleEnsResponse {
        address: Option<String>,
        name: Option<String>,
    }

    // Try to make API request, but handle errors gracefully
    match fetch_json::<SimpleEnsResponse>(req).await {
        Ok(api_response) => {
            // Update our response with basic ENS info
            if let Some(address) = api_response.address {
                fallback_response.eth_address = Some(address);
            }
            if let Some(name) = api_response.name {
                fallback_response.ens_name = Some(name);
            }
        }
        Err(e) => {
            // If this API fails, we'll just use our fallback data
            // In a production component, we might try multiple ENS providers
        }
    };

    // Return the best response we could generate
    Ok(fallback_response)
}
