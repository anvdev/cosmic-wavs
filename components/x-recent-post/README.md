# X Recent Post Component

This WAVS component fetches the most recent post from an X (Twitter) user.

## Features

- Looks up a user by username to get their unique ID
- Fetches the most recent tweet from the user's timeline
- Returns structured data including username, tweet text, and timestamp

## Usage

### Input

The component accepts a Twitter username as input:

```solidity
function getRecentTweet(string username) external;
```

### Output

The component returns a JSON object with the following structure:

```json
{
  "username": "string",
  "user_id": "string",
  "tweet_id": "string",
  "tweet_text": "string",
  "created_at": "string",
  "profile_name": "string"
}
```

### Environment Variables

The component requires the following environment variable:

- `WAVS_ENV_X_BEARER_TOKEN`: X API Bearer Token for authentication

## Example

To run the component from the command line:

```bash
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "dabit3"`
export COMPONENT_FILENAME=x_recent_post.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_X_BEARER_TOKEN\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```