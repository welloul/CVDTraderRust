use crate::core::state::{self, GlobalState};
use crate::market_data::candles::CandleBuilder;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::Result;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const WS_URL: &str = "wss://api.hyperliquid.xyz/ws";

pub struct MarketDataHandler {
    pub coin: String,
    callbacks:
        Vec<Arc<dyn Fn(Value) -> futures_util::future::BoxFuture<'static, ()> + Send + Sync>>,
    is_running: bool,
    last_message_time: std::time::Instant,
    latency_samples: Vec<f64>,
    candle_builder: CandleBuilder,
    state: Arc<Mutex<GlobalState>>,
}

impl MarketDataHandler {
    pub fn new(coin: String, state: Arc<Mutex<GlobalState>>) -> Self {
        Self {
            coin,
            callbacks: Vec::new(),
            is_running: false,
            last_message_time: std::time::Instant::now(),
            latency_samples: Vec::new(),
            candle_builder: CandleBuilder::new(1), // 1-minute candles
            state,
        }
    }

    pub fn add_callback<F, Fut>(&mut self, callback: F)
    where
        F: Fn(Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let callback = Arc::new(move |event| {
            Box::pin(callback(event)) as futures_util::future::BoxFuture<'static, ()>
        });
        self.callbacks.push(callback);
    }

    pub async fn connect(&mut self) {
        self.is_running = true;
        let mut retry_count = 0;

        while self.is_running {
            match self.connect_ws().await {
                Ok(_) => {
                    retry_count = 0;
                }
                Err(e) => {
                    tracing::error!("WebSocket error for {}: {}", self.coin, e);
                    retry_count += 1;
                    let delay = std::cmp::min(2u64.pow(retry_count), 30);
                    tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                }
            }
        }
    }

    async fn connect_ws(&mut self) -> Result<()> {
                tracing::info!("Connecting to Hyperliquid WebSocket for {}", self.coin);
        let (ws_stream, _) = connect_async(WS_URL).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to trades
        let sub_msg = serde_json::json!({
            "method": "subscribe",
            "subscription": {
                "type": "trades",
                "coin": &self.coin
            }
        });

        write.send(Message::Text(sub_msg.to_string())).await?;
        tracing::info!("Subscribed to trades for {}", self.coin);

        // Start heartbeat monitor
        let heartbeat_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(50));
            loop {
                interval.tick().await;
                let ping = serde_json::json!({ "method": "ping" });
                if let Err(_) = write.send(Message::Text(ping.to_string())).await {
                    break;
                }
                // tracing::debug!("Sent WebSocket ping");
            }
        });

        let mut msg_count = 0;

        // Message handling loop
        while self.is_running {
            match tokio::time::timeout(std::time::Duration::from_secs(60), read.next()).await {
                Ok(Some(message)) => {
                    if let Ok(Message::Text(msg)) = message {
                        // Ignore pong messages
                        if msg.contains("pong") {
                            continue;
                        }

                        msg_count += 1;
                        if msg_count % 100 == 0 {
                            tracing::info!("Received message {} for {}", msg_count, self.coin);
                        }
                        self.handle_message(&msg).await;
                    }
                }
                Ok(None) => break,
                Err(_) => {
                    tracing::warn!("WebSocket timeout for {}, reconnecting", self.coin);
                    break;
                }
            }
        }

        heartbeat_handle.abort();
        Ok(())
    }

    async fn handle_message(&mut self, msg: &str) {
        let receive_time = std::time::Instant::now();
        self.last_message_time = receive_time;

        if msg.contains("trades") && self.coin == "SOL" {
            tracing::info!("SOL RAW MSG: {}", msg);
            match serde_json::from_str::<Value>(msg) {
                Ok(v) => tracing::info!("SOL PARSED VALUE: {:?}", v),
                Err(e) => tracing::error!("SOL JSON ERROR: {}", e),
            }
        }

        if let Ok(data) = serde_json::from_str::<Value>(msg) {
            if let (Some(channel), Some(trades_data)) = (
                data.get("channel").and_then(|c| c.as_str()),
                data.get("data").and_then(|d| d.as_array()),
            ) {
                if channel == "trades" {
                    for trade in trades_data {
                        self.process_trade(trade, receive_time).await;
                    }
                }
            }
        }
    }

    async fn process_trade(&mut self, trade: &Value, receive_time: std::time::Instant) {
        let px = if let Some(p) = trade.get("px") {
            if let Some(f) = p.as_f64() { f }
            else if let Some(s) = p.as_str() { s.parse::<f64>().ok().unwrap_or(0.0) }
            else { 0.0 }
        } else { 0.0 };

        let sz = if let Some(s) = trade.get("sz") {
            if let Some(f) = s.as_f64() { f }
            else if let Some(st) = s.as_str() { st.parse::<f64>().ok().unwrap_or(0.0) }
            else { 0.0 }
        } else { 0.0 };

        let is_buy = trade.get("side").and_then(|s| s.as_str()) == Some("B");

        // Handle timestamp conversion
        let trade_ts_ns = trade.get("time").and_then(|t| t.as_i64()).unwrap_or(0) as f64;
        let trade_ts_ms = if trade_ts_ns > 1e15 {
            (trade_ts_ns / 1e6) as i64
        } else if trade_ts_ns > 1e12 {
            trade_ts_ns as i64
        } else {
            (trade_ts_ns * 1000.0) as i64
        };

        if self.coin == "SOL" {
            tracing::info!("DEBUG SOL: px={}, sz={}, ts={}, buy={}", px, sz, trade_ts_ms, is_buy);
        }

        // Calculate latency
        let network_latency_ms = receive_time.elapsed().as_millis() as f64;
        self.latency_samples.push(network_latency_ms);
        if self.latency_samples.len() > 100 {
            self.latency_samples.remove(0);
        }

        // Update global state latency
        {
            let mut state = self.state.lock().await;
            state.update_latency(&self.coin, network_latency_ms);
        }

        // Process candle building
        if let Some(finished_candle) =
            self.candle_builder
                .process_trade(trade_ts_ms, px, sz, is_buy)
        {
            // Candle finished, dispatch candle event
            let candle_event = serde_json::json!({
                "type": "candle_closed",
                "coin": &self.coin,
                "price": finished_candle.close,
                "latency_ms": network_latency_ms,
                "closed_candle_1m": {
                    "start_time": finished_candle.timestamp,
                    "open": finished_candle.open,
                    "high": finished_candle.high,
                    "low": finished_candle.low,
                    "close": finished_candle.close,
                    "volume": finished_candle.volume,
                    "cvd": finished_candle.cvd,
                    "poc": finished_candle.poc
                },
                "vwap": 0.0,  // TODO: Implement VWAP
                "indicators": {}  // TODO: Implement indicators
            });

            // Dispatch candle event to callbacks
            for callback in &self.callbacks {
                callback(candle_event.clone()).await;
            }
        }

        // Create market data event for every trade
        let event = serde_json::json!({
            "type": "market_data",
            "coin": &self.coin,
            "price": px,
            "size": sz,
            "is_buy": is_buy,
            "timestamp": trade_ts_ms as f64 / 1000.0,
            "latency_ms": network_latency_ms,
            "vwap": 0.0,
            "indicators": {}
        });

        // Dispatch to callbacks
        for callback in &self.callbacks {
            callback(event.clone()).await;
        }
    }

    pub async fn stop(&mut self) {
        self.is_running = false;
        //         println!("[INFO] "Stopped MarketDataHandler for {}", self.coin);
    }
}
