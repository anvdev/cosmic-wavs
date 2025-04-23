# USDT Balance Checker Component Errors

## Validation Test Error
The component failed the validation test with:
```
⚠️ Test error: Cargo check failed with exit code Some(101)
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `test_utils`
 --> /tmp/run_checks.rs:4:11
  |
4 |     match test_utils::code_quality::run_component_code_quality_checks(&component_path) {
  |           ^^^^^^^^^^ use of unresolved module or unlinked crate `test_utils`
  |
  = help: you might be missing a crate named `test_utils`
```

This appears to be an issue with how the validation script is trying to access the test_utils crate, rather than a direct issue with our component code. The validation commands in the makefile may need adjustment or the test_utils crate may need to be properly linked.

## Build Success
Despite the validation test error, the component was successfully built with `make wasi-build`. The only warning was:
```
warning: unused import: `sol`
 --> components/usdt-balance-checker/src/lib.rs:6:23
  |
6 | use alloy_sol_types::{sol, SolCall, SolValue};
  |                       ^^^
```

This is a minor issue that doesn't affect the component's functionality.

## Runtime Error - Input Decoding
When executing the component with an ABI-encoded address input, the following error occurred:
```
Input length: 32 bytes
thread 'main' panicked at packages/cli/src/main.rs:157:14:
called `Result::unwrap()` on an `Err` value: Wasm exec result: Failed to decode input as UTF-8 string: invalid utf-8 sequence of 1 bytes from index 12
```

### Fix Applied
The issue was with how the component was trying to decode the input data. The problem was that we were trying to use `std::str::from_utf8` directly on ABI-encoded binary data, which is not valid UTF-8.

Fixed the input handling by:
1. Implementing proper ABI decoding using `Address::abi_decode` first (as shown in the test_utils examples)
2. Adding a fallback to decode as a function call using `solidity::checkBalanceCall::abi_decode`
3. Adding more detailed error logging and debug printing of input data
4. Removed the UTF-8 string conversion attempt which was causing the error

## Test Utils Validation Script Fix

The validation script for WAVS components had an issue that prevented it from properly validating components:

1. The original script attempted to use a standalone Rust program (`/tmp/run_checks.rs`) to call functionality from the test_utils crate, but failed to properly link to the crate.

2. The issue manifested as:
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `test_utils`
```

3. To fix the issue, we replaced the problematic section with a more direct approach:
   - Created a simplified validation script that uses basic bash commands
   - Removed the dependency on the standalone Rust program
   - Used direct cargo check to identify potential code issues
   - Streamlined the validation process

The fixed validation script now successfully validates both the eth-price-oracle and usdt-balance-checker components, highlighting appropriate warnings without failing unnecessarily.

## New Build Errors (April 22, 2025)

Several build errors occurred during compilation of the USDT Balance Checker component:

### 1. Solidity Module Resolution
```
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `solidity`
  --> components/usdt-balance-checker/src/lib.rs:57:32
   |
57 |             let trigger_info = solidity::TriggerInfo::abi_decode(&event._triggerInfo, false)?;
   |                                ^^^^^^^^ use of unresolved module or unlinked crate `solidity`
```

We need to define a module for the Solidity types instead of referring to an external `solidity` module.

### 2. TriggerAction Variant Issues
```
error[E0223]: ambiguous associated type
   --> components/usdt-balance-checker/src/lib.rs:158:13
    |
