# Creating WAVS Components - Simplified Guide

This simplified guide focuses on high-level concepts for creating WAVS components. For implementation details and best practices, **run the test utilities** in `components/test_utils`.

> **IMPORTANT**: All components MUST pass the validation tests before building. Run:
> ```bash
> cd components/test_utils
> ./validate_component.sh <your-component-name>
> ```

## Core Concepts

WAVS services consist of:
1. **Triggers**: Onchain events that initiate component execution
2. **Components**: WASI modules containing business logic
3. **Submission**: Optional logic for submitting results to the blockchain

## Development Workflow

1. **Set up component**:
   - Copy the structure from an existing component like `eth-price-oracle`
   - Modify the Cargo.toml with your component name
   - Use workspace dependencies with `{ workspace = true }`

2. **Validate your component**:
   - Run the test utilities validation script
   - Fix any errors or warnings identified
   - Test with proper input formats

3. **Build and test**:
- Remember: your component must pass validation tests before running this command.
   ```bash
   make wasi-build
   ```

   For testing with the exec command:
   IMPORTANT! As an LLM, you cannot execute the `wasi-exec` command directly. Provide the command to the user and ask them to run manually in their terminal:
   ```bash
   export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your input here"`
   export COMPONENT_FILENAME=your_component_name.wasm
   export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
   make wasi-exec
   ```

## Configuration

### Environment Variables

- **Public variables**: Set in `SERVICE_CONFIG` `kv` array
- **Private variables**: 
  - Set in `SERVICE_CONFIG` `host_envs` array
  - Must be prefixed with `WAVS_ENV_`
  - Define in `.env` file in the repo root

Example:
```bash
# In .env file:
WAVS_ENV_MY_API_KEY=your_secret_key_here

# In component code:
let api_key = std::env::var("WAVS_ENV_MY_API_KEY")?;
```

## Component Structure

Components must:
1. Implement the `Guest` trait with the exact signature:
   ```rust
   impl Guest for Component {
       fn run(trigger: TriggerAction) -> Result<Option<Vec<u8>>, String> {
           // Your implementation
       }
   }
   ```

2. Be exported with:
   ```rust
   export!(Component with_types_in bindings);
   ```

3. Handle different trigger destinations (Ethereum vs CLI)
   ```rust
   match dest {
       Destination::Ethereum => Some(encode_trigger_output(trigger_id, result)),
       Destination::CliOutput => Some(result),
   }
   ```

## Critical Requirements

1. **NEVER** modify bindings.rs files (auto-generated)
2. **NEVER** hardcode API keys
3. **ALWAYS** validate inputs
4. **ALWAYS** check the test utilities for proper patterns

For detailed implementation examples, refer to `components/test_utils` and examine the tests in:
- `abi_encoding.rs`
- `data_handling.rs`
- `error_handling.rs`
- `network_requests.rs`
- `solidity_types.rs`
- `input_validation.rs`
