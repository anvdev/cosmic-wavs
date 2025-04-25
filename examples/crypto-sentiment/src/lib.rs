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
    function analyzeCryptoSentiment(string cryptoName) external;
}

// Define sentiment analysis result structure - MUST derive Clone
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SentimentResult {
    crypto_name: String,
    sentiment_score: f64,
    articles_analyzed: u32,
    most_positive_headline: String,
    most_negative_headline: String,
    sources: Vec<String>,
    timestamp: String,
}

// Define news API response structures
#[derive(Debug, Deserialize, Clone)]
struct NewsApiResponse {
    #[serde(rename = "Data")]
    data: Vec<NewsArticle>,
}

#[derive(Debug, Deserialize, Clone)]
struct NewsArticle {
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "body")]
    body: String,
    #[serde(rename = "source")]
    source: String,
}

// Define solidity module for trigger handling
mod solidity {
    use alloy_sol_macro::sol;
    pub use ITypes::*;

    sol!("../../src/interfaces/ITypes.sol");
}

// Component struct declaration
struct Component;
export!(Component with_types_in bindings);

// Main component implementation
impl Guest for Component {
    fn run(action: TriggerAction) -> std::result::Result<Option<Vec<u8>>, String> {
        let (trigger_id, req, dest) =
            decode_trigger_event(action.data).map_err(|e| e.to_string())?;

        // Clone request data to avoid ownership issues
        let req_clone = req.clone();

        // Decode the input string using proper ABI decoding
        let crypto_name =
            if let Ok(decoded) = analyzeCryptoSentimentCall::abi_decode(&req_clone, false) {
                // Successfully decoded as function call
                decoded.cryptoName
            } else {
                // Try decoding just as a string parameter
                match String::abi_decode(&req_clone, false) {
                    Ok(s) => s,
                    Err(e) => return Err(format!("Failed to decode input as ABI string: {}", e)),
                }
            };

        // Analyze sentiment
        let res = block_on(async move {
            let sentiment_result = analyze_crypto_sentiment(&crypto_name).await?;
            serde_json::to_vec(&sentiment_result)
                .map_err(|e| format!("Failed to serialize result: {}", e))
        })?;

        // Return the result based on destination
        let output = match dest {
            Destination::Ethereum => Some(encode_trigger_output(trigger_id, &res)),
            Destination::CliOutput => Some(res),
        };

        Ok(output)
    }
}

// Helper function to decode trigger event
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

// Helper function to encode trigger output
pub fn encode_trigger_output(trigger_id: u64, output: impl AsRef<[u8]>) -> Vec<u8> {
    solidity::DataWithId { triggerId: trigger_id, data: output.as_ref().to_vec().into() }
        .abi_encode()
}

// Main sentiment analysis function
async fn analyze_crypto_sentiment(crypto_name: &str) -> Result<SentimentResult, String> {
    // Get API key from environment
    let api_key = std::env::var("WAVS_ENV_NEWS_API_KEY")
        .map_err(|_| "Failed to get NEWS_API_KEY from environment variables".to_string())?;

    // Fetch news articles
    let articles = fetch_crypto_news(crypto_name, &api_key).await?;

    if articles.is_empty() {
        return Err(format!("No news articles found for {}", crypto_name));
    }

    // Perform sentiment analysis
    let mut overall_score = 0.0;
    let mut sources = Vec::new();
    let mut most_positive_headline = ("", -1.0);
    let mut most_negative_headline = ("", 1.0);

    for article in &articles {
        // Add source if not already included
        if !sources.contains(&article.source) {
            sources.push(article.source.to_string());
        }

        // Calculate sentiment score for this article
        let title_score = calculate_sentiment(&article.title);
        let body_score = calculate_sentiment(&article.body);

        // Weight title more heavily than body (3:1 ratio)
        let article_score = (title_score * 3.0 + body_score) / 4.0;
        overall_score += article_score;

        // Update most positive/negative headlines
        if article_score > most_positive_headline.1 {
            most_positive_headline = (article.title.as_str(), article_score);
        }
        if article_score < most_negative_headline.1 {
            most_negative_headline = (article.title.as_str(), article_score);
        }
    }

    // Normalize overall score
    let average_score = overall_score / articles.len() as f64;

    // Create result
    let result = SentimentResult {
        crypto_name: crypto_name.to_string(),
        sentiment_score: (average_score * 100.0).round() / 100.0, // Round to 2 decimal places
        articles_analyzed: articles.len() as u32,
        most_positive_headline: most_positive_headline.0.to_string(),
        most_negative_headline: most_negative_headline.0.to_string(),
        sources,
        timestamp: get_current_timestamp(),
    };

    Ok(result)
}

