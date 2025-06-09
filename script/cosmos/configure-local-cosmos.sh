#!/bin/sh
 
# Define default environment variables
PASSWORD=${PASSWORD:-1234567890}
STAKE=${STAKE_TOKEN:-ubtsg}
CHAIN_ID=${CHAIN_ID:-sub-2}
MONIKER=${MONIKER:-node001}
TIMEOUT_COMMIT=${TIMEOUT_COMMIT:-5s}
BLOCK_GAS_LIMIT=${GAS_LIMIT:-10000000}
UNSAFE_CORS=${UNSAFE_CORS:-}

# Define paths and binary specific for cosmos node docker image
VAL1HOME="$HOME/.bitsongd"


CONFIG_DEFAULTS=".cosmos/defaults"
CONFIG_DEST=".cosmos/data"
GENESIS_FILE="$CONFIG_DEST/genesis.json"
APP_TOML="$CONFIG_DEST/app.toml"
CONFIG_TOML="$CONFIG_DEST/config.toml"
CLIENT_TOML="$VAL1HOME/client.toml"
# BALANCES="$VAL1HOME/default-balances"

# Define port configurations
VAL1_API_PORT=1317
VAL1_GRPC_PORT=8016
VAL1_GRPC_WEB_PORT=9016
VAL1_PROXY_APP_PORT=26658
VAL1_RPC_PORT=26657
# VAL1_PPROF_PORT=6060
VAL1_P2P_PORT=26656
defaultCoins="1000000000$STAKE"
delegate="100000000$STAKE"

# Validate required environment variables
if [ -z "$VAL1HOME" ] || [ -z "$VAL1_PROXY_APP_PORT" ] || [ -z "$VAL1_RPC_PORT" ] || [ -z "$VAL1_P2P_PORT" ] || [ -z "$VAL1_API_PORT" ] || [ -z "$VAL1_GRPC_PORT" ] || [ -z "$VAL1_GRPC_WEB_PORT" ] ||  [ -z "$CHAIN_ID" ] || [ -z "$defaultCoins" ] || [ -z "$delegate" ]; then
  echo "Error: Missing required variables (VAL1HOME, VAL1_PROXY_APP_PORT, VAL1_RPC_PORT, VAL1_P2P_PORT, VAL1_API_PORT, VAL1_GRPC_PORT, VAL1_GRPC_WEB_PORT, BIND, CHAIN_ID, defaultCoins, delegate)"
  exit 1
fi

# Ensure jq is available
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is not installed"
  exit 1
fi


## create new paths from defaults to modify
mkdir -p "$CONFIG_DEST"
cp "$CONFIG_DEFAULTS/genesis.json" "$CONFIG_DEST/genesis.json"
cp "$CONFIG_DEFAULTS/app.toml" "$CONFIG_DEST/app.toml"
cp "$CONFIG_DEFAULTS/app.toml" "$CONFIG_DEST/config.toml"
cp "$CONFIG_DEFAULTS/client.toml" "$CONFIG_DEST/client.toml"

## modify the default file templates via jq
jq ".app_state.crisis.constant_fee.denom = \"$STAKE\" |
    .app_state.staking.params.bond_denom = \"$STAKE\" |
    .app_state.mint.params.blocks_per_year = \"20000000\" |
    .app_state.mint.params.mint_denom = \"$STAKE\" |
    .app_state.merkledrop.params.creation_fee.denom = \"$STAKE\" |
    .app_state.gov.voting_params.voting_period = \"24s\" |
    .app_state.gov.params.min_deposit[0].denom = \"$STAKE\" |
    .app_state.gov.params.voting_period = \"15s\" |
    .app_state.slashing.params.signed_blocks_window = \"10\" |
    .app_state.slashing.params.min_signed_per_window = \"1.000000000000000000\" |
    .app_state.gov.params.expedited_voting_period = \"8s\" |
    .app_state.fantoken.params.burn_fee.denom = \"$STAKE\" |
    .app_state.fantoken.params.issue_fee.denom = \"$STAKE\" |
    .app_state.fantoken.params.mint_fee.denom = \"$STAKE\" |
    .consensus_params.block.max_gas = \"$BLOCK_GAS_LIMIT\" |
    .consensus_params.block.time_iota_ms = \"10\"" "$GENESIS_FILE" > "$CONFIG_DEST/tmp.json"

if [ $? -ne 0 ]; then
  echo "Error: Failed to modify genesis.json"
  exit 1
fi

mv "$CONFIG_DEST/tmp.json" "$GENESIS_FILE"
if [ $? -ne 0 ]; then
  echo "Error: Failed to replace genesis.json"
  exit 1
fi

# Modify config.toml
sed -i.bak -e "s|^proxy_app *=.*|proxy_app = \"tcp://127.0.0.1:$VAL1_PROXY_APP_PORT\"|" "$CONFIG_DEST/config.toml" &&
sed -i.bak "/^\[rpc\]/,/^\[/ s|laddr *=.*|laddr = \"tcp://127.0.0.1:$VAL1_RPC_PORT\"|" "$CONFIG_DEST/config.toml" &&
sed -i.bak "/^\[rpc\]/,/^\[/ s|address *=.*|address = \"tcp://127.0.0.1:$VAL1_RPC_PORT\"|" "$CONFIG_DEST/config.toml" &&
sed -i.bak "/^\[p2p\]/,/^\[/ s|laddr *=.*|laddr = \"tcp://0.0.0.0:$VAL1_P2P_PORT\"|" "$CONFIG_DEST/config.toml" &&
sed -i.bak -e "s|^grpc_laddr *=.*|grpc_laddr = \"\"|" "$CONFIG_DEST/config.toml" &&
sed -i.bak -e "s|timeout_commit = \"5s\"|timeout_commit = \"$TIMEOUT_COMMIT\"|" "$CONFIG_DEST/config.toml"

if [ $? -ne 0 ]; then
  echo "Error: Failed to modify config.toml"
  exit 1
fi

# Modify app.toml
sed -i.bak "/^\[api\]/,/^\[/ s|enable = false|enable = true|" "$CONFIG_DEST/app.toml" &&
sed -i.bak "/^\[api\]/,/^\[/ s|minimum-gas-prices *=.*|minimum-gas-prices = \"0.0$STAKE\"|" "$CONFIG_DEST/app.toml" &&
sed -i.bak "/^\[api\]/,/^\[/ s|address *=.*|address = \"tcp://0.0.0.0:$VAL1_API_PORT\"|" "$CONFIG_DEST/app.toml" &&
sed -i.bak "/^\[grpc\]/,/^\[/ s|address *=.*|address = \"localhost:$VAL1_GRPC_PORT\"|" "$CONFIG_DEST/app.toml" &&
sed -i.bak "/^\[grpc-web\]/,/^\[/ s|address *=.*|address = \"localhost:$VAL1_GRPC_WEB_PORT\"|" "$CONFIG_DEST/app.toml"

if [ $? -ne 0 ]; then
  echo "Error: Failed to modify app.toml"
  exit 1
fi

# Apply CORS settings if enabled
if [ -n "$UNSAFE_CORS" ]; then
  echo "Unsafe CORS set... updating app.toml and config.toml"
  sed -i.bak -e "s|enabled-unsafe-cors = false|enabled-unsafe-cors = true|" "$CONFIG_DEST/app.toml"
  sed -i.bak -e "s|cors_allowed_origins = \[\]|cors_allowed_origins = [\"*\"]|" "$CONFIG_DEST/config.toml"
fi
 