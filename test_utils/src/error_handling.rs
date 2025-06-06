//! Examples and tests for proper error handling in WAVS components
//! 
//! This module demonstrates proper error handling techniques,
//! especially for Option and Result types.

use std::collections::HashMap;

/// EXAMPLE 1: Proper handling of Option types
#[test]
fn test_option_handling() {
    let config_map = build_test_config();
    
    // WRONG: Using map_err() on Option types will cause a build error
    // The following line would fail to compile:
    // let wrong_config = get_eth_chain_config(&config_map, "mainnet").map_err(|e| e.to_string())?;
    
    // CORRECT: For Option types, use ok_or_else() to convert to Result
    let config_result = get_eth_chain_config(&config_map, "mainnet")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string());
    
    assert!(config_result.is_ok());
    let config = config_result.unwrap();
    assert_eq!(config, "https://eth-mainnet.example.com/v1/rpc");
    
    // CORRECT: Using if let for Option handling
    if let Some(config) = get_eth_chain_config(&config_map, "mainnet") {
        assert_eq!(config, "https://eth-mainnet.example.com/v1/rpc");
    } else {
        panic!("Should have found config");
    }
    
    // Test the negative case
    let missing_config_result = get_eth_chain_config(&config_map, "nonexistent")
        .ok_or_else(|| "Failed to get Ethereum chain config".to_string());
    
    assert!(missing_config_result.is_err());
    assert_eq!(
        missing_config_result.unwrap_err(),
        "Failed to get Ethereum chain config"
    );
}

/// EXAMPLE 2: Proper handling of Result types
#[test]
fn test_result_handling() {
    // CORRECT: For Result types, use map_err()
    let balance_result = fetch_balance("0x1234").map_err(|e| format!("Balance fetch failed: {}", e));
    
    assert!(balance_result.is_err());
    assert!(balance_result.unwrap_err().contains("Balance fetch failed"));
    
    // CORRECT: Chaining multiple results with the ? operator
    fn process_balance() -> Result<u64, String> {
        let balance = fetch_balance("0x5678").map_err(|e| format!("Balance fetch failed: {}", e))?;
        Ok(balance * 2)
    }
    
    let process_result = process_balance();
    assert!(process_result.is_err());
}

/// EXAMPLE 3: Providing meaningful error messages
#[test]
fn test_error_messages() {
    // BETTER: Provide context in error messages
    let result = parse_address("invalid-address").map_err(|e| {
        format!("Failed to parse address 'invalid-address': {}", e)
    });
    
    assert!(result.is_err());
    let error_msg = result.unwrap_err();
    
    // Error should contain both the original value and the error
    assert!(error_msg.contains("invalid-address"));
    assert!(error_msg.contains("Failed to parse address"));
}

/// EXAMPLE 4: Safe error propagation
#[test]
fn test_error_propagation() {
    // Test function that demonstrates proper error propagation
    fn fetch_and_process_data(address: &str) -> Result<u64, String> {
        // Step 1: Parse the address, with proper error context
        let _ = parse_address(address)
            .map_err(|e| format!("Invalid address format '{}': {}", address, e))?;
        
        // Step 2: Fetch balance, with proper error context
        let balance = fetch_balance(address)
            .map_err(|e| format!("Failed to fetch balance for {}: {}", address, e))?;
        
        // Step 3: Process the result
        Ok(balance)
    }
    
    let result = fetch_and_process_data("invalid");
    assert!(result.is_err());
    
    // Error should contain context from the appropriate step
    let error = result.unwrap_err();
    assert!(error.contains("Invalid address format 'invalid'"));
}

// Helper functions for the tests

fn build_test_config() -> HashMap<String, String> {
    let mut config = HashMap::new();
    config.insert("mainnet".to_string(), "https://eth-mainnet.example.com/v1/rpc".to_string());
    config.insert("goerli".to_string(), "https://eth-goerli.example.com/v1/rpc".to_string());
    config
}

fn get_eth_chain_config<'a>(config: &'a HashMap<String, String>, network: &str) -> Option<&'a String> {
    config.get(network)
}

fn fetch_balance(address: &str) -> Result<u64, &'static str> {
    // Simulated error for testing
    Err("RPC connection failed")
}

fn parse_address(address: &str) -> Result<String, &'static str> {
    // Simplified validation for testing
    if address.starts_with("0x") && address.len() >= 42 {
        Ok(address.to_string())
    } else {
        Err("Invalid Ethereum address format")
    }
}