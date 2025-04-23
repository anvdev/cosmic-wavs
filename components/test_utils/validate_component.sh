#!/bin/bash
# Component validation script - FIXED VERSION
# Runs test utilities to validate a component before building

set -e  # Exit on any error

if [ -z "$1" ]; then
  echo "Usage: $0 <component-directory-name>"
  echo "Example: $0 eth-price-oracle"
  exit 1
fi

COMPONENT_NAME=$1
COMPONENT_DIR="../$COMPONENT_NAME"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if component directory exists
if [ ! -d "$COMPONENT_DIR" ]; then
  echo "❌ Error: Component directory $COMPONENT_DIR not found"
  exit 1
fi

echo "🔍 Validating component: $COMPONENT_NAME"

# 1. Check for String::from_utf8 usage on ABI data
echo "📝 Checking for common String::from_utf8 misuse..."
grep_result=$(grep -r "String::from_utf8" "$COMPONENT_DIR" | grep -v "test" | grep -v "# CORRECT" || true)
if [ ! -z "$grep_result" ]; then
  echo "⚠️  Warning: Found String::from_utf8 usage. Ensure it's not being used on ABI-encoded data."
  echo "$grep_result"
fi

# 2. Check for proper Clone derivation on API structs
echo "📝 Checking for Clone derivation on structs..."
STRUCTS_WITHOUT_CLONE=$(grep -r -A 5 "struct" "$COMPONENT_DIR/src" | grep -B 5 "Deserialize" | grep -v "Clone" || true)
if [ ! -z "$STRUCTS_WITHOUT_CLONE" ]; then
  echo "⚠️  Warning: Found structs that might be missing Clone derivation:"
  echo "$STRUCTS_WITHOUT_CLONE"
fi

# 3. Check for map_err on Option types
echo "📝 Checking for map_err on Option types..."
if grep -r "get_eth_chain_config.*map_err" "$COMPONENT_DIR" > /dev/null; then
  echo "❌ Error: Found map_err used on Option types. Use ok_or_else instead."
  grep -r "get_eth_chain_config.*map_err" "$COMPONENT_DIR"
  exit 1
fi

