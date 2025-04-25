# ENS Domain Resolver Component Plan

## Overview
This component will resolve Ethereum Name Service (ENS) domains to addresses and metadata. It will:
1. Accept an ENS name (like "vitalik.eth") and return the corresponding Ethereum address
2. Include additional ENS metadata like avatar, expiration date, and other profile information
3. Support reverse lookups from addresses to ENS names

## Implementation Details

### Data Flow
1. Receive input (ENS name or ETH address)
2. Determine if it's a forward (ENS → address) or reverse (address → ENS) lookup
3. Call appropriate ENS API endpoint
4. Parse and format the response
5. Return data in standardized JSON format

### API Selection
We'll use the ENS Graph API (`https://api.thegraph.com/subgraphs/name/ensdomains/ens`) for basic resolution and the ENS Metadata Service for richer profile data.

### Component Structure
- Input function: `resolveEnsDomain(string input)` (accepts either ENS name or ETH address)
- Response structure: JSON with resolved data and metadata
- Error handling: Clear error messages for resolution failures

## Validation Checklist

1. Component structure:
   - [x] Implements Guest trait
   - [x] Exports component correctly
   - [x] Properly handles TriggerAction and TriggerData

2. ABI handling:
   - [x] Properly decodes function calls
   - [x] Avoids String::from_utf8 on ABI data

3. Data ownership:
   - [x] All API structures derive Clone
   - [x] Clones data before use
   - [x] Avoids moving out of collections

4. Error handling:
   - [x] Uses ok_or_else() for Option types
   - [x] Uses map_err() for Result types
   - [x] Provides descriptive error messages

5. Imports:
   - [x] Includes all required traits and types
   - [x] Uses correct import paths
   - [x] Properly imports SolCall for encoding
   - [x] Each and every method and type is used properly and has the proper import
   - [x] Both structs and their traits are imported
   - [x] Verify all required imports are imported properly
   - [x] All dependencies are in Cargo.toml with `{workspace = true}`
   - [x] Any unused imports are removed

6. Component structure:
   - [x] Uses proper sol! macro with correct syntax
   - [x] Correctly defines Solidity types in solidity module
   - [x] Implements required functions

7. Security:
   - [x] No hardcoded API keys or secrets
   - [x] Uses environment variables for sensitive data

8. Dependencies:
   - [x] Uses workspace dependencies correctly
   - [x] Includes all required dependencies

9. Solidity types:
   - [x] Properly imports sol macro
   - [x] Uses solidity module correctly
   - [x] Handles numeric conversions safely
   - [x] Uses .to_string() for all string literals in struct initialization

10. Network requests:
    - [x] Uses block_on for async functions
    - [x] Uses fetch_json with correct headers
    - [x] Handles API responses correctly

## Implementation Plan

1. Create Cargo.toml with required dependencies
2. Implement lib.rs with:
   - Proper imports and Solidity type definitions
   - Component struct and Guest trait implementation
   - Helper functions for ENS resolution and data formatting
   - Error handling with descriptive messages
3. Validate component
4. Build component
5. Test with sample ENS names and addresses
