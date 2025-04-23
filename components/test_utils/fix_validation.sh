#!/bin/bash
# Fix script for component validation
# This script fixes the issue with the test_utils crate not being found

echo "Fixing component validation script..."

# The issue is in the validation script trying to link to test_utils
# Let's modify the script to use cargo directly instead of a standalone Rust program

# Create a backup
cp validate_component.sh validate_component.sh.bak

# Replace the problematic section
sed -i.bak '85,120s/# Use the new verify_required_imports function via a simple Rust program.*/# Use cargo directly to check for code quality issues\ncd "..\/.."\nCOMPONENT_NAME_SIMPLE=$(basename "$COMPONENT_DIR")\ncargo check -p "$COMPONENT_NAME_SIMPLE" --message-format=json 2>\&1 | grep -i "unused\|import\|cannot find\|unresolved\|not in scope\|no function or associated item named\|no method named\|multiple applicable items in scope"\nCODE_CHECK_STATUS=$?\nif [ $CODE_CHECK_STATUS -eq 0 ]; then\n  echo "⚠️ Warning: Found potential code quality issues in component."\nelse\n  echo "✅ No code quality issues found"\nfi\ncd "$SCRIPT_DIR"/' validate_component.sh

# Make the script executable
chmod +x validate_component.sh

echo "Fix applied! You can now run 'make validate-component COMPONENT=your-component-name'"