#!/bin/sh

# Set the configuration directory
CONFIG_DIR="/root/.bitsongd/config"

# Define the binary name (assumed to be in the Docker image's PATH)
BIND="/usr/bin/bitsongd"

# Define the test-keys file path (optional, can be used if external keys are provided)
TEST_KEYS_PATH="/root/test-keys"

# Define the gas token denom
GAS_TOKEN="ubtsg"

# Define variables for val1 node
VAL1HOME="/root/.bitsongd"
VAL1_API_PORT=1317
VAL1_GRPC_PORT=9090
VAL1_GRPC_WEB_PORT=9091
VAL1_PROXY_APP_PORT=26658
VAL1_RPC_PORT=26657
VAL1_PPROF_PORT=6060
VAL1_P2P_PORT=26656
CHAINID="sub-1"
defaultCoins="1000000000ubtsg"  # Default balance for genesis account
delegate="100000000ubtsg"       # Amount to delegate for validator

# Validate required environment variables
# Allow BIND and CHAINID to be overridden by environment variables if set
BIND=${BIND_ENV:-$BIND}
CHAINID=${CHAINID_ENV:-$CHAINID}

if [ -z "$VAL1HOME" ] || [ -z "$VAL1_PROXY_APP_PORT" ] || [ -z "$VAL1_RPC_PORT" ] || [ -z "$VAL1_P2P_PORT" ] || [ -z "$VAL1_API_PORT" ] || [ -z "$VAL1_GRPC_PORT" ] || [ -z "$VAL1_GRPC_WEB_PORT" ] || [ -z "$BIND" ] || [ -z "$CHAINID" ] || [ -z "$defaultCoins" ] || [ -z "$delegate" ]; then
  echo "Error: Missing required variables (VAL1HOME, VAL1_PROXY_APP_PORT, VAL1_RPC_PORT, VAL1_P2P_PORT, VAL1_API_PORT, VAL1_GRPC_PORT, VAL1_GRPC_WEB_PORT, BIND, CHAINID, defaultCoins, delegate)"
  exit 1
fi

# Ensure jq and other dependencies are available
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is not installed in the Docker image"
  exit 1
fi

# Initialize the node if not already initialized
if [ ! -d "$VAL1HOME/config" ]; then
  echo "Initializing node at $VAL1HOME with chain ID $CHAINID..."
  $BIND --home "$VAL1HOME" init "$CHAINID" --chain-id "$CHAINID"
  if [ $? -ne 0 ]; then
    echo "Error: Node initialization failed"
    exit 1
  fi
fi

# Modify val1 genesis with testing params
jq ".app_state.crisis.constant_fee.denom = \"$GAS_TOKEN\" |
      .app_state.staking.params.bond_denom = \"$GAS_TOKEN\" |
      .app_state.mint.params.blocks_per_year = \"20000000\" |
      .app_state.mint.params.mint_denom = \"$GAS_TOKEN\" |
      .app_state.merkledrop.params.creation_fee.denom = \"$GAS_TOKEN\" |
      .app_state.gov.voting_params.voting_period = \"24s\" |
      .app_state.gov.params.min_deposit[0].denom = \"$GAS_TOKEN\" |
      .app_state.gov.params.voting_period = \"15s\" |
      .app_state.slashing.params.signed_blocks_window = \"10\" |
      .app_state.slashing.params.min_signed_per_window = \"1.000000000000000000\" |
      .app_state.gov.params.expedited_voting_period = \"8s\" |
      .app_state.fantoken.params.burn_fee.denom = \"$GAS_TOKEN\" |
      .app_state.fantoken.params.issue_fee.denom = \"$GAS_TOKEN\" |
      .app_state.fantoken.params.mint_fee.denom = \"$GAS_TOKEN\"" "$VAL1HOME/config/genesis.json" > "$VAL1HOME/config/tmp.json"

# Check if jq command was successful
if [ $? -ne 0 ]; then
  echo "Error: Failed to modify genesis.json"
  exit 1
