use serde_json::Value;
use crate::core::rounding::RoundingUtil;
use crate::hyperliquid::Exchange;
use crate::persistence::Repository;
use anyhow::{Result, Context};
use crate::core::state::{GlobalState, ActiveOrder};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ExecutionGateway {
    exchange: Option<Exchange>,
    rounding_util: RoundingUtil,
    state: Arc<Mutex<GlobalState>>,
    ttl_tracker: Option<Arc<Mutex<super::ttl::OrderTTLTracker>>>,
}

impl ExecutionGateway {
    pub fn new(
        exchange: Option<Exchange>,
        rounding_util: RoundingUtil,
        state: Arc<Mutex<GlobalState>>,
        ttl_tracker: Option<Arc<Mutex<super::ttl::OrderTTLTracker>>>
    ) -> Self {
        Self {
            exchange,
            rounding_util,
            state,
            ttl_tracker,
        }
    }

    pub async fn execute_limit_order(
        &self,
        coin: &str,
        is_buy: bool,
        sz: f64,
        limit_px: f64,
        _stop_loss: f64,
        _take_profit: f64,
    ) -> Result<Option<Value>> {
        let state_lock = self.state.lock().await;
        let execution_mode = state_lock.config.execution.mode.clone();
        drop(state_lock);

        // Round sizes and prices
        let rounded_sz_str = self.rounding_util.round_size(coin, sz);
        let rounded_sz: f64 = rounded_sz_str.parse().unwrap_or(0.0);
        let rounded_px_str = self.rounding_util.round_price(coin, limit_px);
        let rounded_px: f64 = rounded_px_str.parse().unwrap_or(0.0);

        if rounded_sz <= 0.0 {
            return Ok(None);
        }

        if execution_mode == "dryrun" {
            let mut state_lock = self.state.lock().await;
            
            // Initialize wallet balance for simulation if empty
            if state_lock.wallet_balance == 0.0 {
                state_lock.wallet_balance = 1000.0;
            }

            let new_pos = crate::core::state::Position {
                coin: coin.to_string(),
                size: rounded_sz,
                entry_price: rounded_px,
                leverage: 1.0,
                unrealized_pnl: 0.0,
                stop_loss: _stop_loss,
                take_profit: _take_profit,
                breakeven: rounded_px,
                side: if is_buy { "LONG".to_string() } else { "SHORT".to_string() },
                opened_at: chrono::Utc::now().to_rfc3339(),
                entry_reason: "Strategy Signal".to_string(),
                sl_modifications: Vec::new(),
                tp_50_hit: false,
                trailing_sl: 0.0,
                original_tp: _take_profit,
            };

            state_lock.positions.insert(coin.to_string(), new_pos);
            drop(state_lock);

            // Simulate success
            return Ok(Some(serde_json::json!({
                "status": "ok",
                "response": {
                    "data": {
                        "statuses": [{
                            "filled": {"oid": 12345}
                        }]
                    }
                }
            })));
        }

        // Live/Testnet execution
        let order_params = serde_json::json!({
            "type": "order",
            "coin": coin,
            "side": if is_buy { "B" } else { "A" },
            "sz": rounded_sz,
            "px": rounded_px,
            "orderType": {
                "limit": {
                    "tif": "Alo"  // Post-only
                }
            }
        });

        let exchange = match self.exchange {
            Some(ref ex) => ex,
            None => return Err(anyhow::anyhow!("Exchange not configured for real execution")),
        };

        match exchange.place_order(order_params).await {
            Ok(result) => {

                // Track the order
                if let Some(statuses) = result.get("response")
                    .and_then(|r| r.get("data"))
                    .and_then(|d| d.get("statuses"))
                    .and_then(|s| s.as_array()) {

                    if let Some(status) = statuses.get(0) {
                        let oid = if let Some(filled) = status.get("filled") {
                            filled.get("oid").and_then(|o| o.as_i64())
                        } else if let Some(resting) = status.get("resting") {
                            resting.get("oid").and_then(|o| o.as_i64())
                        } else {
                            None
                        };

                        if let Some(order_id) = oid {
                            let mut state_lock = self.state.lock().await;
                            state_lock.active_orders.insert(order_id as i64, ActiveOrder {
                                oid: order_id as i64,
                                coin: coin.to_string(),
                                is_buy,
                                sz: rounded_sz,
                                limit_px: rounded_px,
                                order_type: "limit".to_string(),
                            });

                            // Track in TTL tracker
                            if let Some(ref tracker) = self.ttl_tracker {
                                // TODO: Implement TTL tracking
                            }
                        }
                    }
                }

                Ok(Some(result))
            }
            Err(e) => {
                Err(anyhow::anyhow!(e))
            }
        }
    }

    pub async fn close_position(
        &self,
        coin: &str,
        size: f64,
        is_long: bool,
    ) -> Result<Option<Value>> {
        let state_lock = self.state.lock().await;
        let execution_mode = state_lock.config.execution.mode.clone();
        drop(state_lock);

        // Market order to close position
        let order_params = serde_json::json!({
            "type": "order",
            "coin": coin,
            "side": if is_long { "A" } else { "B" }, // Opposite side to close
            "sz": size,
            "px": 0,  // Market order
            "orderType": {
                "market": {}
            }
        });

        let mut exit_price_str = "0.0".to_string();
        if execution_mode == "dryrun" {
            let state_lock = self.state.lock().await;
            if let Some(coin_data) = state_lock.market_data.get(coin) {
                if let Some(prices) = coin_data.get("price").and_then(|p| p.as_array()) {
                    if let Some(last_price) = prices.last().and_then(|p| p.as_f64()) {
                        exit_price_str = last_price.to_string();
                    }
                }
            }
        }

        let result = match execution_mode.as_str() {
            "dryrun" => Ok(Some(serde_json::json!({"status": "ok", "response": {"data": {"statuses": [{"filled": {"avgPx": exit_price_str, "oid": 0}}]}}}))),
            _ => if let Some(ref exch) = self.exchange {
                exch.place_order(order_params).await.map(Some).map_err(|e| anyhow::anyhow!(e))
            } else {
                Err(anyhow::anyhow!("Exchange not configured for real execution"))
            },
        };

        if let Ok(Some(ref res)) = result {
            let mut state_lock = self.state.lock().await;
            if let Some(position) = state_lock.positions.remove(coin) {
                // Simplified PNL calculation for the export
                let exit_price = res["response"]["data"]["statuses"][0]["filled"]["avgPx"]
                    .as_str()
                    .unwrap_or("0.0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                
                let pnl = if position.side == "LONG" {
                    (exit_price - position.entry_price) * position.size
                } else {
                    (position.entry_price - exit_price) * position.size
                };

                state_lock.wallet_balance += pnl;

                let closed_trade = crate::core::state::ClosedTrade {
                    id: uuid::Uuid::new_v4().to_string(),
                    coin: coin.to_string(),
                    side: position.side.clone(),
                    size: position.size,
                    entry_price: position.entry_price,
                    exit_price: if exit_price == 0.0 { position.entry_price } else { exit_price },
                    pnl,
                    reason: "SL/TP or Manual".to_string(),
                    entry_reason: Some(position.entry_reason.clone()),
                    sl_modifications: position.sl_modifications.clone(),
                    opened_at: position.opened_at.clone(),
                    closed_at: chrono::Utc::now().to_rfc3339(),
                };

                state_lock.add_closed_trade(closed_trade);
            }
        }

        result
    }
}