//! Examples and tests for proper data handling in WAVS components
//! 
//! This module demonstrates how to properly handle data ownership,
//! cloning, and other common pitfalls when working with data in
//! WAVS components.

use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{sol, SolValue};
use serde::{Deserialize, Serialize};

// Example for API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeatherApiResponse {
    location: String,
    temperature: f64,
    conditions: String,
    timestamp: u64,
}

// CORRECT: Always derive Clone for API response structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PriceData {
    symbol: String,
    price: f64,
    timestamp: String,
}

// WRONG: Missing Clone derivation - will cause ownership issues
#[derive(Debug, Serialize, Deserialize)]
struct BadPriceData {
    symbol: String,
    price: f64,
    timestamp: String,
}

/// EXAMPLE 1: Properly cloning data before use to avoid ownership issues
#[test]
fn test_proper_data_cloning() {
    // Sample data
    let data = vec![1, 2, 3, 4, 5];
    
    // CORRECT: Create owned clone before using
    let data_clone = data.clone();
    let result = process_data(&data_clone);
    
    // We can still use the original data after processing
    assert_eq!(data, vec![1, 2, 3, 4, 5]);
    assert_eq!(result, 15);
    
    // WRONG: Creating a temporary clone that is immediately dropped
    // This code would compile but is inefficient
    #[allow(clippy::redundant_clone)]
    let result2 = process_data(&data.clone());
    assert_eq!(result2, 15);
}

fn process_data(data: &[u8]) -> i32 {
    data.iter().map(|&x| x as i32).sum()
}

/// EXAMPLE 2: Proper handling of collection elements to avoid "move out of index" errors
#[test]
fn test_collection_element_handling() {
    let api_responses = vec![
        WeatherApiResponse {
            location: "New York".to_string(),
            temperature: 72.5,
            conditions: "Sunny".to_string(),
            timestamp: 1682553600,
        },
        WeatherApiResponse {
            location: "London".to_string(),
            temperature: 63.2,
            conditions: "Cloudy".to_string(),
            timestamp: 1682553600,
        },
    ];
    
    // WRONG: This would cause "move out of index" errors in actual code
    // let first_location = api_responses[0].location; // This moves out of the vector
    // let _ = process_location(first_location);
    // let second_location = api_responses[0].location; // Error: value borrowed after move
    
    // CORRECT: Clone the value to avoid moving out of the collection
    let first_location = api_responses[0].location.clone();
    let _ = process_location(first_location);
    let second_location = api_responses[0].location.clone();
    let _ = process_location(second_location);
    
    // The vector is still intact
    assert_eq!(api_responses.len(), 2);
}

fn process_location(location: String) -> String {
    format!("Processing location: {}", location)
}

/// EXAMPLE 3: Proper handling of processing order to avoid "borrow of partially moved value" errors
#[test]
fn test_processing_order() {
    let api_response = WeatherApiResponse {
        location: "Tokyo".to_string(),
        temperature: 75.2,
        conditions: "Partly Cloudy".to_string(),
        timestamp: 1682553600,
    };
    
    // CORRECT: Process the complete struct first, then move fields
    let json_data = serde_json::to_string(&api_response).unwrap();
    let weather_struct = SimpleWeather {
        city: api_response.location,
        temp: api_response.temperature,
    };
    
    assert!(!json_data.is_empty());
    assert_eq!(weather_struct.city, "Tokyo");
    
    // Re-create the test data for the second example
    let api_response = WeatherApiResponse {
        location: "Tokyo".to_string(),
        temperature: 75.2,
        conditions: "Partly Cloudy".to_string(),
        timestamp: 1682553600,
    };
    
    // ALSO CORRECT: Use clone to avoid ownership issues entirely
    let weather_struct = SimpleWeather {
        city: api_response.location.clone(),
        temp: api_response.temperature,
    };
    let json_data = serde_json::to_string(&api_response).unwrap();
    
    assert!(!json_data.is_empty());
    assert_eq!(weather_struct.city, "Tokyo");
}

#[derive(Debug)]
struct SimpleWeather {
    city: String,
    temp: f64,
}

/// EXAMPLE 4: Safe bytes/string handling for Solidity data fields
#[test]
fn test_solidity_bytes_handling() {
    // Define a test ABI-encoded message
    let message = "Test message";
    let encoded_message = message.as_bytes().to_vec();
    
    // CORRECT: Properly convert Vec<u8> to Bytes for Solidity data fields
    let solidity_bytes = Bytes::from(encoded_message.clone());
    
    // Verify it works correctly
    assert_eq!(solidity_bytes.len(), encoded_message.len());
    assert_eq!(solidity_bytes.as_ref(), encoded_message.as_slice());
}

/// EXAMPLE 5: Numeric type conversions - correct ways to convert between Rust and Solidity types
#[test]
fn test_numeric_conversions() {
    // Sample temperature value (293.00K)
    let temperature: u128 = 29300;
    
    // CORRECT: String parsing method - works reliably for all numeric types
    let temperature_uint256: U256 = temperature.to_string().parse().unwrap();
    assert_eq!(temperature_uint256, U256::from(29300));
    
    // CORRECT: Explicit type conversion for struct fields
    let decimals: i32 = 6;
    let decimals_u32 = decimals as u32; // explicit cast required between integer types
    assert_eq!(decimals_u32, 6u32);
    
    // CORRECT: Use a loop for exponentiation with U256
    let mut divisor = U256::from(1);
    for _ in 0..decimals {
        divisor = divisor * U256::from(10);
    }
    assert_eq!(divisor, U256::from(1_000_000)); // 10^6
}

/// EXAMPLE 6: Always derive Clone for API response structs
#[test]
fn test_api_response_cloning() {
    // Create a good price data struct (with Clone)
    let price_data = PriceData {
        symbol: "ETH".to_string(),
        price: 3500.0,
        timestamp: "2023-04-26T12:00:00Z".to_string(),
    };
    
    // We can clone this struct
    let price_data_clone = price_data.clone();
    assert_eq!(price_data.symbol, price_data_clone.symbol);
    
    // NOTE: This test is just to demonstrate the importance of deriving Clone.
    // In real code, the bad struct (without Clone) would fail to compile when used
    // in scenarios requiring cloning.
}