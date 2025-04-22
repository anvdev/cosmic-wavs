//! Examples and tests for working with Solidity types in WAVS components
//! 
//! This module demonstrates proper handling of Solidity types,
//! including definitions, conversions, and common pitfalls.

use alloy_primitives::{Address, Bytes, U256, hex};
use alloy_sol_types::{sol, SolCall, SolType, SolValue};
use std::str::FromStr;

// Define Solidity types for testing
sol! {
    struct TokenInfo {
        address tokenAddress;
        uint256 decimals;
        string symbol;
        bytes data;
    }
    
    struct UserData {
        address user;
        uint256 balance;
        bool isActive;
    }
    
    function transferToken(address token, address to, uint256 amount) external;
}

/// EXAMPLE 1: Defining and using Solidity structs
#[test]
fn test_solidity_struct_usage() {
    // Create a TokenInfo struct
    let token_info = TokenInfo {
        tokenAddress: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        decimals: U256::from(18),
        symbol: "ETH".to_string(),
        data: Bytes::from(vec![1, 2, 3, 4]),
    };
    
    // We can access fields directly
    assert_eq!(token_info.symbol, "ETH");
    assert_eq!(token_info.decimals, U256::from(18));
    
    // We can encode it to ABI format
    let encoded = token_info.abi_encode();
    
    // And decode it back - use fully qualified syntax to avoid ambiguity
    let decoded = <TokenInfo as SolValue>::abi_decode(&encoded, false).unwrap();
    
    // The values should match
    assert_eq!(decoded.tokenAddress, token_info.tokenAddress);
    assert_eq!(decoded.decimals, token_info.decimals);
    assert_eq!(decoded.symbol, token_info.symbol);
    assert_eq!(decoded.data, token_info.data);
}

/// EXAMPLE 2: Creating function calls
#[test]
fn test_solidity_function_calls() {
    // Create a transfer token call
    let transfer_call = transferTokenCall {
        token: Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(), // USDC
        to: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        amount: U256::from(1000000), // 1 USDC (6 decimals)
    };
    
    // Encode the call to ABI format
    let encoded = transfer_call.abi_encode();
    
    // You can decode it back - using SolCall trait for function calls
    let decoded = transferTokenCall::abi_decode(&encoded, false).unwrap();
    
    // The values should match
    assert_eq!(decoded.token, transfer_call.token);
    assert_eq!(decoded.to, transfer_call.to);
    assert_eq!(decoded.amount, transfer_call.amount);
}

/// EXAMPLE 3: Working with numeric types safely
#[test]
fn test_numeric_conversions() {
    // Create a U256 from a Rust primitive
    let amount: u64 = 1_000_000_000_000_000_000; // 1 ETH in wei
    
    // WRONG: Using .into() can fail for large numbers
    // let amount_uint: U256 = amount.into(); // This is fragile
    
    // CORRECT: Use parsing from string to handle any size safely
    let amount_uint: U256 = amount.to_string().parse().unwrap();
    assert_eq!(amount_uint, U256::from(1_000_000_000_000_000_000_u64));
    
    // CORRECT: For smaller values, from() is also safe
    let small_amount: u64 = 1000;
    let small_uint = U256::from(small_amount);
    assert_eq!(small_uint, U256::from(1000));
    
    // CORRECT: Working with decimals
    let decimals: u8 = 18;
    let divisor = calculate_divisor(decimals);
    assert_eq!(divisor, U256::from(10).pow(U256::from(18)));
    
    // Check the division works correctly
    let one_token = amount_uint / divisor;
    assert_eq!(one_token, U256::from(1));
}

/// EXAMPLE 4: Handling addresses
#[test]
fn test_address_handling() {
    // Creating addresses from hex strings
    let address_str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    
    // CORRECT: Using FromStr trait
    let address1 = Address::from_str(address_str).unwrap();
    
    // ALSO CORRECT: Using parse with type annotation
    let address2: Address = address_str.parse().unwrap();
    
    assert_eq!(address1, address2);
    
    // CORRECT: Creating from raw bytes
    let bytes = hex::decode("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    let address3 = Address::from_slice(&bytes);
    
    assert_eq!(address1, address3);
}

/// EXAMPLE 5: Using sol! macro for function and struct definitions
#[test]
fn test_sol_macro() {
    // You can define Solidity types where they're needed
    sol! {
        struct LocalStruct {
            uint256 id;
            string name;
        }
    }
    
    // Create an instance of the locally defined struct
    let local_struct = LocalStruct {
        id: U256::from(123),
        name: "Local type".to_string(),
    };
    
    assert_eq!(local_struct.id, U256::from(123));
    assert_eq!(local_struct.name, "Local type");
}

/// EXAMPLE 6: Handling ABI method ambiguity
#[test]
fn test_abi_method_ambiguity() {
    // Create a user data struct
    let user_data = UserData {
        user: Address::from_str("0x1234567890123456789012345678901234567890").unwrap(),
        balance: U256::from(1000),
        isActive: true,
    };
    
    // Encode it
    let encoded = user_data.abi_encode();
    
    // When using abi_decode, specify which trait implementation to use
    // to avoid "multiple applicable items in scope" errors
    let decoded = <UserData as SolValue>::abi_decode(&encoded, false).unwrap();
    
    assert_eq!(decoded.user, user_data.user);
    assert_eq!(decoded.balance, user_data.balance);
    assert_eq!(decoded.isActive, user_data.isActive);
}

// Helper function to calculate 10^decimals as U256
fn calculate_divisor(decimals: u8) -> U256 {
    let mut divisor = U256::from(1);
    for _ in 0..decimals {
        divisor = divisor * U256::from(10);
    }
    divisor
}