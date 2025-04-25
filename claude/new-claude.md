# WAVS Component Creation Guide with WAVSai

This guide contains essential information for creating WAVS components that build and run without errors. Follow the WAVSai process for guaranteed error-free components.

## WAVSai Component Creation Process

When you ask me to create a WAVS component, I'll follow this systematic process to ensure it works perfectly on the first try:

1. **Understand Requirements**: I'll clarify exactly what the component needs to do
2. **Research Phase**: I'll research any unknown APIs or services needed
3. **Plan Component**: I'll create a detailed implementation plan
4. **Pre-validate Design**: I'll check for potential errors before coding
5. **Implement Component**: I'll create the component files (Cargo.toml, lib.rs)
6. **Validate Component**: I'll verify against common error patterns
7. **Generate User Instructions**: I'll provide clear usage instructions

## Component Structure

A WAVS component needs:
1. `Cargo.toml` - Dependencies configuration
2. `src/lib.rs` - Component implementation
3. `src/bindings.rs` - Auto-generated, never edit

## WAVSai Pre-Implementation Validation Checklist

Before writing any component code, I'll verify these critical aspects:

1. **Imports and Dependencies**
   - [ ] Each and every method and type is used properly and has the proper import
   - [ ] Both structs and their traits are imported
   - [ ] Verify all required imports are imported properly
   - [ ] All dependencies are in Cargo.toml with `{workspace = true}`
   - [ ] Any unused imports are removed

2. **Component Structure**
   - [ ] Component will implement Guest trait
   - [ ] Component will be exported with correct export! macro
   - [ ] All required helper functions will be implemented

3. **ABI Handling**
   - [ ] No String::from_utf8 on ABI data
   - [ ] Proper ABI decoding with fallbacks planned
   - [ ] ABI encoding for responses implemented

4. **Type System**
   - [ ] All type conversions will be explicit
   - [ ] Blockchain-specific types will be used correctly
   - [ ] No implicit conversions between numeric types

5. **Error Handling**
   - [ ] ok_or_else used for Option types
   - [ ] map_err used for Result types
   - [ ] All potential errors are handled

6. **Memory Management**
   - [ ] All data will be cloned before use
   - [ ] All API response structures will derive Clone
   - [ ] No moving out of collections

## Creating a Component

### 1. Cargo.toml Template

```toml
[package]
name = "your-component-name"
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
package = "component:your-component-name"
target = "wavs:worker/layer-trigger-world@0.3.0"
```

CRITICAL: Never use direct version numbers - always use `{ workspace = true }`.

### 2. Component Implementation (lib.rs)

#### Basic Structure

```rust
// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wstd::runtime::block_on;

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Component struct declaration 
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) = 
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;
            
        // 1. Decode input data
        // 2. Process data
        // 3. Return encoded output
        
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(result),
        };
        Ok(output)
    }
}
```

#### Trigger Event Handling

```rust
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
```

## Critical Components for Error-Free Code

### 1. ABI Handling

NEVER use `String::from_utf8` on ABI-encoded data. This will ALWAYS fail with "invalid utf-8 sequence".

```rust
// WRONG - Will fail
let input_string = String::from_utf8(abi_encoded_data)?;

// CORRECT - Use proper ABI decoding
let req_clone = req.clone(); // Clone first

// IMPORTANT: For consistency, ALWAYS use string inputs in all components,
// even for numeric, boolean, or other data types. Parse to the required type afterwards.

// Decode the data using proper ABI decoding
let parameter = 
    if let Ok(decoded) = YourFunctionCall::abi_decode(&req_clone, false) {
        // Successfully decoded as function call
        decoded.parameter
    } else {
        // Try decoding just as a string parameter
        match String::abi_decode(&req_clone, false) {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
        }
    };
    
// For numeric parameters, parse from the string
// Example: When you need a number but input is a string:
let number = parameter.parse::<u64>().map_err(|e| format!("Invalid number: {}", e))?;
```

### 2. Solidity Types Definition

```rust
// Define Solidity function signature that matches your input format
sol! {
    function checkBalance(string wallet) external;
}

// Define Solidity return structure (if needed)
sol! {
    struct BalanceData {
        address wallet;
        uint256 balance;
        string tokenSymbol;
        bool success;
    }
}

// Define Solidity interfaces for contracts
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// Create separate solidity module - IMPORTANT!
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
    
    // Define your other Solidity types here
}
```

### 3. Data Structure Ownership

ALWAYS derive `Clone` for API response data structures:

