use rust_decimal::Decimal;


use super::fee::{calculate_maker_rebate, calculate_taker_fee};
use super::order_book::{LocalOrderBook, Side};

#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub max_maker_price: Decimal,
    pub min_profit_margin: Decimal,
    pub maker_rebate_rate: Decimal,
}

#[derive(Debug, Clone)]
pub struct TradeSignal {
    pub maker_side: Side,
    pub maker_price: Decimal,
    pub taker_side: Side,
    pub taker_price: Decimal,
    pub quantity: Decimal,
    pub expected_profit: Decimal,
}

pub struct ArbitrageEngine {
    config: StrategyConfig,
}

impl ArbitrageEngine {
    pub fn new(config: StrategyConfig) -> Self {
        Self { config }
    }

    pub fn check_arbitrage(&self, order_book: &LocalOrderBook) -> Option<TradeSignal> {
        // Check YES side as maker
        if let Some(signal) = self.check_side(order_book, Side::Yes) {
            return Some(signal);
        }

        // Check NO side as maker
        if let Some(signal) = self.check_side(order_book, Side::No) {
            return Some(signal);
        }

        None
    }

    fn check_side(&self, order_book: &LocalOrderBook, maker_side: Side) -> Option<TradeSignal> {
        // Get best bid for maker side (we place limit order here)
        let (maker_price, maker_size) = order_book.best_bid(&maker_side)?;

        // Only consider low prices for maker
        if maker_price > self.config.max_maker_price {
            return None;
        }

        // Get opposite side for taker (immediate hedge)
        let taker_side = match maker_side {
            Side::Yes => Side::No,
            Side::No => Side::Yes,
        };

        // Get best ask for taker side (we market buy here)
        let (taker_price, taker_size) = order_book.best_ask(&taker_side)?;

        // Calculate costs
        let quantity = maker_size.min(taker_size); // Use minimum available
        
        let maker_cost = maker_price - calculate_maker_rebate(maker_price, quantity, self.config.maker_rebate_rate) / quantity;
        let taker_cost = taker_price + calculate_taker_fee(taker_price, quantity) / quantity;
        
        let total_cost = maker_cost + taker_cost;
        let one = Decimal::from(1);

        // Check if profitable
        if total_cost < (one - self.config.min_profit_margin) {
            let expected_profit = (one - total_cost) * quantity;
            
            return Some(TradeSignal {
                maker_side,
                maker_price,
                taker_side,
                taker_price,
                quantity,
                expected_profit,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_arbitrage_detection() {
        let config = StrategyConfig {
            max_maker_price: dec!(0.30),
            min_profit_margin: dec!(0.005),
            maker_rebate_rate: dec!(0.0005),
        };

        let engine = ArbitrageEngine::new(config);
        
        let mut ob = LocalOrderBook::new("test".to_string());
        
        // Setup profitable scenario
        // YES @ 0.18, NO @ 0.78
        // Total cost: ~0.18 + ~0.78 = ~0.96 < 1.00
        ob.update_side(
            Side::Yes,
            vec![(dec!(0.18), dec!(100))],
            vec![(dec!(0.22), dec!(100))],
        );
        ob.update_side(
            Side::No,
            vec![(dec!(0.77), dec!(100))],
            vec![(dec!(0.78), dec!(100))],
        );

        let signal = engine.check_arbitrage(&ob);
        assert!(signal.is_some());
        
        if let Some(s) = signal {
            assert_eq!(s.maker_price, dec!(0.18));
            assert!(s.expected_profit > Decimal::ZERO);
        }
    }

    #[test]
    fn test_no_arbitrage_high_price() {
        let config = StrategyConfig {
            max_maker_price: dec!(0.30),
            min_profit_margin: dec!(0.005),
            maker_rebate_rate: dec!(0.0005),
        };

        let engine = ArbitrageEngine::new(config);
        
        let mut ob = LocalOrderBook::new("test".to_string());
        
        // Maker price too high (> 0.30)
        ob.update_side(
            Side::Yes,
            vec![(dec!(0.50), dec!(100))],
            vec![(dec!(0.52), dec!(100))],
        );
        ob.update_side(
            Side::No,
            vec![(dec!(0.48), dec!(100))],
            vec![(dec!(0.50), dec!(100))],
        );

        let signal = engine.check_arbitrage(&ob);
        assert!(signal.is_none());
    }
}
