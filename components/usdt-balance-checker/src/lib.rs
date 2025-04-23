use std::str::FromStr;

pub mod bindings;
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

use alloy_network::Ethereum;
use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::ethereum::new_eth_provider;
use wstd::runtime::block_on;

// Define the USDT ERC20 interface
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// Module for Solidity types
mod solidity {
    use alloy_primitives::Bytes;
    use alloy_sol_macro::sol;

    sol! {
        struct TriggerInfo {
            uint64 triggerId;
            bytes data;
        }

        event NewTrigger(bytes _triggerInfo);

        struct DataWithId {
            uint64 triggerId;
            bytes data;
        }

        function checkBalance(address wallet) external view returns (uint256);
    }
}

// Response structure with formatted balance
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UsdtBalanceResponse {
    address: String,
    balance_raw: String,
    balance_formatted: String,
    decimals: u8,
}

// Define destinations for component output
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
    solidity::DataWithId { triggerId: trigger_id, data: Bytes::from(output.as_ref().to_vec()) }
        .abi_encode()
}

// Main component implementation
struct UsdtBalanceChecker;

async fn query_usdt_balance(address: Address) -> Result<UsdtBalanceResponse, String> {
    // USDT contract address on Ethereum mainnet
    let usdt_contract = Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7")
        .map_err(|e| format!("Invalid USDT contract address: {}", e))?;

    // Get Ethereum provider from chain config
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

    let provider: RootProvider<Ethereum> =
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    // Create balance query call
    let balance_call = IERC20::balanceOfCall { owner: address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_contract)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    // Query balance
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance: U256 = U256::from_be_slice(&result);

    // Get decimals
    let decimals_call = IERC20::decimalsCall {};
    let decimals_tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(usdt_contract)),
        input: TransactionInput { input: Some(decimals_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    let decimals_result = provider.call(&decimals_tx).await.map_err(|e| e.to_string())?;
    let decimals: u8 = decimals_result[0]; // First byte contains the uint8 value

    // Format balance with decimals
    let mut divisor = U256::from(1);
    for _ in 0..decimals {
        divisor = divisor * U256::from(10);
    }

    let whole_part = balance / divisor;
    let fractional_part = balance % divisor;

    // Format the fractional part with leading zeros
    let fractional_str = fractional_part.to_string();

    // Calculate padding with safety checks to prevent overflow
    let padding = if fractional_str.len() >= decimals as usize {
        0 // No padding needed if fractional part is already long enough
    } else {
        // Safe subtraction to get padding amount
        decimals as usize - fractional_str.len()
    };

    // Limit maximum padding to prevent capacity overflow (100 is a safe upper bound)
    let safe_padding = std::cmp::min(padding, 100);

    // Create padding string and format the fractional part
    let padding_zeros = "0".repeat(safe_padding);
    let fractional_str = format!("{}{}", padding_zeros, fractional_str);

    // Combine for formatted balance
    let formatted_balance = format!("{}.{}", whole_part, fractional_str);

    Ok(UsdtBalanceResponse {
        address: address.to_string(),
        balance_raw: balance.to_string(),
        balance_formatted: formatted_balance,
        decimals,
    })
}

impl Guest for UsdtBalanceChecker {
    fn run(trigger: TriggerAction) -> Result<Option<Vec<u8>>, String> {
        // Decode trigger data
        let (trigger_id, input_data, destination) = decode_trigger_event(trigger.data)
            .map_err(|e| format!("Failed to decode trigger event: {}", e))?;

        println!("Input length: {} bytes", input_data.len());
        let hex_display: Vec<String> =
            input_data.iter().take(8).map(|b| format!("{:02x}", b)).collect();
        println!("First 8 bytes: {}", hex_display.join(" "));

        // Try to decode the input as an address
        let address = if let Ok(call) = solidity::checkBalanceCall::abi_decode(&input_data, false) {
            // If it was formatted as a function call (from cast abi-encode "f(address)")
            call.wallet
        } else {
            // Try to decode directly as an address
            Address::abi_decode(&input_data, false)
                .map_err(|e| format!("Failed to decode address: {}", e))?
        };

        println!("Querying USDT balance for address: {}", address);

        // Query USDT balance
        let balance_response = block_on(async { query_usdt_balance(address).await })?;

        // Serialize the result to JSON
        let json_result = serde_json::to_string(&balance_response)
            .map_err(|e| format!("Failed to serialize response: {}", e))?;

        // Return based on destination
        let result = match destination {
            Destination::Ethereum => encode_trigger_output(trigger_id, json_result.as_bytes()),
            Destination::CliOutput => json_result.into_bytes(),
        };

        Ok(Some(result))
    }
}

// Export the component
export!(UsdtBalanceChecker with_types_in bindings);
