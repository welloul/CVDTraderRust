use rusqlite::Connection;
use tokio_rusqlite::Connection as AsyncConnection;
use anyhow::Result;
use std::path::Path;

#[derive(Clone)]
pub struct Database {
    path: String,
}

impl Database {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn initialize(&self) -> Result<()> {
        let conn = Connection::open(&self.path)?;

        // Create tables
        conn.execute_batch(
            r#"
            -- Configuration table
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Positions table
            CREATE TABLE IF NOT EXISTS positions (
                coin TEXT PRIMARY KEY,
                size REAL NOT NULL,
                entry_price REAL NOT NULL,
                leverage REAL NOT NULL,
                unrealized_pnl REAL NOT NULL DEFAULT 0.0,
                stop_loss REAL,
                take_profit REAL,
                breakeven REAL,
                side TEXT NOT NULL,
                opened_at TEXT NOT NULL,
                entry_reason TEXT,
                sl_modifications TEXT, -- JSON array
                tp_50_hit BOOLEAN DEFAULT FALSE,
                trailing_sl REAL,
                original_tp REAL,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Active orders table
            CREATE TABLE IF NOT EXISTS active_orders (
                oid INTEGER PRIMARY KEY,
                coin TEXT NOT NULL,
                is_buy BOOLEAN NOT NULL,
                sz REAL NOT NULL,
                limit_px REAL NOT NULL,
                order_type TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Closed trades table
            CREATE TABLE IF NOT EXISTS closed_trades (
                id TEXT PRIMARY KEY,
                coin TEXT NOT NULL,
                side TEXT NOT NULL,
                size REAL NOT NULL,
                entry_price REAL NOT NULL,
                exit_price REAL NOT NULL,
                pnl REAL NOT NULL,
                reason TEXT NOT NULL,
                entry_reason TEXT,
                sl_modifications TEXT, -- JSON array
                opened_at TEXT NOT NULL,
                closed_at TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            -- Performance metrics table
            CREATE TABLE IF NOT EXISTS performance_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                metric_type TEXT NOT NULL, -- 'latency', 'pnl', 'error_rate', etc.
                metric_name TEXT NOT NULL,
                value REAL NOT NULL,
                coin TEXT, -- Optional, for coin-specific metrics
                metadata TEXT -- JSON additional data
            );

            -- System health table
            CREATE TABLE IF NOT EXISTS system_health (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                component TEXT NOT NULL,
                status TEXT NOT NULL, -- 'healthy', 'degraded', 'unhealthy'
                message TEXT,
                metrics TEXT -- JSON performance metrics
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_positions_coin ON positions(coin);
            CREATE INDEX IF NOT EXISTS idx_active_orders_coin ON active_orders(coin);
            CREATE INDEX IF NOT EXISTS idx_closed_trades_coin ON closed_trades(coin);
            CREATE INDEX IF NOT EXISTS idx_closed_trades_closed_at ON closed_trades(closed_at);
            CREATE INDEX IF NOT EXISTS idx_performance_timestamp ON performance_metrics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_performance_type_name ON performance_metrics(metric_type, metric_name);
            CREATE INDEX IF NOT EXISTS idx_health_timestamp ON system_health(timestamp);
            CREATE INDEX IF NOT EXISTS idx_health_component ON system_health(component);
            "#
        )?;

        //         println!("[INFO] "Database initialized", path = %self.path);
        Ok(())
    }

    pub async fn get_async_connection(
        &self,
    ) -> Result<AsyncConnection> {
        let conn = AsyncConnection::open(&self.path).await?;
        Ok(conn)
    }

    pub fn backup(&self, backup_path: &Path) -> Result<()> {
        let _conn = Connection::open(&self.path)?;
        let _backup_conn = Connection::open(backup_path)?;

        // Perform SQLite backup
        // conn.backup(rusqlite::DatabaseName::Main, &backup_conn, None)?;
        // TODO: Implement backup with correct rusqlite API

        //         println!("[INFO] "Database backup completed", from = %self.path, to = %backup_path.display());
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.get_async_connection().await?;

        let stats = conn.call(|conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    (SELECT COUNT(*) FROM positions) as position_count,
                    (SELECT COUNT(*) FROM active_orders) as active_order_count,
                    (SELECT COUNT(*) FROM closed_trades) as closed_trade_count,
                    (SELECT COUNT(*) FROM performance_metrics) as metric_count,
                    (SELECT COUNT(*) FROM system_health WHERE status != 'healthy') as unhealthy_events
                "#,
            )?;

            let stats = stmt.query_row([], |row| {
                Ok(DatabaseStats {
                    position_count: row.get(0)?,
                    active_order_count: row.get(1)?,
                    closed_trade_count: row.get(2)?,
                    metric_count: row.get(3)?,
                    unhealthy_events: row.get(4)?,
                })
            })?;

            Ok(stats)
        }).await?;

        Ok(stats)
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub position_count: i64,
    pub active_order_count: i64,
    pub closed_trade_count: i64,
    pub metric_count: i64,
    pub unhealthy_events: i64,
}

impl Default for Database {
    fn default() -> Self {
        Self::new("cvd_trader.db".to_string())
    }
}
