pub mod signer;
pub mod orders;

use anyhow::Result;
use crossbeam_channel::Receiver;
use rust_decimal::Decimal;
use std::str::FromStr;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

use crate::strategy::arbitrage::TradeSignal;
use orders::OrderClient;

pub struct ExecutionManager {
    order_client: OrderClient,
    signal_rx: Receiver<TradeSignal>,
    market_id: String,
}

impl ExecutionManager {
    pub fn new(
        order_client: OrderClient,
        signal_rx: Receiver<TradeSignal>,
        market_id: String,
    ) -> Self {
        Self {
            order_client,
            signal_rx,
            market_id,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Execution Manager running");

        loop {
            match self.signal_rx.recv() {
                Ok(signal) => {
                    if let Err(e) = self.execute_arbitrage(signal).await {
                        error!("Failed to execute arbitrage: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Signal channel closed: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn execute_arbitrage(&self, signal: TradeSignal) -> Result<()> {
        info!("💰 Executing arbitrage trade");

        // Step 1: Place maker limit order
        let maker_order_id = self.order_client.place_limit_order(
            &self.market_id,
            &signal.maker_side,
            signal.maker_price,
            signal.quantity,
        ).await?;

        // Step 2: Monitor fills
        let filled_qty = self.monitor_fills(&maker_order_id).await?;

        // Step 3: Immediate hedge with taker order
        if filled_qty > Decimal::ZERO {
            info!("🔄 Hedging {} units", filled_qty);
            
            self.order_client.place_market_order(
                &self.market_id,
                &signal.taker_side,
                filled_qty,
            ).await?;

            info!("✅ Arbitrage completed! Filled: {}", filled_qty);
        }

        Ok(())
    }

    async fn monitor_fills(&self, order_id: &str) -> Result<Decimal> {
        let mut total_filled = Decimal::ZERO;
        let max_wait = Duration::from_secs(60);
        let poll_interval = Duration::from_millis(100);
        let start = tokio::time::Instant::now();

        loop {
            if start.elapsed() > max_wait {
                warn!("Order monitoring timeout, cancelling order");
                self.order_client.cancel_order(order_id).await?;
                break;
            }

            let status = self.order_client.get_order_status(order_id).await?;
            let filled = Decimal::from_str(&status.filled_size)?;

            if filled > total_filled {
                let delta = filled - total_filled;
                info!("📊 Partial fill detected: {} (total: {})", delta, filled);
                total_filled = filled;
                
                // TODO: Immediate partial hedge here
            }

            if status.status == "FILLED" {
                info!("✅ Order fully filled: {}", filled);
                total_filled = filled;
                break;
            }

            if status.status == "CANCELLED" || status.status == "REJECTED" {
                warn!("Order {} status: {}", order_id, status.status);
                break;
            }

            sleep(poll_interval).await;
        }

        Ok(total_filled)
    }
}
