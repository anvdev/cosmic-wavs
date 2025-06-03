#!/bin/bash

set -e

FORK_RPC_URL=${FORK_RPC_URL:-"https://ethereum-holesky-rpc.publicnode.com"}
DEPLOY_ENV=$(sh ./script/get-deploy-status.sh)


# echo "Starting all local services using Rust library..."

# # Use the Rust implementation to start all local services
# cd script/cw-orch-wavs
# cargo run --bin wavs start-all ${FORK_RPC_URL:+--fork-rpc-url "$FORK_RPC_URL"}
# cd ../..

## == Start watcher ==
rm $LOG_FILE 2> /dev/null || true


## == Base Anvil Testnet Fork ==
if [ "$DEPLOY_ENV" = "TESTNET" ]; then
  echo "Running in testnet mode, nothing to do"
  exit 0
fi

if [ "$DEPLOY_ENV" = "LOCAL" ]; then
  anvil --fork-url ${FORK_RPC_URL} --port ${PORT} &
  anvil_pid=$!
  trap "kill -9 $anvil_pid && echo -e '\nKilled anvil'" EXIT
  while ! cast block-number --rpc-url http://localhost:${PORT} > /dev/null 2>&1
  do
    sleep 0.25
  done

  FILES="-f docker-compose.yml -f telemetry/docker-compose.yml"
  docker compose ${FILES} up --force-recreate -d
  trap "docker compose ${FILES} down --remove-orphans && docker kill wavs-1 wavs-aggregator-1 > /dev/null 2>&1 && echo -e '\nKilled IPFS + Local WARG, and wavs instances'" EXIT

  echo "Started..."
  wait
fi