# 4. Check for proper import of essential traits and types
echo "📝 Checking for essential imports..."
if grep -r "FromStr" "$COMPONENT_DIR"/src/*.rs > /dev/null && ! grep -r "use std::str::FromStr" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "⚠️  Warning: Found FromStr usage but std::str::FromStr might not be imported"
fi

# Check for TxKind import issues - specific check for common error
if grep -r "alloy_rpc_types::eth::TxKind" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "❌ Error: Found incorrect TxKind import path. Use alloy_primitives::TxKind instead of alloy_rpc_types::eth::TxKind."
  echo "This is a critical error that will prevent component compilation."
  echo "Fix: 1. Add 'use alloy_primitives::{Address, TxKind, U256};' (or add TxKind to existing import)"
  echo "     2. Replace 'alloy_rpc_types::eth::TxKind::Call' with 'TxKind::Call'"
  grep -r "alloy_rpc_types::eth::TxKind" "$COMPONENT_DIR"/src/*.rs
  exit 1
fi

# Check for TxKind usage without import
if grep -r "::Call" "$COMPONENT_DIR"/src/*.rs > /dev/null && ! grep -r "use.*TxKind" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "⚠️  Warning: Found potential TxKind usage but TxKind might not be properly imported."
  echo "Make sure to import TxKind from alloy_primitives: use alloy_primitives::TxKind;"
fi

# 5. Check for proper export! macro usage and syntax
echo "📝 Checking for proper component export..."
if ! grep -r "export!" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "❌ Error: export! macro not found. Components must use export! macro."
  exit 1
fi

# Check for correct export! macro syntax with with_types_in
if grep -r "export!" "$COMPONENT_DIR"/src/*.rs > /dev/null && ! grep -r "export!.*with_types_in bindings" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "❌ Error: Incorrect export! macro syntax. Use 'export!(YourComponent with_types_in bindings)' instead of just 'export!(YourComponent)'."
  grep -r "export!" "$COMPONENT_DIR"/src/*.rs
  exit 1
fi

# 6. Check for hardcoded API keys
echo "📝 Checking for hardcoded API keys..."
if grep -r "key=.*[0-9a-zA-Z]\{8,\}" "$COMPONENT_DIR" --include="*.rs" > /dev/null; then
  echo "❌ Error: Found possible hardcoded API key. Use environment variables instead."
  grep -r "key=.*[0-9a-zA-Z]\{8,\}" "$COMPONENT_DIR" --include="*.rs"
  exit 1
fi

# 7. Check for proper workspace dependency usage
echo "📝 Checking for proper workspace dependency usage..."
if grep -r "version = \"[0-9]" "$COMPONENT_DIR/Cargo.toml" > /dev/null; then
  echo "⚠️  Warning: Found direct version numbers in Cargo.toml. Use workspace = true instead."
  grep -r "version = \"[0-9]" "$COMPONENT_DIR/Cargo.toml"
fi

# 8. Check for unused imports with cargo check
echo "📝 Checking for unused imports and code issues..."
cd "$SCRIPT_DIR/../.."
COMPONENT_NAME_SIMPLE=$(basename "$COMPONENT_DIR")

# Simply use cargo check to find warnings/errors
cargo check -p "$COMPONENT_NAME_SIMPLE" 2>&1 | grep -i "warning\|error" || true

cd "$SCRIPT_DIR"

# 9. Check for unbounded string.repeat operations
echo "📝 Checking for string capacity overflow risks..."
if grep -r "\.repeat(" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  # First, collect all .repeat() calls
  ALL_REPEATS=$(grep -r "\.repeat(" "$COMPONENT_DIR"/src/*.rs | grep -v "\"\.repeat" || true)
  
  # Define patterns that indicate safety
  SAFETY_PATTERNS="std::cmp::min\|::min(\|min(\|if \|safe_\|bounded_\|max_\|limit\|const \|// SAFE:"
  
  # Define risky variable patterns that often lead to unbounded repetition
  RISKY_VARS="len\|size\|count\|width\|height\|padding\|indent\|offset\|spaces\|zeros\|chars\|decimals\|digits"
  
  # Check each .repeat() call for safety
  echo "$ALL_REPEATS" | while read -r line; do
    # Skip if it contains a safety pattern
    if echo "$line" | grep -q "$SAFETY_PATTERNS"; then
      continue
    fi
    
    # Check if it contains any risky variable
    if echo "$line" | grep -q -i "$RISKY_VARS"; then
      # Collect file and line number for error reporting
      file_path=$(echo "$line" | cut -d':' -f1)
      line_num=$(echo "$line" | cut -d':' -f2)
      
      # Get a few lines before for context (to check for bounds checks above)
      context_before=$(tail -n +$((line_num - 5)) "$file_path" 2>/dev/null | head -n 5)
      
      # If the context contains safety patterns, it might be safe
      if echo "$context_before" | grep -q "$SAFETY_PATTERNS"; then
        continue
      fi
      
      # If we reach here, we have a potentially unsafe .repeat() call
      echo "❌ Error: Found potentially unbounded string.repeat operation in $file_path line $line_num:"
      echo "$line" | sed 's/^[^:]*:[^:]*://'
      echo 
      echo "This can cause capacity overflow errors. Options to fix:"
      echo "  1. Add a direct safety check: \".repeat(std::cmp::min(variable, 100))\""
      echo "  2. Use a bounded variable: \"let safe_value = std::cmp::min(variable, MAX_SIZE); .repeat(safe_value)\""
      echo "  3. Add a safety comment if manually verified: \"// SAFE: bounded by check above\""
      exit 1
    fi
  done
fi

echo "✅ Component validation checks complete!"
echo "🚀 Component is ready for building. Run the following command to build:"
echo "    cd ../.. && make wasi-build"
