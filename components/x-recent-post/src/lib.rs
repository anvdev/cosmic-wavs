// Required imports
use alloy_sol_types::{sol, SolCall, SolValue};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use wavs_wasi_chain::decode_event_log_data;
use wavs_wasi_chain::http::{fetch_json, http_request_get};
use wstd::{http::HeaderValue, runtime::block_on};

pub mod bindings; // Never edit bindings.rs!
use crate::bindings::wavs::worker::layer_types::{TriggerData, TriggerDataEthContractEvent};
use crate::bindings::{export, Guest, TriggerAction};

// Define destination for output
pub enum Destination {
    Ethereum,
    CliOutput,
}

// Define Solidity function signature that matches input format
sol! {
    function getRecentTweet(string username) external;
}

// Define Solidity interfaces for trigger events
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// API Response Types
#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserLookupResponse {
    data: UserData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct UserData {
    id: String,
    name: String,
    username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TweetsResponse {
    data: Option<Vec<TweetData>>,
    meta: Option<TweetMeta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TweetData {
    id: String,
    text: String,
    created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TweetMeta {
    result_count: Option<i32>,
}

// Response data structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct RecentTweetData {
    username: String,
    user_id: String,
    tweet_id: String,
    tweet_text: String,
    created_at: String,
    profile_name: String,
}

// Component struct declaration
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone and decode input data
        let req_clone = req.clone();

        // Decode the username parameter using proper ABI decoding
        let username = if let Ok(decoded) = getRecentTweetCall::abi_decode(&req_clone, false) {
            // Successfully decoded as function call
            decoded.username
        } else {
            // Try decoding just as a string parameter
            match String::abi_decode(&req_clone, false) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to decode input as string: {}", e)),
            }
        };

        // Get the recent tweet data
        let result = block_on(async move {
            let tweet_data = fetch_recent_tweet(&username).await?;
            serde_json::to_vec(&tweet_data).map_err(|e| e.to_string())
        })?;

        // Return encoded output based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &result)),
            Destination::CliOutput => Some(result),
        };

        Ok(output)
    }
}

// Trigger event handling
pub fn decode_trigger_event(trigger_data: TriggerData) -> Result<(u64, Vec<u8>, Destination)> {
    match trigger_data {
        TriggerData::EthContractEvent(TriggerDataEthContractEvent { log, .. }) => {
            let event: solidity::NewTrigger = decode_event_log_data!(log)?;
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

// Function to get the most recent tweet for a username
async fn fetch_recent_tweet(username: &str) -> Result<RecentTweetData, String> {
    // Get API token from environment
    let bearer_token = std::env::var("WAVS_ENV_X_BEARER_TOKEN")
        .map_err(|_| "Failed to get X_BEARER_TOKEN from environment variables".to_string())?;

    // 1. First lookup the user by username to get the user ID
    let user_id = get_user_id(username, &bearer_token).await?;

    // 2. Then get the user's recent tweets
    let tweet = get_recent_tweets(&user_id, &bearer_token).await?;

    Ok(tweet)
}

// Function to get user ID from username
async fn get_user_id(username: &str, bearer_token: &str) -> Result<UserData, String> {
    // Create API URL for user lookup
    let url = format!("https://api.twitter.com/2/users/by/username/{}", username);

    // Create request with headers
    let mut req = http_request_get(&url)
        .map_err(|e| format!("Failed to create user lookup request: {}", e))?;

    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", bearer_token))
            .map_err(|e| format!("Failed to create authorization header: {}", e))?,
    );
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Make the request and parse response
    let response: UserLookupResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch user data: {}", e))?;

    Ok(response.data)
}

// Function to get recent tweets
async fn get_recent_tweets(
    user_id: &UserData,
    bearer_token: &str,
) -> Result<RecentTweetData, String> {
    // Create API URL for tweets
    let url = format!(
        "https://api.twitter.com/2/users/{}/tweets?max_results=5&tweet.fields=created_at",
        user_id.id
    );

    // Create request with headers
    let mut req =
        http_request_get(&url).map_err(|e| format!("Failed to create tweets request: {}", e))?;

    req.headers_mut().insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", bearer_token))
            .map_err(|e| format!("Failed to create authorization header: {}", e))?,
    );
    req.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));

    // Make the request and parse response
    let response: TweetsResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch tweets: {}", e))?;

    // Get the most recent tweet
    let tweet = match response.data {
        Some(tweets) if !tweets.is_empty() => {
            let recent_tweet = &tweets[0];
            RecentTweetData {
                username: user_id.username.to_string(),
                user_id: user_id.id.to_string(),
                tweet_id: recent_tweet.id.to_string(),
                tweet_text: recent_tweet.text.to_string(),
                created_at: recent_tweet
                    .created_at
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string()),
                profile_name: user_id.name.to_string(),
            }
        }
        _ => return Err("No tweets found for this user".to_string()),
    };

    Ok(tweet)
}
