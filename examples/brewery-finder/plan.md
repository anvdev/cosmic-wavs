# Brewery Finder Component Plan

## Overview
This component will query the OpenBrewery API to find breweries by zip code. It takes a zip code as input, queries the API, and returns information about breweries in that area.

## API Details
- API: OpenBrewery DB
- Endpoint: `https://api.openbrewerydb.org/v1/breweries?by_postal={postal_code}`
- No authentication required
- Response: JSON array of brewery objects

## Component Structure

### Imports
```rust
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};
```

### Solidity Types
```rust
sol! {
    function findBreweriesByZip(string zipCode) external;
}
```

### Response Structure
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Brewery {
    id: String,
    name: String,
    brewery_type: String,
    street: Option<String>,
    address_2: Option<String>,
    address_3: Option<String>,
    city: String,
    state: String,
    postal_code: String,
    country: String,
    longitude: Option<String>,
    latitude: Option<String>,
    phone: Option<String>,
    website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BreweryResponse {
    zip_code: String,
    breweries: Vec<Brewery>,
    count: usize,
    timestamp: String,
}
```

## Flow
1. Decode trigger event
2. Extract zip code from input
3. Query OpenBrewery API
4. Parse response
5. Format and return results

## Validation Checklist

1. Common errors:
   - [x] ALWAYS use `{ workspace = true }` in your component Cargo.toml. Explicit versions go in the root Cargo.toml.
   - [x] ALWAYS verify API response structures by using curl or the WebFetch tool on the endpoints.
   - [x] ALWAYS Read any documentation given to you in a prompt
   - [x] ALWAYS implement the Guest trait and export your component
   - [x] ALWAYS use `export!(Component with_types_in bindings)`
   - [x] ALWAYS use `clone()` before consuming data to avoid ownership issues
   - [x] ALWAYS derive `Clone` for API response data structures
   - [x] ALWAYS decode ABI data properly, never with `String::from_utf8`
   - [x] ALWAYS use `ok_or_else()` for Option types, `map_err()` for Result types
   - [x] ALWAYS use string parameters for CLI testing (`cast abi-encode "f(string)" "5"` instead of `f(uint256)`)
   - [x] ALWAYS use `.to_string()` to convert string literals (&str) to String types in struct field assignments
   - [x] NEVER edit bindings.rs - it's auto-generated

2. Component structure:
   - [x] Implements Guest trait
   - [x] Exports component correctly
   - [x] Properly handles TriggerAction and TriggerData

3. ABI handling:
   - [x] Properly decodes function calls
   - [x] Avoids String::from_utf8 on ABI data

4. Data ownership:
   - [x] All API structures derive Clone
   - [x] Clones data before use
   - [x] Avoids moving out of collections
   - [x] Avoids all ownership issues and "Move out of index" errors

5. Error handling:
   - [x] Uses ok_or_else() for Option types
   - [x] Uses map_err() for Result types
   - [x] Provides descriptive error messages

6. Imports:
   - [x] Includes all required traits and types
   - [x] Uses correct import paths
   - [x] Properly imports SolCall for encoding
   - [x] Each and every method and type is used properly and has the proper import
   - [x] Both structs and their traits are imported
   - [x] Verify all required imports are imported properly
   - [x] All dependencies are in Cargo.toml with `{workspace = true}`
   - [x] Any unused imports are removed

7. Component structure:
   - [x] Uses proper sol! macro with correct syntax
   - [x] Correctly defines Solidity types in solidity module
   - [x] Implements required functions

8. Security:
   - [x] No hardcoded API keys or secrets
   - [x] Uses environment variables for sensitive data (not needed for this API)

9. Dependencies:
   - [x] Uses workspace dependencies correctly
   - [x] Includes all required dependencies

10. Network requests:
    - [x] Uses block_on for async functions
    - [x] Uses fetch_json with correct headers
    - [x] ALL API endpoints have been tested with curl or the WebFetch tool and responses handled correctly