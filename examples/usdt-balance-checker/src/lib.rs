use alloy_network::Ethereum;
use alloy_primitives::TxKind;
use alloy_primitives::{Address, Bytes, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::str::FromStr;
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::ethereum::new_eth_provider;
use wstd::runtime::block_on;

pub mod bindings; // bindings are auto-generated during the build process
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define USDT ERC20 interface
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// Define our Solidity input type
sol! {
    function checkUsdtBalance(string wallet) external;
}

// Destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Fixed USDT contract address on Ethereum mainnet
const USDT_CONTRACT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

// Response structure with Clone derivation to avoid ownership issues
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsdtBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    usdt_contract: String,
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
    solidity::DataWithId { triggerId: trigger_id, data: Bytes::from(output.as_ref().to_vec()) }
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

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the wallet address string using proper ABI decoding
        let wallet_address_str =
            if let Ok(decoded) = checkUsdtBalanceCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.wallet
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Checking USDT balance for wallet: {}", wallet_address_str);

        // Run the balance check and return the result
        let res = block_on(async move {
            let balance_data = get_usdt_balance(&wallet_address_str).await?;
            println!("Balance data: {:?}", balance_data);
            serde_json::to_vec(&balance_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

async fn get_usdt_balance(wallet_address_str: &str) -> Result<UsdtBalanceData, String> {
    // Parse the wallet address
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address format '{}': {}", wallet_address_str, e))?;

    // Parse the USDT contract address
    let usdt_address = Address::from_str(USDT_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid USDT contract address: {}", e))?;

    // Get Ethereum provider
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

    let provider: RootProvider<Ethereum> =
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    // Create balanceOf call to get the USDT balance
    let balance_call = IERC20::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    // Execute call to get raw balance
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance_raw: U256 = U256::from_be_slice(&result);

    // Get decimals for formatting
    let decimals_call = IERC20::decimalsCall {};
    let tx_decimals = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_address)),
        input: TransactionInput { input: Some(decimals_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    // Execute call to get decimals
    let result_decimals = provider.call(&tx_decimals).await.map_err(|e| e.to_string())?;
    let decimals: u8 = result_decimals[31]; // Extract last byte for uint8

    // Format the balance with proper decimals
    let formatted_balance = format_token_amount(balance_raw, decimals);

    // Get current timestamp
    let timestamp = get_current_timestamp();

    Ok(UsdtBalanceData {
        wallet: wallet_address_str.to_string(),
        balance_raw: balance_raw.to_string(),
        balance_formatted: formatted_balance,
        usdt_contract: USDT_CONTRACT_ADDRESS.to_string(),
        timestamp,
    })
}

// Format token amount using decimals
fn format_token_amount(amount: U256, decimals: u8) -> String {
    if amount.is_zero() {
        return "0".to_string();
    }

    let amount_str = amount.to_string();
    let amount_len = amount_str.len();

    if amount_len <= decimals as usize {
        // Amount smaller than one token unit (e.g. less than 1 USDT)
        let padding = decimals as usize - amount_len;
        // Handle padding with safety bounds
        let zeros = "0".repeat(min(padding, 100)); // SAFE: bounded by min()
        let formatted = format!("0.{}{}", zeros, amount_str);
        return formatted;
    } else {
        // Amount has both whole number and fractional parts
        let decimal_pos = amount_len - decimals as usize;
        let (whole, fractional) = amount_str.split_at(decimal_pos);

        // Remove trailing zeros from fractional part
        let trimmed_fractional = fractional.trim_end_matches('0').to_string();

        // If fractional part is now empty, return just the whole part
        if trimmed_fractional.is_empty() {
            return whole.to_string();
        }

        return format!("{}.{}", whole, trimmed_fractional);
    }
}

// Get current timestamp in ISO 8601 format
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    format!("{}", now)
}
