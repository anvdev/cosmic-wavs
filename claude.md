# Creating WAVS components

You started a new job at Layer Labs. Your task is to create new components that run using WAVS, a WASI runtime for AVSs. Read this guide carefully, as it will orient you on how to build services. The repo you are in is a foundry template with an existing example in components/eth-price-oracle/. This repo uses a makefile to streamline commands.

## Creating New Components

When creating a new component, you must follow these steps completely to avoid common errors:

**Phase 1: Prepare**
1. Read claude.md
2. Study the component example in /components/eth-price-oracle
3. Read /components/test_utils/validate_component.sh in its entirety. It contains information on common errors when creating components.
4. Read the files in components/test_utils/src. They contain more best-practice code examples.

**Phase 2: Create**
5. Copy the `eth-price-oracle` component's Cargo.toml and modify the name
6. Copy bindings.rs from `eth-price-oracle`
7. Never edit bindings.rs
8. Create a lib.rs file similar to the `eth-price-oracle`


**Phase 3: Test**
9.  Double-check your component code against validate_component.sh, eth-price-oracle example, and claude.md to ensure it is made correctly
10. Run `make validate-component COMPONENT=your-component-name` to run the validation tests.
11. You must fix all errors and warnings from the validation test before proceeding or the component will not build properly.
12. Repeat steps 10 and 11 until the component passes all tests.

**Phase 4: Build**

13. Proceed only after there are no more errors or warnings when running `make validate-component COMPONENT=your-component-name`.
14. Run `make wasi-build`: this command builds every component in the /component directory, generates bindings automatically (you do not ever need to edit a bindings.rs file!), automatically compiles components to WASM, and places them automatically in the /compiled folder.
15. If the build fails, you will need to create fixes, pass validation checks again, and build again. ALWAYS AVOID BUILDING MORE THAN ONCE. ALL ERRORS SHOULD BE CAUGHT BEFORE EVER RUNNING THE BUILD COMMAND.

**Phase 5: Test**

16. Prepare the `make wasi-exec` command. IMPORTANT! As an LLM, you cannot execute the `wasi-exec` command ever. Always provide the command to the user in the terminal and ask them to run it manually:

```bash
# ONLY use this format for component input data:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your long string here"`

# CRITICAL: When handling ABI-encoded inputs:
# - NEVER use String::from_utf8 directly on binary data
# - ABI-encoded data is binary and must be handled according to its format

export COMPONENT_FILENAME=eth_price_oracle.wasm # the filename of your compiled component.
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'" # The service config

make wasi-exec
```

## Environment variables

You can set public and private variables in components.
1. public
   - set in `SERVICE_CONFIG` `kv` array
2. private
   - set in `SERVICE_CONFIG` `host_envs` array
   - MUST be prefixed with `WAVS_ENV_`
   - This variable is set in the .env file in the root of this repo. Use the following steps:
     1. see if `.env` exists: `ls -la .env`
     - If there is no .env file, use `cp .env.example .env`.
     1. Before adding, check if the variable already exists to avoid duplicates
     1. Add your private variable WITHOUT quotes:
```bash
sed -i '' '1i\
WAVS_ENV_MY_API_KEY=your_secret_key_here
' .env
```

Use in component:

```rust
let endpoint = std::env::var("api_endpoint")?;
let api_key = std::env::var("WAVS_ENV_MY_API_KEY")?;
```

IMPORTANT: NEVER hardcode API keys directly in components. Always store API keys and other sensitive data as environment variables using the method above. Do not use quotes in the .env file values as they may cause URL formatting errors.

Set in command:

```bash
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your long string here"` # your input data for testing the component.
export COMPONENT_FILENAME=eth_price_oracle.wasm # the filename of your compiled component.
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_MY_API_KEY\"],\"kv\":[[\"api_endpoint\",\"https://api.example.com\"]],\"workflow_id\":\"default\",\"component_id\":\"default\"}'" # public variable set in kv.

