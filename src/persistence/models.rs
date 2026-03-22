use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

// Database models that mirror the in-memory structures
// These are used for persistence and can be slightly different from runtime models

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPosition {
    pub coin: String,
    pub size: f64,
    pub entry_price: f64,
    pub leverage: f64,
    pub unrealized_pnl: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub breakeven: f64,
    pub side: String,
    pub opened_at: String,
    pub entry_reason: Option<String>,
    pub sl_modifications: Vec<String>,
    pub tp_50_hit: bool,
    pub trailing_sl: Option<f64>,
    pub original_tp: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbActiveOrder {
    pub oid: i64,
    pub coin: String,
    pub is_buy: bool,
    pub sz: f64,
    pub limit_px: f64,
    pub order_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbClosedTrade {
    pub id: String,
    pub coin: String,
    pub side: String,
    pub size: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub pnl: f64,
    pub reason: String,
    pub entry_reason: Option<String>,
    pub sl_modifications: Vec<String>,
    pub opened_at: String,
    pub closed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbConfig {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPerformanceMetric {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub metric_type: String, // 'latency', 'pnl', 'error_rate', 'throughput'
    pub metric_name: String,
    pub value: f64,
    pub coin: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSystemHealth {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

// Conversion implementations between in-memory and database models
impl From<crate::core::state::Position> for DbPosition {
    fn from(pos: crate::core::state::Position) -> Self {
        DbPosition {
            coin: pos.coin,
            size: pos.size,
            entry_price: pos.entry_price,
            leverage: pos.leverage,
            unrealized_pnl: pos.unrealized_pnl,
            stop_loss: Some(pos.stop_loss),
            take_profit: Some(pos.take_profit),
            breakeven: pos.breakeven,
            side: pos.side,
            opened_at: pos.opened_at,
            entry_reason: Some(pos.entry_reason),
            sl_modifications: pos.sl_modifications,
            tp_50_hit: pos.tp_50_hit,
            trailing_sl: Some(pos.trailing_sl),
            original_tp: Some(pos.original_tp),
        }
    }
}

impl From<crate::core::state::ActiveOrder> for DbActiveOrder {
    fn from(order: crate::core::state::ActiveOrder) -> Self {
        DbActiveOrder {
            oid: order.oid,
            coin: order.coin,
            is_buy: order.is_buy,
            sz: order.sz,
            limit_px: order.limit_px,
            order_type: order.order_type,
        }
    }
}

impl From<crate::core::state::ClosedTrade> for DbClosedTrade {
    fn from(trade: crate::core::state::ClosedTrade) -> Self {
        DbClosedTrade {
            id: trade.id,
            coin: trade.coin,
            side: trade.side,
            size: trade.size,
            entry_price: trade.entry_price,
            exit_price: trade.exit_price,
            pnl: trade.pnl,
            reason: trade.reason,
            entry_reason: trade.entry_reason,
            sl_modifications: trade.sl_modifications,
            opened_at: trade.opened_at,
            closed_at: trade.closed_at,
        }
    }
}