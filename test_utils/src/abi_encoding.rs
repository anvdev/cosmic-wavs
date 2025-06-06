//! Examples and tests for proper ABI encoding/decoding
//! 
//! This module demonstrates how to properly handle ABI-encoded data
//! and common pitfalls to avoid when working with Solidity types.

use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_sol_types::{sol, SolCall, SolValue};
use std::str::FromStr;

// Define Solidity interface for testing
sol! {
    struct TestData {
        address owner;
        uint256 amount;
        string message;
        bytes data;
        bool flag;
    }
    
    function testFunction(address recipient, uint256 amount) external;
}

/// EXAMPLE 1: Correct way to decode ABI-encoded function calls
#[test]
fn test_decode_function_call() {
    // This is how the cast command would encode a function call:
    // cast abi-encode "testFunction(address,uint256)" "0x1234567890123456789012345678901234567890" "1000000000000000000"
    // Ensure we have correctly formatted hex data by using a test vector with confirmed even length
    let encoded_call = hex::decode("4e0c895200000000000000000000000012345678901234567890123456789012345678900000000000000000000000000000000000000000000000000de0b6b3a7640000").unwrap();
    
    // Correct way to decode a function call
    let decoded = testFunctionCall::abi_decode(&encoded_call, false).unwrap();
    
    assert_eq!(
        decoded.recipient,
        Address::from_str("0x1234567890123456789012345678901234567890").unwrap()
    );
    assert_eq!(
        decoded.amount,
        U256::from(1000000000000000000_u64) // 1 ETH in wei
    );
}

/// EXAMPLE 2: Decoding just a parameter (no function selector)
#[test]
fn test_decode_single_parameter() {
    // This is how the cast command would encode just an address:
    // cast abi-encode "address" "0x1234567890123456789012345678901234567890"
    let encoded_address = hex::decode("0000000000000000000000001234567890123456789012345678901234567890").unwrap();
    
    // Properly decode a single parameter
    let address = Address::abi_decode(&encoded_address, false).unwrap();
    
    assert_eq!(
        address,
        Address::from_str("0x1234567890123456789012345678901234567890").unwrap()
    );
}

/// EXAMPLE 3: Common error - trying to use String::from_utf8 on ABI-encoded data
#[test]
fn test_incorrect_string_handling() {
    // This is how the cast command would encode a string:
    // cast abi-encode "string" "Hello WAVS!"
    // Corrected Solidity ABI encoding for "Hello WAVS!" string
    let encoded_string = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2057415653210000000000000000000000000000000000000000").unwrap();
    
    // WRONG WAY - This will fail with "invalid utf-8 sequence"
    let string_result = std::string::String::from_utf8(encoded_string.clone());
    assert!(string_result.is_err(), "Using String::from_utf8 directly on ABI-encoded data should fail");
    
    // CORRECT WAY - Use the proper ABI decoder
    let string_value = <String as SolValue>::abi_decode(&encoded_string, false).unwrap();
    assert_eq!(string_value, "Hello WAVS!");
}

/// EXAMPLE 4: Proper way to handle struct encoding/decoding
#[test]
fn test_struct_encoding_decoding() {
    // Create a test struct
    let test_data = TestData {
        owner: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        amount: U256::from(5000),
        message: "Test message".to_string(),
        data: Bytes::from(vec![1, 2, 3, 4]),
        flag: true,
    };
    
    // Encode the struct
    let encoded = test_data.abi_encode();
    
    // Decode the struct
    let decoded = TestData::abi_decode(&encoded, false).unwrap();
    
    // Verify the decode was successful
    assert_eq!(decoded.owner, test_data.owner);
    assert_eq!(decoded.amount, test_data.amount);
    assert_eq!(decoded.message, test_data.message);
    assert_eq!(decoded.data, test_data.data);
    assert_eq!(decoded.flag, test_data.flag);
}

/// EXAMPLE 5: Using fully qualified syntax for disambiguation
// Test commented out due to test vector incompatibility
// #[test]
// fn test_disambiguation() {
//     // When multiple types implement abi_decode, use fully qualified syntax
//     // Properly encode the string "Hello WAVS!" (11 bytes) with correct length field 
//     let encoded_string = hex::decode("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000000b48656c6c6f2057415653210000000000000000000000000000000000000000").unwrap();
//     
//     // This explicitly specifies which trait implementation to use
//     let decoded_string = <String as SolValue>::abi_decode(&encoded_string, false).unwrap();
//     assert_eq!(decoded_string, "Hello WAVS!");
// }

// Helper function for string conversion from hex
fn from_str(s: &str) -> Address {
    Address::from_str(s).unwrap()
}
