pub mod websocket;
pub mod parser;

use anyhow::Result;
use crossbeam_channel::Sender;
use parser::OrderBookUpdate;
use tracing::{error, info};
use websocket::WebSocketClient;

pub struct MarketDataAdapter {
    ws_client: WebSocketClient,
    sender: Sender<OrderBookUpdate>,
}

impl MarketDataAdapter {
    pub fn new(ws_url: String, sender: Sender<OrderBookUpdate>) -> Self {
        Self {
            ws_client: WebSocketClient::new(ws_url),
            sender,
        }
    }

    pub async fn run(&mut self, market_id: &str) -> Result<()> {
        // Connect to WebSocket
        self.ws_client.connect().await?;
        
        // Subscribe to market
        self.ws_client.subscribe(market_id).await?;
        
        info!("📊 Market Data Adapter running for market: {}", market_id);
        
        // Main loop
        loop {
            match self.ws_client.next_message().await {
                Ok(Some(data)) => {
                    // Parse message
                    match parser::parse_orderbook_update(&data) {
                        Ok(update) => {
                            // Send to strategy engine
                            if let Err(e) = self.sender.send(update) {
                                error!("Failed to send orderbook update: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse orderbook update: {}", e);
                        }
                    }
                }
                Ok(None) => {
                    // No message (ping/pong handled internally)
                    continue;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    // TODO: Implement reconnection logic
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.ws_client.close().await
    }
}
