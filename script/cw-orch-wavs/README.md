
## Requirements

### Cw-Orchestrator
https://orchestrator.abstract.money/setup/workspace/deploy.html
### Anvil
### WAVS Service
## Workflow
 

To run the `wavs.rs` binary with the specified subcommands and options, you can use the following `cargo run` commands.

### 1. Deploy WAVS infrastructure and contracts

```bash
cargo run -- deploy --network <network> --docker-compose <docker-compose-file>
```

| Flag | Description | Required |
| --- | --- | --- |
| `--network` | Network to deploy on (e.g. main, testnet, local) | Yes |
| `--docker-compose` | Path to docker compose file for deploying eth & cosmos network | Yes |

Example:
```bash
cargo run -- deploy --network main --docker-compose ./path/to/docker-compose.yml
```

### 2. Build WAVS service configuration

```bash
cargo run -- build-service [--config <config-file>]
```

| Flag | Description | Required |
| --- | --- | --- |
| `--config` | Custom config file location (optional) | No |

Example:
```bash
cargo run -- build-service --config ./path/to/config.toml
```

### 3. Create a new deployer wallet

```bash
cargo run -- create-deployer [--rpc-url <rpc-url>] [--env <env>]
```

| Flag | Description | Required |
| --- | --- | --- |
| `--rpc-url` | RPC URL for funding wallet (optional) | No |
| `--env` | Deployment environment (LOCAL, TESTNET) (optional) | No |

Example:
```bash
cargo run -- create-deployer --rpc-url http://localhost:8545 --env LOCAL
```

### 4. Create a new aggregator

```bash
cargo run -- create-aggregator [--index <index>] [--rpc-url <rpc-url>] [--env <env>]
```

| Flag | Description | Required | Default |
| --- | --- | --- | --- |
| `--index` | Aggregator index number | No | 1 |
| `--rpc-url` | RPC URL for funding wallet (optional) | No |  |
| `--env` | Deployment environment (LOCAL, TESTNET) (optional) | No |  |

Example:
```bash
cargo run -- create-aggregator --index 2 --rpc-url http://localhost:8545 --env TESTNET
```

### 5. Deploy Cosmos WAVS service

```bash
cargo run -- deploy-cosmos [--component <component>] [--rpc-url <rpc-url>] [--chain-id <chain-id>] [--trigger-event <trigger-event>] [--start]
```

| Flag | Description | Required | Default |
| --- | --- | --- | --- |
| `--component` | Component filename | No | cosmic-wavs-demo-infusion.wasm |
| `--rpc-url` | Cosmos RPC URL | No | http://localhost:26657 |
| `--chain-id` | Cosmos chain ID | No | sub-1 |
| `--trigger-event` | Trigger event name | No | cw-infusion |
| `--start` | Start service after deployment | No |  |

Example:
```bash
cargo run -- deploy-cosmos --component cosmic-wavs-demo-infusion.wasm --rpc-url http://localhost:26657 --chain-id sub-1 --trigger-event cw-infusion --start
```

### 6. Upload component to WAVS

```bash
cargo run -- upload --component <component> [--endpoint <endpoint>]
```

| Flag | Description | Required | Default |
| --- | --- | --- | --- |
| `--component` | Component filename | Yes |  |
| `--endpoint` | WAVS endpoint | No | http://localhost:8000 |

Example:
```bash
cargo run -- upload --component my-component.wasm --endpoint http://localhost:8000
```

### 7. Deploy WAVS service

```bash
cargo run -- deploy-service --service-url <service-url> [--wavs-endpoint <wavs-endpoint>]
```

| Flag | Description | Required | Default |
| --- | --- | --- | --- |
| `--service-url` | Service URL (IPFS hash or HTTP URL) | Yes |  |
| `--wavs-endpoint` | WAVS endpoint (optional) | No |  |

Example:
```bash
cargo run -- deploy-service --service-url https://example.com/service --wavs-endpoint http://localhost:8000
```

### 8. Start all local services

```bash
cargo run -- start-all [--fork-rpc-url <fork-rpc-url>]
```

| Flag | Description | Required |
| --- | --- | --- |
| `--fork-rpc-url` | Fork RPC URL for Anvil (optional) | No |

Example:
```bash
cargo run -- start-all --fork-rpc-url http://localhost:8545
```

 