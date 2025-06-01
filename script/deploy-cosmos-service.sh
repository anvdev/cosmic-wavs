#!/bin/bash

set -e

: '''
# Deploy Cosmos WAVS Service using the Rust library
# This script now calls the Rust implementation for better maintainability

Usage:
    sh ./script/deploy-cosmos-service.sh [--start]

Environment Variables:
- COSMOS_RPC_URL: Cosmos chain RPC endpoint (default: http://localhost:26657)
- COSMOS_CHAIN_ID: Cosmos chain ID (default: sub-1)
- TRIGGER_EVENT: Cosmos event type to listen for (default: cw-infusion)
- COMPONENT_FILENAME: WASM component file name
- WAVS_CONTROLLER_ADDRESS: Cosmos address for WAVS controller
- WAVS_CONTROLLER_MNEMONIC: Mnemonic for WAVS controller
'''

# == Defaults ==
COSMOS_RPC_URL=${COSMOS_RPC_URL:-"http://localhost:26657"}
COSMOS_CHAIN_ID=${COSMOS_CHAIN_ID:-"sub-1"}
TRIGGER_EVENT=${TRIGGER_EVENT:-"cw-infusion"}
COMPONENT_FILENAME=${COMPONENT_FILENAME:-"cosmic-wavs-demo-infusion.wasm"}

# Determine if we should start the service
START_FLAG=""
if [ "$1" = "--start" ]; then
    START_FLAG="--start"
fi

echo "Deploying Cosmos WAVS service using Rust library..."

# Change to the cw-orch-wavs directory and run the Rust command
cd script/cw-orch-wavs
cargo run --bin wavs deploy-cosmos \
    --component "$COMPONENT_FILENAME" \
    --rpc-url "$COSMOS_RPC_URL" \
    --chain-id "$COSMOS_CHAIN_ID" \
    --trigger-event "$TRIGGER_EVENT" \
    $START_FLAG
cd ../..