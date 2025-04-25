# WAVSai Knowledge Base

This document serves as the comprehensive knowledge base for Claude to create error-free WAVS components. Follow this guide systematically when creating components to ensure they work perfectly on the first attempt.

## Component Creation Process

Always follow this exact sequence when creating a WAVS component:

1. **Understand Requirements**: Clarify what the component needs to do
2. **Research Phase**: Research any unknown APIs or services
3. **Plan Component**: Create a detailed implementation plan
4. **Pre-validate Design**: Check for potential errors before coding
5. **Implement Component**: Create the component files
6. **Validate Component**: Verify against common error patterns
7. **Generate User Instructions**: Provide clear usage instructions

## Component Structure

A WAVS component consists of exactly these files:

1. `Cargo.toml` - Package and dependency configuration
2. `src/lib.rs` - Component implementation code
3. `src/bindings.rs` - Auto-generated bindings (never edit this)

### Cargo.toml Structure

Always use this exact template for `Cargo.toml` with appropriate dependencies:

```toml
[package]
name = "component-name"
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

# Add for blockchain interactions
alloy-primitives = { workspace = true }
alloy-provider = { workspace = true }
alloy-rpc-types = { workspace = true }
alloy-network = { workspace = true }

[lib]
crate-type = ["cdylib"]

[profile.release]
codegen-units = 1
opt-level = "s"
debug = false
strip = true
lto = true

[package.metadata.component]
package = "component:component-name"
target = "wavs:worker/layer-trigger-world@0.3.0"
```

## Component Implementation Structure

Always structure `lib.rs` files in this exact order:

1. **Imports** - All required imports
2. **Bindings Module** - Declaration of the bindings module
3. **Solidity Type Definitions** - Define all Solidity types and interfaces
4. **Data Structures** - Define all data structures
5. **Component Declaration** - Declare the component and export it
6. **Implementation** - Implement the Guest trait
7. **Helper Functions** - Implement all helper functions

### Standard Imports

Always include these imports for blockchain components:

```rust
// Core imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wstd::runtime::block_on;

// Blockchain-specific imports
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use wavs_wasi_chain::ethereum::new_eth_provider;

// Binding imports
pub mod bindings;
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};
```

### Common Component Template

Always follow this base template for components:

```rust
// Standard imports here...

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Define input function signature
sol! {
    function yourFunctionName(string param1) external;
}

// Create separate solidity module for ITypes
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// Response data structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct YourResponseType {
    // Fields here
    timestamp: String,
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
        
        // Decode the input parameter
        let parameter = 
            if let Ok(decoded) = yourFunctionNameCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.param1
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };
        
        // Process the request
        let res = block_on(async move {
            let result_data = process_request(&parameter).await?;
            serde_json::to_vec(&result_data).map_err(|e| e.to_string())
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

// Main processing function
async fn process_request(parameter: &str) -> Result<YourResponseType, String> {
    // Implementation here
    
    Ok(YourResponseType {
        // Fields here
        timestamp: get_current_timestamp(),
    })
}
```

## Common Error Patterns and Prevention

### 1. ABI Data Handling Errors

**Problem**: Using `String::from_utf8` on ABI-encoded data.

**Prevention**: NEVER use `String::from_utf8` on ABI-encoded data. Always use proper ABI decoding:

```rust
// WRONG - Will always fail
let input_string = String::from_utf8(abi_encoded_data)?;

// CORRECT - Use proper ABI decoding
let parameter = 
    if let Ok(decoded) = YourFunctionCall::abi_decode(&req_clone, false) {
        decoded.parameter
    } else {
        match String::abi_decode(&req_clone, false) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
        }
    };
```

### 2. Trait Import Errors

**Problem**: Missing trait imports for methods.

**Prevention**: Always import both structs AND their required traits:

```rust
// WRONG - Missing Provider trait
use alloy_provider::RootProvider;

// CORRECT - Include both struct and trait
use alloy_provider::{Provider, RootProvider};
```

### 3. Type Conversion Errors

**Problem**: Incorrect type conversions or missing conversions.

