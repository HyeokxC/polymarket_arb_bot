use anyhow::Result;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use super::signer::TxSigner;
use crate::strategy::order_book::Side;

#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    pub market: String,
    pub side: String,
    pub price: String,
    pub size: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderResponse {
    pub order_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderStatus {
    pub order_id: String,
    pub status: String,
    pub filled_size: String,
    pub remaining_size: String,
}

pub struct OrderClient {
    http_client: Client,
    api_url: String,
    signer: TxSigner,
}

impl OrderClient {
    pub fn new(api_url: String, signer: TxSigner) -> Self {
        let http_client = Client::builder()
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_url,
            signer,
        }
    }

    pub async fn place_limit_order(
        &self,
        market_id: &str,
        side: &Side,
        price: Decimal,
        size: Decimal,
    ) -> Result<String> {
        let side_str = match side {
            Side::Yes => "YES",
            Side::No => "NO",
        };

        let timestamp = chrono::Utc::now().timestamp_millis();
        
        // Create message to sign
        let message = format!("{}:{}:{}:{}", market_id, side_str, price, size);
        let signature = self.signer.sign_message(message.as_bytes())?;

        let request = CreateOrderRequest {
            market: market_id.to_string(),
            side: side_str.to_string(),
            price: price.to_string(),
            size: size.to_string(),
            order_type: "LIMIT".to_string(),
            signature,
            timestamp,
        };

        let url = format!("{}/order", self.api_url);
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let order_response: CreateOrderResponse = response.json().await?;
            info!("✅ Limit order placed: {}", order_response.order_id);
            Ok(order_response.order_id)
        } else {
            let error_text = response.text().await?;
            error!("Failed to place order: {}", error_text);
            Err(anyhow::anyhow!("Order placement failed: {}", error_text))
        }
    }

    pub async fn place_market_order(
        &self,
        market_id: &str,
        side: &Side,
        size: Decimal,
    ) -> Result<String> {
        let side_str = match side {
            Side::Yes => "YES",
            Side::No => "NO",
        };

        let timestamp = chrono::Utc::now().timestamp_millis();
        
        let message = format!("{}:{}:{}", market_id, side_str, size);
        let signature = self.signer.sign_message(message.as_bytes())?;

        let request = CreateOrderRequest {
            market: market_id.to_string(),
            side: side_str.to_string(),
            price: "0".to_string(), // Market order
            size: size.to_string(),
            order_type: "MARKET".to_string(),
            signature,
            timestamp,
        };

        let url = format!("{}/order", self.api_url);
        let response = self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            let order_response: CreateOrderResponse = response.json().await?;
            info!("✅ Market order placed: {}", order_response.order_id);
            Ok(order_response.order_id)
        } else {
            let error_text = response.text().await?;
            error!("Failed to place market order: {}", error_text);
            Err(anyhow::anyhow!("Market order failed: {}", error_text))
        }
    }

    pub async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus> {
        let url = format!("{}/order/{}", self.api_url, order_id);
        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            let status: OrderStatus = response.json().await?;
            Ok(status)
        } else {
            Err(anyhow::anyhow!("Failed to get order status"))
        }
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let url = format!("{}/order/{}", self.api_url, order_id);
        let response = self.http_client.delete(&url).send().await?;

        if response.status().is_success() {
            info!("🗑️  Order cancelled: {}", order_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to cancel order"))
        }
    }
}
