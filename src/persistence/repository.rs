use anyhow::{Result, Context};
use crate::persistence::{database::Database, models::*};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Repository {
    pub db: Database,
}

impl Repository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn save_position(
        &self,
        position: &DbPosition,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;

        // Clone all data that will be moved into the closure
        let coin = position.coin.clone();
        let size = position.size;
        let entry_price = position.entry_price;
        let leverage = position.leverage;
        let unrealized_pnl = position.unrealized_pnl;
        let stop_loss = position.stop_loss;
        let take_profit = position.take_profit;
        let breakeven = position.breakeven;
        let side = position.side.clone();
        let opened_at = position.opened_at.clone();
        let entry_reason = position.entry_reason.clone();
        let sl_modifications =
            serde_json::to_string(&position.sl_modifications).unwrap_or_default();
        let tp_50_hit = position.tp_50_hit;
        let trailing_sl = position.trailing_sl;
        let original_tp = position.original_tp;

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO positions
                (coin, size, entry_price, leverage, unrealized_pnl, stop_loss, take_profit,
                 breakeven, side, opened_at, entry_reason, sl_modifications, tp_50_hit,
                 trailing_sl, original_tp, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                "#,
                (
                    &coin,
                    &size,
                    &entry_price,
                    &leverage,
                    &unrealized_pnl,
                    &stop_loss,
                    &take_profit,
                    &breakeven,
                    &side,
                    &opened_at,
                    &entry_reason,
                    &sl_modifications,
                    &tp_50_hit,
                    &trailing_sl,
                    &original_tp,
                ),
            )?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn load_positions(
        &self,
    ) -> Result<HashMap<String, DbPosition>> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let positions = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                "SELECT coin, size, entry_price, leverage, unrealized_pnl, stop_loss, take_profit,
                        breakeven, side, opened_at, entry_reason, sl_modifications, tp_50_hit,
                        trailing_sl, original_tp FROM positions"
            )?;

                let position_iter = stmt.query_map([], |row| {
                    Ok(DbPosition {
                        coin: row.get(0)?,
                        size: row.get(1)?,
                        entry_price: row.get(2)?,
                        leverage: row.get(3)?,
                        unrealized_pnl: row.get(4)?,
                        stop_loss: row.get(5)?,
                        take_profit: row.get(6)?,
                        breakeven: row.get(7)?,
                        side: row.get(8)?,
                        opened_at: row.get(9)?,
                        entry_reason: row.get(10)?,
                        sl_modifications: serde_json::from_str(&row.get::<_, String>(11)?)
                            .unwrap_or_default(),
                        tp_50_hit: row.get(12)?,
                        trailing_sl: row.get(13)?,
                        original_tp: row.get(14)?,
                    })
                })?;

                let mut positions = HashMap::new();
                for position in position_iter {
                    let pos = position?;
                    positions.insert(pos.coin.clone(), pos);
                }

                Ok(positions)
            })
            .await?;

        Ok(positions)
    }

    pub async fn save_active_order(
        &self,
        order: &DbActiveOrder,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let oid = order.oid.clone();
        let coin = order.coin.clone();
        let is_buy = order.is_buy;
        let sz = order.sz;
        let limit_px = order.limit_px;
        let order_type = order.order_type.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT OR REPLACE INTO active_orders
                (oid, coin, is_buy, sz, limit_px, order_type, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                "#,
                (&oid, &coin, &is_buy, &sz, &limit_px, &order_type),
            )?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn load_active_orders(
        &self,
    ) -> Result<HashMap<i64, DbActiveOrder>> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let orders = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT oid, coin, is_buy, sz, limit_px, order_type FROM active_orders",
                )?;

                let order_iter = stmt.query_map([], |row| {
                    Ok(DbActiveOrder {
                        oid: row.get(0)?,
                        coin: row.get(1)?,
                        is_buy: row.get(2)?,
                        sz: row.get(3)?,
                        limit_px: row.get(4)?,
                        order_type: row.get(5)?,
                    })
                })?;

                let mut orders = HashMap::new();
                for order in order_iter {
                    let ord = order?;
                    orders.insert(ord.oid, ord);
                }

                Ok(orders)
            })
            .await?;

        Ok(orders)
    }

    pub async fn save_closed_trade(
        &self,
        trade: &DbClosedTrade,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;

        // Clone all data that will be moved into the closure
        let id = trade.id.clone();
        let coin = trade.coin.clone();
        let side = trade.side.clone();
        let size = trade.size;
        let entry_price = trade.entry_price;
        let exit_price = trade.exit_price;
        let pnl = trade.pnl;
        let reason = trade.reason.clone();
        let entry_reason = trade.entry_reason.clone();
        let sl_modifications = serde_json::to_string(&trade.sl_modifications).unwrap_or_default();
        let opened_at = trade.opened_at.clone();
        let closed_at = trade.closed_at.clone();

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT OR IGNORE INTO closed_trades
                (id, coin, side, size, entry_price, exit_price, pnl, reason,
                 entry_reason, sl_modifications, opened_at, closed_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                (
                    &id,
                    &coin,
                    &side,
                    &size,
                    &entry_price,
                    &exit_price,
                    &pnl,
                    &reason,
                    &entry_reason,
                    &sl_modifications,
                    &opened_at,
                    &closed_at,
                ),
            )?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn save_config(
        &self,
        config: &HashMap<String, String>,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let config_clone = config.clone();

        conn.call(move |conn| {
            for (key, value) in config_clone {
                conn.execute(
                    "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)",
                    (key, value),
                )?;
            }
            Ok(())
        }).await?;

        Ok(())
    }

    pub async fn load_config(&self) -> Result<HashMap<String, String>> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let config = conn
            .call(move |conn| {
                let mut stmt = conn.prepare("SELECT key, value FROM config")?;
                let config_iter = stmt.query_map([], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                })?;

                let mut config = HashMap::new();
                for item in config_iter {
                    let (key, value) = item?;
                    config.insert(key, value);
                }

                Ok(config)
            })
            .await?;

        Ok(config)
    }

    pub async fn save_performance_metric(
        &self,
        metric: &DbPerformanceMetric,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let timestamp = metric.timestamp;
        let metric_type = metric.metric_type.clone();
        let metric_name = metric.metric_name.clone();
        let value = metric.value;
        let coin = metric.coin.clone();
        let metadata = metric
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO performance_metrics
                (timestamp, metric_type, metric_name, value, coin, metadata)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                (
                    &timestamp,
                    &metric_type,
                    &metric_name,
                    &value,
                    &coin,
                    &metadata,
                ),
            )?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn save_system_health(
        &self,
        health: &DbSystemHealth,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;

        // Clone all data that will be moved into the closure
        let timestamp = health.timestamp;
        let component = health.component.clone();
        let status = serde_json::to_string(&health.status).unwrap_or_default();
        let message = health.message.clone();
        let metrics = health
            .metrics
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_default());

        conn.call(move |conn| {
            conn.execute(
                r#"
                INSERT INTO system_health
                (timestamp, component, status, message, metrics)
                VALUES (?, ?, ?, ?, ?)
                "#,
                (&timestamp, &component, &status, &message, &metrics),
            )?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    pub async fn get_recent_metrics(
        &self,
        limit: i64,
    ) -> Result<Vec<DbPerformanceMetric>> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;
        let metrics = conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, timestamp, metric_type, metric_name, value, coin, metadata
                 FROM performance_metrics ORDER BY timestamp DESC LIMIT ?",
                )?;
                let mut metrics = Vec::new();
                for result in stmt.query_map([limit], |row| {
                    Ok(DbPerformanceMetric {
                        id: Some(row.get(0)?),
                        timestamp: row.get(1)?,
                        metric_type: row.get(2)?,
                        metric_name: row.get(3)?,
                        value: row.get(4)?,
                        coin: row.get(5)?,
                        metadata: row
                            .get::<_, Option<String>>(6)?
                            .and_then(|m| serde_json::from_str(&m).ok()),
                    })
                })? {
                    metrics.push(result?);
                }
                Ok(metrics)
            })
            .await?;

        Ok(metrics)
    }

    pub async fn cleanup_old_data(
        &self,
        days_to_keep: i64,
    ) -> Result<()> {
        let conn = self.db.get_async_connection().await.context("Failed to get DB connection")?;

        conn.call(move |conn| {
            // Clean up old performance metrics (keep last 30 days)
            conn.execute(
                "DELETE FROM performance_metrics WHERE timestamp < datetime('now', '-' || ? || ' days')",
                [days_to_keep],
            )?;

            // Clean up old system health records (keep last 7 days)
            conn.execute(
                "DELETE FROM system_health WHERE timestamp < datetime('now', '-7 days')",
                [],
            )?;

            Ok(())
        }).await?;

        //         println!("[INFO] "Database cleanup completed", days_kept = days_to_keep);
        Ok(())
    }
}
