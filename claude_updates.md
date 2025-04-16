# Proposed Updates to WAVS Component Documentation

Based on our experience building a weather API component, here are proposed updates to the documentation to help developers avoid common issues:

## 1. Input Data Handling

### Input Data Formatting and Handling

When testing components with `make wasi-exec`, you must format the input data appropriately for your component:

```markdown
### Input Data Formatting and Handling

When testing components with `make wasi-exec`, you must format the input data appropriately for your component:

1. **String inputs (bytes32)**: Use `cast format-bytes32-string` for string inputs up to 31 characters:
   ```bash
   export TRIGGER_DATA_INPUT=`cast format-bytes32-string "90210"` # For zip codes, short strings, etc.
   ```

2. **Numeric inputs (uint256, etc.)**: Use `cast abi-encode` for numeric types:
   ```bash
   export TRIGGER_DATA_INPUT=`cast abi-encode "f(uint256)" 123456` # For numeric inputs
   ```

3. **Custom struct inputs**: Use `cast abi-encode` with appropriate type definition:
   ```bash
   export TRIGGER_DATA_INPUT=`cast abi-encode "f((uint256,string))" 123 "example"` # For struct (uint256,string)
   ```

4. **Raw hex data**: Provide hex-encoded data directly:
   ```bash
   export TRIGGER_DATA_INPUT="0x1234abcd" # For custom binary formats
   ```

Always ensure your component correctly decodes the input format you've chosen:

```rust
// For bytes32 string inputs (with null byte padding)
let raw_string = String::from_utf8(trigger_data.clone())
    .map_err(|e| format!("Failed to parse data: {}", e))?;
let clean_string = raw_string.trim_end_matches('\0');

// For numeric/complex inputs via ABI encoding
let decoded_value = YourType::abi_decode(&trigger_data, false)?;
```

### Working with Bytes32 Input Data

When testing components with `make wasi-exec` and `cast format-bytes32-string`, your input will include null bytes (0x00) as padding to 32 bytes. Always trim these when converting to strings:

```rust
// Convert bytes to string and trim null bytes
let raw_string = String::from_utf8(trigger_data.clone())
    .map_err(|e| format!("Failed to parse data: {}", e))?;

// Get only the actual string by trimming the null bytes
let clean_string = raw_string.trim_end_matches('\0');
```

This is especially important when using the string in URLs or other contexts that require clean strings.
```

## 2. Rust Ownership

Add a section on Rust ownership considerations:

```markdown
### Rust Ownership Considerations

Remember that Rust's ownership system will move values when they're passed to functions like `String::from_utf8()`. If you need to use that data again:

```rust
// WRONG: This will fail with "use of moved value"
let data_string = String::from_utf8(trigger_data)
    .map_err(|e| format!("Failed to parse data: {}", e))?;
println!("Raw bytes: {:?}", trigger_data); // Error: trigger_data was moved

// CORRECT: Clone the data before consumption
let data_string = String::from_utf8(trigger_data.clone())
    .map_err(|e| format!("Failed to parse data: {}", e))?;
println!("Raw bytes: {:?}", trigger_data); // Works fine
```

When dealing with large data, consider if you actually need the original data after conversion, as cloning has performance implications.
```

## 3. Type Conversion Between Solidity and Rust

Expand the type conversion section:

```markdown
### Type Conversion Between Solidity and Rust

When working with Solidity types in Rust, pay special attention to:

1. **Bytes Conversion**: When returning data to Ethereum, `Vec<u8>` must be converted to `Bytes`:

```rust
// WRONG: This will cause a type mismatch error
let data = DataWithId {
    triggerId: trigger_id,
    data: weather_data.abi_encode(), // Error: expected Bytes, found Vec<u8>
};

// CORRECT: Convert Vec<u8> to Bytes using .into()
let data = DataWithId {
    triggerId: trigger_id,
    data: weather_data.abi_encode().into(), // Works correctly
};
```

2. **Field Access in Generated Types**: Always check the actual structure in the Solidity interface file. Field names and structures in generated Rust types exactly match the Solidity definitions:

```rust
// WRONG: Assuming field structure without checking ITypes.sol
event._triggerInfo.triggerId // May fail if structure doesn't match

// CORRECT: First understand the Solidity structure, then access
// Example: If NewTrigger event emits bytes, you'll need to decode those bytes first
event._triggerInfo.to_vec() // Access raw bytes for further processing
```

### Event Structure and Field Access

Always check the actual structure in Solidity interface files before accessing fields. For example, in ITypes.sol:

```solidity
event NewTrigger(bytes _triggerInfo);
```

This event doesn't directly provide `triggerId` and `data` fields. You must first decode the bytes:

```rust
// CORRECT approach:
// 1. Decode the event to get the bytes parameter
let event: solidity::NewTrigger = decode_event_log_data!(log)?;
// 2. Decode the bytes into the actual struct
let trigger_info = solidity::TriggerInfo::abi_decode(&event._triggerInfo, false)?;
// 3. Now you can access the fields
let trigger_id = trigger_info.triggerId;
let data = trigger_info.data;
```

Assuming the structure without checking will cause compilation errors.
```

