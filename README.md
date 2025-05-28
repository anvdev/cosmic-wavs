# Cosmic-WAVS

<div align="center">

[![Bannger](/imgs/readme-banner.png)](https://youtu.be/jyl7kbie41w)

</div>

 
<div align="center">

> [!WARNING]
> **Experimental Use Only**: This template is designed for experimentation. Results may vary. This template is not intended for production use. Use at your own risk and for experimental purposes only.
Repo template: https://github.com/Lay3rLabs/wavs-foundry-template

</div>


## Goals
- Track cosmwasm nft burn events emitted from Cosmos Chain
- Authentication action to perform via Wavs operator keys
- Broadcast authorized action to Cosmos Chain

## Design 

### wavs + x/smart-accounts  
 For this implemenetation, WAVS services will use authentication capabilities provided by the [x/smart-account module](https://github.com/permissionlessweb/go-bitsong/tree/main/x/smart-account) to perform on chain actions. This is implemented by 
registration of an smart-contract authenticator to a secp256k1 key account. This repo contains a few example of making use of this workflow. Our bls12-381 compatible account authentication example can be found here [btsg-wavs](https://github.com/permissionlessweb/bs-accounts/blob/cleanup/contracts/smart-accounts/btsg-wavs/src/contract.rs#L100), and is used to allow a set of operator for a given AVS instance authenticate actions for this account to perform.


### custom AVS logic
 
Here we design our AVS to perform custom logic. This demo has logic that filters any new burn event that has occured on the chain the cw-infusion contract is deployed on, in order to trigger its custom filtering workflow:
```rs
TriggerData::CosmosContractEvent(TriggerDataCosmosContractEvent {event,..}) => {
            // Extract event type and data from Cosmos event
            let event_type = Some(event.ty.clone());
            if let Some(et) = event_type.as_ref() {
                if et.as_str() == "wasm" {
                    // Look for burn action
                    if let Some(action_attr) = event.attributes.iter().find(|(k, _)| k == "action")
                    {
                               if action_attr.1 == "burn" {
                                /// custom logic...
                               }
                    }
                }
            }}
```

We can also implement custom logic, such as deterministic queries to determine any msgs that the AVS should perform:
```rs
 // 2.query contract the check if operators need to update assigned cw-infuser state
    let res: Vec<cw_infusions::wavs::WavsRecordResponse> = cosm_guery
        .contract_smart(
            &Address::new_cosmos_string(&cw_infuser_addr, None)?,
            &cw_infuser::msg::QueryMsg::WavsRecord {
                nfts: vec![nft_addr.to_string()],
                burner: None,
            },
        )
        .await?;

    // 3. form msgs for operators to sign
    let mut infusions = vec![];
    for record in res {
        if let Some(count) = record.count {
            // implement custom WAVS action here
        }
    }

```
For this demo, any burn event will trigger the AVS to check if any infusion in the cw-infuser address paired to it has the specific nft collection as an eligible collection.

If there are none, no messages are formed, otherwise a message to update the global contract state is signed via the preferred Ecdsa authorization method.
```rs

```

We still need to handle error responses, in order to resubmit transactions via governance override.
We still need to implement aggregated consensus if there are more than one operator.

## Cw-Orch-Wavs
All-in-one scripting library for deploying & testing

## Demo Requirements & Actions
- 1 Cosmos Chain: Cosmwasm + Smart Account enabled
    - deploy cw-infuser, cw721-base, & cw-wavs
    - register wavs-managed-account with custom authenticator
- 1 Ethereum Chain: 
    - deploy core eignlayer contracts
- 1 Wavs Operator:
    - deploy & register


<!-- ## Video tutorial

Follow along with the video tutorial:



You can see an example of the finished component [here](https://github.com/Lay3rLabs/WAVS-Claude-Template/tree/warpcast-eigen-counter/components/warpcast-eigen-counter). -->

## Setup
<!-- 
1. Follow the instructions to set up an account and install Claude Code: [Claude Code installation](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview)

2. Clone this repo:

```sh
git clone https://github.com/Lay3rLabs/WAVS-Claude-Template.git

cd WAVS-Claude-Template
```

3. Follow all of the setup instructions in the next section: -->

### System Requirements

<details>
<summary>Core (Docker, Compose, Make, JQ, Node v21+)</summary>

### Docker
- **MacOS**: `brew install --cask docker`
- **Linux**: `sudo apt -y install docker.io`
- **Windows WSL**: [docker desktop wsl](https://docs.docker.com/desktop/wsl/#turn-on-docker-desktop-wsl-2) & `sudo chmod 666 /var/run/docker.sock`
- [Docker Documentation](https://docs.docker.com/get-started/get-docker/)

### Docker Compose
- **MacOS**: Already installed with Docker installer
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
</details>

<details>

<summary>Rust v1.84+</summary>

### Rust Installation

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup toolchain install stable
rustup target add wasm32-wasip2
```

### Upgrade Rust

```bash
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

```bash
# Install required cargo components
# https://github.com/bytecodealliance/cargo-component#installation
cargo install cargo-binstall
cargo binstall cargo-component warg-cli wkg --locked --no-confirm --force

# Configure default registry
wkg config --default-registry wa.dev
```

</details>

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

> [!NOTE]
> If you are running on a Mac with an ARM chip, you will need to do the following:
> - Set up Rosetta: `softwareupdate --install-rosetta`
> - Enable Rosetta (Docker Desktop: Settings -> General -> enable "Use Rosetta for x86_64/amd64 emulation on Apple Silicon")
>
> Configure one of the following networking:
> - Docker Desktop: Settings -> Resources -> Network -> 'Enable Host Networking'
> - `brew install chipmk/tap/docker-mac-net-connect && sudo brew services start chipmk/tap/docker-mac-net-connect`

### Start Environment

Start an Ethereum node (anvil), the WAVS service, and deploy [eigenlayer](https://www.eigenlayer.xyz/) contracts to the local network.

```bash
cp .env.example .env

# Start the backend
#
# This must remain running in your terminal. Use another terminal to run other commands.
# You can stop the services with `ctrl+c`. Some MacOS terminals require pressing it twice.
make start-all
```

### Deploy Contract

Upload your service's trigger and submission contracts. The trigger contract is where WAVS will watch for events, and the submission contract is where the AVS service operator will submit the result on chain.

```bash
export SERVICE_MANAGER_ADDR=`make get-eigen-service-manager-from-deploy`
forge script ./script/Deploy.s.sol ${SERVICE_MANAGER_ADDR} --sig "run(string)" --rpc-url http://localhost:8545 --broadcast
```

> [!TIP]
> You can see the deployed trigger address with `make get-trigger-from-deploy`
> and the deployed submission address with `make get-service-handler-from-deploy`

## Deploy Service

Provide the cw-infuser contract address to register to the service.
```bash
# Your component filename in the `/compiled` folder
export COMPONENT_FILENAME=your_component.wasm
# Your service config. Make sure to include any necessary variables.
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
TRIGGER_EVENT="NewTrigger(bytes)" make deploy-service
```

## Trigger the Service


todo: implmeent description for triggering service via nft mint

<!-- Anyone can now call the [trigger contract](./src/contracts/WavsTrigger.sol) which emits the trigger event WAVS is watching for from the previous step. WAVS then calls the service and saves the result on-chain.

```bash
export TRIGGER_DATA_INPUT=merch
export SERVICE_TRIGGER_ADDR=`make get-trigger-from-deploy`
forge script ./script/Trigger.s.sol ${SERVICE_TRIGGER_ADDR} ${TRIGGER_DATA_INPUT} --sig "run(string,string)" --rpc-url http://localhost:8545 --broadcast -v 4
``` -->

## Show the result

Query the latest submission contract id from the previous request made.

```bash
# Get the latest TriggerId and show the result via `script/ShowResult.s.sol`
make show-result
```
