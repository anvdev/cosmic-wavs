#!/bin/bash

cd `git rev-parse --show-toplevel` || exit

if [ ! -f .env ]; then
    cp .env.example .env
    if [ $? -ne 0 ]; then
        echo "Failed to copy .env.example to .env"
        exit 1
    fi
fi


# Extract TRIGGER_ORIGIN from the file
TRIGGER_ORIGIN=$(grep "^TRIGGER_ORIGIN=" .env | cut -d '=' -f2)
TRIGGER_ORIGIN_ENV=$(echo "$TRIGGER_ORIGIN" | tr '[:lower:]' '[:upper:]')
echo "$TRIGGER_ORIGIN_ENV"
