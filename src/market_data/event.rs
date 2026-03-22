use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataEvent {
    pub coin: String,
    pub price: f64,
    pub latency_ms: f64,
    pub vwap: f64,
    pub closed_candle_1m: Option<Value>,
}

impl MarketDataEvent {
    pub fn from_value(value: Value) -> Option<Self> {
        let coin = value.get("coin")?.as_str()?.to_string();
        let price = value.get("price")?.as_f64()?;
        let latency_ms = value.get("latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let vwap = value.get("vwap").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let closed_candle_1m = value.get("closed_candle_1m").cloned();

        Some(Self {
            coin,
            price,
            latency_ms,
            vwap,
            closed_candle_1m,
        })
    }
}