// Function to fetch crypto news articles
async fn fetch_crypto_news(crypto_name: &str, api_key: &str) -> Result<Vec<NewsArticle>, String> {
    // Create CryptoCompare API URL (free tier, public API)
    let url = format!(
        "https://min-api.cryptocompare.com/data/v2/news/?categories={}&api_key={}",
        crypto_name, api_key
    );

    // Create request with headers
    let mut req = http_request_get(&url).map_err(|e| format!("Failed to create request: {}", e))?;

    req.headers_mut().insert("Accept", HeaderValue::from_static("application/json"));

    // Make API request
    let response: NewsApiResponse =
        fetch_json(req).await.map_err(|e| format!("Failed to fetch news data: {}", e))?;

    // Return articles (limited to 10 for efficiency)
    let limited_articles = response.data.into_iter().take(10).collect();

    Ok(limited_articles)
}

// Simple sentiment analysis function
fn calculate_sentiment(text: &str) -> f64 {
    let text = text.to_lowercase();

    // Define positive and negative words/phrases specific to crypto
    let positive_terms = [
        "bullish",
        "rally",
        "surge",
        "gain",
        "soar",
        "rise",
        "up",
        "high",
        "growth",
        "positive",
        "adopt",
        "partnership",
        "success",
        "launch",
        "breakthrough",
        "outperform",
        "beat",
        "strong",
        "opportunity",
        "potential",
        "support",
        "backed",
        "confidence",
        "progress",
        "recovery",
        "buy",
    ];

    let negative_terms = [
        "bearish",
        "crash",
        "drop",
        "fall",
        "plunge",
        "decline",
        "down",
        "low",
        "loss",
        "negative",
        "ban",
        "regulation",
        "concern",
        "risk",
        "fear",
        "volatile",
        "uncertainty",
        "weak",
        "threat",
        "problem",
        "issue",
        "warn",
        "caution",
        "trouble",
        "struggle",
        "sell",
        "dump",
        "liquidation",
    ];

    // Count term occurrences
    let mut positive_count = 0;
    let mut negative_count = 0;

    for term in positive_terms.iter() {
        positive_count += text.matches(term).count();
    }

    for term in negative_terms.iter() {
        negative_count += text.matches(term).count();
    }

    // Calculate sentiment score (-1.0 to 1.0)
    if positive_count == 0 && negative_count == 0 {
        return 0.0; // Neutral if no terms found
    }

    let total = positive_count + negative_count;
    let sentiment = (positive_count as f64 - negative_count as f64) / total as f64;

    // Return sentiment score between -1.0 and 1.0
    sentiment
}

// Helper to get current timestamp in ISO format
fn get_current_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let since_epoch = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0));

    let secs = since_epoch.as_secs();

    // Format as ISO timestamp (simplified)
    let year = 1970 + (secs / 31536000);
    let month = (secs % 31536000) / 2592000 + 1;
    let day = ((secs % 31536000) % 2592000) / 86400 + 1;
    let hour = (((secs % 31536000) % 2592000) % 86400) / 3600;
    let minute = ((((secs % 31536000) % 2592000) % 86400) % 3600) / 60;
    let second = ((((secs % 31536000) % 2592000) % 86400) % 3600) % 60;

    // Use if/else clamping to avoid importing any additional functions
    let month_clamped = if month > 12 { 12 } else { month };
    let day_clamped = if day > 31 { 31 } else { day };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month_clamped, day_clamped, hour, minute, second
    )
}
