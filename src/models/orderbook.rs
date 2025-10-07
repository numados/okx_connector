use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderbookError {
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Empty response data")]
    EmptyData,
    #[error("Invalid price data: NaN or infinite values")]
    InvalidPriceData,
    #[error("Invalid timestamp format: {0}")]
    InvalidTimestamp(#[from] std::num::ParseIntError),
}

/// Represents an order book with asks and bids
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orderbook {
    /// Ask orders (sell orders), sorted in ascending order by price
    pub asks: Vec<(f64, f64)>,
    /// Bid orders (buy orders), sorted in descending order by price
    pub bids: Vec<(f64, f64)>,
    /// Timestamp of the order book data
    pub ts: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderbookSnapshotResponse {
    code: String,
    msg: String,
    data: Vec<RawOrderbookData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawOrderbookData {
    asks: Vec<(f64, f64)>,
    bids: Vec<(f64, f64)>,
    ts: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderbookUpdate {
    asks: Vec<(f64, f64)>,
    bids: Vec<(f64, f64)>,
}

impl Orderbook {
    /// Creates a new Orderbook from a JSON snapshot string
    pub fn from_snapshot(data: &str) -> Result<Self, OrderbookError> {
        let response: OrderbookSnapshotResponse = serde_json::from_str(data)?;
        let raw_data = response
            .data
            .into_iter()
            .next()
            .ok_or(OrderbookError::EmptyData)?;

        let ts = raw_data.ts.parse::<u64>()?;

        let mut orderbook = Orderbook {
            asks: raw_data.asks,
            bids: raw_data.bids,
            ts,
        };

        orderbook.sort_order_book()?;
        Ok(orderbook)
    }

    /// Applies an incremental update to the order book
    pub fn apply_update(&mut self, update: &str) -> Result<(), OrderbookError> {
        let update: OrderbookUpdate = serde_json::from_str(update)?;
        self.asks.extend(update.asks);
        self.bids.extend(update.bids);
        self.sort_order_book()?;
        Ok(())
    }

    /// Sorts the order book: asks in ascending order, bids in descending order
    fn sort_order_book(&mut self) -> Result<(), OrderbookError> {
        // Validate that all prices are valid (not NaN or infinite)
        for (price, _) in self.asks.iter().chain(self.bids.iter()) {
            if !price.is_finite() {
                return Err(OrderbookError::InvalidPriceData);
            }
        }

        // Sort asks in ascending order by price
        self.asks
            .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

        // Sort bids in descending order by price
        self.bids
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_from_snapshot() {
        let data = r#"{"code":"0","msg":"","data":[{"asks":[[41006.8,0.60030921]],"bids":[[41006.3,0.30178210]],"ts":"1621447077008"}]}"#;
        let orderbook = Orderbook::from_snapshot(data).unwrap();
        assert_eq!(orderbook.asks.len(), 1);
        assert_eq!(orderbook.bids.len(), 1);
        assert_eq!(orderbook.asks[0], (41006.8, 0.60030921));
        assert_eq!(orderbook.bids[0], (41006.3, 0.30178210));
        assert_eq!(orderbook.ts, 1621447077008);
    }

    #[test]
    fn test_orderbook_apply_update() {
        let data = r#"{"code":"0","msg":"","data":[{"asks":[[41006.8,0.60030921]],"bids":[[41006.3,0.30178210]],"ts":"1621447077008"}]}"#;
        let mut orderbook = Orderbook::from_snapshot(data).unwrap();
        let update = r#"{"asks":[[41007.0,0.20000000]],"bids":[[41005.0,0.10000000]]}"#;
        orderbook.apply_update(update).unwrap();
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks[1], (41007.0, 0.20000000));
        assert_eq!(orderbook.bids[1], (41005.0, 0.10000000));
    }

    #[test]
    fn test_orderbook_sort_order_book() {
        let mut orderbook = Orderbook {
            asks: vec![(41007.0, 0.20000000), (41006.8, 0.60030921)],
            bids: vec![(41005.0, 0.10000000), (41006.3, 0.30178210)],
            ts: 1621447077008,
        };
        orderbook.sort_order_book().unwrap();
        assert_eq!(
            orderbook.asks,
            vec![(41006.8, 0.60030921), (41007.0, 0.20000000)]
        );
        assert_eq!(
            orderbook.bids,
            vec![(41006.3, 0.30178210), (41005.0, 0.10000000)]
        );
    }

    #[test]
    fn test_orderbook_invalid_price_data() {
        let mut orderbook = Orderbook {
            asks: vec![(f64::NAN, 0.20000000)],
            bids: vec![(41006.3, 0.30178210)],
            ts: 1621447077008,
        };
        assert!(orderbook.sort_order_book().is_err());
    }
}
