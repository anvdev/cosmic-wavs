use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

pub enum Destination {
    Ethereum,
    CliOutput,
}

sol! {
    function findBreweriesByZip(string zipCode) external;
}

mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Brewery {
    id: String,
    name: String,
    brewery_type: String,
    street: Option<String>,
    address_2: Option<String>,
    address_3: Option<String>,
    city: String,
    state: String,
    postal_code: String,
    country: String,
    longitude: Option<f64>,
    latitude: Option<f64>,
    phone: Option<String>,
    website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreweryResponse {
    zip_code: String,
    breweries: Vec<Brewery>,
    count: usize,
    timestamp: String,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Decode trigger event
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the zip code string using proper ABI decoding
        let zip_code = if let Ok(decoded) = findBreweriesByZipCall::abi_decode(&req_clone, false) {
            // Successfully decoded as function call
            decoded.zipCode
        } else {
            // Try decoding just as a string parameter
            match String::abi_decode(&req_clone, false) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
            }
        };

        // Query brewery data
        let res = block_on(async move {
            let brewery_data = find_breweries_by_zip(&zip_code).await?;
            serde_json::to_vec(&brewery_data).map_err(|e| e.to_string())
        })?;

        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

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

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

async fn find_breweries_by_zip(zip_code: &str) -> Result<BreweryResponse, String> {
    // Create API URL
    let url = format!("https://api.openbrewerydb.org/v1/breweries?by_postal={}", zip_code);

    // Create request with headers
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

    // Make API request
    let breweries: Vec<Brewery> =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch brewery data: {}", e))?;

    // Get current timestamp
    let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(n) => n.as_secs().to_string(),
        Err(_) => "Unknown time".to_string(),
    };

    // Create response object
    Ok(BreweryResponse {
        zip_code: zip_code.to_string(),
        count: breweries.len(),
        breweries,
        timestamp,
    })
}
