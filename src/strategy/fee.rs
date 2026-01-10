use rust_decimal::Decimal;
use std::str::FromStr;

/// Calculate Polymarket taker fee rate based on price
/// Formula: fee_rate = 2 * price * (1 - price) * 0.0312
/// Maximum fee: 1.56% at price = 0.50
pub fn calculate_taker_fee_rate(price: Decimal) -> Decimal {
    let two = Decimal::from(2);
    let one = Decimal::from(1);
    let max_fee = Decimal::from_str("0.0312").expect("Valid decimal");
    
    two * price * (one - price) * max_fee
}

/// Calculate absolute fee amount for a given price and quantity
pub fn calculate_taker_fee(price: Decimal, quantity: Decimal) -> Decimal {
    let fee_rate = calculate_taker_fee_rate(price);
    price * quantity * fee_rate
}

/// Estimate maker rebate (placeholder - actual rebate may vary)
pub fn calculate_maker_rebate(price: Decimal, quantity: Decimal, rebate_rate: Decimal) -> Decimal {
    price * quantity * rebate_rate
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_fee_at_extremes() {
        // At 0.01 and 0.99, fee should be near 0
        let fee_low = calculate_taker_fee_rate(dec!(0.01));
        let fee_high = calculate_taker_fee_rate(dec!(0.99));
        
        assert!(fee_low < dec!(0.001));
        assert!(fee_high < dec!(0.001));
    }

    #[test]
    fn test_fee_at_midpoint() {
        // At 0.50, fee should be maximum (1.56%)
        let fee_mid = calculate_taker_fee_rate(dec!(0.50));
        
        // 2 * 0.5 * 0.5 * 0.0312 = 0.0156
        assert_eq!(fee_mid, dec!(0.0156));
    }

    #[test]
    fn test_fee_symmetry() {
        // Fee at 0.30 should equal fee at 0.70
        let fee_30 = calculate_taker_fee_rate(dec!(0.30));
        let fee_70 = calculate_taker_fee_rate(dec!(0.70));
        
        assert_eq!(fee_30, fee_70);
    }

    #[test]
    fn test_absolute_fee() {
        // $100 at 0.50 should cost $1.56 in fees
        let fee = calculate_taker_fee(dec!(0.50), dec!(100));
        
        // 0.50 * 100 * 0.0156 = 0.78
        assert_eq!(fee, dec!(0.78));
    }
}
