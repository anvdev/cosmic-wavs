# Creating WAVS Components - Simplified Guide

This simplified guide focuses on high-level concepts for creating WAVS components. For implementation details and best practices, **run the test utilities** in `components/test_utils`.

> **IMPORTANT**: All components MUST pass the validation tests before building. Run:
> ```bash
> cd components/test_utils
> ./validate_component.sh <your-component-name>
> ```




## Creating New Components

When creating a new component, you must follow these steps completely to avoid common errors:

**Phase 1: Prepare**
1. Read claude.md
2. Study the component example in /components/eth-price-oracle
3. Read /components/test_utils/validate_component.sh in its entirety. It contains information on common errors when creating components.
4. Read the files in components/test_utils/src. They contain more best-practice code examples.

**Phase 2: Create**
5. Copy the `eth-price-oracle` component's Cargo.toml and modify the name
6. Create a lib.rs file similar to the `eth-price-oracle`
7. Copy the `eth-price-oracle` component's Cargo.toml and modify the name
8. Never edit bindings.rs

**Phase 3: Test**
9.  Double-check your component code against validate_component.sh, eth-price-oracle example, and claude.md to ensure it is made correctly
10. Run `make validate-component COMPONENT=your-component-name` to run the validation tests.
11. You must fix all errors and warnings from the validation test before proceeding or the component will not build properly.
12. Repeat steps 10 and 11 until the component passes all tests.

**Phase 4: Build**

13. Proceed only after there are no more errors or warnings when running `make validate-component COMPONENT=your-component-name`.
14. Run `make wasi-build`: this command builds every component in the /component directory, generates bindings automatically (you do not ever need to edit a bindings.rs file!), automatically compiles components to WASM, and places them automatically in the /compiled folder.
15. If the build fails, you will need to create fixes, pass validation checks again, and build again. ALWAYS AVOID BUILDING MORE THAN ONCE. ALL ERRORS SHOULD BE CAUGHT BEFORE EVER RUNNING THE BUILD COMMAND.

**Phase 5: Test**

16. Prepare the `make wasi-exec` command. IMPORTANT! As an LLM, you cannot execute the `wasi-exec` command ever. Always provide the command to the user in the terminal and ask them to run it manually:

```bash
# ONLY use this format for component input data:
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "your long string here"`

# CRITICAL: When handling ABI-encoded inputs:
# - NEVER use String::from_utf8 directly on binary data
# - ABI-encoded data is binary and must be handled according to its format

export COMPONENT_FILENAME=eth_price_oracle.wasm # the filename of your compiled component.
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'" # The service config

make wasi-exec
```



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
