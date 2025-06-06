#!/bin/bash

 
echo "Configuring cosmos node"
sh script/cosmos/configure-cosmos.sh
 
FILES="-f script/cosmos/docker-compose.yaml"
docker compose ${FILES} pull
docker compose ${FILES} up --force-recreate -d
trap "docker compose ${FILES} down --remove-orphans && docker kill wavs-1 wavs-aggregator-1 > /dev/null 2>&1 && echo -e '\nKilled IPFS + Local WARG, and wavs instances'" EXIT

echo "Started Cosmos node network..."

# Keep the script running to prevent premature exit
while true; do
  sleep 3600  # Sleep for an hour per iteration to reduce resource usage
done
