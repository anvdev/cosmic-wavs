// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Define Solidity function signature that matches input format
sol! {
    function calculateSquare(string number) external;
}

// Define result data structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SquareResult {
    input: String,
    square: String,
}

// Define solidity module for trigger handling
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

        // Decode the input string using proper ABI decoding
        let input_str = if let Ok(decoded) = calculateSquareCall::abi_decode(&req_clone, false) {
            // Successfully decoded as function call
            decoded.number
        } else {
            // Try decoding just as a string parameter
            match String::abi_decode(&req_clone, false) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
            }
        };

        // Parse the input string to a number
        let number = input_str.parse::<u64>().map_err(|e| format!("Invalid number: {}", e))?;

        // Calculate the square
        let square = number * number;

        // Create the result structure
        let result = SquareResult { input: input_str.to_string(), square: square.to_string() };

        // Serialize the result to JSON
        let json_result = serde_json::to_vec(&result)
            .map_err(|e| format!("Failed to serialize result: {}", e))?;

        // Return the result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &json_result)),
            Destination::CliOutput => Some(json_result),
        };

        Ok(output)
    }
}

// Helper function to decode trigger event
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
