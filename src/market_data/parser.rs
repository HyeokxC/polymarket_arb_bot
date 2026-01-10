use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    pub market_id: String,
    pub timestamp: i64,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub size: String,
}

impl PriceLevel {
    pub fn price_decimal(&self) -> Result<Decimal> {
        Ok(Decimal::from_str(&self.price)?)
    }

    pub fn size_decimal(&self) -> Result<Decimal> {
        Ok(Decimal::from_str(&self.size)?)
    }
}

pub fn parse_orderbook_update(data: &[u8]) -> Result<OrderBookUpdate> {
    // Using serde_json for now - can optimize to simd-json later
    let update: OrderBookUpdate = serde_json::from_slice(data)?;
    Ok(update)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_orderbook() {
        let json = r#"{
            "market_id": "test-market",
            "timestamp": 1704844800000,
            "bids": [
                {"price": "0.20", "size": "100"},
                {"price": "0.19", "size": "200"}
            ],
            "asks": [
                {"price": "0.21", "size": "150"},
                {"price": "0.22", "size": "250"}
            ]
        }"#;

        let result = parse_orderbook_update(json.as_bytes());
        assert!(result.is_ok());

        let update = result.unwrap();
        assert_eq!(update.market_id, "test-market");
        assert_eq!(update.bids.len(), 2);
        assert_eq!(update.asks.len(), 2);
    }

    #[test]
    fn test_price_level_conversion() {
        let level = PriceLevel {
            price: "0.50".to_string(),
            size: "100.5".to_string(),
        };

        let price = level.price_decimal().unwrap();
        let size = level.size_decimal().unwrap();

        assert_eq!(price, Decimal::from_str("0.50").unwrap());
        assert_eq!(size, Decimal::from_str("100.5").unwrap());
    }
}
