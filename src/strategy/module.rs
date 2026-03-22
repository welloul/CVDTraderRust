use crate::core::config::StrategyConfig;
use crate::core::state::GlobalState;
use crate::execution::gateway::ExecutionGateway;
use crate::execution::ttl::OrderTTLTracker;
use crate::market_data::candles::Candle;
use crate::market_data::event::MarketDataEvent;
use crate::risk::manager::RiskManager;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct StrategyModule {
    state: Arc<Mutex<GlobalState>>,
    execution: Option<Arc<Mutex<ExecutionGateway>>>,
    risk: Arc<Mutex<RiskManager>>,
    ttl_tracker: Option<Arc<Mutex<OrderTTLTracker>>>,
    candle_history: HashMap<String, Vec<Candle>>,
    cvd_flip_streak: HashMap<String, i32>,
}

impl StrategyModule {
    pub fn new(
        state: Arc<Mutex<GlobalState>>,
        execution: Option<Arc<Mutex<ExecutionGateway>>>,
        risk: Arc<Mutex<RiskManager>>,
        ttl_tracker: Option<Arc<Mutex<OrderTTLTracker>>>,
    ) -> Self {
        Self {
            state,
            execution,
            risk,
            ttl_tracker,
            candle_history: HashMap::new(),
            cvd_flip_streak: HashMap::new(),
        }
    }

    pub async fn on_market_data(&mut self, event: MarketDataEvent) {
        let latency = event.latency_ms;

        // Risk check latency
        let mut risk = self.risk.lock().await;
        risk.check_latency(latency);
        drop(risk);

        let coin = event.coin.clone();
        let price = event.price;

        // Initialize market data bucket
        {
            let mut state = self.state.lock().await;
            if !state.market_data.contains_key(&coin) {
                let mut coin_data = HashMap::new();
                coin_data.insert("candles".to_string(), serde_json::json!([]));
                coin_data.insert("cvd".to_string(), serde_json::json!([]));
                coin_data.insert("price".to_string(), serde_json::json!([price]));
                coin_data.insert("indicators".to_string(), serde_json::json!({}));
                state.market_data.insert(coin.clone(), coin_data);
            } else if let Some(coin_data) = state.market_data.get_mut(&coin) {
                coin_data.insert("price".to_string(), serde_json::json!([price]));
                // Indicators logic: if they were in the Value, they'd be here. 
                // For now, let's keep it simple or expand MarketDataEvent.
            }
        }

        // Update simulated PnL
        self.update_simulated_pnl(&coin, price).await;

        // Check SL/TP
        self.check_sl_tp(&coin, price).await;

        // Process closed candle
        if let Some(ref closed_candle) = event.closed_candle_1m {
            self.process_closed_candle(&coin, closed_candle).await;
        }

        let vwap = event.vwap;

        // Signal evaluation
        if let Some(ref closed_candle) = event.closed_candle_1m {
            self.evaluate_signal(&coin, closed_candle, vwap).await;
        }
    }