# REMEMBER: As an LLM, you cannot execute this command directly.
# Provide these instructions to the user and ask them to run manually in their terminal:
make wasi-exec
```

## Foundry Template structure

The foundry template is made up of the following main files:

```bash
wavs-foundry-template/
├── README.md
├── makefile               # Commands, variables, and configs
├── components/            # WASI components
│   ├── eth-price-oracle/
│   │   ├── Cargo.toml     # Component dependencies
│   │   ├── lib.rs         # Main Component logic
│   │   └── bindings.rs    # Auto-generated by `make build` NEVER EDIT A bindings.rs FILE
│   │
│   └── test_utils/        # Test utilities for components
│       ├── src/           # Source code for test utilities
│       │   ├── lib.rs     # Main library entry point
│       │   ├── abi_encoding.rs      # ABI encoding/decoding
│       │   ├── code_quality.rs      # Code quality validation
│       │   ├── data_handling.rs     # Data structure
│       │   ├── input_validation.rs  # Input validation
│       │   ├── solidity_types.rs    # Solidity type handling
│       │   ├── network_requests.rs  # Network request
│       │   └── error_handling.rs    # Error handling
│       │
│       ├── Cargo.toml     # Test utils dependencies
│       ├── README.md      # Documentation
│       └── validate_component.sh # Shell script for component validation
├── compiled/              # WASM files compiled by `make build`
├── src/
│   ├── contracts/        # Trigger and submission contracts
│   └── interfaces/       # Solidity interfaces
├── script/               # Scripts used in makefile commands
├── cli.toml              # CLI configuration
├── wavs.toml             # WAVS service configuration
├── Cargo.toml            # Workspace dependencies
├── docs/                 # Documentation
└── .env                  # Private environment variables
```

- The `README` file contains the tutorial commands.
- The `makefile` contains commands for building and deploying the service. It also contains variables and configs for the service.
- The components directory contains the component logic for your service. Running `make wasi-build` will automatically generate bindings and compile components into the `compiled` directory.
- The src directory contains the Solidity contract and interfaces.
- The script directory contains the scripts used in the makefile commands to deploy, trigger, and test the service.
- The `.env` file contains private environment variables and keys. Use `cp .env.example .env` to copy the example `.env` file.
- The `test_utils/` directory contains examples and validation tests. Read this to learn how to make components properly.

## WAVS services

The basic service is made up of a trigger, a component, and submission logic (optional).

[Trigger](#triggers): any onchain event emitted from a contract.

[Component](#components): the main logic of a WAVS service. Components are responsible for processing the trigger data and executing the business logic.

[Submission](#submission): handles the logic for submitting a component's output to the blockchain.

## Triggers

A trigger prompts a WAVS service to run. Operators listen for the trigger event specified by the service and execute the corresponding component off-chain. Triggers can be any onchain event emitted from any contract.

WAVS doesn't interpret the contents of event triggers. Instead, it passes the raw log data to components, which can decode and process the data according to their specific needs.


1. When a service is deployed, it is configured with a trigger address and event, a wasi component, and a submission contract (optional).

2. Registered operators listen to chain logs. Each operator maintains lookup maps and verifies events against registered triggers.

3. When a trigger event is emitted, operators pick up the event and verify the event matches the registered trigger.

4. If a match is found, WAVS creates a `TriggerAction` that wraps the trigger event data.

5. The TriggerAction is converted to a WASI-compatible format and passed to the component where it is decoded and processed.


```rust
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
```


## Components

WASI components contain the main logic of a WAVS service. They are responsible for processing the trigger data and executing the business logic of a service.

A basic component has three main parts:

- Decoding incoming trigger data.
- Processing the data (this is the custom logic of your component).
- Encoding and returning the result for submission (if applicable).


After being passed the `TriggerAction`, the component decodes it using the `decode_event_log_data!` macro from the [`wavs-wasi-chain`](https://docs.rs/wavs-wasi-chain/latest/wavs_wasi_chain/all.html#functions) crate.

Components must implement the `Guest` trait, which is the main interface between your component and the WAVS runtime. The `run` function is the entry point for processing triggers and MUST have EXACTLY this signature:

```rust
impl Guest for YourComponent {
    fn run(trigger: TriggerAction) -> Result<Option<Vec<u8>>, String> {
        // Your implementation here...
    }
}
```

If you need to submit results to the blockchain, results need to be encoded using `abi_encode()`.

The `sol!` macro from `alloy_sol_types` is used to define Solidity types in Rust. It generates Rust structs and implementations that match your Solidity types, including ABI encoding/decoding methods.

IMPORTANT: The example in eth-price-oracle is just one example of a specific component. You will need to make sure that you use appropriate input formats for your component.

The most common error in components is trying to use `String::from_utf8` on ABI-encoded data. This will ALWAYS fail with "invalid utf-8 sequence". Remember:
- NEVER use `String::from_utf8` on ABI-encoded data
- ABI-encoded data is binary and must be handled according to its format

IMPORTANT: Always import the required traits if using these methods:
```rust
use alloy_sol_types::{sol, SolCall, SolValue}; // SolCall needed for abi_encode() on call structs
```

When using methods like `abi_decode` that are implemented by multiple traits, use fully qualified syntax to avoid ambiguity errors:
```rust
// Explicitly specify which trait implementation to use
let trigger_info = <TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
```

Bindings are automatically generated for any files in the `/components` and `/src` directories when the `make build` command is run. Never edit the bindings.rs file.

## Common Type and Data Handling Issues

When building components, you'll need to handle several common issues:

### 1. Rust-Solidity Type Conversions

Solidity and Rust have different type systems. For numeric types:

```rust
// String parsing method - works reliably for all numeric types including Uint<256,4>
let temperature: u128 = 29300;
let temperature_uint256 = temperature.to_string().parse().unwrap();

