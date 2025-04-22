#!/bin/bash
# Run all tests in the test_utils crate

cd "$(dirname "$0")"
cargo test -- --nocapture