```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseData {
    field1: String,
    field2: u64,
    // other fields
}
```

ALWAYS clone data before use to avoid ownership issues:

```rust
// WRONG - Creates temporary that is dropped immediately
let result = process_data(&data.clone());

// CORRECT - Create variable to hold the cloned data
let data_clone = data.clone();
let result = process_data(&data_clone);
```

### 4. Network Requests

```rust
use wstd::runtime::block_on;  // Required for async
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::http::HeaderValue;

async fn make_request() -> Result<ResponseType, String> {
    let url = format!("https://api.example.com/endpoint?param={}", param);
    
    // Create request with headers
    let mut req = http_request_get(&url)
        .map_err(|e| format!("Failed to create request: {}", e))?;
    
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    
    // Parse JSON response - response type MUST derive Clone
    let response: ResponseType = fetch_json(req).await
        .map_err(|e| format!("Failed to fetch data: {}", e))?;
    
    Ok(response)
}

// Use block_on in component logic
fn process_data() -> Result<ResponseType, String> {
    block_on(async { make_request().await })
}
```

### 5. Option/Result Handling

```rust
// WRONG - Option types don't have map_err
let config = get_eth_chain_config("mainnet").map_err(|e| e.to_string())?;

// CORRECT - For Option types, use ok_or_else()
let config = get_eth_chain_config("mainnet")
    .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

// CORRECT - For Result types, use map_err()
let balance = fetch_balance(address).await
    .map_err(|e| format!("Balance fetch failed: {}", e))?;
```

### 6. Blockchain Interactions

```rust
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use std::str::FromStr; // Required for parsing addresses

async fn query_blockchain(address_str: &str) -> Result<ResponseData, String> {
    // Parse address
    let address = Address::from_str(address_str)
        .map_err(|e| format!("Invalid address format: {}", e))?;
    
    // Get chain configuration from environment
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get chain config".to_string())?;
    
    // Create provider
    let provider: RootProvider<Ethereum> = 
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());
    
    // Create contract call
    let contract_call = IERC20::balanceOfCall { owner: address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(contract_address)),
        input: TransactionInput { 
            input: Some(contract_call.abi_encode().into()), 
            data: None 
        },
        ..Default::default()
    };
    
    // Execute call
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance: U256 = U256::from_be_slice(&result);
    
    Ok(ResponseData { /* your data here */ })
}
```

### 7. Numeric Type Handling 

```rust
// WRONG - Using .into() for numeric conversions between types
let temp_uint: U256 = temperature.into(); // DON'T DO THIS

// CORRECT - String parsing method works reliably for all numeric types
let temperature: u128 = 29300;
let temperature_uint256 = temperature.to_string().parse::<U256>().unwrap();

// CORRECT - Always use explicit casts between numeric types
let decimals: u8 = decimals_u32 as u8;

// CORRECT - Handling token decimals correctly
let mut divisor = U256::from(1);
for _ in 0..decimals {
    divisor = divisor * U256::from(10);
}
let formatted_amount = amount / divisor;
```

## Component Examples by Task

### 1. ENS Name to Address Resolver

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

### 2. Token Balance Checker

