#!/bin/bash

cd `git rev-parse --show-toplevel` || exit

if [ ! -f .env ]; then
    cp .env.example .env
    if [ $? -ne 0 ]; then
        echo "Failed to copy .env.example to .env"
        exit 1
    fi
fi


# Extract TRIGGER_DEST from the file
TRIGGER_DEST=$(grep "^TRIGGER_DEST=" .env | cut -d '=' -f2)
TRIGGER_DEST_ENV=$(echo "$TRIGGER_DEST" | tr '[:lower:]' '[:upper:]')
echo "$TRIGGER_DEST_ENV"
