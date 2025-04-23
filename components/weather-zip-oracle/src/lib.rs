use alloy_sol_types::{SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings; // bindings are auto-generated during the build process
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

pub enum Destination {
    Ethereum,
    CliOutput,
}

pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let log_clone = log.clone();
            let event: solidity::NewTrigger = decode_event_log_data!(log_clone)?;
            let trigger_info =
                <solidity::TriggerInfo as SolValue>::abi_decode(&event._triggerInfo, false)?;
            Ok((trigger_info.triggerId, trigger_info.data.to_vec(), Destination::Ethereum))
        }
        TriggerData::Raw(data) => Ok((0, data.clone(), Destination::CliOutput)),
        _ => Err(anyhow::anyhow!("Unsupported trigger data type")),
    }
}

pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");

    // Define a simple struct representing the function that encodes string input
    sol! {
        function getWeatherByZip(string zipCode) external;
    }
}

// Response structures with Clone derivation to avoid ownership issues
#[derive(Serialize, Deserialize, Debug, Clone)]
struct WeatherData {
    zip_code: String,
    city: String,
    country: String,
    temperature: f64,
    feels_like: f64,
    description: String,
    humidity: u32,
    wind_speed: f64,
    timestamp: String,
}

// OpenWeather API response structures
#[derive(Serialize, Deserialize, Debug, Clone)]
struct WeatherResponse {
    weather: Vec<Weather>,
    main: Main,
    wind: Wind,
    sys: Sys,
    name: String,
    dt: u64,
    cod: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Weather {
    description: String,
    main: String,
    icon: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Main {
    temp: f64,
    feels_like: f64,
    temp_min: f64,
    temp_max: f64,
    pressure: u32,
    humidity: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Wind {
    speed: f64,
    deg: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Sys {
    country: String,
}

struct Component;
export!(Component with_types_in bindings);

impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the zip code string using proper ABI decoding
        let zip_code =
            if let Ok(decoded) = solidity::getWeatherByZipCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.zipCode
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        println!("Getting weather for zip code: {}", zip_code);

        // Run the weather query and return the result
        let res = block_on(async move {
            let weather_data = get_weather_by_zip(&zip_code).await?;
            println!("Weather data: {:?}", weather_data);
            serde_json::to_vec(&weather_data).map_err(|e| e.to_string())
        })?;

        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };
        Ok(output)
    }
}

async fn get_weather_by_zip(zip_code: &str) -> Result<WeatherData, String> {
    // Get API key from environment variable
    let api_key = std::env::var("WAVS_ENV_OPENWEATHER_API_KEY")
        .map_err(|_| "Failed to get OPENWEATHER_API_KEY from environment variables. Make sure it's set in the .env file".to_string())?;

    // Validate zip code format
    if !is_valid_zip_code(zip_code) {
        return Err(format!("Invalid zip code format: {}", zip_code));
    }

    // Format URL with proper encoding
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?zip={},us&appid={}&units=imperial",
        zip_code, api_key
    );

    // Debug print URL (without API key)
    println!("Debug - Request URL (redacted API key): {}", url.replace(&api_key, "[REDACTED]"));

    // Create request with proper headers
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;
    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Parse JSON response
    let response: WeatherResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch weather data: {}", e))?;

    // Check for error response
    if response.cod != 200 {
        return Err(format!("API Error: {}", response.cod));
    }

    // Get current timestamp
    let timestamp = response.dt.to_string();

    // Extract weather information
    let weather_description = if !response.weather.is_empty() {
        response.weather[0].description.clone()
    } else {
        "Unknown".to_string()
    };

    Ok(WeatherData {
        zip_code: zip_code.to_string(),
        city: response.name,
        country: response.sys.country,
        temperature: response.main.temp,
        feels_like: response.main.feels_like,
        description: weather_description,
        humidity: response.main.humidity,
        wind_speed: response.wind.speed,
        timestamp,
    })
}

// Validate US zip code format
fn is_valid_zip_code(zip_code: &str) -> bool {
    // Basic validation for US zip codes (5 digits)
    let zip_len = zip_code.len();
    if zip_len != 5 {
        return false;
    }

    // Check that all characters are digits
    zip_code.chars().all(|c| c.is_digit(10))
}
