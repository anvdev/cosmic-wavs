#!/usr/bin/make -f

# Check if user is in docker group to determine if sudo is needed
SUDO := $(shell if groups | grep -q docker; then echo ''; else echo 'sudo'; fi)

# Define common variables
CARGO=cargo
COIN_MARKET_CAP_ID?=1
COMPONENT_FILENAME?=evm_price_oracle.wasm
CREDENTIAL?=""
DOCKER_IMAGE?=ghcr.io/lay3rlabs/wavs:99aa44aAdd
MIDDLEWARE_DOCKER_IMAGE?=ghcr.io/lay3rlabs/wavs-middleware:cd0ca86
IPFS_ENDPOINT?=http://127.0.0.1:5001
RPC_URL?=http://127.0.0.1:8545
SERVICE_FILE?=.docker/service.json
SERVICE_SUBMISSION_ADDR?=`jq -r .deployedTo .docker/submit.json`
SERVICE_TRIGGER_ADDR?=`jq -r .deployedTo .docker/trigger.json`
WASI_BUILD_DIR ?= ""
WAVS_CMD ?= $(SUDO) docker run --rm --network host $$(test -f .env && echo "--env-file ./.env") -v $$(pwd):/data ${DOCKER_IMAGE} wavs-cli
WAVS_ENDPOINT?="http://127.0.0.1:8000"
ENV_FILE?=.env
-include ${ENV_FILE}

# Default target is build
default: build

## build: building the project
build: _build_forge wasi-build

## wasi-build: building WAVS wasi components | WASI_BUILD_DIR
wasi-build:
	@./script/build_components.sh $(WASI_BUILD_DIR)

## wasi-exec: executing the WAVS wasi component(s) | COMPONENT_FILENAME, COIN_MARKET_CAP_ID
wasi-exec: pull-image
	@$(WAVS_CMD) exec --log-level=info --data /data/.docker --home /data \
	--component "/data/compiled/$(COMPONENT_FILENAME)" \
	--input `cast format-bytes32-string $(COIN_MARKET_CAP_ID)`

## clean: cleaning the project files
clean: clean-docker
	@forge clean
	@$(CARGO) clean
	@rm -rf cache
	@rm -rf out
	@rm -rf broadcast

## clean-docker: remove unused docker containers
clean-docker:
	@$(SUDO) docker rm -v $(shell $(SUDO) docker ps -a --filter status=exited -q) > /dev/null 2>&1 || true

## fmt: formatting solidity and rust code
fmt:
	@forge fmt --check
	@$(CARGO) fmt

## test: running all tests (Solidity and Rust)
test:
	@forge test
	@echo "Running component test utilities..."
	@if [ -d "./test_utils" ]; then \
		cd test_utils && cargo test; \
	else \
		echo "Warning: Test utilities not found. Skipping component tests."; \
	fi

## setup: install initial dependencies
setup: check-requirements
	@forge install
	@npm install

## start-all-local: starting anvil and core services (like IPFS for example)
start-all-local: clean-docker setup-env
	@sh ./script/start_all.sh

## get-trigger-from-deploy: getting the trigger address from the script deploy
get-trigger-from-deploy:
	@jq -r '.deployedTo' "./.docker/trigger.json"

## get-submit-from-deploy: getting the submit address from the script deploy
get-submit-from-deploy:
	@jq -r '.deployedTo' "./.docker/submit.json"

## wavs-cli: running wavs-cli in docker
wavs-cli:
	@$(WAVS_CMD) $(filter-out $@,$(MAKECMDGOALS))

## upload-component: uploading the WAVS component | COMPONENT_FILENAME, WAVS_ENDPOINT
upload-component:
	@if [ -z "${COMPONENT_FILENAME}" ]; then \
		echo "Error: COMPONENT_FILENAME is not set. Please set it to your WAVS component filename."; \
		exit 1; \
	fi
	@cd script/cw-orch-wavs && cargo run --bin wavs upload --component ${COMPONENT_FILENAME} --endpoint ${WAVS_ENDPOINT}

