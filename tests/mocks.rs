use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock implementations for external dependencies

#[derive(Clone)]
pub struct MockExchange {
    pub orders: Arc<Mutex<Vec<Value>>>,
    pub should_succeed: bool,
}

impl MockExchange {
    pub fn new() -> Self {
        Self {
            orders: Arc::new(Mutex::new(Vec::new())),
            should_succeed: true,
        }
    }

    pub fn set_should_succeed(&mut self, succeed: bool) {
        self.should_succeed = succeed;
    }

    pub async fn place_order(&self, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let mut orders = self.orders.lock().await;
        orders.push(params.clone());

        if self.should_succeed {
            Ok(json!({
                "status": "ok",
                "response": {
                    "data": {
                        "statuses": [{
                            "filled": {
                                "oid": 12345
                            }
                        }]
                    }
                }
            }))
        } else {
            Err("Mock exchange error".into())
        }
    }

    pub async fn cancel_order(&self, _params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        if self.should_succeed {
            Ok(json!({ "status": "cancelled" }))
        } else {
            Err("Mock cancel error".into())
        }
    }

    pub async fn get_order_count(&self) -> usize {
        let orders = self.orders.lock().await;
        orders.len()
    }
}

#[derive(Clone)]
pub struct MockInfo {
    pub meta_calls: Arc<Mutex<usize>>,
    pub user_state_calls: Arc<Mutex<usize>>,
    pub should_succeed: bool,
}

impl MockInfo {
    pub fn new() -> Self {
        Self {
            meta_calls: Arc::new(Mutex::new(0)),
            user_state_calls: Arc::new(Mutex::new(0)),
            should_succeed: true,
        }
    }

    pub async fn meta(&self) -> Result<Value, Box<dyn std::error::Error>> {
        let mut calls = self.meta_calls.lock().await;
        *calls += 1;

        if self.should_succeed {
            Ok(json!({
                "universe": [
                    {
                        "name": "BTC",
                        "szDecimals": 2,
                        "tickSize": 0.01
                    },
                    {
                        "name": "ETH",
                        "szDecimals": 4,
                        "tickSize": 0.01
                    }
                ]
            }))
        } else {
            Err("Mock meta error".into())
        }
    }

    pub async fn user_state(&self, _address: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let mut calls = self.user_state_calls.lock().await;
        *calls += 1;

        if self.should_succeed {
            Ok(json!({
                "positions": [],
                "wallet_balance": 10000.0
            }))
        } else {
            Err("Mock user state error".into())
        }
    }

    pub async fn open_orders(&self, _address: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
        if self.should_succeed {
            Ok(vec![])
        } else {
            Err("Mock open orders error".into())
        }
    }
}

// Test utilities
pub fn create_test_trade(price: f64, size: f64, is_buy: bool, timestamp: Option<i64>) -> Value {
    let ts = timestamp.unwrap_or(1640995200000);
    json!({
        "px": price.to_string(),
        "sz": size.to_string(),
        "side": if is_buy { "B" } else { "A" },
        "time": ts.to_string()
    })
}

pub fn create_test_market_data_event(coin: &str, price: f64, size: f64, is_buy: bool, latency: f64) -> Value {
    json!({
        "type": "market_data",
        "coin": coin,
        "price": price,
        "size": size,
        "is_buy": is_buy,
        "timestamp": 1640995200.0,
        "latency_ms": latency,
        "vwap": 0.0,
        "indicators": {}
    })
}

pub fn create_test_candle_event(coin: &str, timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64, cvd: f64) -> Value {
    json!({
        "type": "candle_closed",
        "coin": coin,
        "closed_candle_1m": {
            "start_time": timestamp,
            "open": open,
            "high": high,
            "low": low,
            "close": close,
            "volume": volume,
            "cvd": cvd,
            "poc": close
        },
        "vwap": 0.0,
        "indicators": {}
    })
}