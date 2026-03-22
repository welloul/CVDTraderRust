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
        
        let price = if let Some(p) = value.get("price") {
            if let Some(f) = p.as_f64() { f }
            else if let Some(s) = p.as_str() { s.parse::<f64>().ok()? }
            else { return None; }
        } else { return None; };

        let latency_ms = value.get("latency_ms").and_then(|v| v.as_f64()).unwrap_or(0.0);
        
        let vwap = if let Some(v) = value.get("vwap") {
            if let Some(f) = v.as_f64() { f }
            else if let Some(s) = v.as_str() { s.parse::<f64>().ok().unwrap_or(0.0) }
            else { 0.0 }
        } else { 0.0 };

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
