#!/bin/bash

if [ -z "$REGISTRY" ]; then
    echo "REGISTRY is not set. Please set the REGISTRY environment variable." && exit 1
fi
if [ -z "$PKG_NAME" ]; then
    echo "PKG_NAME is not set. Please set the PKG_NAME environment variable." && exit 1
fi
if [ -z "$PKG_VERSION" ]; then
    echo "PKG_VERSION is not set. Please set the PKG_VERSION environment variable." && exit 1
fi
if [ -z "$COMPONENT_FILENAME" ]; then
    echo "COMPONENT_FILENAME is not set. Please set the COMPONENT_FILENAME environment variable." && exit 1
fi
if [ -z "$PKG_NAMESPACE" ]; then
    echo "PKG_NAMESPACE is not set. Please set the PKG_NAMESPACE environment variable." && exit 1
fi

# ===

cd `git rev-parse --show-toplevel` || exit

PROTOCOL="https"
if [[ "$REGISTRY" == *"localhost"* ]] || [[ "$REGISTRY" == *"127.0.0.1"* ]]; then
    PROTOCOL="http"
fi
echo "Publishing to registry (${PROTOCOL}://${REGISTRY})..."


output=$(warg publish release --registry ${PROTOCOL}://${REGISTRY} --name ${PKG_NAMESPACE}:${PKG_NAME} --version ${PKG_VERSION} ./compiled/${COMPONENT_FILENAME} 2>&1 || true)
exit_code=$?

# Check for specific error conditions in the output
if [[ $exit_code -ne 0 ]]; then
    if [[ "$output" =~ "failed to prove inclusion" ]]; then
        echo "Package uploaded to local registry successfully..."
    elif [[ "$output" =~ "error sending request for url" ]]; then
        echo "NOTE: Check to make sure you are running the registry locally"
        echo "${output}"
        exit 1
    else
        echo "Unknown error occurred ${output}"
        exit 1
    fi
fi

exit 0
