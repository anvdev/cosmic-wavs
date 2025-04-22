use alloy_sol_types::SolValue;
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
            let trigger_info = solidity::TriggerInfo::abi_decode(&event._triggerInfo, false)?;
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
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // For debugging
        println!("Input length: {} bytes", req.len());
        let hex_display: Vec<String> = req.iter().take(8).map(|b| format!("{:02x}", b)).collect();
        println!("First 8 bytes: {}", hex_display.join(" "));

        // The req will now be ABI-encoded string data
        // Extract the string from the ABI encoding (skip function selector and offset)
        let string_data = if req.len() >= 68 {
            // Standard ABI encoding for a string has:
            // - 4 bytes function selector (if part of a function call)
            // - 32 bytes for the offset
            // - 32 bytes for the length
            // - Actual string data (padded to 32 byte multiple)

            // Skip the first 4 bytes (function selector) if present
            let start_pos = if req[0] == 0x46 && req[1] == 0x8a && req[2] == 0x46 && req[3] == 0x9d
            {
                4
            } else {
                0
            };

            // Get the string length from the second 32-byte chunk (may need to skip selector first)
            let length_pos = start_pos + 32;
            let mut length_bytes = [0u8; 32];
            if req.len() >= length_pos + 32 {
                length_bytes.copy_from_slice(&req[length_pos..length_pos + 32]);
            } else {
                return Err("Invalid input format: not enough data for string length".to_string());
            }

            let length = u64::from_be_bytes([
                length_bytes[24],
                length_bytes[25],
                length_bytes[26],
                length_bytes[27],
                length_bytes[28],
                length_bytes[29],
                length_bytes[30],
                length_bytes[31],
            ]) as usize;

            // Get the actual string data
            let string_pos = length_pos + 32;
            if req.len() >= string_pos + length {
                let string_data = &req[string_pos..string_pos + length];
                std::str::from_utf8(string_data)
                    .map_err(|e| format!("Invalid UTF-8 in string: {}", e))?
            } else {
                return Err("Invalid input format: not enough data for string content".to_string());
            }
        } else {
            return Err("Invalid input format: too short for ABI-encoded string".to_string());
        };

        println!("Decoded string input: {}", string_data);

        // Parse the first character as a hex digit for the ID
        let id = string_data.chars().next().ok_or("Empty input")?;
        let id = id.to_digit(16).ok_or("Invalid hex digit")? as u64;

        let res = block_on(async move {
            let resp_data = get_price_feed(id).await?;
            println!("resp_data: {:?}", resp_data);
            serde_json::to_vec(&resp_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

async fn get_price_feed(id: u64) -> Result<PriceFeedData, String> {
    let url = format!(
        "https://api.coinmarketcap.com/data-api/v3/cryptocurrency/detail?id={}&range=1h",
        id
    );

    let current_time = std::time::SystemTime::now().elapsed().unwrap().as_secs();

    let mut req = http_request_get(&url).map_err(|e| e.to_string())?;
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    req.headers_mut()
        .insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36"));
    req.headers_mut().insert(
        "Cookie",
        HeaderValue::from_str(&format!("myrandom_cookie={}", current_time)).unwrap(),
    );

    let json: Root = fetch_json(req).await.map_err(|e| e.to_string())?;

    Ok(PriceFeedData {
        symbol: json.data.symbol,
        price: json.data.statistics.price,
        timestamp: json.status.timestamp,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceFeedData {
    symbol: String,
    timestamp: String,
    price: f64,
}

/// -----
/// <https://transform.tools/json-to-rust-serde>
/// Generated from <https://api.coinmarketcap.com/data-api/v3/cryptocurrency/detail?id=1&range=1h>
/// -----
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub data: Data,
    pub status: Status,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub id: f64,
    pub name: String,
    pub symbol: String,
    pub statistics: Statistics,
    pub description: String,
    pub category: String,
    pub slug: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    pub price: f64,
    #[serde(rename = "totalSupply")]
    pub total_supply: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoinBitesVideo {
    pub id: String,
    pub category: String,
    #[serde(rename = "videoUrl")]
    pub video_url: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "previewImage")]
    pub preview_image: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub timestamp: String,
    pub error_code: String,
    pub error_message: String,
    pub elapsed: String,
    pub credit_count: f64,
}
