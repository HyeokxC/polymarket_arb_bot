use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub polymarket: PolymarketConfig,
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolymarketConfig {
    pub api_key: String,
    pub secret: String,
    pub ws_url: String,
    pub rest_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    pub max_maker_price: f64,
    pub min_profit_margin: f64,
    pub maker_rebate_rate: f64,
    pub default_order_size: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiskConfig {
    pub daily_loss_limit: f64,
    pub max_position_size: f64,
    pub kill_switch_loss_threshold: f64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();

        let polymarket = PolymarketConfig {
            api_key: env::var("POLYMARKET_API_KEY")?,
            secret: env::var("POLYMARKET_SECRET")?,
            ws_url: env::var("POLYMARKET_WS_URL")
                .unwrap_or_else(|_| "wss://ws-subscriptions-clob.polymarket.com/ws/market".to_string()),
            rest_url: env::var("POLYMARKET_REST_URL")
                .unwrap_or_else(|_| "https://clob.polymarket.com".to_string()),
        };

        let strategy = StrategyConfig {
            max_maker_price: env::var("MAX_MAKER_PRICE")
                .unwrap_or_else(|_| "0.30".to_string())
                .parse()?,
            min_profit_margin: env::var("MIN_PROFIT_MARGIN")
                .unwrap_or_else(|_| "0.005".to_string())
                .parse()?,
            maker_rebate_rate: env::var("MAKER_REBATE_RATE")
                .unwrap_or_else(|_| "0.0005".to_string())
                .parse()?,
            default_order_size: env::var("DEFAULT_ORDER_SIZE")
                .unwrap_or_else(|_| "100.0".to_string())
                .parse()?,
        };

        let risk = RiskConfig {
            daily_loss_limit: env::var("DAILY_LOSS_LIMIT")
                .unwrap_or_else(|_| "100.0".to_string())
                .parse()?,
            max_position_size: env::var("MAX_POSITION_SIZE")
                .unwrap_or_else(|_| "1000.0".to_string())
                .parse()?,
            kill_switch_loss_threshold: env::var("KILL_SWITCH_LOSS_THRESHOLD")
                .unwrap_or_else(|_| "0.02".to_string())
                .parse()?,
        };

        Ok(AppConfig {
            polymarket,
            strategy,
            risk,
        })
    }
}
