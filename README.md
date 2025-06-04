# Cosmic-WAVS

<div align="center">

[![Bannger](/imgs/readme-banner.png)](https://youtu.be/jyl7kbie41w)

</div>


<div align="center">

> [!WARNING]
> **Experimental Use Only**: This template is designed for experimentation. Results may vary. This template is not intended for production use. Use at your own risk and for experimental purposes only.
Repo template: https://github.com/Lay3rLabs/wavs-foundry-template

</div>


## Cw-Orch-Wavs
All-in-one scripting library for deploying & testing
 
## Setup


### System Requirements

<details>
<summary>Core (Docker, Compose, Make, JQ, Node v21+, Foundry)</summary>

## Ubuntu Base
- **Linux**: `sudo apt update && sudo apt install build-essential`

### Docker

If prompted, remove containerd with `sudo apt remove containerd.io`.

- **MacOS**: `brew install --cask docker`
- **Linux**: `sudo apt -y install docker.io`
- **Windows WSL**: [docker desktop wsl](https://docs.docker.com/desktop/wsl/#turn-on-docker-desktop-wsl-2) & `sudo chmod 666 /var/run/docker.sock`
- [Docker Documentation](https://docs.docker.com/get-started/get-docker/)

### Docker Compose
- **MacOS**: Already installed with Docker installer
> `sudo apt remove docker-compose-plugin` may be required if you get a `dpkg` error
- **Linux + Windows WSL**: `sudo apt-get install docker-compose-v2`
- [Compose Documentation](https://docs.docker.com/compose/)

### Make
- **MacOS**: `brew install make`
- **Linux + Windows WSL**: `sudo apt -y install make`
- [Make Documentation](https://www.gnu.org/software/make/manual/make.html)

### JQ
- **MacOS**: `brew install jq`
- **Linux + Windows WSL**: `sudo apt -y install jq`
- [JQ Documentation](https://jqlang.org/download/)

### Node.js
- **Required Version**: v21+
- [Installation via NVM](https://github.com/nvm-sh/nvm?tab=readme-ov-file#installing-and-updating)

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash
nvm install --lts
```

### Foundry
```bash docci-ignore
curl -L https://foundry.paradigm.xyz | bash && $HOME/.foundry/bin/foundryup
```

</details>

<details>

<summary>Rust v1.85+</summary>

### Rust Installation

```bash docci-ignore
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup toolchain install stable
rustup target add wasm32-wasip2
```

### Upgrade Rust

```bash docci-ignore
# Remove old targets if present
rustup target remove wasm32-wasi || true
rustup target remove wasm32-wasip1 || true

# Update and add required target
rustup update stable
rustup target add wasm32-wasip2
```

</details>

<details>
<summary>Cargo Components</summary>

### Install Cargo Components

On Ubuntu LTS, if you later encounter errors like:

```bash
wkg: /lib/x86_64-linux-gnu/libm.so.6: version `GLIBC_2.38' not found (required by wkg)
wkg: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by wkg)
```

If GLIB is out of date. Consider updating your system using:
```bash
sudo do-release-upgrade
```


```bash docci-ignore
# Install required cargo components
# https://github.com/bytecodealliance/cargo-component#installation
cargo install cargo-binstall
cargo binstall cargo-component wasm-tools warg-cli wkg --locked --no-confirm --force

# Configure default registry
# Found at: $HOME/.config/wasm-pkg/config.toml
wkg config --default-registry wa.dev
```

</details>

## Create Project

```bash docci-ignore
# if foundry is not installed:
# `curl -L https://foundry.paradigm.xyz | bash && $HOME/.foundry/bin/foundryup`
forge init --template Lay3rLabs/wavs-foundry-template my-wavs --branch main
```

> \[!TIP]
> Run `make help` to see all available commands and environment variable overrides.

### Solidity

Install the required packages to build the Solidity contracts. This project supports both [submodules](./.gitmodules) and [npm packages](./package.json).

```bash
# Install packages (npm & submodules)
make setup

# Build the contracts
forge build

# Run the solidity tests
forge test
```

## Build WASI components

Now build the WASI components into the `compiled` output directory.

> \[!WARNING]
> If you get: `error: no registry configured for namespace "wavs"`
>
> run, `wkg config --default-registry wa.dev`

> \[!WARNING]
> If you get: `failed to find the 'wasm32-wasip1' target and 'rustup' is not available`
>
> `brew uninstall rust` & install it from <https://rustup.rs>

```bash
# Remove `WASI_BUILD_DIR` to build all components.
warg reset
WASI_BUILD_DIR=components/evm-price-oracle make wasi-build
```

## Testing the Price Feed Component Locally

How to test the component locally for business logic validation before on-chain deployment. An ID of 1 for the oracle component is Bitcoin.

```bash
COIN_MARKET_CAP_ID=1 make wasi-exec
```
 

Expected output:

```shell docci-ignore
input id: 1
resp_data: PriceFeedData {
    symbol: "BTC",
    timestamp: "2025-04-01T00:00:00.000Z",
    price: 82717.27035239758
}
INFO Fuel used: 653415

Result (hex encoded):
7b2273796d626f6c223a22425443222c2274696d657374616d70223a22323032352d30342d30315430303a34...

Result (utf8):
{"symbol":"BTC","timestamp":"2025-04-01T00:00:00.000Z","price":82717.27035239758}
 

### Testing 

 After successfully building your component, it's time to test it. The following command can be used to test your component logic without deploying WAVS. Make sure to replace the placeholders with the correct inputs.

```sh
# This is the input data passed to your component
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your parameter here"`
# The name of your compiled component
export COMPONENT_FILENAME=your_component_name.wasm
# If you are using an API key, make sure it is properly set in your .env file
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
# IMPORTANT: Claude can't run this command without system permission. It is always best for the user to run this command.
make wasi-exec
```

## Running WAVS locally

> If you are running on a Mac with an ARM chip, you will need to do the following:
> - Set up Rosetta: `softwareupdate --install-rosetta`
> - Enable Rosetta (Docker Desktop: Settings -> General -> enable "Use Rosetta for x86_64/amd64 emulation on Apple Silicon")
>
> Configure one of the following networking:
> - Docker Desktop: Settings -> Resources -> Network -> 'Enable Host Networking'
> - `brew install chipmk/tap/docker-mac-net-connect && sudo brew services start chipmk/tap/docker-mac-net-connect`

## Start Environment
 

Start an Ethereum node (anvil), cosmos node (cosmos-omnibus) the WAVS service, and deploy [eigenlayer](https://www.eigenlayer.xyz/) contracts to the local network.

### Enable Telemetry (optional)

Set Log Level:
  - Open the `.env` file.
  - Set the `log_level` variable for wavs to debug to ensure detailed logs are captured.

> \[!NOTE]
To see details on how to access both traces and metrics, please check out [Telemetry Documentation](telemetry/telemetry.md).

### Start the backend

```bash docci-background docci-delay-after=5
### Enable Telemetry (optional)

Set Log Level:
  - Open the `.env` file.
  - Set the `log_level` variable for wavs to debug to ensure detailed logs are captured.

> \[!NOTE]
To see details on how to access both traces and metrics, please check out [Telemetry Documentation](telemetry/telemetry.md).

### Start the backend

```bash docci-background docci-delay-after=5
# This must remain running in your terminal. Use another terminal to run other commands.
# You can stop the services with `ctrl+c`. Some MacOS terminals require pressing it twice.
cp .env.example .env

# update the .env for either LOCAL or TESTNET

# Starts anvil + IPFS, WARG, Jaeger, cosmos-node, and prometheus.
make start-all-local
```

## Create Deployer, upload Eigenlayer

```bash
# local: create deployer & auto fund. testnet: create & iterate check balance
sh ./script/create-deployer.sh

## Deploy Eigenlayer from Deployer
docker run --rm --network host --env-file .env -v ./.nodes:/root/.nodes ghcr.io/lay3rlabs/wavs-middleware:0.4.0-beta.6 deploy
```

## Deploy Service Contracts

**Key Concepts:**

*   **WavsServiceManager** - `SERVICE_MANAGER_ADDR` is the address of the Eigenlayer service manager contract. It was deployed in the previous step.
*   **Trigger Contract:** Any contract that emits events, then WAVS monitors. When a relevant event occurs, WAVS triggers the execution of your WebAssembly component.
*   **Submission Contract:** This contract is used by the AVS service operator to submit the results generated by the WAVS component on-chain.

 After WavsServiceManager has been deployed, you deploy the trigger and submission contracts which depends on the service manager. The service manager will verify that a submission is valid (from an authorized operator) before saving it to the blockchain. The trigger contract is any arbitrary contract that emits some event that WAVS will watch for.

# ## Multichain Deployment
Wavs supports having the trigger contract deployed on a different chain than the service submission contract on the L1 *(Ethereum for now because that is where Eigenlayer is deployed)*. 
If your looking to design your multichain wavs, ensure you replace the default deployment logic for the trigger contract with your trigger specific implementation. 


```bash docci-delay-per-cmd=2
export RPC_URL=`sh ./script/get-rpc.sh`
export DEPLOYER_PK=$(cat .nodes/deployer)
export SERVICE_MANAGER_ADDRESS=$(jq -r '.addresses.WavsServiceManager' .nodes/avs_deploy.json)

forge create SimpleSubmit --json --broadcast -r ${RPC_URL} --private-key "${DEPLOYER_PK}" --constructor-args "${SERVICE_MANAGER_ADDRESS}" > .docker/submit.json
export SERVICE_SUBMISSION_ADDR=`jq -r '.deployedTo' .docker/submit.json`

forge create SimpleTrigger --json --broadcast -r ${RPC_URL} --private-key "${DEPLOYER_PK}" > .docker/trigger.json
export SERVICE_TRIGGER_ADDR=`jq -r '.deployedTo' .docker/trigger.json`

# for cosmos chains, make use of cw-orch, connects to docker image 
# cargo bin deploy-trigger --network local 
```

## Deploy Service

Deploy the compiled component with the contract information from the previous steps. Review the [makefile](./Makefile) for more details and configuration options.`TRIGGER_EVENT` is the event that the trigger contract emits and WAVS watches for. By altering `SERVICE_TRIGGER_ADDR` you can watch events for contracts others have deployed.

```bash docci-delay-per-cmd=3
# ** Testnet Setup: https://wa.dev/account/credentials
#
# If you get errors:
# warg reset --registry http://127.0.0.1:8090

export COMPONENT_FILENAME=evm_price_oracle.wasm
export REGISTRY=`sh ./script/get-registry.sh`
export PKG_NAME="evmrustoracle"
export PKG_VERSION="0.1.0"
export PKG_NAMESPACE=`sh ./script/get-wasi-namespace.sh`

# Upload the component to the registry
# local or wa.dev depending on DEPLOY_ENV in .env
sh script/upload-to-wasi-registry.sh

# Testnet: set values (default: local if not set)
# export TRIGGER_CHAIN=holesky
# export SUBMIT_CHAIN=holesky

# Package not found with wa.dev? -- make sure it is public
export AGGREGATOR_URL=http://127.0.0.1:8001
REGISTRY=${REGISTRY} sh ./script/build_service.sh
```

## Upload to IPFS

```bash
# Upload service.json to IPFS
export SERVICE_FILE=.docker/service.json

# local: 127.0.0.1:5001
# testnet: https://app.pinata.cloud/. set PINATA_API_KEY to JWT token in .env
export ipfs_cid=`SERVICE_FILE=${SERVICE_FILE} make upload-to-ipfs`

# LOCAL: http://127.0.0.1:8080
# TESTNET: https://gateway.pinata.cloud/
export IPFS_GATEWAY="$(sh script/get-ipfs-gateway.sh)/ipfs/"

export IPFS_URI="ipfs://${ipfs_cid}"
curl "${IPFS_GATEWAY}${ipfs_cid}"

cast send ${SERVICE_MANAGER_ADDRESS} 'setServiceURI(string)' "${IPFS_URI}" -r ${RPC_URL} --private-key ${DEPLOYER_PK}
```

## Start Aggregator

```bash
sh ./script/create-aggregator.sh 1

IPFS_GATEWAY=${IPFS_GATEWAY} sh ./infra/aggregator-1/start.sh

wget -q --header="Content-Type: application/json" --post-data="{\"uri\": \"${IPFS_URI}\"}" ${AGGREGATOR_URL}/register-service -O -
```

## Start WAVS

```bash
sh ./script/create-operator.sh 1

IPFS_GATEWAY=${IPFS_GATEWAY} sh ./infra/wavs-1/start.sh

# Deploy the service JSON to WAVS so it now watches and submits.
# 'opt in' for WAVS to watch (this is before we register to Eigenlayer)
WAVS_ENDPOINT=http://127.0.0.1:8000 SERVICE_URL=${IPFS_URI} IPFS_GATEWAY=${IPFS_GATEWAY} make deploy-service
```

## Register service specific operator

Each service gets their own key path (hd_path). The first service starts at 1 and increments from there. Get the service ID

```bash
export SERVICE_ID=`curl -s http://localhost:8000/app | jq -r '.services[0].id'`
export HD_INDEX=`curl -s http://localhost:8000/service-key/${SERVICE_ID} | jq -rc '.secp256k1.hd_index'`

source infra/wavs-1/.env
export AVS_PRIVATE_KEY=`cast wallet private-key --mnemonic-path "$WAVS_SUBMISSION_MNEMONIC" --mnemonic-index ${HD_INDEX}`

# Register the operator with the WAVS service manager
export SERVICE_MANAGER_ADDRESS=`jq -r '.addresses.WavsServiceManager' .nodes/avs_deploy.json`
DELEGATION=0.001ether AVS_PRIVATE_KEY=${AVS_PRIVATE_KEY} make V=1 operator-register

# Verify registration
SERVICE_MANAGER_ADDRESS=${SERVICE_MANAGER_ADDRESS} make operator-list
```

## Trigger the Service
todo: implmeent description for triggering service via nft mint

Anyone can now call the [trigger contract](./src/contracts/WavsTrigger.sol) which emits the trigger event WAVS is watching for from the previous step. WAVS then calls the service and saves the result on-chain.

```bash
# Request BTC from CMC
export COIN_MARKET_CAP_ID=1
# Get the trigger address from previous Deploy forge script
export SERVICE_TRIGGER_ADDR=`make get-trigger-from-deploy`
# Execute on the trigger contract, WAVS will pick this up and submit the result
# on chain via the operators.

# uses FUNDED_KEY as the executor (local: anvil account)
source .env

forge script ./script/Trigger.s.sol ${SERVICE_TRIGGER_ADDR} ${COIN_MARKET_CAP_ID} --sig 'run(string,string)' --rpc-url ${RPC_URL} --broadcast
```

## Show the result

Query the latest submission contract id from the previous request made.

```bash docci-delay-per-cmd=2 docci-output-contains="1"
RPC_URL=${RPC_URL} make get-trigger
```

```bash docci-delay-per-cmd=2 docci-output-contains="BTC"
TRIGGER_ID=1 RPC_URL=${RPC_URL} make show-result
```

## Update Threshold

```bash docci-ignore
export ECDSA_CONTRACT=`cat .nodes/avs_deploy.json | jq -r '.addresses.stakeRegistry'`

TOTAL_WEIGHT=`cast call ${ECDSA_CONTRACT} "getLastCheckpointTotalWeight()(uint256)" --rpc-url ${RPC_URL} --json | jq -r '.[0]'`
TWO_THIRDS=`echo $((TOTAL_WEIGHT * 2 / 3))`

cast send ${ECDSA_CONTRACT} "updateStakeThreshold(uint256)" ${TWO_THIRDS} --rpc-url ${RPC_URL} --private-key ${FUNDED_KEY}

make operator-list
```

# Claude Code

To spin up a sandboxed instance of [Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) in a Docker container that only has access to this project's files, run the following command:

```bash docci-ignore
npm run claude-code
# or with no restrictions (--dangerously-skip-permissions)
npm run claude-code:unrestricted
```

```bash docci-delay-per-cmd=2 docci-output-contains="1"
RPC_URL=${RPC_URL} make get-trigger
```

```bash docci-delay-per-cmd=2 docci-output-contains="BTC"
TRIGGER_ID=1 RPC_URL=${RPC_URL} make show-result
```

## Update Threshold

```bash docci-ignore
export ECDSA_CONTRACT=`cat .nodes/avs_deploy.json | jq -r '.addresses.stakeRegistry'`

TOTAL_WEIGHT=`cast call ${ECDSA_CONTRACT} "getLastCheckpointTotalWeight()(uint256)" --rpc-url ${RPC_URL} --json | jq -r '.[0]'`
TWO_THIRDS=`echo $((TOTAL_WEIGHT * 2 / 3))`

cast send ${ECDSA_CONTRACT} "updateStakeThreshold(uint256)" ${TWO_THIRDS} --rpc-url ${RPC_URL} --private-key ${FUNDED_KEY}

make operator-list
```

# Claude Code

To spin up a sandboxed instance of [Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview) in a Docker container that only has access to this project's files, run the following command:

```bash docci-ignore
npm run claude-code
# or with no restrictions (--dangerously-skip-permissions)
npm run claude-code:unrestricted
```