// AVOID .into() for numeric conversions to Solidity types - it often fails
// let temp_uint = temperature.into(); // DON'T DO THIS - will often fail

// ALWAYS use explicit type conversions for struct fields with specific types
let decimals = 6; // inferred as i32/usize
struct_field: decimals as u32, // explicit cast required between integer types

// CORRECT - Use a loop for exponentiation with U256 (pow expects U256, not u32/u8)
let mut divisor = U256::from(1);
for _ in 0..decimals {
    divisor = divisor * U256::from(10);
}
```

### 2. Binary Data Handling

When working with Solidity structs that contain binary data:

```rust
// IMPORTANT: Always convert Vec<u8> to Bytes explicitly for Solidity data fields
// Use the correct import path: use wavs_wasi_chain::ethereum::alloy_primitives::Bytes;
let data_with_id = solidity::DataWithId {
    triggerId: trigger_id,
    data: Bytes::from(result.abi_encode()), // Convert Vec<u8> to Bytes
};
```

### 3. Input and Output Handling

Components can receive trigger data in two ways:
1. Via an onchain event (after deployment)
2. Via the `wasi-exec` command (for testing)

When testing, use the following input format:

| Input Type | Command | Code Handling |
|------------|---------|--------------|
| String | `cast abi-encode "f(string)" "text"` | Extract from ABI string format |

CRITICAL: NEVER use `String::from_utf8` on ABI-encoded data. ABI-encoded data is binary and must be handled according to its format.

Debug input format:
```rust
println!("Input length: {} bytes", data.len());
let hex_display: Vec<String> = data.iter().take(8).map(|b| format!("{:02x}", b)).collect();
println!("First 8 bytes: {}", hex_display.join(" "));
```

#### Common Input Handling Mistakes

1. **Using String::from_utf8 on ABI-encoded data**
   ```rust
   // WRONG - This will fail with "invalid utf-8 sequence":
   let input = String::from_utf8(abi_encoded_data)?;
   
   // CORRECT - Process ABI-encoded data according to its type
   // For strings, follow the ABI string format specification
   ```

4. **Not cloning data before use**
   ```rust
    // WRONG - Creates temporary that is immediately dropped:
    let result = process_data(&data.clone());  // The clone is dropped after this line

    // CORRECT - Create variable to hold the owned value:
    let data_clone = data.clone();
    let result = process_data(&data_clone);  // data_clone lives as long as needed
   ```


### 4. Network Requests

Components can make HTTP requests using the `wavs-wasi-chain` crate and `block_on` for async handling:

```rust
use wstd::runtime::block_on;  // Required for running async code
use wavs_wasi_chain::http::{fetch_json, http_request_get}; // Correct import path
use wstd::http::HeaderValue; // For setting headers

// IMPORTANT: All API response structs MUST derive Clone
#[derive(Serialize, Deserialize, Debug, Clone)] // Clone prevents ownership errors
struct ResponseType {
    field1: String,
    field2: u64,
}

// The request function must be async
async fn make_request() -> Result<ResponseType, String> {
    // IMPORTANT: When formatting URLs with parameters, be careful with special characters
    // For debugging URL issues, always print the URL before making the request
    let url = "https://api.example.com/endpoint";
    println!("Debug - Request URL: {}", url);
    // Create request with proper headers
    let mut req = http_request_get(&url)
        .map_err(|e| format!("Failed to create request: {}", e))?;
    
    // Add appropriate headers 
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    
    // Parse JSON response
    let response: ResponseType = fetch_json(req).await
        .map_err(|e| format!("Failed to fetch data: {}", e))?;
    Ok(response)
}

