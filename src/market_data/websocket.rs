use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, Serialize)]
pub struct SubscribeMessage {
    pub market: String,
    #[serde(rename = "type")]
    pub msg_type: String,
}

pub struct WebSocketClient {
    url: String,
    stream: Option<WsStream>,
}

impl WebSocketClient {
    pub fn new(url: String) -> Self {
        Self { url, stream: None }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.url);
        
        let (ws_stream, _) = connect_async(&self.url).await?;
        
        info!("✅ WebSocket connected");
        self.stream = Some(ws_stream);
        
        Ok(())
    }

    pub async fn subscribe(&mut self, market_id: &str) -> Result<()> {
        let subscribe_msg = SubscribeMessage {
            market: market_id.to_string(),
            msg_type: "subscribe".to_string(),
        };

        let json = serde_json::to_string(&subscribe_msg)?;
        
        if let Some(stream) = &mut self.stream {
            stream.send(Message::Text(json)).await?;
            info!("📡 Subscribed to market: {}", market_id);
        }

        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<Vec<u8>>> {
        if let Some(stream) = &mut self.stream {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => Ok(Some(text.into_bytes())),
                Some(Ok(Message::Binary(data))) => Ok(Some(data)),
                Some(Ok(Message::Ping(_))) => {
                    // Respond to ping
                    stream.send(Message::Pong(vec![])).await?;
                    Ok(None)
                }
                Some(Ok(Message::Pong(_))) => Ok(None),
                Some(Ok(Message::Close(_))) => {
                    warn!("WebSocket closed by server");
                    Ok(None)
                }
                Some(Ok(Message::Frame(_))) => {
                    // Raw frame - ignore
                    Ok(None)
                }
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    Err(e.into())
                }
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(mut stream) = self.stream.take() {
            stream.close(None).await?;
            info!("WebSocket connection closed");
        }
        Ok(())
    }
}