## deploy-service: deploying the WAVS component service json | SERVICE_URL, CREDENTIAL, WAVS_ENDPOINT
deploy-service:
	@if [ -z "${SERVICE_URL}" ]; then \
		echo "Error: SERVICE_URL is not set. Set SERVICE_URL to a link / ipfs url."; \
		exit 1; \
	fi
	@if [ -n "${WAVS_ENDPOINT}" ]; then \
		if [ "$$(curl -s -o /dev/null -w "%{http_code}" ${WAVS_ENDPOINT}/app)" != "200" ]; then \
			echo "Error: WAVS_ENDPOINT is not reachable. Please check the WAVS_ENDPOINT."; \
			exit 1; \
		fi; \
	fi
	@$(WAVS_CMD) deploy-service --service-url ${SERVICE_URL} --log-level=debug --data /data/.docker --home /data $(if $(WAVS_ENDPOINT),--wavs-endpoint $(WAVS_ENDPOINT),)

# deploy-service:
# 	@if [ -z "${SERVICE_URL}" ]; then \
# 		echo "Error: SERVICE_URL is not set. Set SERVICE_URL to a link / ipfs url."; \
# 		exit 1; \
# 	fi
# 	@cd script/cw-orch-wavs && cargo run --bin wavs deploy-service --service-url ${SERVICE_URL} $(if $(WAVS_ENDPOINT),--wavs-endpoint $(WAVS_ENDPOINT),)


## get-trigger: get the trigger id | SERVICE_TRIGGER_ADDR, RPC_URL
get-trigger:
	@forge script ./script/ShowResult.s.sol ${SERVICE_TRIGGER_ADDR} --sig 'trigger(string)' --rpc-url $(RPC_URL) --broadcast
## get-trigger-cosmos: get the cosmos trigger id | COSMOS_RPC_URL, COSMOS_CHAIN_ID
get-trigger-cosmos:
	@echo "Querying Cosmos chain for trigger events..."
	@curl -s "${COSMOS_RPC_URL}/abci_query?path=\"/custom/wasm/smart/${WAVS_CONTRACT_ADDRESS}&data={\\\"trigger_count\\\":{}}\""

## deploy-cosmos-service: deploy a WAVS service for Cosmos triggers | COMPONENT_FILENAME, COSMOS_RPC_URL
deploy-cosmos-service:
	@sh ./script/deploy-cosmos-service.sh

## start-cosmos-service: deploy and start a WAVS service for Cosmos triggers | COMPONENT_FILENAME
start-cosmos-service:
	@sh ./script/deploy-cosmos-service.sh --start

TRIGGER_ID?=1
## show-result: showing the result | SERVICE_SUBMISSION_ADDR, TRIGGER_ID, RPC_URL
show-result:
	@forge script ./script/ShowResult.s.sol ${SERVICE_SUBMISSION_ADDR} ${TRIGGER_ID} --sig 'data(string,uint64)' --rpc-url $(RPC_URL) --broadcast


PINATA_API_KEY?=""
## upload-to-ipfs: uploading a service config to IPFS | SERVICE_FILE, [PINATA_API_KEY]
upload-to-ipfs:
	@if [ `sh script/get-deploy-status.sh` = "LOCAL" ]; then \
		curl -X POST "http://127.0.0.1:5001/api/v0/add?pin=true" -H "Content-Type: multipart/form-data" -F file=@${SERVICE_FILE} | jq -r .Hash; \
	else \
		if [ -z "${PINATA_API_KEY}" ]; then \
			echo "Error: PINATA_API_KEY is not set. Please set it to your Pinata API key -- https://app.pinata.cloud/developers/api-keys."; \
			exit 1; \
		fi; \
		curl -X POST --url https://uploads.pinata.cloud/v3/files --header "Authorization: Bearer ${PINATA_API_KEY}" --header 'Content-Type: multipart/form-data' --form file=@${SERVICE_FILE} --form network=public --form name=service-`date +"%b-%d-%Y"`.json | jq -r .data.cid; \
	fi

