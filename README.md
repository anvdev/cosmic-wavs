# [Cosmos-WAVS] 
 -->


> [!WARNING]
> **Experimental Use Only**: This template is designed for experimentation. Results may vary. This template is not intended for production use. Use at your own risk and for experimental purposes only.
Repo template: https://github.com/Lay3rLabs/wavs-foundry-template



## Goals
- Track cosmwasm nft burn events emitted from Cosmos Chain
- Authentication action to perform via Wavs operator keys
- Broadcast authorized action to Cosmos Chain


## WAVS Actions To Explore
1. Operator Fee Allocations:
    - Smart contract logic allocation of fees routed to contract owner for operator incentives
2. Proof of Task - Aggregated Operator Key Signatures For Action Authorization:
    - **Action Authorization**: Query & Write to smart contract state on Cosmos via action occuring on Eth Or Btc
3. Execution Service Method:
    - Determining which operator broadcast the gls & actions to perform
4. TODO: Caching of Failed Transactions: 


 

<!-- ## Video tutorial

Follow along with the video tutorial:

[![Watch the video](/img/video.png)](https://youtu.be/jyl7kbie41w)

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
<!-- 
## Create Components with Claude Code

After following all setup instructions and installing Claude Code, you are ready to make a component!

1. In the root of your project, run the following command to start Claude Code:

```sh
claude
```

1. Enter your one-shot prompt. In this example, we're creating a component that can check how many times a Warpcast user has used the word EigenLayer in a post. You can see an example of the finished component [here](https://github.com/Lay3rLabs/WAVS-Claude-Template/tree/warpcast-eigen-counter/components/warpcast-eigen-counter).

```
Let's make a new component that takes the input of a warpcast username (like dabit3), counts the number of times they have mentioned EigenLayer, and returns that number and the user's wallet address.


Make sure you handle endpoint responses and cast data correctly:

- https://hoyt.farcaster.xyz:2281/v1/userNameProofByName?name=dabit3
- https://hoyt.farcaster.xyz:2281/v1/castsByFid?fid=235510
```

3. Claude will start creating your component. Review Claude's work and accept changes that Claude makes. Make sure to double check what Claude is doing and be safe about accepting changes.

4. Claude will make a new component and files, and run validation tests on the component using the `make validate-component COMPONENT=your-component` command.

5. Claude may need to make changes after running the Validation tests. After making changes, Claude will build the component using the `make wasi-build` command. -->

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
<!-- 
Claude may try to run the `make wasi-exec` command themselves. You should prompt Claude to give you the command instead, as Claude can't run it without permissions.

> [!WARNING]
> If you get: `error: no registry configured for namespace "wavs"`
>
> run, `wkg config --default-registry wa.dev`

> [!WARNING]
> If you get: `failed to find the 'wasm32-wasip1' target and 'rustup' is not available`
>
> `brew uninstall rust` & install it from <https://rustup.rs> -->


<!-- 7. Your component should execute. If there are any errors, share them with Claude for troubleshooting. -->

<!-- ## Tips for working with Claude

- While this repo contains a [claude.md](/claude.md) file with enough context for creating simple components, Claude Code may inevitably run into problems.
- Feel free to update [claude.md](/claude.md) for your specific purposes or if you run into regular errors.
- Claude can sometimes try to over-engineer its fixes for errors. If you feel it is not being productive, delete the component, clear claude with `/clear`, and try again. You may need to adjust your prompt.
- If you are building a complex component, it may be helpful to have Claude build a simple component first and then expand upon it.
- Claude may try to fix warnings unnecessarily. You can Tell Claude to ignore minor warnings and any errors found in bindings.rs (it is auto-generated).

### Prompting

This repo is designed to be used with short prompts for simple components. However, often times, Claude will do better with more context.

- Provide relevant documentation (preferably as an `.md` file or other ai-digestible content).
- Provide endpoints.
- You may need to provide API response structure if Claude is just not understanding responses.
- Be specific about what you want Claude to build.
- Be patient.

## Examples

The [`/examples`](/examples/) directory contains multiple one-shot examples built by Claude. These serve as a knowledge base for Claude. Explore the examples for ideas, or try to build one of the examples yourself. Remember to delete the example that you want to build before prompting Claude, otherwise it may just copy it directly.

## Troubleshooting

- You can ask Claude to fix errors it may not be able to catch when executing components. Make sure to give Claude full context of the error.
- LLMs can be unpredictable. Minimal prompts provide a lot of room for creativity/error. If Claude is not able to fix an error after trying, sometimes deleting the component, clearing Claude history with `/clear` and starting fresh can help.
- Claude may try to edit the bindings.rs file to "fix" it. Claude never needs to do this.
- Claude is supposed to provide you with the `make wasi-exec` command. Sometimes it will try to run this itself. It can't. Ask it to give you the command.
- When copying and pasting the full `make wasi-exec` command, be careful with line breaks, especially in the `SERVICE_CONFIG`. You may need to reformat long lines to avoid break. -->



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

Deploy the compiled component with the contracts from the previous steps. Review the [makefile](./Makefile) for more details and configuration options.`TRIGGER_EVENT` is the event that the trigger contract emits and WAVS watches for. By altering `SERVICE_TRIGGER_ADDR` you can watch events for contracts others have deployed.

```bash
# Your component filename in the `/compiled` folder
export COMPONENT_FILENAME=your_component.wasm
# Your service config. Make sure to include any necessary variables.
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
TRIGGER_EVENT="NewTrigger(bytes)" make deploy-service
```

## Trigger the Service

Anyone can now call the [trigger contract](./src/contracts/WavsTrigger.sol) which emits the trigger event WAVS is watching for from the previous step. WAVS then calls the service and saves the result on-chain.

```bash
export TRIGGER_DATA_INPUT=dabit3
export SERVICE_TRIGGER_ADDR=`make get-trigger-from-deploy`
forge script ./script/Trigger.s.sol ${SERVICE_TRIGGER_ADDR} ${TRIGGER_DATA_INPUT} --sig "run(string,string)" --rpc-url http://localhost:8545 --broadcast -v 4
```

## Show the result

Query the latest submission contract id from the previous request made.

```bash
# Get the latest TriggerId and show the result via `script/ShowResult.s.sol`
make show-result
```
