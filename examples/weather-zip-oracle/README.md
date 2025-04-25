# Weather ZIP Oracle Component

## Overview

The Weather ZIP Oracle component fetches current weather data for a given US ZIP code using the OpenWeatherMap API.

## What It Does

- Accepts a US ZIP code as input
- Validates the ZIP code format
- Queries the OpenWeatherMap API
- Returns comprehensive weather data for the specified location

## Key Features

- Input validation for proper ZIP code format
- Secure API key handling via environment variables
- Comprehensive weather information including temperature, feels like, humidity, wind speed
- Proper error handling for API responses

## Input Format

The component expects a 5-digit US ZIP code as a string:

```
"10001"
```

## Output Format

```json
{
  "zip_code": "10001",
  "city": "New York",
  "country": "US",
  "temperature": 72.5,
  "feels_like": 71.2,
  "description": "partly cloudy",
  "humidity": 45,
  "wind_speed": 8.5,
  "timestamp": "1713993600"
}
```

## WASI Execution Command

Note: You must set the `WAVS_ENV_OPENWEATHER_API_KEY` environment variable with a valid OpenWeatherMap API key in your.env file before running this component.

```bash
# Use this command to test the component locally:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "10001"`
export COMPONENT_FILENAME=weather_zip_oracle.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_OPENWEATHER_API_KEY\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

