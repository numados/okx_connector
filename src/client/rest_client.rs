use crate::models::Orderbook;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::num::ParseIntError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum OKXClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Failed to deserialize response: {0}")]
    DeserializationError(#[from] serde_json::Error),
    #[error("Failed to parse float: {0}")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Unexpected response structure: {0}")]
    UnexpectedResponseStructure(String),
}

/// Internal representation of raw order book data from the API
#[derive(Debug, Serialize, Deserialize)]
struct RawOrderbook {
    asks: Vec<(String, String, String, String)>,
    bids: Vec<(String, String, String, String)>,
    ts: String,
}

impl RawOrderbook {
    fn parse_to_orderbook(&self) -> Result<Orderbook, OKXClientError> {
        Ok(Orderbook {
            asks: self.parse_vec(&self.asks)?,
            bids: self.parse_vec(&self.bids)?,
            ts: self.ts.parse::<u64>()?,
        })
    }

    fn parse_vec(
        &self,
        vec: &[(String, String, String, String)],
    ) -> Result<Vec<(f64, f64)>, OKXClientError> {
        vec.iter()
            .map(|(price, amount, _, _)| Ok((price.parse::<f64>()?, amount.parse::<f64>()?)))
            .collect()
    }
}

pub struct OKXRestClient {
    base_url: Url,
    client: Client,
}

impl OKXRestClient {
    pub fn new(base_url: &str) -> Result<Self, OKXClientError> {
        Ok(OKXRestClient {
            base_url: Url::parse(base_url)?,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("OKX-Rust-Client/1.0")
                .build()?,
        })
    }

    pub async fn get_order_book(&self, symbol: &str) -> Result<Orderbook, OKXClientError> {
        let url = self
            .base_url
            .join(&format!("api/v5/market/books?instId={}", symbol))?;
        let response_text = self.client.get(url).send().await?.text().await?;

        let response_value: Value = serde_json::from_str(&response_text)?;

        // Check if the response has the expected structure
        let orderbook_data = response_value["data"]
            .as_array()
            .and_then(|arr| arr.first())
            .ok_or_else(|| {
                OKXClientError::UnexpectedResponseStructure("Missing 'data' array or empty".into())
            })?;

        let raw_orderbook: RawOrderbook = serde_json::from_value(orderbook_data.clone())?;
        raw_orderbook.parse_to_orderbook()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_order_book() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/v5/market/books"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "code": "0",
                "msg": "",
                "data": [{
                    "asks": [["50000", "1", "0", "7"]],
                    "bids": [["49999", "1", "0", "6"]],
                    "ts": "1719335318504"
                }]
            })))
            .mount(&mock_server)
            .await;

        let client = OKXRestClient::new(&mock_server.uri()).unwrap();

        let orderbook = client.get_order_book("BTC-USDT").await.unwrap();

        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks[0], (50000.0, 1.0));
        assert_eq!(orderbook.bids[0], (49999.0, 1.0));
        assert_eq!(orderbook.ts, 1719335318504);
    }
}
