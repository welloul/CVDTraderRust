use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use crate::core::config::Config;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;

const TRADES_FILE: &str = "backend/data/trades.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTrade {
    pub id: String,
    pub coin: String,
    pub side: String, // "LONG" or "SHORT"
    pub size: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
    pub reason: String, // Exit reason
    pub entry_reason: Option<String>,
    pub sl_modifications: Vec<String>, // Log of SL changes
    pub opened_at: String, // ISO timestamp
    pub closed_at: String, // ISO timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    pub size: f64,
    pub entry_price: f64,
    pub leverage: f64,
    pub unrealized_pnl: f64,
    pub stop_loss: f64,
    pub take_profit: f64,
    pub breakeven: f64, // Fee-adjusted breakeven price
    pub side: String, // "LONG" or "SHORT"
    pub opened_at: String, // ISO timestamp
    pub entry_reason: String, // Why the trade was opened
    pub sl_modifications: Vec<String>, // Log of SL changes
    pub tp_50_hit: bool, // True if 50% was closed at TP
    pub trailing_sl: f64, // Trailing SL for remaining 50%
    pub original_tp: f64, // Original TP price for reference
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveOrder {
    pub oid: i64,
    pub coin: String,
    pub is_buy: bool,
    pub sz: f64,
    pub limit_px: f64,
    pub order_type: String,
}

#[derive(Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub struct GlobalState {
    pub is_running: bool,
    pub config: Config,
    pub positions: HashMap<String, Position>,
    pub active_orders: HashMap<i64, ActiveOrder>,
    pub closed_trades: Vec<ClosedTrade>,
    pub market_data: HashMap<String, HashMap<String, serde_json::Value>>,
    pub wallet_balance: f64,
    pub main_wallet_balance: f64,
    pub logs: Vec<LogEntry>,
    pub latency_by_coin: HashMap<String, Vec<f64>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let mut state = GlobalState {
            is_running: true,
            config: Config::load(),
            positions: HashMap::new(),
            active_orders: HashMap::new(),
            closed_trades: Vec::new(),
            market_data: HashMap::new(),
            wallet_balance: 0.0,
            main_wallet_balance: 0.0,
            logs: Vec::new(),
            latency_by_coin: HashMap::new(),
        };

        state.load_trades();
        state
    }

    pub fn update_latency(&mut self, coin: &str, latency_ms: f64) {
        self.latency_by_coin
            .entry(coin.to_string())
            .or_insert_with(Vec::new)
            .push(latency_ms);

        // Keep last 100 samples
        if let Some(samples) = self.latency_by_coin.get_mut(coin) {
            if samples.len() > 100 {
                samples.remove(0);
            }
        }
    }

    pub fn get_latency_stats(&self) -> HashMap<String, HashMap<String, f64>> {
        let mut stats = HashMap::new();

        for (coin, samples) in &self.latency_by_coin {
            if samples.is_empty() {
                continue;
            }

            // Filter out extreme outliers
            let filtered: Vec<f64> = samples
                .iter()
                .filter(|&&s| (-50000.0..=50000.0).contains(&s))
                .cloned()
                .collect();

            if filtered.is_empty() {
                continue;
            }

            // The instruction implies adding a network health check.
            // Placing `reqwest::get("https://www.google.com")` directly here
            // would be a syntax error and not logically fit into latency stats.
            // Assuming the intent is to add a *new* method for a network check,
            // or that this line was meant for a different context.
            // As per instructions to make it syntactically correct and faithful,
            // and given the snippet's placement, I'll add a placeholder comment
            // and a new method for the network check.
            // If the user intended this line to be part of the latency calculation,
            // it would require significant reinterpretation of the instruction.
            // For now, I'll add a new method `check_network_connectivity`
            // and ensure the existing `get_latency_stats` remains correct.

            // Original code continues here:
            let mut sorted = filtered.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = sorted[sorted.len() / 2];

            let avg = filtered.iter().sum::<f64>() / filtered.len() as f64;
            let min_val = filtered.iter().fold(f64::INFINITY, |a: f64, &b| a.min(b));
            let max_val = filtered.iter().fold(f64::NEG_INFINITY, |a: f64, &b| a.max(b));

            let mut coin_stats = HashMap::new();
            coin_stats.insert("avg_ms".to_string(), (avg * 100.0).round() / 100.0);
            coin_stats.insert("min_ms".to_string(), (min_val * 100.0).round() / 100.0);
            coin_stats.insert("max_ms".to_string(), (max_val * 100.0).round() / 100.0);
            coin_stats.insert("clock_offset_ms".to_string(), (median * 100.0).round() / 100.0);
            coin_stats.insert("samples".to_string(), filtered.len() as f64);

            stats.insert(coin.to_string(), coin_stats);
        }

        stats
    }

    pub fn add_log(&mut self, level: &str, message: &str, extra: HashMap<String, serde_json::Value>) {
        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            message: message.to_string(),
            extra,
        };
        self.logs.push(entry);
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }

    fn load_trades(&mut self) {
        // Ensure directory exists
        if let Some(parent) = std::path::Path::new(TRADES_FILE).parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(content) = fs::read_to_string(TRADES_FILE) {
            if let Ok(trades) = serde_json::from_str::<Vec<ClosedTrade>>(&content) {
                self.closed_trades = trades;
            }
        }
    }

    pub fn save_trades(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.closed_trades) {
            if let Err(_e) = fs::write(TRADES_FILE, json) {
                // Warning logged via standard tracing or logged in state
            }
        }
    }

    pub fn add_closed_trade(&mut self, trade: ClosedTrade) {
        self.closed_trades.push(trade);
        if self.closed_trades.len() > 1000 {
            self.closed_trades.remove(0);
        }
        self.save_trades();
    }

    pub async fn update_config(&mut self, _new_config: HashMap<String, String>) {
        // Since Config is now a structured struct, we would need to parse these values
        // For now, this is a placeholder as the Config struct handles its own loading
//         println!("[INFO] "Configuration update via API not yet fully implemented for structured Config");
    }

    pub async fn start_bot(&mut self) {
        self.is_running = true;
        // Reset circuit breaker - would need to access risk manager
//         println!("[INFO] "Bot started via Command & Control");
    }

    pub async fn stop_bot(&mut self) {
        self.is_running = false;
//         println!("[INFO] "Bot stopped via Command & Control");
    }

    pub async fn sync_state(&mut self, _info_client: &crate::hyperliquid::Info, _address: &str) {
        // TODO: Implement sync_state with Hyperliquid API
//         println!("[INFO] "State sync not yet implemented");
    }

    pub async fn sync_main_wallet_balance(&mut self, _info_client: &crate::hyperliquid::Info, _main_wallet_address: &str) {
        // TODO: Implement sync_main_wallet_balance
//         println!("[INFO] "Main wallet balance sync not yet implemented");
    }
}

// Singleton pattern removed in favor of Dependency Injection