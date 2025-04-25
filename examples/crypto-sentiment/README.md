# Crypto News Sentiment Analyzer

This WAVS component fetches recent news about a specified cryptocurrency and analyzes the sentiment of those articles. The sentiment analysis can be used for trading signals and could be a valuable input for automated trading strategies.

## Features

- Retrieves recent news articles about a specified cryptocurrency
- Performs sentiment analysis on the article headlines and content
- Calculates an overall sentiment score (range from -1.0 to 1.0)
- Identifies the most positive and most negative headlines
- Lists the sources of the analyzed news
- Includes a timestamp of when the analysis was performed

## Input Parameters

The component accepts a single string parameter:
- **cryptoName**: The name or symbol of the cryptocurrency (e.g., "BTC", "Bitcoin", "ETH", "Ethereum")

## Output Format

The component returns a JSON response with the following structure:

```json
{
  "cryptoName": "Bitcoin",
  "sentimentScore": 0.25,
  "articlesAnalyzed": 10,
  "mostPositiveHeadline": "Bitcoin Adoption Soars as Major Companies Add to Reserves",
  "mostNegativeHeadline": "Regulatory Concerns Continue to Pressure Bitcoin Prices",
  "sources": ["CoinDesk", "CryptoNews", "Bloomberg"],
  "timestamp": "2023-04-24T14:32:15Z"
}
```

## Sentiment Score Interpretation

- **Greater than 0.5**: Very positive news sentiment
- **0.1 to 0.5**: Moderately positive news sentiment
- **-0.1 to 0.1**: Neutral news sentiment
- **-0.5 to -0.1**: Moderately negative news sentiment
- **Less than -0.5**: Very negative news sentiment

## Usage

To test with the WASI executor:

```bash
# Set the crypto name (in this example "Bitcoin")
export TRIGGER_DATA_INPUT=`cast abi-encode "f(string)" "Bitcoin"`
export COMPONENT_FILENAME=crypto_sentiment.wasm
export SERVICE_CONFIG="'{\"fuel_limit\":100000000,\"max_gas\":5000000,\"host_envs\":[\"WAVS_ENV_NEWS_API_KEY\"],\"kv\":[],\"workflow_id\":\"default\",\"component_id\":\"default\"}'"
make wasi-exec
```

## API Keys and Environment Variables

This component requires an API key to access cryptocurrency news from CryptoCompare. The following environment variable must be set:

- `WAVS_ENV_NEWS_API_KEY`: API key for accessing the CryptoCompare news service

You can sign up for a free API key at: https://min-api.cryptocompare.com/

The free tier includes a limited number of API calls per day, which should be sufficient for testing and small-scale usage.

## Implementation Details

### API Integration
We'll use the Crypto Compare News API to fetch recent cryptocurrency news. This API provides:
- Recent news articles related to specific cryptocurrencies
- Article metadata like title, body, published time, source

### Sentiment Analysis
Since we don't have direct access to advanced NLP libraries in WebAssembly, we'll use:
- A simple approach counting positive/negative words and phrases
- A predefined dictionary of crypto-specific positive/negative terms
- Basic text processing to normalize content before analysis

### Response Structure
The response will include:
- Overall sentiment score (-1.0 to 1.0 range)
- Number of articles analyzed
- Most positive and most negative headlines
- Sources of the analyzed news
- Timestamp of the analysis

## Component Structure
- `Cargo.toml`: Dependencies configuration
- `src/lib.rs`: Main component implementation
- `src/bindings.rs`: Auto-generated code (never edit)

## Checklist

- ✅ Implement Guest trait and export component correctly
- ✅ Properly handle TriggerAction and TriggerData
- ✅ Properly decode ABI function calls
- ✅ Avoid String::from_utf8 on ABI data
- ✅ Derive Clone for response structure
- ✅ Clone data before use
- ✅ Use ok_or_else() for Option types
- ✅ Use map_err() for Result types
- ✅ Include all required imports
- ✅ Use proper sol! macro syntax
- ✅ No hardcoded API keys (use environment variables)
- ✅ Use workspace dependencies correctly
- ✅ Handle numeric conversions safely
- ✅ Use .to_string() for string literals in struct
- ✅ Use block_on for async network requests

## Testing Approach
Test the component with various inputs:
- Common cryptocurrencies (BTC, ETH)
- Less common cryptocurrencies
- Edge cases (empty string, invalid names)
- Test during different market conditions
