# Brewery Finder Component

A WAVS component that queries the OpenBrewery API to find breweries by zip code.

## Overview

This component takes a zip code as input and returns information about breweries in that area. It uses the OpenBrewery DB API which is free and does not require authentication.

## Input Format

The component expects a string parameter containing a zip code:

```solidity
function findBreweriesByZip(string zipCode) external;
```

## Response Format

The component returns a JSON object with the following structure:

```json
{
  "zip_code": "12345",
  "breweries": [
    {
      "id": "string",
      "name": "string",
      "brewery_type": "string",
      "street": "string",
      "address_2": "string",
      "address_3": "string",
      "city": "string",
      "state": "string",
      "postal_code": "string",
      "country": "string",
      "longitude": "string",
      "latitude": "string",
      "phone": "string",
      "website_url": "string"
    }
  ],
  "count": 0,
  "timestamp": "string"
}
```

## API Details

- API: OpenBrewery DB (https://www.openbrewerydb.org/)
- Endpoint: `https://api.openbrewerydb.org/v1/breweries?by_postal={postal_code}`
- No authentication required
- The API supports both 5-digit and 9-digit postal codes
- For 9-digit codes, include a hyphen (e.g., "12345-6789")

## Usage

```bash
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "44107"`
export COMPONENT_FILENAME=brewery_finder.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```