## operator-list: listing the AVS operators | ENV_FILE
operator-list:
	@if [ -z "${WAVS_SERVICE_MANAGER_ADDRESS}" ]; then \
		echo "Error: WAVS_SERVICE_MANAGER_ADDRESS is not set. Please set it to the deployed WAVS stake registry." && exit 1; \
	fi
	@docker run --rm --network host --env-file ${ENV_FILE} \
		-e WAVS_SERVICE_MANAGER_ADDRESS=${WAVS_SERVICE_MANAGER_ADDRESS} \
		-v ./.nodes:/root/.nodes --entrypoint /wavs/list_operator.sh ${MIDDLEWARE_DOCKER_IMAGE}

AVS_PRIVATE_KEY?=""
DELEGATION?="0.001ether"
## operator-register: listing the AVS operators | ENV_FILE, AVS_PRIVATE_KEY
operator-register:
	@if [ -z "${AVS_PRIVATE_KEY}" ]; then \
		echo "Error: AVS_PRIVATE_KEY is not set. Please set it to your AVS private key." && exit 1; \
	fi
	@if [ -z "${WAVS_SERVICE_MANAGER_ADDRESS}" ]; then \
		echo "Error: WAVS_SERVICE_MANAGER_ADDRESS is not set. Please set it to the deployed WAVS service manager." && exit 1; \
	fi
	@docker run --rm --network host \
		-e WAVS_SERVICE_MANAGER_ADDRESS=${WAVS_SERVICE_MANAGER_ADDRESS} \
		--env-file ${ENV_FILE} \
		-v ./.nodes:/root/.nodes \
		--entrypoint /wavs/register.sh ${MIDDLEWARE_DOCKER_IMAGE} "${AVS_PRIVATE_KEY}" "${DELEGATION}"


## update-submodules: update the git submodules
update-submodules:
	@git submodule update --init --recursive

# Declare phony targets
.PHONY: build clean fmt bindings test

.PHONY: help
help: Makefile
	@echo
	@echo " Choose a command run"
	@echo
	@sed -n 's/^##//p' $< | column -t -s ':' |  sed -e 's/^/ /'
	@echo

# helpers
_build_forge:
	@forge build

.PHONY: setup-env
setup-env:
	@if [ ! -f .env ]; then \
		if [ -f .env.example ]; then \
			echo "Creating .env file from .env.example..."; \
			cp .env.example .env; \
			echo ".env file created successfully!"; \
		fi; \
	fi

pull-image:
	@if ! docker image inspect ${DOCKER_IMAGE} &>/dev/null; then \
		echo "Image ${DOCKER_IMAGE} not found. Pulling..."; \
		$(SUDO) docker pull ${DOCKER_IMAGE}; \
	fi

# check versions

## check-requirements: verify system requirements are installed
check-requirements: check-node check-jq check-cargo

check-command:
	@command -v $(1) > /dev/null 2>&1 || (echo "Command $(1) not found. Please install $(1), reference the System Requirements section"; exit 1)

.PHONY: check-node
check-node:
	@$(call check-command,node)
	@NODE_VERSION=$$(node --version); \
	MAJOR_VERSION=$$(echo $$NODE_VERSION | sed 's/^v\([0-9]*\)\..*/\1/'); \
	if [ $$MAJOR_VERSION -lt 21 ]; then \
		echo "Error: Node.js version $$NODE_VERSION is less than the required v21."; \
		echo "Please upgrade Node.js to v21 or higher."; \
		exit 1; \
	fi

.PHONY: check-jq
check-jq:
	@$(call check-command,jq)

.PHONY: check-cargo
check-cargo:
	@$(call check-command,cargo)
