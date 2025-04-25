# Square Number Component

A simple WAVS component that takes a number as input and returns the square of that number.

## Functionality

This component:
1. Accepts a string input containing a number
2. Parses the string to a u64 integer
3. Calculates the square of the number
4. Returns a JSON response with both the input and the result

## Testing

To test with the WASI executor:

```bash
# Using string input "5" (per CLAUDE.md guidelines)
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "5"`
export COMPONENT_FILENAME=square_number.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## Expected Output

```json
{
  "input": "5",
  "square": "25"
}
```

## Structure
- Implement Guest trait
- Parse input string to number
- Calculate square
- Return result

## Checklist

- ✅ Implement Guest trait and export component correctly
- ✅ Properly handle TriggerAction and TriggerData
- ✅ Properly decode ABI function calls
- ✅ Avoid String::from_utf8 on ABI data
- ✅ Derive Clone for response structure
- ✅ Clone data before use
- ✅ Use ok_or_else() for Option types
- ✅ Use map_err() for Result types
- ✅ Include all required imports
- ✅ Use proper sol! macro syntax
- ✅ No hardcoded secrets
- ✅ Use workspace dependencies correctly
- ✅ Handle numeric conversions safely
- ✅ Use .to_string() for string literals in struct
- ✅ Use block_on if needed for async

## Implementation Notes
- Accept string input (per CLAUDE.md guidelines)
- Parse string to u64
- Calculate square
- Return JSON result
