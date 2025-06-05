#!/bin/sh
set -o errexit -o nounset -o pipefail

# Docker entrypoint script for a Cosmos node using bitsongd
CURRENT_DIR=$(pwd)
echo $CURRENT_DIR
# Define paths
CONFIG_SRC="$HOME/.cosmos/data"  

CONFIG_DIR="$HOME/.bitsongd/config"
VAL1HOME="$HOME/.bitsongd"

# Define default environment variables
PASSWORD=${PASSWORD:-1234567890}
STAKE=${STAKE_TOKEN:-ubtsg}
FEE=${FEE_TOKEN:-ubtsg}
CHAIN_ID=${CHAIN_ID:-sub-2}  
MONIKER=${MONIKER:-node001}
KEYRING="--keyring-backend test"
defaultCoins="1000000000$STAKE"
delegate="100000000$STAKE"

# Define CLI alias
BSD="docker run --rm -it -v $HOME/.bitsongd:/root/.bitsongd bitsong:v0.12323 bitsongd"

# Ensure the configuration directory exists
mkdir -p "$CONFIG_DIR"



# Check if configuration files exist in CONFIG_SRC and copy them
if [ -f "$CONFIG_SRC/genesis.json" ] && [ -f "$CONFIG_SRC/app.toml" ] && [ -f "$CONFIG_SRC/config.toml" ]; then
  echo "Copying configuration files from $CONFIG_SRC to $CONFIG_DIR..."
  cp "$CONFIG_SRC/genesis.json" "$CONFIG_DIR/genesis.json"
  cp "$CONFIG_SRC/app.toml" "$CONFIG_DIR/app.toml"
  cp "$CONFIG_SRC/config.toml" "$CONFIG_DIR/config.toml"
else
  echo "Error: One or more configuration files (genesis.json, app.toml, config.toml) missing in $CONFIG_SRC"
  exit 1
fi

# Check if client.toml exists and copy it if present
if [ -f "$CONFIG_SRC/client.toml" ]; then
  echo "Copying client.toml from $CONFIG_SRC to $VAL1HOME..."
  cp "$CONFIG_SRC/client.toml" "$VAL1HOME/client.toml"
else
  echo "Warning: client.toml not found in $CONFIG_SRC, skipping"
fi

# Check if node is already initialized (i.e., validator key exists)
if ! $BSD keys show validator $KEYRING --home "$VAL1HOME" >/dev/null 2>&1; then
  # Initialize the node if not already initialized
  if [ ! -f "$CONFIG_DIR/config.toml" ] || [ ! -f "$CONFIG_DIR/genesis.json" ]; then
    echo "Initializing node with chain-id $CHAIN_ID and moniker $MONIKER..."
    $BSD init "$MONIKER" --chain-id "$CHAIN_ID" --home "$VAL1HOME"
    if [ $? -ne 0 ]; then
      echo "Error: Failed to initialize node"
      exit 1
    fi
  fi

  # Create validator key
  echo "Creating validator key..."
  mkdir -p "$VAL1HOME/keys"
  (echo "$PASSWORD"; echo "$PASSWORD") | $BSD keys add validator $KEYRING --home "$VAL1HOME" --output json > "$VAL1HOME/keys/val.json" 2>&1
  if [ $? -ne 0 ]; then
    echo "Error: Failed to add validator key"
    exit 1
  fi

  # Add genesis account
  echo "Adding genesis account for validator..."
  validator_address=$($BSD keys show validator "$KEYRING" --home "$VAL1HOME" -a)
  $BSD genesis add-genesis-account "$validator_address" "$defaultCoins,1000000000$FEE,5000000000uusd" --home "$VAL1HOME"
  if [ $? -ne 0 ]; then
    echo "Error: Failed to add genesis account for validator"
    exit 1
  fi

  # Generate and collect gentx
  echo "Generating gentx for validator..."
  (echo "$PASSWORD"; echo "$PASSWORD"; echo "$PASSWORD") | $BSD genesis gentx validator "$delegate" --chain-id "$CHAIN_ID" --amount="$delegate" $KEYRING --home "$VAL1HOME"
  if [ $? -ne 0 ]; then
    echo "Error: Failed to generate gentx for validator"
    exit 1
  fi

  echo "Collecting gentxs..."
  $BSD genesis collect-gentxs --home "$VAL1HOME"
  if [ $? -ne 0 ]; then
    echo "Error: Failed to collect gentxs"
    exit 1
  fi
else
  echo "Node already initialized, skipping key creation and genesis setup"
fi

# Start the Cosmos node
echo "Starting Cosmos node..."
exec $BSD start --log_level info --home "$VAL1HOME"