fi

# Move temporary genesis file to replace the original
mv "$VAL1HOME/config/tmp.json" "$VAL1HOME/config/genesis.json"
if [ $? -ne 0 ]; then
  echo "Error: Failed to replace genesis.json"
  exit 1
fi

# Modify app & config.toml
# config.toml
sed -i.bak -e "s|^proxy_app *=.*|proxy_app = \"tcp://127.0.0.1:$VAL1_PROXY_APP_PORT\"|" "$VAL1HOME/config/config.toml" &&
sed -i.bak "/^\[rpc\]/,/^\[/ s|laddr *=.*|laddr = \"tcp://127.0.0.1:$VAL1_RPC_PORT\"|" "$VAL1HOME/config/config.toml" &&
sed -i.bak "/^\[rpc\]/,/^\[/ s|address *=.*|address = \"tcp://127.0.0.1:$VAL1_RPC_PORT\"|" "$VAL1HOME/config/config.toml" &&
sed -i.bak "/^\[p2p\]/,/^\[/ s|laddr *=.*|laddr = \"tcp://0.0.0.0:$VAL1_P2P_PORT\"|" "$VAL1HOME/config/config.toml" &&
sed -i.bak -e "s|^grpc_laddr *=.*|grpc_laddr = \"\"|" "$VAL1HOME/config/config.toml" &&
# app.toml
sed -i.bak "/^\[api\]/,/^\[/ s|minimum-gas-prices *=.*|minimum-gas-prices = \"0.0$GAS_TOKEN\"|" "$VAL1HOME/config/app.toml" &&
sed -i.bak "/^\[api\]/,/^\[/ s|address *=.*|address = \"tcp://0.0.0.0:$VAL1_API_PORT\"|" "$VAL1HOME/config/app.toml" &&
sed -i.bak "/^\[grpc\]/,/^\[/ s|address *=.*|address = \"localhost:$VAL1_GRPC_PORT\"|" "$VAL1HOME/config/app.toml" &&
sed -i.bak "/^\[grpc-web\]/,/^\[/ s|address *=.*|address = \"localhost:$VAL1_GRPC_WEB_PORT\"|" "$VAL1HOME/config/app.toml"

# Check if sed commands were successful
if [ $? -ne 0 ]; then
  echo "Error: Failed to modify config.toml or app.toml"
  exit 1
fi

# Setup test keys
#  todo: import and generate keys based on json file generated with this format
# {
#     "members": [
#         {
#             "cosmos": {
#                 "ed12259": {},
#                 "secp256k1": {}
#             },
#             "eth": {
#                 "bls12": {}
#             }
#         }
#     ]
# }
# Create a validator key if it doesn't exist
if [ ! -f "$VAL1HOME/test-keys/val.json" ]; then
  echo "Creating validator key..."
  yes | $BIND --home "$VAL1HOME" keys add validator1 --output json > "$VAL1HOME/test-keys/val.json" 2>&1
  if [ $? -ne 0 ]; then
    echo "Error: Failed to add validator1 key"
    exit 1
  fi
fi

# Add genesis account
$BIND --home "$VAL1HOME" genesis add-genesis-account "$($BIND --home "$VAL1HOME" keys show validator1 -a)" "$defaultCoins"
if [ $? -ne 0 ]; then
  echo "Error: Failed to add genesis account for validator1"
  exit 1
fi

# Generate & collect gentx
$BIND --home "$VAL1HOME" genesis gentx validator1 "$delegate" --chain-id "$CHAINID"
if [ $? -ne 0 ]; then
  echo "Error: Failed to generate gentx for validator1"
  exit 1
fi

sleep 1

$BIND --home "$VAL1HOME" genesis collect-gentxs
if [ $? -ne 0 ]; then
  echo "Error: Failed to collect gentxs"
  exit 1
fi

sleep 1

# Start the Cosmos node
echo "Starting Cosmos node..."
exec $BIND --home "$VAL1HOME" start --log_level info