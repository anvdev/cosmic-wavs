#!/bin/sh
set -o errexit -o nounset -o pipefail

# Docker entrypoint script for a Cosmos node using bitsongd
echo "Current directory: $(pwd)"
echo "Directory contents: $(ls -la)"

# Define directories
HOME="/bitsong"
CONFIG_SRC="$HOME/.defaults"
VAL1HOME="$HOME/.bitsongd"
CONFIG_DIR="$VAL1HOME/config"

# Define default environment variables
PASSWORD=${PASSWORD:-1234567890}
STAKE=${STAKE_TOKEN:-ubtsg}
FEE=${FEE_TOKEN:-uthiol}
CHAIN_ID=${CHAIN_ID:-sub-2}
MONIKER=${MONIKER:-node001}
KEYRING="--keyring-backend test"
defaultCoins="1000000000$STAKE,1000000000$FEE,5000000000uusd"
delegate="100000000$STAKE"

# Define CLI alias
BSD="bitsongd"

# Remove existing home directory to ensure clean initialization
rm -rf "$VAL1HOME"

# Initialize the node with chain-id and moniker
echo "Initializing node with chain-id $CHAIN_ID and moniker $MONIKER..."
$BSD init "$MONIKER" --chain-id "$CHAIN_ID" --home "$VAL1HOME" -o
if [ $? -ne 0 ]; then
  echo "Error: Failed to initialize node"
  exit 1
fi

# Ensure CONFIG_DIR exists
mkdir -p "$CONFIG_DIR"

# Move configuration files from CONFIG_SRC to CONFIG_DIR if they exist
echo "Checking for configuration files in $CONFIG_SRC..."
for file in genesis.json priv_validator_key.json node_key.json app.toml config.toml; do
  if [ -f "$CONFIG_SRC/$file" ]; then
    echo "Moving $file from $CONFIG_SRC to $CONFIG_DIR..."
    mv "$CONFIG_SRC/$file" "$CONFIG_DIR/$file"
  else
    echo "Warning: $file not found in $CONFIG_SRC, using default initialized file."
  fi
done

# Verify moved files
echo "Current directory: $(pwd)"
echo "Contents of $CONFIG_DIR:"
ls -la "$CONFIG_DIR"

# Set keyring backend
echo "Configuring keyring backend..."
$BSD config keyring-backend test --home "$VAL1HOME"

# Create validator key
echo "Creating validator key..."
echo "$PASSWORD" | $BSD keys add validator --home "$VAL1HOME" --keyring-backend test
if [ $? -ne 0 ]; then
  echo "Error: Failed to create validator key"
  exit 1
fi

# Generate and collect gentx
echo "Adding genesis account and creating gentx for validator..."
validator_address=$($BSD keys show validator -a --home "$VAL1HOME" --keyring-backend test)
$BSD genesis add-genesis-account "$validator_address" "$defaultCoins" --home "$VAL1HOME"
if [ $? -ne 0 ]; then
  echo "Error: Failed to add genesis account"
  exit 1
fi

echo "Generating gentx for validator..."
$BSD genesis gentx validator "$delegate" \
  --home="$VAL1HOME" \
  --chain-id="$CHAIN_ID" \
  --moniker="validator" \
  --commission-max-change-rate=0.01 \
  --commission-max-rate=1.0 \
  --commission-rate=0.07 \
  --keyring-backend test
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

# Verify final genesis file
echo "Contents of final genesis.json:"
cat "$CONFIG_DIR/genesis.json"

# Start the Cosmos node
echo "Starting Cosmos node..."
exec $BSD start --log_level info --home "$VAL1HOME"