```rust
// IMPORTS
use alloy_network::Ethereum;
use alloy_primitives::{Address, TxKind, U256};
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

pub mod bindings;
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// TOKEN INTERFACE
sol! {
    interface IERC20 {
        function balanceOf(address owner) external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

// INPUT FUNCTION SIGNATURE 
sol! {
    function checkTokenBalance(string wallet) external;
}

// FIXED CONTRACT ADDRESS
const TOKEN_CONTRACT_ADDRESS: &str = "0x..."; // Your token contract address

// RESPONSE STRUCTURE - MUST DERIVE CLONE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenBalanceData {
    wallet: String,
    balance_raw: String,
    balance_formatted: String,
    token_contract: String,
    timestamp: String,
}

// COMPONENT IMPLEMENTATION
struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Decode trigger data
        let (trigger_id, req, dest) = 
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;
        
        // Clone request data to avoid ownership issues
        let req_clone = req.clone();
        
        // Decode the wallet address string using proper ABI decoding
        let wallet_address_str = 
            if let Ok(decoded) = checkTokenBalanceCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.wallet
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };
        
        // Check token balance
        let res = block_on(async move {
            let balance_data = get_token_balance(&wallet_address_str).await?;
            serde_json::to_vec(&balance_data).map_err(|e| e.to_string())
        })?;
        
        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

// BALANCE CHECKER IMPLEMENTATION
async fn get_token_balance(wallet_address_str: &str) -> Result<TokenBalanceData, String> {
    // Parse wallet address
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address: {}", e))?;
    
    // Parse token contract address
    let token_address = Address::from_str(TOKEN_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid token address: {}", e))?;
    
    // Get Ethereum provider
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;
    
    let provider: RootProvider<Ethereum> = 
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());
    
    // Get token balance
    let balance_call = IERC20::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(token_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };
    
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance_raw: U256 = U256::from_be_slice(&result);
    
    // Get token decimals
    let decimals_call = IERC20::decimalsCall {};
    let tx_decimals = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(token_address)),
        input: TransactionInput { input: Some(decimals_call.abi_encode().into()), data: None },
        ..Default::default()
    };
    
    let result_decimals = provider.call(&tx_decimals).await.map_err(|e| e.to_string())?;
    let decimals: u8 = result_decimals[31]; // Last byte for uint8
    
    // Format balance
    let formatted_balance = format_token_amount(balance_raw, decimals);
    
    // Return data
    Ok(TokenBalanceData {
        wallet: wallet_address_str.to_string(),
        balance_raw: balance_raw.to_string(),
        balance_formatted: formatted_balance,
        token_contract: TOKEN_CONTRACT_ADDRESS.to_string(),
        timestamp: get_current_timestamp(),
    })
}
```

### 3. API Data Fetcher

```rust
// IMPORTS
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// INPUT FUNCTION SIGNATURE
sol! {
    function fetchApiData(string param) external;
}

// RESPONSE STRUCTURE - MUST DERIVE CLONE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse {
    field1: String,
    field2: u64,
    // other fields
}

// RESULT DATA STRUCTURE - MUST DERIVE CLONE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResultData {
    input_param: String,
    result: String,
    timestamp: String,
}

// COMPONENT IMPLEMENTATION
struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Decode trigger data
        let (trigger_id, req, dest) = 
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;
        
        // Clone request data to avoid ownership issues
        let req_clone = req.clone();
        
        // Decode the parameter string using proper ABI decoding
        let param = 
            if let Ok(decoded) = fetchApiDataCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.param
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };
        
        // Make API request
        let res = block_on(async move {
            let api_data = fetch_api_data(&param).await?;
            serde_json::to_vec(&api_data).map_err(|e| e.to_string())
        })?;
        
        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

// API FETCHER IMPLEMENTATION
async fn fetch_api_data(param: &str) -> Result<ResultData, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_API_KEY")
        .map_err(|_| "Failed to get API_KEY from environment variables".to_string())?;
    
    // Create API URL
    let url = format!(
        "https://api.example.com/endpoint?param={}&apikey={}",
        param, api_key
    );
    
    // Create request with headers
    let mut req = http_request_get(&url)
        .map_err(|e| format!("Failed to create request: {}", e))?;
    
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    
    // Make API request
    let api_response: ApiResponse = fetch_json(req).await
        .map_err(|e| format!("Failed to fetch data: {}", e))?;
    
    // Process and return data
    Ok(ResultData {
        input_param: param.to_string(),
        result: format!("{}: {}", api_response.field1, api_response.field2),
        timestamp: get_current_timestamp(),
    })
}
```

### 4. NFT Ownership Checker

