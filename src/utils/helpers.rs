use crate::models::Orderbook;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
struct OrderbookResponse {
    code: String,
    msg: String,
    data: Vec<OrderbookData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderbookData {
    asks: Vec<[String; 2]>,
    bids: Vec<[String; 2]>,
    ts: String,
}

#[derive(Error, Debug)]
pub enum OrderbookError {
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Float parsing error: {0}")]
    FloatParseError(#[from] std::num::ParseFloatError),
    #[error("Integer parsing error: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
    #[error("Missing or invalid data: {0}")]
    InvalidData(String),
}

pub fn parse_order_book(data: &str) -> Result<Orderbook, OrderbookError> {
    let response: OrderbookResponse = serde_json::from_str(data)?;

    let orderbook_data = response
        .data
        .first()
        .ok_or_else(|| OrderbookError::InvalidData("Empty 'data' array".into()))?;

    Ok(Orderbook {
        asks: parse_orders(&orderbook_data.asks)?,
        bids: parse_orders(&orderbook_data.bids)?,
        ts: orderbook_data.ts.parse::<u64>()?,
    })
}

fn parse_orders(orders: &[[String; 2]]) -> Result<Vec<(f64, f64)>, OrderbookError> {
    orders
        .iter()
        .map(|[price, amount]| Ok((price.parse::<f64>()?, amount.parse::<f64>()?)))
        .collect()
}

pub fn validate_order_book_data(data: &str) -> Result<(), OrderbookError> {
    let v: serde_json::Value = serde_json::from_str(data)?;

    if !v.is_object() {
        return Err(OrderbookError::InvalidData(
            "Data is not a valid JSON object".into(),
        ));
    }

    if let Some(obj) = v.as_object() {
        if !obj.contains_key("asks") || !obj.contains_key("bids") {
            return Err(OrderbookError::InvalidData(
                "Missing required fields: 'asks' or 'bids'".into(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_order_book() {
        let data = r#"{"code":"0","msg":"","data":[{"asks":[["41006.8","0.60030921"]],"bids":[["41006.3","0.30178210"]],"ts":"1621447077008"}]}"#;
        let orderbook = parse_order_book(data);
        assert!(
            orderbook.is_ok(),
            "Failed to parse orderbook: {:?}",
            orderbook.err()
        );
        let orderbook = orderbook.unwrap();
        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks[0], (41006.8, 0.60030921));
        assert_eq!(orderbook.bids[0], (41006.3, 0.30178210));
        assert_eq!(orderbook.ts, 1621447077008);
    }

    #[test]
    fn test_validate_order_book_data() {
        let valid_data = r#"{"asks":[["41006.8","0.60030921"]],"bids":[["41006.3","0.30178210"]],"ts":"1621447077008"}"#;
        let invalid_data = r#"{"ask":[["41006.8","0.60030921"]],"bids":[["41006.3","0.30178210"]],"ts":"1621447077008"}"#;
        assert!(
            validate_order_book_data(valid_data).is_ok(),
            "Valid data failed validation"
        );
        assert!(
            validate_order_book_data(invalid_data).is_err(),
            "Invalid data passed validation"
        );
    }
}
