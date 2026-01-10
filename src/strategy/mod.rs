pub mod order_book;
pub mod arbitrage;
pub mod fee;

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use rust_decimal::Decimal;
use tracing::{info, warn};


use crate::market_data::parser::OrderBookUpdate;
use arbitrage::{ArbitrageEngine, StrategyConfig, TradeSignal};
use order_book::{LocalOrderBook, Side};

pub struct StrategyEngine {
    order_book: LocalOrderBook,
    arb_engine: ArbitrageEngine,
    market_data_rx: Receiver<OrderBookUpdate>,
    signal_tx: Sender<TradeSignal>,
}

impl StrategyEngine {
    pub fn new(
        market_id: String,
        config: StrategyConfig,
        market_data_rx: Receiver<OrderBookUpdate>,
        signal_tx: Sender<TradeSignal>,
    ) -> Self {
        Self {
            order_book: LocalOrderBook::new(market_id),
            arb_engine: ArbitrageEngine::new(config),
            market_data_rx,
            signal_tx,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("⚙️  Strategy Engine running");

        loop {
            // Receive orderbook updates
            match self.market_data_rx.recv() {
                Ok(update) => {
                    // Update local orderbook
                    self.update_orderbook(update)?;

                    // Check for arbitrage opportunities
                    if let Some(signal) = self.arb_engine.check_arbitrage(&self.order_book) {
                        info!(
                            "🎯 Arbitrage opportunity detected! Maker: {:?}@{}, Taker: {:?}@{}, Profit: {}",
                            signal.maker_side, signal.maker_price, signal.taker_side, signal.taker_price, signal.expected_profit
                        );

                        // Send signal to execution engine
                        if let Err(e) = self.signal_tx.send(signal) {
                            warn!("Failed to send trade signal: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Market data channel closed: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    fn update_orderbook(&mut self, update: OrderBookUpdate) -> Result<()> {
        // Parse price levels
        let bids: Vec<(Decimal, Decimal)> = update
            .bids
            .iter()
            .filter_map(|level| {
                match (level.price_decimal(), level.size_decimal()) {
                    (Ok(p), Ok(s)) => Some((p, s)),
                    _ => None,
                }
            })
            .collect();

        let asks: Vec<(Decimal, Decimal)> = update
            .asks
            .iter()
            .filter_map(|level| {
                match (level.price_decimal(), level.size_decimal()) {
                    (Ok(p), Ok(s)) => Some((p, s)),
                    _ => None,
                }
            })
            .collect();

        // Update orderbook (assuming YES side for now)
        // TODO: Determine side from update metadata
        self.order_book.update_side(Side::Yes, bids, asks);

        Ok(())
    }
}
