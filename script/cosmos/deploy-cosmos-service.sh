#!/bin/bash

set -e

: '''
# Deploy Cosmos WAVS Service using the Rust library
# This script now calls the Rust implementation for better maintainability

Usage:
    sh ./script/deploy-cosmos-service.sh [--start]

Environment Variables:
- NETWORK: local
 
'''
# == Defaults ==

# Determine if we should start the service
START_FLAG=""
if [ "$1" = "--start" ]; then
    START_FLAG="--start"
fi


echo "Deploying Cosmos WAVS demo using Cw-Orchestrator..."
# configure files for docker entrypoint
cp ./script/template/.env.example.cosmos script/cw-orch-wavs/.env
sh script/cosmos/setup-local-cosmos.sh


# Change to the cw-orch-wavs directory and run the Rust command
cd script/cw-orch-wavs

cargo run --bin $START_FLAG
cd ../..