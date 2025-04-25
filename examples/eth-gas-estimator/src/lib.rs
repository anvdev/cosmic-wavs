use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings; // bindings are auto-generated during the build process
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

pub enum Destination {
    Ethereum,
    CliOutput,
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

    // Define a simple struct representing the function that encodes string input
    sol! {
        function getGasEstimates() external;
    }
}

// Response structures with Clone derivation to avoid ownership issues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GasPriceData {
    slow: SpeedTier,
    average: SpeedTier,
    fast: SpeedTier,
    timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpeedTier {
    price: String,
    time_minutes: String,
}

// Gas Estimator API response structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlocknativeResponse {
    system: String,
    network: String,
    unit: String,
    #[serde(rename = "maxPrice")]
    max_price: f64,
    #[serde(rename = "currentBlockNumber")]
    current_block_number: u64,
    #[serde(rename = "msSinceLastBlock")]
    ms_since_last_block: u64,
    #[serde(rename = "blockPrices")]
    block_prices: Vec<BlockPrices>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BlockPrices {
    #[serde(rename = "blockNumber")]
    block_number: u64,
    #[serde(rename = "estimatedTransactionCount")]
    estimated_transaction_count: u64,
    #[serde(rename = "baseFeePerGas")]
    base_fee_per_gas: f64,
    #[serde(rename = "estimatedPrices")]
    estimated_prices: Vec<EstimatedPrice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EstimatedPrice {
    confidence: u8,
    price: f64,
    #[serde(rename = "maxPriorityFeePerGas")]
    max_priority_fee_per_gas: f64,
    #[serde(rename = "maxFeePerGas")]
    max_fee_per_gas: f64,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // We're not really using the input in this case, but we still decode it properly
        // to ensure compatibility with standard contract function calls
        if let Ok(_) = solidity::getGasEstimatesCall::abi_decode(&req_clone, false) {
            println!("Retrieving gas estimates");
        } else {
            // Try decoding just as a string parameter as fallback
            match String::abi_decode(&req_clone, false) {
                Ok(s) => println!("Input parameter: {}", s),
                Err(e) => {
                    println!("Ignoring decode error and proceeding: {}", e);
                    // We don't error out here since we don't need input for gas estimation
                }
            };
        }

        // Fetch gas price data
        let res = block_on(async move {
            let gas_data = get_gas_prices().await?;
            println!("Gas data: {:?}", gas_data);
            serde_json::to_vec(&gas_data).map_err(|e| e.to_string())
        })?;

        // Return data based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

async fn get_gas_prices() -> Result<GasPriceData, String> {
    // Using Blocknative public Gas API which doesn't require an API key
    let url = "https://api.blocknative.com/gasprices/blockprices?chainid=1";

    // Create request with headers
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut()
        .insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36"));

    // Parse JSON response
    let response: BlocknativeResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch gas data: {}", e))?;

    // Get current timestamp
    let timestamp = get_current_timestamp();

    // Extract the prices for different confidence levels
    // Default to first block in the response if available
    if response.block_prices.is_empty() {
        return Err("No gas price data available".to_string());
    }

    let block_prices = &response.block_prices[0];
    if block_prices.estimated_prices.is_empty() {
        return Err("No estimated prices available".to_string());
    }

    // Find prices with different confidence levels
    // 99% confidence = fast, 80% = average, 60% = slow
    let fast_price = find_price_by_confidence(&block_prices.estimated_prices, 99)
        .unwrap_or_else(|| &block_prices.estimated_prices[0]);

    let average_price = find_price_by_confidence(&block_prices.estimated_prices, 80)
        .unwrap_or_else(|| {
            &block_prices.estimated_prices[min(1, block_prices.estimated_prices.len() - 1)]
        });

    let slow_price =
        find_price_by_confidence(&block_prices.estimated_prices, 60).unwrap_or_else(|| {
            &block_prices.estimated_prices[min(2, block_prices.estimated_prices.len() - 1)]
        });

    // Create the gas price data structure
    Ok(GasPriceData {
        slow: SpeedTier {
            price: format!("{:.2}", slow_price.price),
            time_minutes: "10-15".to_string(),
        },
        average: SpeedTier {
            price: format!("{:.2}", average_price.price),
            time_minutes: "5-10".to_string(),
        },
        fast: SpeedTier {
            price: format!("{:.2}", fast_price.price),
            time_minutes: "1-3".to_string(),
        },
        timestamp,
    })
}

// Helper function to find a price by confidence level
fn find_price_by_confidence(
    prices: &[EstimatedPrice],
    target_confidence: u8,
) -> Option<&EstimatedPrice> {
    for price in prices {
        if price.confidence == target_confidence {
            return Some(price);
        }
    }
    None
}

// We're using std::cmp::min imported at the top

// Get current timestamp in seconds
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    format!("{}", now)
}
