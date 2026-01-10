use anyhow::Result;
use rust_decimal::Decimal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::warn;


pub struct KillSwitch {
    active: Arc<AtomicBool>,
    daily_loss: Decimal,
    daily_loss_limit: Decimal,
    loss_threshold: Decimal,
    consecutive_failures: u32,
    max_consecutive_failures: u32,
}

impl KillSwitch {
    pub fn new(daily_loss_limit: Decimal, loss_threshold: Decimal) -> Self {
        Self {
            active: Arc::new(AtomicBool::new(false)),
            daily_loss: Decimal::ZERO,
            daily_loss_limit,
            loss_threshold,
            consecutive_failures: 0,
            max_consecutive_failures: 3,
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn activate(&self, reason: &str) {
        warn!("🚨 KILL SWITCH ACTIVATED: {}", reason);
        self.active.store(true, Ordering::Relaxed);
    }

    pub fn deactivate(&self) {
        warn!("✅ Kill switch deactivated");
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn record_trade(&mut self, profit: Decimal) -> Result<()> {
        if profit < Decimal::ZERO {
            // Loss detected
            self.daily_loss += profit.abs();
            self.consecutive_failures += 1;

            // Check daily loss limit
            if self.daily_loss >= self.daily_loss_limit {
                self.activate(&format!(
                    "Daily loss limit reached: {} / {}",
                    self.daily_loss, self.daily_loss_limit
                ));
                return Err(anyhow::anyhow!("Daily loss limit exceeded"));
            }

            // Check consecutive failures
            if self.consecutive_failures >= self.max_consecutive_failures {
                self.activate(&format!(
                    "Consecutive failures: {}",
                    self.consecutive_failures
                ));
                return Err(anyhow::anyhow!("Too many consecutive failures"));
            }

            // Check single trade loss threshold
            let loss_pct = profit.abs() / self.daily_loss_limit;
            if loss_pct >= self.loss_threshold {
                self.activate(&format!(
                    "Single trade loss threshold exceeded: {:.2}%",
                    loss_pct * Decimal::from(100)
                ));
                return Err(anyhow::anyhow!("Loss threshold exceeded"));
            }
        } else {
            // Profit - reset consecutive failures
            self.consecutive_failures = 0;
        }

        Ok(())
    }

    pub fn reset_daily(&mut self) {
        self.daily_loss = Decimal::ZERO;
        self.consecutive_failures = 0;
        self.deactivate();
    }

    pub fn get_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_kill_switch_daily_limit() {
        let mut ks = KillSwitch::new(dec!(100), dec!(0.50)); // 50% threshold
        
        // Record losses (each under 50% threshold)
        assert!(ks.record_trade(dec!(-30)).is_ok());
        assert!(ks.record_trade(dec!(-40)).is_ok());
        
        // This should trigger kill switch (total: 110 > 100)
        let result = ks.record_trade(dec!(-40));
        assert!(result.is_err());
        assert!(ks.is_active());
    }


    #[test]
    fn test_consecutive_failures() {
        let mut ks = KillSwitch::new(dec!(100), dec!(0.02));
        
        assert!(ks.record_trade(dec!(-1)).is_ok());
        assert!(ks.record_trade(dec!(-1)).is_ok());
        
        // Third consecutive failure should trigger
        let result = ks.record_trade(dec!(-1));
        assert!(result.is_err());
        assert!(ks.is_active());
    }

    #[test]
    fn test_profit_resets_failures() {
        let mut ks = KillSwitch::new(dec!(100), dec!(0.02));
        
        assert!(ks.record_trade(dec!(-1)).is_ok());
        assert!(ks.record_trade(dec!(-1)).is_ok());
        
        // Profit resets counter
        assert!(ks.record_trade(dec!(5)).is_ok());
        
        // Can have 2 more failures
        assert!(ks.record_trade(dec!(-1)).is_ok());
        assert!(ks.record_trade(dec!(-1)).is_ok());
    }
}