**Prevention**: Always be explicit with type conversions, especially with blockchain numeric types:

```rust
// WRONG - Trying to use .into() for numeric conversions
let temp_uint: U256 = temperature.into();

// WRONG - Using u128 directly with U256 operations
let result = gas_price_u128 * U256::from(80); // Error!

// CORRECT - Explicit conversion to U256
let gas_price = U256::from(gas_price_u128);
let result = gas_price * U256::from(80);
```

### 4. Option Type Errors

**Problem**: Using `map_err` on Option types.

**Prevention**: Use `ok_or_else` for Options, `map_err` for Results:

```rust
// WRONG - Option types don't have map_err
let config = get_eth_chain_config("mainnet").map_err(|e| e.to_string())?;

// CORRECT - For Option types
let config = get_eth_chain_config("mainnet")
    .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

// CORRECT - For Result types
let balance = fetch_balance(address).await
    .map_err(|e| format!("Balance fetch failed: {}", e))?;
```

### 5. Export Macro Errors

**Problem**: Incorrect export macro usage.

**Prevention**: Always use the exact correct export macro format:

```rust
// WRONG
export!(Component);

// CORRECT
export!(Component with_types_in bindings);
```

### 6. Memory Management Errors

**Problem**: Ownership issues from not cloning data.

**Prevention**: Always clone data before use to avoid ownership issues:

```rust
// WRONG - Creates ownership issues
let result = process_data(&data);

// CORRECT - Clone data before use
let data_clone = data.clone();
let result = process_data(&data_clone);
```

## Comprehensive Validation Checklist

Before generating any component code, verify that:

1. **Imports and Dependencies**
   - [ ] All required imports are included
   - [ ] Both structs and their traits are imported
   - [ ] All dependencies are in Cargo.toml with `{workspace = true}`

2. **Component Structure**
   - [ ] Component implements Guest trait
   - [ ] Component is exported with correct export! macro
   - [ ] All required helper functions are implemented

3. **ABI Handling**
   - [ ] No String::from_utf8 on ABI data
   - [ ] Proper ABI decoding with fallbacks
   - [ ] ABI encoding for responses

4. **Type System**
   - [ ] All type conversions are explicit
   - [ ] Blockchain-specific types are used correctly
   - [ ] No implicit conversions between numeric types

5. **Error Handling**
   - [ ] ok_or_else used for Option types
   - [ ] map_err used for Result types
   - [ ] All potential errors are handled

6. **Memory Management**
   - [ ] All data is cloned before use
   - [ ] All API response structures derive Clone
   - [ ] No moving out of collections

7. **Async Code**
   - [ ] block_on used for async functions
   - [ ] Proper error handling in async blocks
   - [ ] No .await outside of async functions

## Common Components Reference

### ENS Name to Address Resolver

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

## Testing Guidelines

Always test components with this exact command:

```bash
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your-parameter-here"`
export COMPONENT_FILENAME=your_component_name.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## Known API Integrations

### ENS API

- **Endpoint**: `https://api.ensideas.com/ens/resolve/{ens_name}`
- **Method**: GET
- **Response Format**: JSON with `address` field
- **Error Cases**:
  - Invalid ENS name: Returns 400 error
  - Non-existent ENS name: Returns empty response
  - Rate limiting: May return 429 error

### Ethereum RPC

- **Provider Setup**:
  ```rust
  let chain_config = get_eth_chain_config("mainnet")
      .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;
  
  let provider: RootProvider<Ethereum> = 
      new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());
  ```

- **Common Methods**:
  - `get_gas_price()` - Returns `u128` (must convert to U256)
  - `get_balance(address)` - Returns `U256`
  - `call(&tx)` - For contract interactions

## Final Validation Rules

Before returning a component:

1. Validate imports for completeness
2. Ensure all types are properly defined
3. Verify all error handling is in place
4. Check for Clone derivation on all API structures
5. Verify all data is properly cloned
6. Ensure all ABI encoding/decoding is correct
7. Check export macro is properly used

Only after passing ALL validation checks should you generate the final component code.