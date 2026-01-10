use rust_decimal::Decimal;
use std::collections::BTreeMap;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub struct LocalOrderBook {
    pub market_id: String,
    pub yes_bids: BTreeMap<Decimal, Decimal>, // price -> size
    pub yes_asks: BTreeMap<Decimal, Decimal>,
    pub no_bids: BTreeMap<Decimal, Decimal>,
    pub no_asks: BTreeMap<Decimal, Decimal>,
}

impl LocalOrderBook {
    pub fn new(market_id: String) -> Self {
        Self {
            market_id,
            yes_bids: BTreeMap::new(),
            yes_asks: BTreeMap::new(),
            no_bids: BTreeMap::new(),
            no_asks: BTreeMap::new(),
        }
    }

    pub fn update_side(&mut self, side: Side, bids: Vec<(Decimal, Decimal)>, asks: Vec<(Decimal, Decimal)>) {
        match side {
            Side::Yes => {
                self.yes_bids = bids.into_iter().collect();
                self.yes_asks = asks.into_iter().collect();
            }
            Side::No => {
                self.no_bids = bids.into_iter().collect();
                self.no_asks = asks.into_iter().collect();
            }
        }
    }

    pub fn best_bid(&self, side: &Side) -> Option<(Decimal, Decimal)> {
        let bids = match side {
            Side::Yes => &self.yes_bids,
            Side::No => &self.no_bids,
        };
        
        bids.iter().next_back().map(|(p, s)| (*p, *s))
    }

    pub fn best_ask(&self, side: &Side) -> Option<(Decimal, Decimal)> {
        let asks = match side {
            Side::Yes => &self.yes_asks,
            Side::No => &self.no_asks,
        };
        
        asks.iter().next().map(|(p, s)| (*p, *s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_orderbook_creation() {
        let ob = LocalOrderBook::new("test-market".to_string());
        assert_eq!(ob.market_id, "test-market");
        assert!(ob.yes_bids.is_empty());
    }

    #[test]
    fn test_best_bid_ask() {
        let mut ob = LocalOrderBook::new("test".to_string());
        
        ob.update_side(
            Side::Yes,
            vec![(dec!(0.18), dec!(100)), (dec!(0.17), dec!(200))],
            vec![(dec!(0.22), dec!(150)), (dec!(0.23), dec!(250))],
        );

        let best_bid = ob.best_bid(&Side::Yes).unwrap();
        assert_eq!(best_bid.0, dec!(0.18)); // Highest bid

        let best_ask = ob.best_ask(&Side::Yes).unwrap();
        assert_eq!(best_ask.0, dec!(0.22)); // Lowest ask
    }
}