// Use block_on in the main component logic
fn process_data() -> Result<ResponseType, String> {
    block_on(async { make_request().await })
}
```

CRITICAL: ABI-encoded string inputs must be properly decoded according to the ABI specification before using in URLs or API calls.

### 5. Event Log Decoding

When using the decode_event_log_data! macro:

```rust
// Always clone the log before decoding to avoid ownership errors
let log_clone = log.clone();

// If your function returns Result<_, anyhow::Error>:
let event: solidity::NewTrigger = decode_event_log_data!(log_clone)?;

// If your function returns Result<_, String>:
let event: solidity::NewTrigger = decode_event_log_data!(log_clone)
    .map_err(|e| e.to_string())?;
```

### 6. Option and Result Handling

When working with Option and Result types:

```rust
// CRITICAL: Never use map_err() on Option types - it will cause a build error
// WRONG:
let chain_config = get_eth_chain_config("mainnet").map_err(|e| e.to_string())?;

// CORRECT - For Option types, use ok_or_else() to convert to Result:
let chain_config = get_eth_chain_config("mainnet")
    .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;

// CORRECT - For Result types, use map_err():
let balance = fetch_balance(address).await.map_err(|e| format!("Balance fetch failed: {}", e))?;

// Check if Option has value before using:
if let Some(config) = get_eth_chain_config("mainnet") {
    // Use config here
} else {
    // Handle None case
}
```

### 7. Data Structure Ownership

When working with data structures in Rust, especially with API responses:

```rust
// CRITICAL: Always derive Clone for ALL data structures used in API responses
// Missing Clone derivation is a common source of build errors
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ApiResponse {
    name: String,
    description: String,
    // other fields...
}

// WRONG - Will cause "borrow of partially moved value" errors:
let weather_struct = WeatherData {
    city: api_response.name, // This moves the string from api_response
    // other fields...
};
let json_data = serde_json::to_vec(&api_response)?; // ERROR: api_response partially moved

// CORRECT - Process in the right order:
let json_data = serde_json::to_vec(&api_response)?; // Use the complete struct first
let weather_struct = WeatherData {
    city: api_response.name, // Then move fields
    // other fields...
};

// ALSO CORRECT - Use clone to avoid ownership issues entirely:
let weather_struct = WeatherData {
    city: api_response.name.clone(), // Clone prevents moving from api_response
    // other fields...
};
let json_data = serde_json::to_vec(&api_response)?; // Works fine now
```

## Best Practices

To build reliable components:

1. ALWAYS Follow the [Creating New Components instructions](#creating-new-components) carefully.

2. **Configure dependencies correctly**:
   - Use workspace dependencies with `{ workspace = true }` syntax
   - NEVER specify direct version numbers in component Cargo.toml
   - For new functionality, always add all necessary dependencies to workspace Cargo.toml first, then reference with `{ workspace = true }`
   - Consider standard library alternatives before adding new dependencies
   - Import HTTP functions from the correct submodule: `wavs_wasi_chain::http::{fetch_json, http_request_get}`
   - CRITICAL: Include essential standard library imports:
     - `use std::str::FromStr;` when using `from_str` methods (e.g., for parsing addresses)
     - `use std::cmp::min;` when comparing values
   - CRITICAL: Ensure all methods and types used in your component are properly imported - missing imports will cause compilation errors
   - CRITICAL: Use `export!` macro (NOT `#[export]` attribute) to export your component

4. Before you run the `make wasi-build` command, double check that all types/methods in your component are imported correctly. Then, make sure all dependencies are listed correctly, or there will be an error on the build.
   - ALWAYS AVOID BUILD ERRORS. Run through the component code, imports, and toml files before building.
   - CRITICAL:Double check all code, imports, and toml before building to ensure there will be no errors on the build.


5. **Handle data ownership properly**:
   - ALWAYS clone data before consuming: data.clone() before passing to any function that takes ownership of the data
   - CRITICAL: NEVER use `&data.clone()` pattern which creates temporary values that are immediately dropped. Instead:
     ```rust
     // Wrong - temporary value is dropped:
     let input = std::str::from_utf8(&data.clone())?; 
     
     // Correct - create variable to hold the owned value:
     let data_clone = data.clone();
     let input = std::str::from_utf8(&data_clone)?;
     ```
   - ALWAYS clone logs for decoding: `let log_clone = log.clone()`
   - ALWAYS clone collection elements: `array[0].field.clone()` to avoid "move out of index" errors
   - ALWAYS properly decode ABI-encoded string inputs according to the ABI specification
   - ALWAYS use string parsing for numbers: `value.to_string().parse()` for Solidity numeric types (avoid .into())
   - ALWAYS use correct Bytes import: `use wavs_wasi_chain::ethereum::alloy_primitives::Bytes`
   - Be careful with Solidity types defined with sol! macro - they can't be directly imported between files with syntax like `trigger::sol::Type` - either define them where needed or create a module to export them