    async fn process_closed_candle(&mut self, coin: &str, candle_data: &Value) {
        // Parse candle data
        let candle = Candle {
            timestamp: candle_data
                .get("start_time")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            open: candle_data
                .get("open")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            high: candle_data
                .get("high")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            low: candle_data
                .get("low")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            close: candle_data
                .get("close")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            volume: candle_data
                .get("volume")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            cvd: candle_data
                .get("cvd")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            poc: candle_data
                .get("poc")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        };

        // Add to history
        self.candle_history
            .entry(coin.to_string())
            .or_insert_with(Vec::new)
            .push(candle.clone());

        // Keep last 100 candles
        if let Some(history) = self.candle_history.get_mut(coin) {
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Update state
        let mut state = self.state.lock().await;
        if let Some(coin_data) = state.market_data.get_mut(coin) {
            if let Some(candles_value) = coin_data.get_mut("candles") {
                if let Some(candles_array) = candles_value.as_array_mut() {
                    let candle_json = serde_json::json!({
                        "time": candle.timestamp,
                        "open": candle.open,
                        "high": candle.high,
                        "low": candle.low,
                        "close": candle.close,
                        "cvd": candle.cvd,
                        "poc": candle.poc
                    });
                    candles_array.push(candle_json);
                    if candles_array.len() > 100 {
                        candles_array.remove(0);
                    }
                }
            }
        }

                tracing::info!("1m Candle Closed for {}, close: {}, cvd: {}", coin, candle.close, candle.cvd);
    }

    async fn evaluate_signal(&mut self, coin: &str, candle_data: &Value, vwap: f64) {
        let state = self.state.lock().await;
        if !state.is_running {
            return;
        }
        drop(state);

        // Check if we have enough history
        if let Some(history) = self.candle_history.get(coin) {
            // This section of the provided edit is syntactically incorrect Rust code.
            // It appears to be an attempt to insert SQL DDL and a partial Rust line.
            // To maintain syntactic correctness of the Rust file, this problematic
            // part is omitted. If you intended to add database interaction or
            // scenario tests, please provide valid Rust code for that purpose.
            //
            // Original problematic snippet from instruction:
            // -- System health table
            // CREATE TABLE IF NOT EXISTS system_health (
            // f.state.lock().await;

            let lookback = {
                let state_lock = self.state.lock().await;
                state_lock.config.strategy.lookback
            };

            if history.len() < lookback {
                if history.len() % 5 == 0 {
                    tracing::info!("Warming up {}... {}/{} candles", coin, history.len(), lookback);
                }
                return;
            }

            // Get current candle
            if let Some(current) = history.last() {
                // Simple signal detection - check for price swings and CVD divergence
                // This is a simplified version of the Python CVD strategy

                let high_swing = self.detect_swing_high(history);
                let low_swing = self.detect_swing_low(history);

                if high_swing && self.is_cvd_exhaustion(history, true) {
                    // Short signal - price made high but CVD is weakening
                    let entry_price = current.close;
                    let stop_loss = current.high * 1.01; // 1% above high
                    let take_profit = current.close * 0.98; // 2% profit target

                    tracing::info!("Short signal detected for {}, price: {}", coin, entry_price);
                    self.try_enter_position(coin, false, entry_price, stop_loss, take_profit)
                        .await;
                } else if low_swing && self.is_cvd_exhaustion(history, false) {
                    // Long signal - price made low but CVD is weakening
                    let entry_price = current.close;
                    let stop_loss = current.low * 0.99; // 1% below low
                    let take_profit = current.close * 1.02; // 2% profit target

                    tracing::info!("Long signal detected for {}, price: {}", coin, entry_price);
                    self.try_enter_position(coin, true, entry_price, stop_loss, take_profit)
                        .await;
                }
            }
        }
    }

    fn detect_swing_high(&self, history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_highs: Vec<f64> = history
            .iter()
            .rev()
            .skip(1)
            .take(4)
            .map(|c| c.high)
            .collect();

        // Current high is higher than previous 4 highs
        prev_highs.iter().all(|&h| current.high > h)
    }

    fn detect_swing_low(&self, history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_lows: Vec<f64> = history
            .iter()
            .rev()
            .skip(1)
            .take(4)
            .map(|c| c.low)
            .collect();

        // Current low is lower than previous 4 lows
        prev_lows.iter().all(|&l| current.low < l)
    }

    fn is_cvd_exhaustion(&self, history: &[Candle], is_high_swing: bool) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_candles: Vec<&Candle> = history.iter().rev().skip(1).take(4).collect();

        if is_high_swing {
            // For short signals: Price higher high + CVD lower high (bearish divergence)
            let price_high_swing = prev_candles.iter().all(|c| current.high > c.high);
            let cvd_lower_high = current.cvd < prev_candles[0].cvd && current.cvd < prev_candles[1].cvd;

            price_high_swing && cvd_lower_high
        } else {
            // For long signals: Price lower low + CVD higher low (bullish divergence)
            let price_low_swing = prev_candles.iter().all(|c| current.low < c.low);
            let cvd_higher_low = current.cvd > prev_candles[0].cvd && current.cvd > prev_candles[1].cvd;

            price_low_swing && cvd_higher_low
        }
    }

    pub async fn calculate_breakeven(&self, entry_price: f64, size: f64, is_buy: bool) -> f64 {
        let fee_rate = {
            let state_lock = self.state.lock().await;
            state_lock.config.strategy.fixed_fee_rate
        };
        let fee = entry_price * size * fee_rate;
        if is_buy {
            entry_price + (fee / size)
        } else {
            entry_price - (fee / size)
        }
    }

    async fn update_simulated_pnl(&self, coin: &str, current_price: f64) {
        let mut state = self.state.lock().await;
        if let Some(position) = state.positions.get_mut(coin) {
            if position.size != 0.0 {
                let pnl = if position.side == "LONG" {
                    (current_price - position.entry_price) * position.size.abs()
                } else {
                    (position.entry_price - current_price) * position.size.abs()
                };
                position.unrealized_pnl = pnl;
            }
        }
    }