```rust
// IMPORTS
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

pub mod bindings;
use crate::bindings::host::get_eth_chain_config;
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// NFT INTERFACE
sol! {
    interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
    }
}

// INPUT FUNCTION SIGNATURE
sol! {
    function checkNftOwnership(string wallet) external;
}

// FIXED CONTRACT ADDRESS
const NFT_CONTRACT_ADDRESS: &str = "0xbd3531da5cf5857e7cfaa92426877b022e612cf8"; // Bored Ape contract

// RESPONSE STRUCTURE - MUST DERIVE CLONE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NftOwnershipData {
    wallet: String,
    owns_nft: bool,
    balance: String,
    nft_contract: String,
    contract_name: String,
    timestamp: String,
}

// COMPONENT IMPLEMENTATION
struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        // Decode trigger data
        let (trigger_id, req, dest) = 
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;
        
        // Clone request data to avoid ownership issues
        let req_clone = req.clone();
        
        // Decode the wallet address string using proper ABI decoding
        let wallet_address_str = 
            if let Ok(decoded) = checkNftOwnershipCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.wallet
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };
        
        // Check NFT ownership
        let res = block_on(async move {
            let ownership_data = check_nft_ownership(&wallet_address_str).await?;
            serde_json::to_vec(&ownership_data).map_err(|e| e.to_string())
        })?;
        
        // Return result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

// NFT OWNERSHIP CHECKER IMPLEMENTATION
async fn check_nft_ownership(wallet_address_str: &str) -> Result<NftOwnershipData, String> {
    // Parse wallet address
    let wallet_address = Address::from_str(wallet_address_str)
        .map_err(|e| format!("Invalid wallet address: {}", e))?;
    
    // Parse NFT contract address
    let nft_address = Address::from_str(NFT_CONTRACT_ADDRESS)
        .map_err(|e| format!("Invalid NFT contract address: {}", e))?;
    
    // Get Ethereum provider
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;
    
    let provider: RootProvider<Ethereum> = 
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());
    
    // Check NFT balance
    let balance_call = IERC721::balanceOfCall { owner: wallet_address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(nft_address)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };
    
    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance: U256 = U256::from_be_slice(&result);
    
    // Determine if wallet owns at least one NFT
    let owns_nft = balance > U256::ZERO;
    
    // Return data
    Ok(NftOwnershipData {
        wallet: wallet_address_str.to_string(),
        owns_nft,
        balance: balance.to_string(),
        nft_contract: NFT_CONTRACT_ADDRESS.to_string(),
        contract_name: "BAYC".to_string(),
        timestamp: get_current_timestamp(),
    })
}
```

## Component Execution Process

### Development Workflow

1. Create the component directory and copy the bindings (bindings will be written over during the build):
   ```bash
   mkdir -p components/your-component-name/src
   cp components/eth-price-oracle/src/bindings.rs components/your-component-name/src/
   ```

2. Create Cargo.toml using the provided template

3. Create a plan.md file with an overview of the component you'll make

4. Create lib.rs with proper implementation:
   - Define proper imports
   - Implement Solidity interfaces
   - Create component struct and implementation
   - Implement required helper functions
   - Handle errors and ownership correctly

5. Validate component:
   ```bash
   make validate-component COMPONENT=your-component-name
   ```
   - Fix ALL errors before continuing

6. Build the component:
   ```bash
   make wasi-build
   ```

### Testing Commands

To test with the WASI executor:

```bash
# IMPORTANT: Always use string parameters, even for numeric values!
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your parameter here"`
export COMPONENT_FILENAME=your_component_name.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_API_KEY\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## Common Errors to Avoid

- ✅ ALWAYS use `{ workspace = true }` in Cargo.toml, never explicit versions
- ✅ ALWAYS implement the Guest trait and export your component
- ✅ ALWAYS use `export!(Component with_types_in bindings)`
- ✅ ALWAYS use `clone()` before consuming data to avoid ownership issues
- ✅ ALWAYS derive `Clone` for API response data structures
- ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- ✅ ALWAYS use string parameters for CLI testing (`cast abi-encode "f(string)" "5"` instead of `f(uint256)`)
- ✅ ALWAYS use `.to_string()` to convert string literals (&str) to String types in struct field assignments
- ✅ NEVER edit bindings.rs - it's auto-generated

## Final WAVSai Component Validation Checklist

Before considering the component complete, I'll verify these critical areas:

1. **ABI Data Handling**: 
   - [ ] No String::from_utf8 on ABI-encoded data
   - [ ] Proper cascading decode pattern implemented
   - [ ] ABI encoding for output data

2. **Trait Imports**:
   - [ ] Both structs AND their traits are imported
   - [ ] Provider trait included with RootProvider
   - [ ] SolCall trait imported when abi_encode() is used

3. **Type Conversions**:
   - [ ] Explicit conversions for all numeric types
   - [ ] Proper handling of blockchain-specific types
   - [ ] No reliance on .into() for numeric conversions

4. **Option/Result Handling**:
   - [ ] ok_or_else() used for Option types
   - [ ] map_err() used for Result types
   - [ ] Descriptive error messages provided

5. **Export Macro**:
   - [ ] Correct format: export!(Component with_types_in bindings)

6. **Memory Management**:
   - [ ] All data cloned before use
   - [ ] All API response structs derive Clone
   - [ ] No moving from borrowed data

7. **Component Output**:
   - [ ] Clear instructions for building
   - [ ] Testing commands provided
   - [ ] Expected behavior described

This complete guide will help you create WAVS components that pass validation, build without errors, and execute correctly. By following the WAVSai process, you can achieve error-free components on the first try.
