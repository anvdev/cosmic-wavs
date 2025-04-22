# NFT Checker Component Errors

## Error 1: Multiple applicable items in scope
When building the NFT checker component, encountered:

```
error[E0034]: multiple applicable items in scope
  --> components/nft-checker/src/lib.rs:30:55
   |
30 |             let trigger_info = solidity::TriggerInfo::abi_decode(&event._triggerInfo, false)?;
   |                                                       ^^^^^^^^^^ multiple `abi_decode` found
```

This error occurs because multiple `abi_decode` methods are available for the `TriggerInfo` type:
1. From the `SolCall` trait
2. From an implementation of the `SolType` trait for `ITypes::TriggerInfo`
3. From an implementation of the `SolValue` trait for `ITypes::TriggerInfo`

Fix: Use fully-qualified syntax to specify which trait implementation to use:
```rust
<solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
```

## Error 2: Index out of range
When running the component, encountered:

```
thread '<unnamed>' panicked at components/nft-checker/src/lib.rs:86:44:
range end index 813195944 out of range for slice of length 128
```

This error occurred because we tried to access data beyond the boundaries of the input. The issue was in the way we were handling the ABI-encoded input string.

Fix: Implemented more robust input handling with proper bounds checking:
1. Added debug output for input data
2. Implemented safer bounds checking before accessing any parts of the input
3. Added fallback values when input data couldn't be properly parsed
4. Made the component more resilient to different input formats

## Error 3: NFT Ownership Checker - Range Index Out of Bounds
When running the NFT ownership checker component, encountered:

```
thread '<unnamed>' panicked at components/nft-ownership-checker/src/lib.rs:91:51:
range end index 813185193 out of range for slice of length 160
```

This error occurred in the ABI string decoding logic. The component was trying to access bytes far beyond the boundaries of the input data (813185193 vs 160 available bytes).

Root causes:
1. Incorrect length calculation from ABI-encoded input
2. Missing bounds checking when accessing the data slice
3. Improper handling of the data format

Fix: Implemented a robust `safely_decode_abi_string` function with:
1. Detailed debug logging for input analysis
2. Proper bounds checking at each step of decoding
3. Sanity checks on decoded length values
4. Safe access to data with `saturating_sub` and `std::cmp::min` to prevent out-of-bounds access
5. Step-by-step validation of the ABI encoding structure

The improved implementation handles various edge cases and provides detailed diagnostic information when errors occur, making it easier to debug future issues.

## Error 4: NFT Ownership Checker - Rust Lifetime Error
When building the nft-ownership-checker component, encountered a Rust lifetime error:

```
error[E0597]: `req_clone` does not live long enough
   --> components/nft-ownership-checker/src/lib.rs:111:37
    |
71  |         let wallet_address_str = if req.len() >= 68 {
    |             ------------------ borrow later stored here
...
110 |                 let req_clone = string_data.to_vec();
    |                     --------- binding `req_clone` declared here
111 |                 std::str::from_utf8(&req_clone)
    |                                     ^^^^^^^^^^ borrowed value does not live long enough
112 |                     .map_err(|e| format!("Invalid UTF-8 in string: {}", e))?
113 |             } else {
    |             - `req_clone` dropped here while still borrowed
```

This error occurred because we were trying to create a string slice (`&str`) from a temporary vector that would be dropped at the end of the if-block scope. However, we were attempting to return this reference outside that scope, which Rust doesn't allow because it would leave a dangling reference.

Fix: Created and returned a fully owned String instead of a string slice:

```rust
// WRONG - Trying to return a reference to a temporary value
let req_clone = string_data.to_vec();
std::str::from_utf8(&req_clone).map_err(|e| format!("Invalid UTF-8 in string: {}", e))?

// CORRECT - Create and return a fully owned String
String::from_utf8(string_data.to_vec()).map_err(|e| format!("Invalid UTF-8 in string: {}", e))?
```

## Error 5: NFT Ownership Checker - Type Mismatch
After fixing the lifetime issue, encountered a type mismatch error:

```
error[E0308]: mismatched types
   --> components/nft-ownership-checker/src/lib.rs:128:37
    |
128 |                 check_nft_ownership(wallet_address_str, nft_contract_str).await?;
    |                 ------------------- ^^^^^^^^^^^^^^^^^^ expected `&str`, found `String`
```