6. **Structure your component for both testing and production**:
   - Implement proper destination-based output handling (CLI vs Ethereum)
   - Include detailed error messages with context in all error cases

### Pre-Deployment Checklist

- Verify all type conversions are handled correctly
- Confirm proper error handling throughout
- Test with actual input formats to verify handling
- Ensure all sensitive data is in environment variables
- Validate output format matches expected contract format
- CRITICAL: Verify all data structures used with API responses implement `Clone` - missing this will cause build errors
- Confirm ABI-encoded string inputs are properly decoded
- CRITICAL: Verify correct UTF-8 handling:
  - NEVER use `String::from_utf8` on ABI-encoded data
  - ABI-encoded data is handled according to its format
- CRITICAL: Check your code and imports. Verify all types and methods used in your component are properly imported

## Troubleshooting Common Errors

| Error Type | Symptom | Solution |
|------------|---------|----------|
| Dependency Version | "failed to select a version for..." | Copy Cargo.toml from eth-price-oracle and change the name |
| Missing Dependency | "unresolved module or crate" | Add the dependency to workspace Cargo.toml first, then reference with `{ workspace = true }` |
| Import Path | "unresolved imports http_request_get" | Use: `use wavs_wasi_chain::http::{fetch_json, http_request_get}` |
| Type Conversion | "expected Uint<256, 4>, found u128" | Use string parsing: `value.to_string().parse().unwrap()` |
| Binary Type Mismatch | "expected Bytes, found Vec<u8>" | Use: `Bytes::from(data)` with correct import |
| Event Decoding | "cannot move out of log.data" | ALWAYS clone: `let log_clone = log.clone()` |
| String Handling | URL formatting errors | Process ABI-encoded data properly, debug with `println!("URL: {}", url)` |
| Input Format | ABI-encoded format issues | Always use `cast abi-encode "f(string)" "text"` |
| HTTP Requests | "invalid uri character" | Check for special characters in URLs, use debug prints to identify issues |
| Ownership Issues | "use of moved value" | Clone data before use: `data.clone()` |
| Collection Access | "cannot move out of index" | Clone when accessing: `array[0].field.clone()` |
| TriggerAction Access | "no field data_input on type TriggerAction" | Use trigger.rs module like the example instead of direct access |
| TriggerData Variants | "no variant or associated item named 'X' found for enum `TriggerData`" | Check the actual enum variants in bindings.rs - the common variants are: `EthContractEvent`, `CosmosContractEvent`, and `Raw` (for testing with wasi-exec) |
| Environment Variables | API key access issues | Include in SERVICE_CONFIG "host_envs" array AND in .env file with WAVS_ENV_ prefix |
| Option/Result Handling | "no method named `map_err` found for enum `Option`" | Use `.ok_or_else(|| "error message".to_string())?` for Option types, NEVER `.map_err()`. For example, `get_eth_chain_config("mainnet").ok_or_else(|| "Config not found".to_string())?` |
| Serialization Error | "trait `Serialize` not implemented for struct from sol! macro" | Create a separate struct with `#[derive(Serialize)]` for JSON output |
| Partially Moved Value | "borrow of partially moved value" | Process data in correct order: serialize/use struct *before* moving its fields |
| Solidity Type Import | "could not find `sol` in module" | Define Solidity types where needed - can't import using `trigger::sol::` syntax |
| Solidity Module Structure | "use of unresolved module or unlinked crate `sol`" | Create proper module structure: `mod solidity { use alloy_sol_types::sol; sol! { /* solidity types */ } }` at module level, not inline with other code |
| ABI Method Ambiguity | "multiple applicable items in scope" | Use qualified syntax: `<Type as SolValue>::abi_decode(...)` |
| Missing Trait | "no function or associated item named 'method' found" | Import required traits, e.g., `use std::str::FromStr;` for `from_str` methods |
| String Capacity Overflow | "capacity overflow" or "panicked at alloc/src/slice.rs" | NEVER use unbounded `string.repeat(n)` without checks. Always limit max size: `"0".repeat(std::cmp::min(padding, 100))` and add bounds checking before calculating padding |
| Numeric Formatting | Invalid token decimal formatting | Always check for edge cases: 1) Verify decimals is valid, 2) Add safety checks to prevent negative or massive padding values, 3) Remember token values can be 0 or very large |
| Module Structure | "failed to resolve: could not find X in module" | Create proper module structure: `mod solidity { use alloy_sol_macro::sol; sol! { /* solidity types */ } }` at module level, not inline with other code |
| TxKind Import Path | "failed to resolve: could not find `TxKind` in `eth`" | Use the correct import: `use alloy_primitives::{Address, TxKind, U256};` and then use `TxKind::Call(address)` directly instead of `alloy_rpc_types::eth::TxKind::Call` |

