use crate::logger;
use crate::utils::types::TradePayload;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const DEFAULT_HOST: &str = "wss://ws-live-data.polymarket.com";

#[derive(Debug, Clone, Deserialize)]
pub struct WsMessage {
    pub topic: Option<String>,
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub payload: Option<serde_json::Value>,
}

pub struct RealTimeWsClient {
    host: String,
    on_message: Arc<dyn Fn(WsMessage) + Send + Sync>,
    on_connect: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl RealTimeWsClient {
    pub fn new<F, G>(on_message: F, on_connect: G) -> Self
    where
        F: Fn(WsMessage) + Send + Sync + 'static,
        G: Fn() + Send + Sync + 'static,
    {
        RealTimeWsClient {
            host: std::env::var("USER_REAL_TIME_DATA_URL").unwrap_or_else(|_| DEFAULT_HOST.to_string()),
            on_message: Arc::new(on_message),
            on_connect: Some(Arc::new(on_connect)),
        }
    }

    pub fn connect(&self) {
        let host = self.host.clone();
        let on_message = Arc::clone(&self.on_message);
        let on_connect = self.on_connect.clone();
        tokio::spawn(async move {
            let url = url::Url::parse(&host).expect("invalid ws url");
            match connect_async(url).await {
                Ok((ws, _)) => {
                    if let Some(ref cb) = on_connect {
                        cb();
                    }
                    let (mut write, mut read) = ws.split();
                    let sub = r#"{"auth":{},"type":"subscribe","subscriptions":[{"topic":"activity","type":"trades"}]}"#;
                    if write.send(Message::Text(sub.into())).await.is_err() {
                        return;
                    }
                    logger::info("Subscribed to activity:trades");

                    while let Some(Ok(msg)) = read.next().await {
                        if let Message::Text(t) = msg {
                            if let Ok(m) = serde_json::from_str::<WsMessage>(&t) {
                                on_message(m);
                            }
                        }
                    }
                }
                Err(e) => {
                    logger::error_msg("WebSocket connect failed", Some(&e));
                }
            }
        });
    }
}

pub fn parse_trade_payload(payload: &serde_json::Value) -> Option<TradePayload> {
    serde_json::from_value(payload.clone()).ok()
}