    async fn try_enter_position(
        &mut self,
        coin: &str,
        is_buy: bool,
        price: f64,
        stop_loss: f64,
        take_profit: f64,
    ) {
        let state = self.state.lock().await;

        // Check if we already have a position in this coin
        if state.positions.contains_key(coin) {
            return;
        }

        // Check risk management
        let risk_ok = {
            let risk = self.risk.lock().await;
            risk.check_pre_trade(coin, 0.1, price) // Fixed size for now
        };

        if !risk_ok {
            return;
        }

        drop(state);

        // Execute the order
        if let Some(ref gateway) = self.execution {
            match gateway
                .lock()
                .await
                .execute_limit_order(
                    coin,
                    is_buy,
                    0.1, // Fixed position size
                    price,
                    stop_loss,
                    take_profit,
                )
                .await
            {
                Ok(Some(_result)) => {
                    tracing::info!("Position opened for {}, is_buy: {}, price: {}", coin, is_buy, price);
                    // Position tracking is handled in the gateway
                }
                Ok(None) => {
                    tracing::warn!("Order not executed for {}", coin);
                }
                Err(e) => {
                    tracing::error!("Order execution failed for {}: {:?}", coin, e);
                }
            }
        }
    }

    async fn check_sl_tp(&self, coin: &str, price: f64) {
        let state = self.state.lock().await;
        if let Some(position) = state.positions.get(coin) {
            let should_close = if position.side == "LONG" {
                (position.take_profit > 0.0 && price >= position.take_profit)
                    || (position.stop_loss > 0.0 && price <= position.stop_loss)
            } else {
                (position.take_profit > 0.0 && price <= position.take_profit)
                    || (position.stop_loss > 0.0 && price >= position.stop_loss)
            };

            if should_close {
                //                 println!("[INFO] "SL/TP triggered", coin = %coin, price = price, side = %position.side);

                // Close position
                if let Some(ref gateway) = self.execution {
                    let size = position.size.abs();
                    let is_long = position.side == "LONG";
                    if let Err(e) = gateway
                        .lock()
                        .await
                        .close_position(coin, size, is_long)
                        .await
                    {
                        // //                         eprintln!("[ERROR]",  "Position close failed", coin = %coin, error = %e);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    fn create_test_candle(high: f64, low: f64, cvd: f64) -> Candle {
        Candle {
            timestamp: 0,
            open: 100.0,
            high,
            low,
            close: (high + low) / 2.0,
            volume: 1000.0,
            cvd,
            poc: 100.0,
        }
    }

    #[tokio::test]
    async fn test_swing_high_cvd_exhaustion() {
        let state = Arc::new(Mutex::new(GlobalState::new()));
        let risk = Arc::new(Mutex::new(RiskManager::new(&crate::core::config::RiskConfig::default())));
        let strategy = StrategyModule::new(state, None, risk, None);

        let mut history = vec![
            create_test_candle(101.0, 99.0, 10.0),
            create_test_candle(102.0, 100.0, 20.0),
            create_test_candle(103.0, 101.0, 30.0),
            create_test_candle(104.0, 102.0, 40.0),
        ];

        // Scenario: Price makes higher high (105.0) but CVD is lower (35.0) -> Exhaustion
        let current = create_test_candle(105.0, 103.0, 35.0);
        history.push(current);

        assert!(strategy.detect_swing_high(&history));
        assert!(strategy.is_cvd_exhaustion(&history, true));
    }

    #[tokio::test]
    async fn test_swing_low_cvd_exhaustion() {
        let state = Arc::new(Mutex::new(GlobalState::new()));
        let risk = Arc::new(Mutex::new(RiskManager::new(&crate::core::config::RiskConfig::default())));
        let strategy = StrategyModule::new(state, None, risk, None);

        let mut history = vec![
            create_test_candle(105.0, 103.0, 50.0),
            create_test_candle(104.0, 102.0, 40.0),
            create_test_candle(103.0, 101.0, 30.0),
            create_test_candle(102.0, 100.0, 20.0),
        ];

        // Scenario: Price makes lower low (99.0) but CVD is higher (25.0) -> Absorption
        let current = create_test_candle(101.0, 99.0, 25.0);
        history.push(current);

        assert!(strategy.detect_swing_low(&history));
        assert!(strategy.is_cvd_exhaustion(&history, false));
    }
}
