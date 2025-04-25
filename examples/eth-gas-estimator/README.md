# ETH Gas Estimator Component

## Overview

The ETH Gas Estimator component provides real-time Ethereum gas price estimates with multiple speed tiers for optimal transaction fee selection.

## What It Does

- Fetches current gas prices from the Blocknative Gas API (no API key required)
- Processes data to provide three speed tiers: slow, average, and fast
- Includes estimated confirmation times for each tier
- Returns formatted gas prices in gwei

## Key Features

- No API key required - uses public APIs
- Multiple speed tiers with confidence levels:
  - Fast (99% confidence): Quick confirmations in 1-3 minutes
  - Average (80% confidence): Standard confirmations in 5-10 minutes
  - Slow (60% confidence): Economical confirmations in 10-15 minutes
- Proper error handling for network issues
- Simple interface - no input parameters required

## Input Format

The component doesn't require any specific input parameters:

```
// Empty input is fine
""
```

## Output Format

```json
{
  "slow": {
    "price": "25.5",
    "time_minutes": "10-15"
  },
  "average": {
    "price": "32.2",
    "time_minutes": "5-10"
  },
  "fast": {
    "price": "45.8", 
    "time_minutes": "1-3"
  },
  "timestamp": "1713993600"
}
```

## WASI Execution Command

```bash
# Command for local testing
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" ""`
export COMPONENT_FILENAME=eth_gas_estimator.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## Usage Examples

- **DeFi Applications**: Set optimal gas prices for automated transactions
- **NFT Minting**: Ensure timely transaction processing during high-demand periods
- **DAO Governance**: Help users estimate costs for proposal submissions
- **Wallet Integration**: Provide users with gas price recommendations

## Implementation Notes

- Uses Blocknative's confidence-based gas estimation for reliable predictions
- Falls back to available pricing tiers if exact confidence levels aren't found
- Updates in real-time with each request

# ETH Gas Estimator Component Plan

## Overview
This component will query and return Ethereum gas price estimates for different transaction speed tiers.

## Component Structure
The component will:
1. Query Ethereum gas prices from a public API
2. Process the data to provide multiple speed tiers (slow, average, fast)
3. Return a formatted response with gas prices in gwei and estimated time

## Implementation Details

### API Selection
We'll use the Blocknative Gas API which provides reliable gas price estimates without requiring an API key.

### Data Processing
- Parse API response
- Format gas prices in gwei
- Structure data for multiple speed tiers
- Include estimated confirmation times

### Response Structure
```json
{
  "slow": {
    "price": "25.5",
    "time_minutes": "10-15"
  },
  "average": {
    "price": "32.2",
    "time_minutes": "5-10"
  },
  "fast": {
    "price": "45.8", 
    "time_minutes": "1-3"
  },
  "timestamp": "1682531234"
}
```

## Code Structure
- Define Solidity input type (getGasEstimates)
- Create API request
- Parse API response
- Format gas price output
- Return well-structured response

## Validation Checklist
- ✅ Component implements Guest trait
- ✅ Component exports correctly
- ✅ Properly handles TriggerAction and TriggerData
- ✅ Properly decodes function calls
- ✅ Avoids String::from_utf8 on ABI data
- ✅ All API structures derive Clone
- ✅ Clones data before use
- ✅ Avoids moving out of collections
- ✅ Uses ok_or_else() for Option types
- ✅ Uses map_err() for Result types
- ✅ Provides descriptive error messages
- ✅ Includes all required traits and types
- ✅ Uses correct import paths
- ✅ Properly imports SolCall for encoding
- ✅ Uses proper sol! macro with correct syntax
- ✅ Correctly defines Solidity types in solidity module
- ✅ No hardcoded API keys or secrets
- ✅ Uses environment variables for sensitive data
- ✅ Uses workspace dependencies correctly
- ✅ Properly imports sol macro
- ✅ Uses solidity module correctly
- ✅ Handles numeric conversions safely
- ✅ Uses .to_string() for all string literals in struct initialization
- ✅ Uses block_on for async functions
- ✅ Uses fetch_json with correct headers
- ✅ Handles API responses correctly
