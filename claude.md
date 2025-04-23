# WAVS Component Creation Guide

This guide contains essential information for creating WAVS components that build and run without errors.

## Component Structure

A WAVS component needs:
1. `Cargo.toml` - Dependencies configuration
2. `src/lib.rs` - Component implementation
3. `src/bindings.rs` - Auto-generated, never edit

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
            let log_clone = log.clone(); // Always clone log
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
    solidity::DataWithId { 
        triggerId: trigger_id, 
        data: output.as_ref().to_vec().into() 
    }.abi_encode()
}
```

## Critical Components

### 1. ABI Handling

NEVER use `String::from_utf8` on ABI-encoded data. This will ALWAYS fail with "invalid utf-8 sequence".

```rust
// WRONG - Will fail
let input_string = String::from_utf8(abi_encoded_data)?;

// CORRECT - Use proper ABI decoding
let req_clone = req.clone(); // Clone first

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

Here are templates for common WAVS component tasks:

### 1. Token Balance Checker

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

### 2. API Data Fetcher

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

### 3. NFT Ownership Checker

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

1. Create the component directory:
   ```bash
   mkdir -p components/your-component-name/src
   ```

2. Create Cargo.toml using the provided template

3. Create lib.rs with proper implementation:
   - Define proper imports
   - Implement Solidity interfaces
   - Create component struct and implementation
   - Implement required helper functions
   - Handle errors and ownership correctly

4. Validate component:
   ```bash
   make validate-component COMPONENT=your-component-name
   ```
   - Fix ALL errors before continuing

5. Build the component:
   ```bash
   make wasi-build
   ```

### Testing Commands

To test with the WASI executor:

```bash
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your parameter here"`
export COMPONENT_FILENAME=your_component_name.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_API_KEY\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## Avoid Common Errors

- ✅ ALWAYS use `{ workspace = true }` in Cargo.toml, never explicit versions
- ✅ ALWAYS implement the Guest trait and export your component
- ✅ ALWAYS use `export!(Component with_types_in bindings)`
- ✅ ALWAYS use `clone()` before consuming data to avoid ownership issues
- ✅ ALWAYS derive `Clone` for API response data structures
- ✅ ALWAYS decode ABI data properly, never with `String::from_utf8`
- ✅ ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
- ✅ NEVER edit bindings.rs - it's auto-generated

## Validation Checklist

1. Component structure:
   - Implements Guest trait
   - Exports component correctly
   - Properly handles TriggerAction and TriggerData

2. ABI handling:
   - Properly decodes function calls
   - Avoids String::from_utf8 on ABI data

3. Data ownership:
   - All API structures derive Clone
   - Clones data before use
   - Avoids moving out of collections

4. Error handling:
   - Uses ok_or_else() for Option types
   - Uses map_err() for Result types
   - Provides descriptive error messages

5. Imports:
   - Includes all required traits and types
   - Uses correct import paths
   - Properly imports SolCall for encoding

6. Component structure:
   - Uses proper sol! macro with correct syntax
   - Correctly defines Solidity types in solidity module
   - Implements required functions

7. Security:
   - No hardcoded API keys or secrets
   - Uses environment variables for sensitive data

8. Dependencies:
   - Uses workspace dependencies correctly
   - Includes all required dependencies

9. Solidity types:
   - Properly imports sol macro
   - Uses solidity module correctly
   - Handles numeric conversions safely

10. Network requests:
    - Uses block_on for async functions
    - Uses fetch_json with correct headers
    - Handles API responses correctly

With this guide, you should be able to create any WAVS component that passes validation, builds without errors, and executes correctly.
