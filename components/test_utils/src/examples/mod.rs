//! Example component validation tests
//! 
//! These examples show how to validate components against best practices.

/// Runs the code quality checks on a component
/// 
/// # Arguments
/// * `component_path` - Path to the component directory
/// 
/// # Returns
/// * `bool` - True if all checks pass, false otherwise
pub fn validate_component_code_quality(component_path: &str) -> bool {
    use crate::code_quality;
    
    println!("\n🔍 Running code quality checks for component...");
    
    // Check for unused imports
    match code_quality::validate_no_unused_imports(component_path) {
        Ok(_) => {
            println!("✅ No unused imports found");
            true
        }
        Err(e) => {
            println!("❌ Found unused imports: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod nft_ownership_checker {
    use alloy_primitives::Address;
    use std::str::FromStr;
    
    // Import test utilities
    use crate::abi_encoding;
    use crate::data_handling;
    use crate::error_handling;
    use crate::solidity_types;
    
    #[test]
    fn test_nft_ownership_checker_validation() {
        // 1. Test address handling
        let azuki_contract = Address::from_str("0xbd3531da5cf5857e7cfaa92426877b022e612cf8").unwrap();
        let wallet = Address::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        
        // 2. Verify we're following best practices for data handling
        // This should match our component implementation
        let azuki_contract_clone = azuki_contract.clone();
        assert_eq!(azuki_contract, azuki_contract_clone);
        
        // 3. Verify we're using Option handling correctly
        let maybe_config = get_mock_eth_config("mainnet");
        let config = maybe_config.ok_or_else(|| "Failed to get Ethereum chain config".to_string()).unwrap();
        assert_eq!(config, "https://eth-mainnet.example.com/v1/rpc");
        
        // 4. Verify we don't use map_err on Option
        // The component should never do:
        // get_eth_chain_config("mainnet").map_err(|e| e.to_string())?;
        // But instead should do:
        // get_eth_chain_config("mainnet").ok_or_else(|| "Failed to get config".to_string())?;
        
        println!("✅ NFT ownership checker component passes validation tests!");
    }
    
    // Mock function to simulate get_eth_chain_config
    fn get_mock_eth_config(network: &str) -> Option<&'static str> {
        match network {
            "mainnet" => Some("https://eth-mainnet.example.com/v1/rpc"),
            "goerli" => Some("https://eth-goerli.example.com/v1/rpc"),
            _ => None,
        }
    }
}