# WAVS Component Test Utilities

This library provides essential validation tools for WAVS components. All components **MUST** pass these tests before running the `make wasi-build` command.

## Purpose

These tests consolidate best practices and common error patterns from CLAUDE.md into executable tests, making it easier to:

1. Verify that your component follows proper implementation patterns
2. Avoid common pitfalls that cause build failures or runtime errors
3. Ensure consistent handling of data, errors, and networking

## Running Tests

To validate your component:

```bash
cd components/test_utils
./run_tests.sh
```

## Test Modules

The test utilities are organized into focused modules:

| Module | Description |
|--------|-------------|
| `abi_encoding` | Proper handling of ABI-encoded data, avoiding common String::from_utf8 errors |
| `code_quality` | Code quality checks, including detecting unused imports and other best practices |
| `data_handling` | Correct data ownership, cloning, and avoiding moved value errors |
| `error_handling` | Proper Option/Result handling, avoiding map_err on Option errors |
| `network_requests` | HTTP request setup, error handling, and API key management |
| `solidity_types` | Working with Solidity types, numeric conversions, and struct handling |
| `input_validation` | Input data validation, safe decoding, and defensive programming |

## Common Errors Prevented

These tests help you avoid the following common errors:

1. Using `String::from_utf8` directly on ABI-encoded data
2. Missing Clone derivation on API response structs 
3. Using `map_err()` on Option types instead of `ok_or_else()`
4. Improper Rust-Solidity type conversions
5. Ownership issues with collection elements
6. Using `&data.clone()` pattern creating temporary values
7. Missing trait imports causing "no method" errors
8. Ambiguous method calls requiring fully qualified syntax
9. Unused imports cluttering the code
10. Direct version specifications instead of workspace dependencies

## Integration

Add these lines to your component's Cargo.toml to add test dependencies:

```toml
[dev-dependencies]
test_utils = { path = "../test_utils" }
```

Then implement component-specific tests using the validation patterns shown in the test utilities.