## 4. Example Pattern for Raw Data Handling

Add a concrete example pattern for handling raw binary data:

```markdown
### Complete Pattern for Handling Raw Input Data

Here's a robust pattern for handling raw input data in components, particularly useful when handling string inputs via `cast format-bytes32-string`:

```rust
// In lib.rs
fn run(action: TriggerAction) -> Result<Option<Vec<u8>>, String> {
    // Decode the trigger event
    let (trigger_id, trigger_data, destination) = decode_trigger_event(action.data)?;
    
    // Debug raw data (useful during development)
    println!("Raw data bytes: {:?}", &trigger_data);
    
    // Convert to string, trimming null bytes
    let raw_string = String::from_utf8(trigger_data.clone())
        .map_err(|e| format!("Failed to parse input: {}", e))?;
    let clean_input = raw_string.trim_end_matches('\0');
    
    println!("Cleaned input: '{}'", clean_input);
    
    // Process data with your component logic
    let result = process_data(clean_input)?;
    
    // Encode for appropriate destination (Ethereum or CLI)
    let encoded = encode_data_for_destination(trigger_id, result, destination)?;
    
    Ok(Some(encoded))
}
```
```

## 5. Testing Strategy

Add a section on incremental testing:

```markdown
### Incremental Testing Strategy

When developing a new component, follow this testing strategy to identify issues early:

1. **Build Early and Often**: Use `make wasi-build` frequently to catch compilation errors early.

2. **Add Debug Logging**: Use `println!` liberally to inspect data at every transformation stage.

3. **Test Input Data**: Verify that your input data is properly decoded:
   ```rust
   println!("Raw input: {:?}", trigger_data);
   println!("String input: '{}'", input_string);
   println!("Input length: {}", input_string.len());
   ```

4. **Test Environment Variables**: Verify environment variables are accessible:
   ```rust
   match env::var("WAVS_ENV_API_KEY") {
       Ok(key) => println!("API key found, length: {}", key.len()),
       Err(e) => println!("API key error: {}", e),
   }
   ```

5. **Validate External API Requests**: Print the full URL and request details before making calls.

6. **Inspect Returned Data**: Print the API response format and structure to ensure your parsing will work correctly.

This diagnostic approach will help you identify issues at each stage of development, making debugging much easier.
```

## 6. Common Errors Checklist

Add a checklist section:

```markdown
### Component Building Checklist

Before running `make wasi-exec`, verify:

- ✅ All string inputs are properly trimmed of null bytes (important for bytes32 inputs)
- ✅ All type conversions between Solidity and Rust types are properly handled (Vec<u8> to Bytes, etc.)
- ✅ When consuming data, it's cloned if needed again later (Rust ownership rules)
- ✅ Environment variables are correctly specified in the SERVICE_CONFIG and .env file
- ✅ URLs in HTTP requests are properly formatted and encoded
- ✅ Your input data format (TRIGGER_DATA_INPUT) matches what your component expects
- ✅ Component's Cargo.toml includes all required dependencies
```

## 7. Best Practices for Component Development

Add comprehensive best practices:

```markdown
## Best Practices for Component Development

To ensure you build components correctly the first time:

### 1. Progressive Development and Testing

- **Start with skeleton implementation**: Begin with a basic component that just logs raw input
- **Test incrementally**: Build and test after implementing each step of your component logic
- **Use inspection points**: Add `println!` statements at key transformation points
- **Validate environment variables early**: Check API keys and endpoints before making requests

### 2. Pre-Implementation Planning

- **Document data flow**: Map out exactly how data transforms from trigger input to final output
- **Identify type conversions**: Note every point where data types change
- **List error cases**: Document all possible failure points and how they'll be handled
- **Mock API responses**: Create sample JSON responses to test parsing before integration

### 3. Defensive Coding Patterns

- **Handle all Result/Option types**: Never use `.unwrap()` or `.expect()` in production code
- **Validate input formats**: Check lengths, formats and values before processing
- **Add explicit error messages**: Always include context in error strings (e.g., "Failed to parse ZIP code: {}")
- **Use type-safe conversions**: Avoid direct casting between numeric types

### 4. Security Considerations

- **Never log API keys or sensitive data**: Redact secrets in all log statements
- **Validate all external inputs**: Sanitize any user-provided data
- **Handle large inputs safely**: Set size limits for inputs to prevent resource exhaustion
- **Use timeouts for external services**: Prevent hanging on network requests

### 5. Pre-Deployment Checklist

Before finalizing your component:
- ✅ Remove debug `println!` statements
- ✅ Verify error handling for all external API calls
- ✅ Test with edge cases (empty input, malformed input, etc.)
- ✅ Check resource usage (memory allocation, computation time)
- ✅ Ensure all secrets are properly stored in environment variables
- ✅ Validate component output format matches submission contract expectations
```

These updates will help developers avoid common issues when building WAVS components.
