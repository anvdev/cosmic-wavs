# USDT Balance Checker Component

## Overview

The USDT Balance Checker component retrieves the Tether (USDT) token balance for a given Ethereum wallet address.

## What It Does

- Accepts an Ethereum wallet address as input
- Connects to the Ethereum mainnet
- Queries the USDT token contract for the wallet's balance
- Returns a structured response with both raw and formatted balance

## Key Features

- Direct blockchain interaction using Ethereum RPC
- ERC-20 standard compatibility for token balance checking
- Proper decimal formatting based on the token's decimals
- Robust error handling for invalid addresses

## Input Format

The component expects an Ethereum wallet address as a string:

```
"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
```

## Output Format

```json
{
  "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
  "balance_raw": "15000000",
  "balance_formatted": "15.0",
  "usdt_contract": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
  "timestamp": "1713993600"
}
```

## WASI Execution Command

```bash
# Use this command to test the component locally:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"`
export COMPONENT_FILENAME=usdt_balance_checker.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

Note: Replace the wallet address with a valid Ethereum address you want to check.