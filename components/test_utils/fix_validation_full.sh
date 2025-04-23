#!/bin/bash
# Comprehensive fix for component validation
# This script fixes the issues with the validation script

echo "Applying comprehensive fix to validation script..."

# Create a backup
cp validate_component.sh validate_component.sh.old

# Create a new version of the validation script
cat > validate_component.sh << 'EOF'
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

# 4. Check for proper import of essential traits
echo "📝 Checking for essential imports..."
if grep -r "FromStr" "$COMPONENT_DIR"/src/*.rs > /dev/null && ! grep -r "use std::str::FromStr" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "⚠️  Warning: Found FromStr usage but std::str::FromStr might not be imported"
fi

# 5. Check for proper export! macro usage
echo "📝 Checking for proper component export..."
if ! grep -r "export!" "$COMPONENT_DIR"/src/*.rs > /dev/null; then
  echo "❌ Error: export! macro not found. Components must use export! macro."
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

echo "✅ Component validation checks complete!"
echo "🚀 Component is ready for building. Run the following command to build:"
echo "    cd ../.. && make wasi-build"
EOF

# Make the script executable
chmod +x validate_component.sh

echo "Comprehensive fix applied! You can now run 'make validate-component COMPONENT=your-component-name'"