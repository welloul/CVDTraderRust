use cvd_trader_rust::persistence::{Database, Repository, models::*};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    async fn setup_test_db() -> Repository {
        let db = Database::new(":memory:".to_string());
        db.initialize().unwrap();
        Repository::new(db)
    }

    #[tokio::test]
    async fn test_config_persistence() {
        let repo = setup_test_db().await;

        // Test saving config
        let mut config = HashMap::new();
        config.insert("test_key".to_string(), "test_value".to_string());
        config.insert("execution_mode".to_string(), "dryrun".to_string());

        repo.save_config(&config).await.unwrap();

        // Test loading config
        let loaded_config = repo.load_config().await.unwrap();
        assert_eq!(loaded_config.get("test_key"), Some(&"test_value".to_string()));
        assert_eq!(loaded_config.get("execution_mode"), Some(&"dryrun".to_string()));
    }

    #[tokio::test]
    async fn test_position_persistence() {
        let repo = setup_test_db().await;

        // Create test position
        let position = DbPosition {
            coin: "BTC".to_string(),
            size: 1.5,
            entry_price: 45000.0,
            leverage: 5.0,
            unrealized_pnl: 250.0,
            stop_loss: Some(44000.0),
            take_profit: Some(46000.0),
            breakeven: 45025.0,
            side: "LONG".to_string(),
            opened_at: "2024-01-01T12:00:00Z".to_string(),
            entry_reason: Some("CVD divergence".to_string()),
            sl_modifications: vec!["initial_sl".to_string()],
            tp_50_hit: false,
            trailing_sl: Some(44500.0),
            original_tp: Some(47000.0),
        };

        // Save position
        repo.save_position(&position).await.unwrap();

        // Load positions
        let positions = repo.load_positions().await.unwrap();
        assert_eq!(positions.len(), 1);

        let loaded_pos = positions.get("BTC").unwrap();
        assert_eq!(loaded_pos.coin, "BTC");
        assert_eq!(loaded_pos.size, 1.5);
        assert_eq!(loaded_pos.entry_price, 45000.0);
        assert_eq!(loaded_pos.unrealized_pnl, 250.0);
    }

    #[tokio::test]
    async fn test_active_order_persistence() {
        let repo = setup_test_db().await;

        // Create test order
        let order = DbActiveOrder {
            oid: 12345,
            coin: "ETH".to_string(),
            is_buy: true,
            sz: 10.0,
            limit_px: 3000.0,
            order_type: "limit".to_string(),
        };

        // Save order
        repo.save_active_order(&order).await.unwrap();

        // Load orders
        let orders = repo.load_active_orders().await.unwrap();
        assert_eq!(orders.len(), 1);

        let loaded_order = orders.get(&12345).unwrap();
        assert_eq!(loaded_order.coin, "ETH");
        assert_eq!(loaded_order.is_buy, true);
        assert_eq!(loaded_order.sz, 10.0);
        assert_eq!(loaded_order.limit_px, 3000.0);
    }

    #[tokio::test]
    async fn test_performance_metrics_persistence() {
        let repo = setup_test_db().await;

        // Create test metric
        let metric = DbPerformanceMetric {
            id: None,
            timestamp: chrono::Utc::now(),
            metric_type: "latency".to_string(),
            metric_name: "market_data".to_string(),
            value: 15.5,
            coin: Some("BTC".to_string()),
            metadata: Some(serde_json::json!({"source": "websocket"})),
        };

        // Save metric
        repo.save_performance_metric(&metric).await.unwrap();

        // Load recent metrics
        let metrics = repo.get_recent_metrics(10).await.unwrap();
        assert_eq!(metrics.len(), 1);

        let loaded_metric = &metrics[0];
        assert_eq!(loaded_metric.metric_type, "latency");
        assert_eq!(loaded_metric.metric_name, "market_data");
        assert_eq!(loaded_metric.value, 15.5);
        assert_eq!(loaded_metric.coin, Some("BTC".to_string()));
    }

    #[tokio::test]
    async fn test_system_health_persistence() {
        let repo = setup_test_db().await;

        // Create test health record
        let health = DbSystemHealth {
            id: None,
            timestamp: chrono::Utc::now(),
            component: "database".to_string(),
            status: crate::persistence::models::HealthStatus::Healthy,
            message: Some("All systems operational".to_string()),
            metrics: Some(serde_json::json!({"connections": 5, "latency_ms": 12})),
        };

        // Save health record
        repo.save_system_health(&health).await.unwrap();

        // Verify it was saved (we can't easily query health records directly)
        // This test mainly ensures no panics during save
    }

    #[test]
    fn test_database_initialization() {
        let db = Database::new(":memory:".to_string());
        assert!(db.initialize().is_ok());

        let stats = db.get_stats().unwrap();
        // New database should have zero records
        assert_eq!(stats.position_count, 0);
        assert_eq!(stats.active_order_count, 0);
        assert_eq!(stats.closed_trade_count, 0);
    }
}