The issue was that our check_nft_ownership function expected a &str parameter, but we were passing in a String value.

Fix: Added a reference operator to match the expected function signature:

```rust
// WRONG - Passing a String to a function that expects &str
check_nft_ownership(wallet_address_str, nft_contract_str).await?

// CORRECT - Use a reference to String which coerces to &str
check_nft_ownership(&wallet_address_str, nft_contract_str).await?
```

These fixes resolved all compilation errors and the component was successfully built.

## Error 6: USDT Balance Component - Capacity Overflow
When running the USDT balance component, encountered:

```
Input length: 128 bytes
First 8 bytes: 00 00 00 00 00 00 00 00
Decoded address input: 0x28C6c06298d514Db089934071355E5743bf21d60

thread '<unnamed>' panicked at /Users/evan/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/src/rust/library/alloc/src/slice.rs:569:23:
capacity overflow
```

This error occurs in our component when trying to format the fractional part of the USDT balance. 
The panic is happening in an allocation-related function, specifically in the `repeat` method which 
is likely being called when we do `"0".repeat(padding)`. 

The issue appears to be that the padding value (calculated as `decimals as usize - fractional_str.len()`) 
is extremely large, possibly because:
1. The balance might be very small, resulting in a fractional_str with very few digits
2. The padding calculation might be trying to repeat "0" an unreasonable number of times

This suggests we need to improve our balance formatting logic to handle extreme cases more safely.

## Error 7: USDT Balance Checker - Unused Import Warning
When building the usdt-balance-checker component, encountered:

```
warning: unused import: `sol`
 --> components/usdt-balance-checker/src/lib.rs:7:23
  |
7 | use alloy_sol_types::{sol, SolCall, SolValue};
  |                       ^^^
  |
  = note: `#[warn(unused_imports)]` on by default
```

Fix: Removed the unused `sol` import and updated the import statement to `use alloy_sol_types::{SolCall, SolValue};`.

## Error 8: USDT Balance Component - TxKind Import Error
When building the usdt-balance component, encountered:

```
error[E0433]: failed to resolve: could not find `TxKind` in `eth`
   --> components/usdt-balance/src/lib.rs:139:40
    |
139 |         to: Some(alloy_rpc_types::eth::TxKind::Call(usdt_address)),
    |                                        ^^^^^^ could not find `TxKind` in `eth`
    |
help: consider importing this enum
    |
1   + use alloy_primitives::TxKind;
    |
help: if you import `TxKind`, refer to it directly
    |
139 -         to: Some(alloy_rpc_types::eth::TxKind::Call(usdt_address)),
139 +         to: Some(TxKind::Call(usdt_address)),
    |
```

This error occurred because TxKind is defined in the alloy_primitives crate, not in alloy_rpc_types::eth as we tried to use it.

Fix:
1. Added `TxKind` to the imports from alloy_primitives: `use alloy_primitives::{Address, Bytes, TxKind, U256};`
2. Updated the transaction request to use the correct type: `to: Some(TxKind::Call(usdt_address))`

## Error 9: NFT Ownership Checker - Struct Missing abi_encode Method
When building the nft-ownership-checker component, encountered:

```
error[E0599]: no method named `abi_encode` found for struct `NFTOwnershipResult` in the current scope
   --> components/nft-ownership-checker/src/lib.rs:105:95
    |
105 |             Destination::Ethereum => Some(encode_trigger_output(trigger_id, &ownership_result.abi_encode())),
    |                                                                                               ^^^^^^^^^^
...
113 | struct NFTOwnershipResult {
    | ------------------------- method `abi_encode` not found for this struct
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following traits define an item `abi_encode`, perhaps you need to implement one of them:
            candidate #1: `SolCall`
            candidate #2: `SolConstructor`
            candidate #3: `SolEnum`
            candidate #4: `SolError`
            candidate #5: `SolInterface`
            candidate #6: `SolType`
            candidate #7: `SolValue`
```

This error occurred because we defined a regular Rust struct `NFTOwnershipResult` and tried to use the `abi_encode()` method on it, but this method is only available for Solidity-compatible types defined with the `sol!` macro.

Fix:
1. Replace the custom Rust struct with a Solidity struct definition using the `sol!` macro
2. Update the struct usage to properly handle the Solidity type