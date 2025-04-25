# NFT Ownership Checker Component

## Overview

The NFT Ownership Checker component verifies whether a given Ethereum wallet owns NFTs from the Bored Ape Yacht Club (BAYC) collection.

## What It Does

- Accepts an Ethereum wallet address as input
- Queries the BAYC smart contract on Ethereum mainnet
- Determines if the wallet owns any BAYC NFTs and returns the total count
- Returns a structured response with ownership details

## Key Features

- Direct blockchain interaction using Ethereum RPC
- ERC-721 standard compatibility for NFT balance checking
- Robust error handling for invalid addresses
- Proper ABI encoding/decoding for blockchain compatibility

## Input Format

The component expects an Ethereum wallet address as a string:

```
"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
```

## Output Format

```json
{
  "wallet": "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
  "owns_nft": true,
  "balance": "2",
  "nft_contract": "0xbd3531da5cf5857e7cfaa92426877b022e612cf8",
  "contract_name": "BAYC",
  "timestamp": "1713993600"
}
```

## WASI Execution Command

```bash
# Use this command to test the component locally:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"`
export COMPONENT_FILENAME=nft_ownership_checker.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

Note: Replace the wallet address with a valid Ethereum address you want to check.