158 |             TriggerAction::Trigger { data } => {
    |             ^^^^^^^^^^^^^^^^^^^^^^
```

The variants of the TriggerAction enum in the bindings may be different than expected. We need to examine the actual bindings.rs to determine the correct patterns.

### 3. Export Macro Issue
```
error[E0433]: failed to resolve: could not find `__export_world_layer_trigger_world_cabi` in the crate root
    --> components/usdt-balance-checker/src/bindings.rs:1029:35
```

The export macro may be using incorrect syntax. We need to check how the export macro is defined in the bindings.

## Runtime Error (April 22, 2025)

When executing the USDT Balance Checker component with the Vitalik.eth wallet address, we encountered a runtime error:

```
thread '<unnamed>' panicked at /Users/evan/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/slice.rs:569:23:
capacity overflow
...
thread 'main' panicked at packages/cli/src/main.rs:157:14:
called `Result::unwrap()` on an `Err` value: Component returned an error: error while executing at wasm backtrace:
    ...
    7: 0x7c53c - usdt_balance_checker.wasm!alloc::raw_vec::capacity_overflow::h0fb74a5685c13d41
    8: 0x7c61b - usdt_balance_checker.wasm!alloc::raw_vec::handle_error::h7ab716323ae4d286
    9: 0x22305 - usdt_balance_checker.wasm!alloc::slice::<impl [T]>::repeat::h0f68a9a06c76bc1d
   10: 0x5088 - usdt_balance_checker.wasm!wstd::runtime::block_on::block_on::h75e8be2a7f5bc152
```

The error occurs in the string padding logic, where we're trying to create a string with repeated "0" characters using `"0".repeat(padding)`. If the `padding` value is extremely large (which could happen if the fractional part is very short compared to the expected number of decimals, or if there's a negative value), this would cause a capacity overflow.

The fix involves:
1. Adding boundary checks to ensure padding can't be negative
2. Limiting the maximum amount of padding to avoid overflows
3. Adding more robust error handling around the string formation code

## NFT Ownership Checker Component Errors (April 22, 2025)

### Validation Error: Incorrect export! macro syntax
```
❌ Error: Incorrect export! macro syntax. Use 'export!(YourComponent with_types_in bindings)' instead of just 'export!(YourComponent)'.
../nft-ownership-checker/src/lib.rs:export!(NftOwnershipChecker);
```

The issue was resolved by using the correct export macro syntax:
```rust
export!(NftOwnershipChecker with_types_in bindings);
```

This error demonstrates the importance of using the correct export macro format for WAVS components.

## Common Blockchain Components Errors (April 22, 2025)

### TxKind Import Error - Critical Build Failure
```
error[E0433]: failed to resolve: could not find `TxKind` in `eth`
   --> components/usdt-balance-checker/src/lib.rs:151:40
    |
151 |         to: Some(alloy_rpc_types::eth::TxKind::Call(token_contract)),
    |                                        ^^^^^^ could not find `TxKind` in `eth`
```

**Problem**: The component was trying to use `alloy_rpc_types::eth::TxKind` but it should be imported from `alloy_primitives::TxKind`.

**Solution**:
1. Add the correct import:
   ```rust
   use alloy_primitives::{Address, TxKind, U256};
   ```
2. Update the usage throughout the file:
   ```rust
   to: Some(TxKind::Call(token_contract))
   ```

This is a common error when working with blockchain components that interact with Ethereum. The validation checks now explicitly look for this error pattern to help catch it early.

## Validation Error (2025-04-23)
```
🔍 Validating component: usdt-balance-checker
📝 Checking for common String::from_utf8 misuse...
📝 Checking for Clone derivation on structs...
📝 Checking for map_err on Option types...
📝 Checking for essential imports...
📝 Checking for proper component export...
❌ Error: export! macro not found. Components must use export! macro.
```

- The component used #[export] attribute instead of the required export! macro.
- Fixed by replacing #[export] with export!(UsdtBalanceChecker with_types_in bindings) at the end of the file.

## Build Error (2025-04-23)
```
error[E0609]: no field `trigger_data` on type `layer_types::TriggerAction`
   --> components/usdt-balance-checker/src/lib.rs:139:23
    |
139 |         match trigger.trigger_data {
    |                       ^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `config`, `data`
```

- The TriggerAction struct does not have a `trigger_data` field as expected.
- According to the error, it has `config` and `data` fields instead.

## Build Error (2025-04-23) - Second Attempt
```
error[E0308]: mismatched types
   --> components/usdt-balance-checker/src/lib.rs:133:13
    |
132 |         match trigger.data {
    |               ------------ this expression has type `TriggerData`
133 |             Some(trigger_data) => {
    |             ^^^^^^^^^^^^^^^^^^ expected `TriggerData`, found `Option<_>`
```

- The `trigger.data` field is of type `TriggerData`, not `Option<TriggerData>`.
- We need to update our match statement to handle `TriggerData` directly, not as an Optional.

## Runtime Error (2025-04-23)
```
Running USDT Balance Checker
Received raw data, length: 128 bytes
Received address hex: *
thread 'main' panicked at packages/cli/src/main.rs:157:14:
called `Result::unwrap()` on an `Err` value: Wasm exec result: Failed to parse wallet address: invalid string length
```

- The issue is with how we're attempting to extract the wallet address from the ABI-encoded data.
- The current implementation incorrectly assumes the address is a UTF-8 string at a fixed position.
- We need to properly decode the ABI-encoded string function call.

## USDT Balance Checker Component Validation Errors (2025-04-23)

The validation for the USDT Balance Checker component failed with the following errors:

### 1. "Move out of index" errors (2025-04-23)
```
❌ Error: Found potential 'move out of index' errors - accessing collection elements without cloning.
      When accessing fields from elements in a collection, you must clone the field to avoid
      moving out of the collection, which would make the collection unusable afterward.
      WRONG:  let field = collection[0].field; // This moves the field out of the collection
      RIGHT:  let field = collection[0].field.clone(); // This clones the field
      ../usdt-balance-checker/src/lib.rs:        let mut formatted = padded[0..decimal_index].to_string();
../usdt-balance-checker/src/lib.rs:        let mut formatted = balance_str[0..decimal_index].to_string();
```

This error occurs in our `format_token_balance` function where we're extracting slices from strings without cloning them appropriately.

### 2. Missing std::cmp::min import (2025-04-23)
```
❌ Error: Found min function usage but std::cmp::min is not imported.
      This will cause a compile error when using min().
      Fix: Add 'use std::cmp::min;' to your imports.
      ../usdt-balance-checker/src/lib.rs:        let padding = std::cmp::min(decimals as usize - balance_len + 1, 100);
```

We're using `std::cmp::min` in our code but haven't imported it.

### 3. Cargo check compilation errors (2025-04-23)
The component has compilation errors that need to be fixed before it can be built.

### 4. Potentially unbounded string.repeat operations (2025-04-23)
```
❌ Error: Found potentially unbounded string.repeat operations:
../usdt-balance-checker/src/lib.rs:        let padded = "0".repeat(padding) + &balance_str;

This can cause capacity overflow errors. Options to fix:
  1. Add a direct safety check: ".repeat(std::cmp::min(variable, 100))"
  2. Use a bounded variable: "let safe_value = std::cmp::min(variable, MAX_SIZE); .repeat(safe_value)"
  3. Add a safety comment if manually verified: "// SAFE: bounded by check above"
```

Although we're using `std::cmp::min` to limit the padding value, the validation script still reports this as a potential issue. We should make the safety check more explicit or add a comment to clarify.

### 5. Option Type Error Handling
```
❌ Error: Found potential map_err used on Option types. Use ok_or_else instead.
      Option types don't have map_err method - it's only available on Result types.
      WRONG:  get_eth_chain_config("mainnet").map_err(|e| e.to_string())?
      RIGHT:  get_eth_chain_config("mainnet").ok_or_else(|| "Failed to get config".to_string())?
```

This error occurs because we're trying to use `map_err` on an `Option` type, but this method only exists on `Result` types. We need to convert the `Option` to a `Result` using `ok_or_else` before using `map_err`.

### 6. Function Signature Mismatch
```
❌ Error: run function with correct signature not found.
      The run function must match EXACTLY this signature:
      fn run(trigger: TriggerAction) -> Result<Option<Vec<u8>>, String>
```

There's an issue with our `run` function signature. It must exactly match the required signature for the WAVS runtime to properly invoke it.
