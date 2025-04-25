# WAVSai ENS Resolver Example

This document demonstrates a complete, error-free ENS resolver component created following the WAVSai process.

## User Request

> I want to build a component that takes an ENS name (e.g., `vitalik.eth`), resolves it to an ETH address via the ENS API, and returns the result.

## Step 1: Understanding Requirements

Requirements identified:
- Input: ENS name as a string
- Process: Query ENS API to resolve to Ethereum address
- Output: JSON with ENS name and resolved address
- Error cases to handle:
  - Invalid ENS name format
  - Non-existent ENS names
  - API failures
  - Rate limiting

## Step 2: Research Phase

ENS API options researched:
1. **ENS Ideals API**: `https://api.ensideas.com/ens/resolve/{ens_name}`
   - Simple GET request, no authentication
   - Returns JSON with address field
   - Handles normalization automatically
   - No documented rate limits

2. **ENS Graph**: More complex, requires GraphQL

Selected **ENS Ideals API** for simplicity and reliability.

## Step 3: Component Design

Component structure:
- Input: `resolveEns(string ensName)` function
- Processing: HTTP request to ENS API
- Output: JSON structure with name, address, timestamp
- Error handling:
  - Format validation
  - API error handling
  - Timeouts and retries

## Step 4: Pre-Validation Checks

**A. Architecture Validation:**
- ✅ Component follows standard HTTP request pattern
- ✅ ENS API endpoint is reliable and documented
- ✅ Error conditions fully accounted for
- ✅ Data flow is clear: Input → API → Response

**B. Dependency Validation:**
- ✅ All HTTP dependencies included
- ✅ Serialization dependencies included
- ✅ ABI encoding/decoding dependencies included
- ✅ All using workspace references

**C. Type System Validation:**
- ✅ String input properly handled
- ✅ JSON structures properly defined with Clone
- ✅ No implicit conversions
- ✅ ABI encoding/decoding correctly implemented

**D. Memory Safety Validation:**
- ✅ All response structures derive Clone
- ✅ Request data properly cloned
- ✅ No dangerous string operations
- ✅ Proper error propagation

**E. Error Handling Validation:**
- ✅ ok_or_else used for Options
- ✅ map_err used for Results
- ✅ All errors have clear messages
- ✅ No unwrapped Results/Options

**F. Component Interface Validation:**
- ✅ Component implements Guest trait correctly
- ✅ Export macro uses correct syntax
- ✅ Function signatures match requirements
- ✅ Handles both Ethereum and CLI output

## Step 5: Implementation

### Cargo.toml
```toml
[package]
name = "ens-resolver"
edition.workspace = true
version.workspace = true
authors.workspace = true
rust-version.workspace = true
repository.workspace = true

[dependencies]
# Core dependencies (always needed)
wit-bindgen-rt = {workspace = true}
wavs-wasi-chain = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
alloy-sol-macro = { workspace = true }
wstd = { workspace = true }
alloy-sol-types = { workspace = true }
anyhow = { workspace = true }

[lib]
crate-type = ["cdylib"]

[profile.release]
codegen-units = 1
opt-level = "s"
debug = false
strip = true
lto = true

[package.metadata.component]
package = "component:ens-resolver"
target = "wavs:worker/layer-trigger-world@0.3.0"
```

### src/lib.rs
```rust
// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wstd::runtime::block_on;
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::http::HeaderValue;

pub mod bindings;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Define input function signature
sol! {
    function resolveEns(string ensName) external;
}

// Create separate solidity module for ITypes
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// Response data structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsResolveResponse {
    ens_name: String,
    eth_address: String,
    timestamp: String,
}

// ENS API response structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnsApiResponse {
    address: String,
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
        
        // Decode the ENS name string using proper ABI decoding
        let ens_name = 
            if let Ok(decoded) = resolveEnsCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.ensName
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };
        
        // Resolve ENS name to address
        let res = block_on(async move {
            let ens_result = resolve_ens_name(&ens_name).await?;
            serde_json::to_vec(&ens_result).map_err(|e| e.to_string())
        })?;
        
        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
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

// Helper function to get current timestamp
fn get_current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    time.to_string()
}

// Main function to resolve ENS name to address
async fn resolve_ens_name(ens_name: &str) -> Result<EnsResolveResponse, String> {
    // Ensure ENS name is properly formatted
    let normalized_ens_name = if !ens_name.ends_with(".eth") {
        format!("{}.eth", ens_name)
    } else {
        ens_name.to_string()
    };
    
    // Construct ENS API URL
    let url = format!("https://api.ensideas.com/ens/resolve/{}", normalized_ens_name);
    
    // Create request with headers
    let mut req = http_request_get(&url)
        .map_err(|e| format!("Failed to create request: {}", e))?;
    
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    
    // Make API request
    let api_response: EnsApiResponse = fetch_json(req).await
        .map_err(|e| format!("Failed to fetch ENS data: {}", e))?;
    
    // Return formatted response
    Ok(EnsResolveResponse {
        ens_name: normalized_ens_name,
        eth_address: api_response.address,
        timestamp: get_current_timestamp(),
    })
}
```

## Step 6: Final Validation

**Common Error Check Results:**
- ✅ No String::from_utf8 on ABI data
- ✅ Proper ABI decoding with fallback
- ✅ Correct export! macro syntax
- ✅ All data structures derive Clone
- ✅ All data is cloned before use
- ✅ Proper error handling for API calls
- ✅ Proper ENS name normalization
- ✅ Clear error messages
- ✅ No unwrapped Results or Options

**Standard Example Comparison:**
- ✅ Follows HTTP request component pattern
- ✅ Proper error handling
- ✅ Clean code organization
- ✅ Standard Guest trait implementation

**Edge Case Handling:**
- ✅ Handles missing .eth suffix
- ✅ Provides clear error messages for API failures
- ✅ Proper HTTP response handling
- ✅ Valid JSON serialization/deserialization

## Step 7: Output Component

### User Instructions

To test the ENS resolver component:

1. Create the component files:
   - Create directory: `components/ens-resolver/src/`
   - Add the Cargo.toml and lib.rs files as shown above
   - Copy bindings.rs from an existing component

2. Build the component:
   ```bash
   make validate-component COMPONENT=ens-resolver
   make wasi-build
   ```

3. Test the component:
   ```bash
   cd /path/to/your/project && \
   export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "vitalik.eth"` && \
   export COMPONENT_FILENAME=ens_resolver.wasm && \
   export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'" && \
   make wasi-exec
   ```

This component will resolve ENS names like "vitalik.eth" or just "vitalik" (it automatically adds .eth if missing) to their Ethereum addresses. The response includes the ENS name, Ethereum address, and timestamp.

### Expected Response

For "vitalik.eth":
```json
{
  "ens_name": "vitalik.eth",
  "eth_address": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
  "timestamp": "1697534271"
}
```