For more details on specific topics, refer to `/docs/custom-components.mdx` or https://docs.rs/wavs-wasi-chain/latest/wavs_wasi_chain/all.html#functions.

## Blockchain Interactions

For components that interact directly with blockchains you MUST add the following dependencies:

In workspace Cargo.toml, add:
```toml
alloy-primitives = "0.8.25"  # Core types (Address, U256)
alloy-provider = { version = "0.11.1", default-features = false, features = ["rpc-api"] }
alloy-rpc-types = "0.11.1"  # RPC definitions
alloy-network = "0.11.1"    # Network trait
```

Then in component Cargo.toml, use:
```toml
alloy-primitives = { workspace = true }
alloy-provider = { workspace = true }
alloy-rpc-types = { workspace = true }
alloy-network = { workspace = true }
```

### Example component imports

The following imports can be helpful when creating components that interact with ethereum.
CRITICAL: Ensure all methods and types used in your component are properly imported - missing imports will cause compilation errors

```rust
use alloy_network::Ethereum;                    // Ethereum network types
use alloy_primitives::{hex, Address, TxKind, U256};  // Common blockchain primitives
use alloy_provider::{Provider, RootProvider};   // Blockchain provider interfaces
use alloy_rpc_types::TransactionInput;          // RPC transaction types
use alloy_sol_types::{sol, SolCall, SolValue}; // Solidity type support
use anyhow::Result;                             // Error handling
use serde::{Deserialize, Serialize};            // Serialization support
use wavs_wasi_chain::decode_event_log_data;     // Event log decoding
use wavs_wasi_chain::ethereum::new_eth_provider; // Ethereum provider creation
use wstd::runtime::block_on;                    // Async runtime utilities

pub mod bindings;
use crate::bindings::host::get_eth_chain_config; // Chain config from wavs.toml
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// IMPORTANT: Ensure all methods and types used in your component are imported before using
// Missing imports will cause compilation errors

```

### Example component

```rust
use std::io::Read;

pub mod bindings; // bindings are auto-generated during the build process
use crate::bindings::host::get_eth_chain_config;

use alloy_network::{Ethereum};
use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_provider::{Provider, RootProvider};
use alloy_rpc_types::TransactionInput;
use alloy_sol_types::{sol, SolCall, SolType, SolValue};
use wavs_wasi_chain::ethereum::new_eth_provider;

sol! {
    interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
    }
}

pub async fn query_nft_ownership(address: Address, nft_contract: Address) -> Result<bool, String> {
    // IMPORTANT: get_eth_chain_config returns Option, not Result - use ok_or_else, not map_err
    let chain_config = get_eth_chain_config("mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string())?;
    let provider: RootProvider<Ethereum> =
        new_eth_provider::<Ethereum>(chain_config.http_endpoint.unwrap());

    let balance_call = IERC721::balanceOfCall { owner: address };
    let tx = alloy_rpc_types::eth::TransactionRequest {
        to: Some(TxKind::Call(nft_contract)),
        input: TransactionInput { input: Some(balance_call.abi_encode().into()), data: None },
        ..Default::default()
    };

    let result = provider.call(&tx).await.map_err(|e| e.to_string())?;
    let balance: U256 = U256::from_be_slice(&result);
    Ok(balance > U256::ZERO)
}
```

CRITICAL: All blockchain interactions must use async functions with `block_on`. Never hardcode RPC endpoints - always use chain configuration from wavs.toml.
CRITICAL: Never implement methods on foreign types. Use built-in methods:
  - For addresses: `Address::from_slice()` or `address_str.parse::<Address>()`
  - For RPC calls: `provider.call(&tx)` (single parameter only)


