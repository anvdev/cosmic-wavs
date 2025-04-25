# WAVSai Component Creation Process

This document outlines the exact process Claude should follow when creating a WAVS component to ensure error-free results.

## Process Overview

```
┌─────────────────────┐
│ 1. Understand      │
│    Requirements     │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 2. Research Phase   │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 3. Design Component │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 4. Pre-Validation   │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 5. Implementation   │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 6. Final Validation │
└──────────┬──────────┘
           ▼
┌─────────────────────┐
│ 7. Output Component │
└─────────────────────┘
```

## Detailed Process Steps

### 1. Understand Requirements

1. Parse user's natural language request
2. Identify:
   - Input type and format
   - External APIs or services needed
   - Output type and format
   - Error conditions to handle
3. Confirm understanding by restating requirements in technical terms
4. Classify component based on pre-defined patterns

### 2. Research Phase

1. For each external API or service:
   - Research API documentation
   - Identify authentication requirements
   - Document endpoint URLs and parameters
   - Document response formats
   - Identify potential error cases
   - Note rate limits or other constraints
2. Verify API functionality through examples
3. Determine required Rust crates and dependencies
4. Research any Rust-specific implementation details

### 3. Design Component

1. Define component structure:
   - Required imports
   - Data structures
   - Solidity type definitions
   - Component trait implementations
   - Helper functions
2. Create function declarations with explicit type signatures
3. Document error handling strategy
4. Define validation and testing approach

### 4. Pre-Validation Checks

**A. Architecture Validation:**
- [ ] Component fits WAVSai standard patterns
- [ ] External API calls are properly structured
- [ ] All error conditions are accounted for
- [ ] Data flow is clear and logical

**B. Dependency Validation:**
- [ ] All required dependencies are in Cargo.toml
- [ ] Workspace dependencies are used correctly
- [ ] No conflicting or redundant dependencies

**C. Type System Validation:**
- [ ] All types are properly defined
- [ ] No implicit type conversions
- [ ] Proper handling of blockchain-specific types
- [ ] ABI encoding/decoding is correct

**D. Memory Safety Validation:**
- [ ] All API response structures derive Clone
- [ ] Data is cloned before use where needed
- [ ] No movement out of borrowed data
- [ ] No dangerous string or memory operations

**E. Error Handling Validation:**
- [ ] ok_or_else for Option types
- [ ] map_err for Result types
- [ ] All errors are propagated properly
- [ ] No unwrapped Results or Options

**F. Component Interface Validation:**
- [ ] Component implements Guest trait correctly
- [ ] Export macro is used correctly
- [ ] Function signatures match requirements
- [ ] Component handles all trigger types

### 5. Implementation

1. Generate Cargo.toml file
2. Generate lib.rs file:
   - Start with all imports
   - Define Solidity types and interfaces
   - Implement data structures
   - Declare component and export it
   - Implement Guest trait
   - Implement helper functions
3. Add detailed comments for complex sections
4. Include error handling for all edge cases

### 6. Final Validation

1. Run code through common error check patterns:
   - Check for String::from_utf8 on ABI data
   - Verify Provider trait is imported along with RootProvider
   - Check numeric type conversions, especially with blockchain types
   - Verify all Option types use ok_or_else, not map_err
   - Ensure export macro uses correct format
   - Verify all data is cloned before use

2. Verify component against standard examples
3. Check for any remaining edge cases or errors
4. Ensure all standard validation criteria pass

### 7. Output Component

1. Present the Cargo.toml and lib.rs files
2. Provide clear instructions for running the component
3. Include explanation of design choices if needed
4. Show the exact command for testing the component

## Decision Tree for Common Components

### ENS Name Resolution

```
Requirements: Resolve ENS name to Ethereum address
APIs: ENS API (ensideas.com)
Input: ENS name string
Output: ENS name and resolved Ethereum address
Key Validations:
- Ensure ENS name is properly formatted
- Handle non-existent ENS names
- Handle API rate limiting
- Ensure proper JSON parsing
```

### Token Balance Checking

```
Requirements: Check token balance for wallet
APIs: Ethereum RPC
Input: Wallet address, token contract address
Output: Token balance and formatted amount
Key Validations:
- Verify wallet address format
- Handle contract errors
- Properly format token amounts with decimals
- Convert between U256 and more usable types
```

### Gas Price Estimation

```
Requirements: Estimate gas prices at different speeds
APIs: Ethereum RPC
Input: None or optional parameters
Output: Gas prices at different speeds
Key Validations:
- Properly query gas price using Provider trait
- Convert u128 gas price to U256 for calculations
- Format gas prices in gwei with correct decimals
- Provide reasonable time estimates
```

## Common Error Patterns and Solutions

### ABI Decoding Errors

```
Error: Failed to decode input
Solution: Implement cascading decode pattern:
1. Try to decode as function call first
2. Fall back to decoding as raw string
3. Provide clear error messages for debugging
```

### API Connection Errors

```
Error: Failed to connect to API
Solution: 
1. Include proper error handling for all HTTP requests
2. Add retries for transient failures
3. Provide clear error messages that include the endpoint
```

### Numeric Conversion Errors

```
Error: Cannot multiply u128 by U256
Solution:
1. Always explicitly convert numeric types
2. Use U256::from(u128_value) for conversions
3. Never rely on .into() for numeric conversions
```

### Output Formatting Errors

```
Error: Incorrect token decimal formatting
Solution:
1. Use proper decimal formatting logic
2. Handle edge cases like zero values
3. Avoid string capacity issues with bounds checking
```

## Final Component Quality Checklist

Before returning the final component, verify:

1. **Correctness**
   - [ ] Component implements all requirements
   - [ ] All edge cases are handled
   - [ ] All error conditions have appropriate messaging

2. **Safety**
   - [ ] No memory safety issues
   - [ ] No unchecked indexing or slicing
   - [ ] All external data is validated

3. **Performance**
   - [ ] No unnecessary cloning or copying
   - [ ] Efficient algorithm implementations
   - [ ] Proper async/await usage

4. **Maintainability**
   - [ ] Clear code structure
   - [ ] Detailed comments on complex sections
   - [ ] Consistent naming conventions

5. **Testability**
   - [ ] Clear test instructions
   - [ ] Well-defined inputs and outputs
   - [ ] Deterministic behavior

Only after ALL validation checks pass should the component be returned to the user.