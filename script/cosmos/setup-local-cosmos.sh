#!/bin/bash
 
echo "Configuring cosmos node"
sh script/cosmos/configure-local-cosmos.sh
 
FILES="-f script/cosmos/docker-compose.yaml"
docker compose ${FILES} pull
docker compose ${FILES} up --force-recreate -d
# trap "docker compose ${FILES} down --remove-orphans && docker kill wavs-1 wavs-aggregator-1 > /dev/null 2>&1 && echo -e '\nKilled Cosmos Instance'" EXIT

echo "Started Cosmos node network..."
 