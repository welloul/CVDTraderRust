use cvd_trader_rust::persistence::{Database, Repository};
use cvd_trader_rust::monitoring::{HealthChecker, MetricsCollector, alerts::AlertManager};
use cvd_trader_rust::core::state::GlobalState;
use cvd_trader_rust::market_data::candles::{Candle, CandleBuilder};
use cvd_trader_rust::strategy::module::StrategyModule;
use cvd_trader_rust::risk::manager::RiskManager;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod end_to_end_tests {
    use super::*;

    async fn setup_full_system() -> (Arc<Mutex<GlobalState>>, Repository, HealthChecker, MetricsCollector) {
        // Initialize database
        let db = Database::new(":memory:".to_string());
        db.initialize().unwrap();
        let repository = Repository::new(db);

        // Initialize state
        let state = Arc::new(Mutex::new(GlobalState::new()));

        // Initialize monitoring
        cvd_trader_rust::monitoring::metrics::MetricsCollector::init();
        let metrics = MetricsCollector::new(repository.clone());
        let health = HealthChecker::new(Arc::clone(&state), repository.clone());

        (state, repository, health, metrics)
    }

    #[tokio::test]
    async fn test_complete_system_initialization() {
        let (state, repository, health_checker, metrics_collector) = setup_full_system().await;

        // Verify database is initialized
        let stats = repository.db.get_stats().unwrap();
        assert_eq!(stats.position_count, 0);
        assert_eq!(stats.active_order_count, 0);
        assert_eq!(stats.closed_trade_count, 0);

        // Verify health checker works
        let health_status = health_checker.check_health().await.unwrap();
        assert!(health_status.components.len() >= 3); // At least database, state, memory checks

        // Verify state is properly initialized
        let state_guard = state.lock().await;
        assert!(!state_guard.is_running);
        assert!(state_guard.positions.is_empty());
        assert!(state_guard.active_orders.is_empty());
        assert!(state_guard.closed_trades.is_empty());

        // Verify metrics collector is initialized (implicitly tested by no panics)
        drop(state_guard);
    }

    #[tokio::test]
    async fn test_state_persistence_workflow() {
        let (state, repository, _health, _metrics) = setup_full_system().await;

        // Create a position in state
        {
            let mut state_guard = state.lock().await;
            state_guard.positions.insert("BTC".to_string(), cvd_trader_rust::core::state::Position {
                coin: "BTC".to_string(),
                size: 1.5,
                entry_price: 45000.0,
                leverage: 5.0,
                unrealized_pnl: 250.0,
                stop_loss: 44000.0,
                take_profit: 46000.0,
                breakeven: 45025.0,
                side: "LONG".to_string(),
                opened_at: "2024-01-01T12:00:00Z".to_string(),
                entry_reason: "Test signal".to_string(),
                sl_modifications: vec!["initial".to_string()],
                tp_50_hit: false,
                trailing_sl: 44500.0,
                original_tp: 47000.0,
            });
        }

        // Save state to database
        let positions_to_save = {
            let state_guard = state.lock().await;
            state_guard.positions.values()
                .map(|pos| cvd_trader_rust::persistence::models::DbPosition::from(pos.clone()))
                .collect::<Vec<_>>()
        };

        for position in positions_to_save {
            repository.save_position(&position).await.unwrap();
        }

        // Clear in-memory state
        {
            let mut state_guard = state.lock().await;
            state_guard.positions.clear();
            assert!(state_guard.positions.is_empty());
        }

        // Load state from database
        let loaded_positions = repository.load_positions().await.unwrap();
        assert_eq!(loaded_positions.len(), 1);
        assert!(loaded_positions.contains_key("BTC"));

        let loaded_btc = loaded_positions.get("BTC").unwrap();
        assert_eq!(loaded_btc.coin, "BTC");
        assert_eq!(loaded_btc.size, 1.5);
        assert_eq!(loaded_btc.entry_price, 45000.0);
        assert_eq!(loaded_btc.unrealized_pnl, 250.0);
    }

    #[tokio::test]
    async fn test_configuration_persistence() {
        let (_state, repository, _health, _metrics) = setup_full_system().await;

        // Save configuration
        let mut config = std::collections::HashMap::new();
        config.insert("execution_mode".to_string(), "dryrun".to_string());
        config.insert("max_leverage".to_string(), "5".to_string());
        config.insert("risk_multiplier".to_string(), "2.0".to_string());

        repository.save_config(&config).await.unwrap();

        // Load configuration
        let loaded_config = repository.load_config().await.unwrap();

        assert_eq!(loaded_config.get("execution_mode"), Some(&"dryrun".to_string()));
        assert_eq!(loaded_config.get("max_leverage"), Some(&"5".to_string()));
        assert_eq!(loaded_config.get("risk_multiplier"), Some(&"2.0".to_string()));
    }

    #[tokio::test]
    async fn test_trade_lifecycle_with_persistence() {
        let (state, repository, _health, _metrics) = setup_full_system().await;

        // Simulate a complete trade lifecycle
        let trade_id = "test_trade_123".to_string();

        // 1. Create and save position
        let position = cvd_trader_rust::persistence::models::DbPosition {
            coin: "ETH".to_string(),
            size: 10.0,
            entry_price: 3000.0,
            leverage: 3.0,
            unrealized_pnl: 150.0,
            stop_loss: Some(2900.0),
            take_profit: Some(3200.0),
            breakeven: 3025.0,
            side: "LONG".to_string(),
            opened_at: "2024-01-01T10:00:00Z".to_string(),
            entry_reason: Some("CVD divergence signal".to_string()),
            sl_modifications: vec!["entry_sl".to_string()],
            tp_50_hit: false,
            trailing_sl: Some(2950.0),
            original_tp: Some(3300.0),
        };

        repository.save_position(&position).await.unwrap();

        // 2. Create and save order
        let order = cvd_trader_rust::persistence::models::DbActiveOrder {
            oid: 999,
            coin: "ETH".to_string(),
            is_buy: true,
            sz: 10.0,
            limit_px: 3000.0,
            order_type: "limit".to_string(),
        };

        repository.save_active_order(&order).await.unwrap();

        // 3. Close trade and save to history
        let closed_trade = cvd_trader_rust::persistence::models::DbClosedTrade {
            id: trade_id.clone(),
            coin: "ETH".to_string(),
            side: "LONG".to_string(),
            size: 10.0,
            entry_price: 3000.0,
            exit_price: 3150.0,
            pnl: 1500.0, // 10 * (3150 - 3000) * 3 leverage, minus fees approx
            reason: "take_profit".to_string(),
            entry_reason: Some("CVD divergence signal".to_string()),
            sl_modifications: vec!["entry_sl".to_string(), "breakeven".to_string()],
            opened_at: "2024-01-01T10:00:00Z".to_string(),
            closed_at: "2024-01-01T15:30:00Z".to_string(),
        };

        repository.save_closed_trade(&closed_trade).await.unwrap();

        // 4. Verify persistence
        let positions = repository.load_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert!(positions.contains_key("ETH"));

        let orders = repository.load_active_orders().await.unwrap();
        assert_eq!(orders.len(), 1);
        assert!(orders.contains_key(&999));

        // 5. Verify database stats
        let stats = repository.db.get_stats().unwrap();
        assert_eq!(stats.position_count, 1);
        assert_eq!(stats.active_order_count, 1);
        assert_eq!(stats.closed_trade_count, 1);
    }

    #[tokio::test]
    async fn test_monitoring_system_integration() {
        let (state, repository, health_checker, metrics_collector) = setup_full_system().await;

        // Test health monitoring
        let health_status = health_checker.check_health().await.unwrap();
        assert!(health_status.components.len() > 0);

        // Test metrics collection
        metrics_collector.record_trade_execution("BTC", 150.0, true);
        metrics_collector.record_market_data_latency("BTC", 15.5);
        metrics_collector.record_pnl_update("BTC", 500.0, false);
        metrics_collector.record_signal_generation("BTC", "long", 0.85);

        // Test alert system
        let mut alert_manager = AlertManager::new();
        alert_manager.raise_alert(
            "test_health_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::HealthCheckFailed,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::Medium,
            "database".to_string(),
            "Test health check failure".to_string(),
        ).await;

        let active_alerts = alert_manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);
        assert_eq!(active_alerts[0].id, "test_health_alert");

        // Resolve alert
        alert_manager.resolve_alert("test_health_alert").await;
        let active_alerts = alert_manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 0);

        // Test health persistence
        health_checker.persist_health_status(&health_status).await.unwrap();

        // Test metrics persistence
        metrics_collector.persist_metrics().await.unwrap();
    }

    #[tokio::test]
    async fn test_cvd_strategy_with_persistence() {
        let (_state, _repository, _health, _metrics) = setup_full_system().await;

        // Test CVD candle building
        let mut builder = CandleBuilder::new(1);

        // Simulate buy and sell trades
        builder.process_trade(1640995200000, 45000.0, 1.0, true);   // Buy
        builder.process_trade(1640995201000, 45010.0, 2.0, false);  // Sell
        builder.process_trade(1640995202000, 45005.0, 1.5, true);   // Buy
        builder.process_trade(1640995203000, 45015.0, 0.8, false);  // Sell

        // Complete candle
        let finished = builder.process_trade(1640995260000, 45012.0, 1.2, true);

        assert!(finished.is_some());
        if let Some(candle) = finished {
            // CVD should be: 1.0 - 2.0 + 1.5 - 0.8 = -0.3
            let expected_cvd = 1.0 - 2.0 + 1.5 - 0.8;
            assert_eq!(candle.cvd, expected_cvd);

            // Volume should be sum: 1.0 + 2.0 + 1.5 + 0.8 = 5.3
            assert_eq!(candle.volume, 5.3);

            // Price range should be correct
            assert_eq!(candle.high, 45015.0);
            assert_eq!(candle.low, 45000.0);
        }
    }

    #[tokio::test]
    async fn test_system_under_load_simulation() {
        let (state, repository, health_checker, metrics_collector) = setup_full_system().await;

        // Simulate system load with multiple operations
        const NUM_OPERATIONS: usize = 50;

        // Create multiple positions
        for i in 0..NUM_OPERATIONS {
            let position = cvd_trader_rust::persistence::models::DbPosition {
                coin: format!("COIN{}", i),
                size: (i as f64 + 1.0),
                entry_price: 1000.0 + (i as f64 * 10.0),
                leverage: 5.0,
                unrealized_pnl: (i as f64 * 25.0),
                stop_loss: Some(950.0 + (i as f64 * 10.0)),
                take_profit: Some(1050.0 + (i as f64 * 10.0)),
                breakeven: 1005.0 + (i as f64 * 10.0),
                side: if i % 2 == 0 { "LONG" } else { "SHORT" }.to_string(),
                opened_at: "2024-01-01T12:00:00Z".to_string(),
                entry_reason: Some(format!("Load test {}", i)),
                sl_modifications: vec![],
                tp_50_hit: false,
                trailing_sl: Some(975.0 + (i as f64 * 10.0)),
                original_tp: Some(1100.0 + (i as f64 * 10.0)),
            };

            repository.save_position(&position).await.unwrap();

            // Record metrics for each position
            metrics_collector.record_trade_execution(&position.coin, 100.0 + (i as f64 * 5.0), true);
        }

        // Verify all positions were saved
        let positions = repository.load_positions().await.unwrap();
        assert_eq!(positions.len(), NUM_OPERATIONS);

        // Verify database stats
        let stats = repository.db.get_stats().unwrap();
        assert_eq!(stats.position_count as usize, NUM_OPERATIONS);

        // Test health under load
        let health_status = health_checker.check_health().await.unwrap();
        assert!(health_status.components.len() > 0);

        // Test metrics persistence under load
        metrics_collector.persist_metrics().await.unwrap();
    }

    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        let (state, repository, health_checker, _metrics) = setup_full_system().await;

        // Test health check with system in unhealthy state
        {
            let mut state_guard = state.lock().await;
            state_guard.is_running = false;
        }

        let health_status = health_checker.check_health().await.unwrap();

        // Should detect unhealthy state
        let state_component = health_status.components.iter()
            .find(|c| c.name == "state")
            .unwrap();
        assert_eq!(state_component.status, cvd_trader_rust::monitoring::health::ComponentStatus::Unhealthy);

        // Reset to healthy state
        {
            let mut state_guard = state.lock().await;
            state_guard.is_running = true;
        }

        let health_status = health_checker.check_health().await.unwrap();
        let state_component = health_status.components.iter()
            .find(|c| c.name == "state")
            .unwrap();
        assert_eq!(state_component.status, cvd_trader_rust::monitoring::health::ComponentStatus::Healthy);
    }

    #[tokio::test]
    async fn test_data_consistency_and_integrity() {
        let (state, repository, _health, _metrics) = setup_full_system().await;

        // Test referential integrity with positions and orders
        let position = cvd_trader_rust::persistence::models::DbPosition {
            coin: "INTEGRITY_TEST".to_string(),
            size: 5.0,
            entry_price: 50000.0,
            leverage: 2.0,
            unrealized_pnl: 1000.0,
            stop_loss: Some(49000.0),
            take_profit: Some(51000.0),
            breakeven: 50100.0,
            side: "LONG".to_string(),
            opened_at: "2024-01-01T12:00:00Z".to_string(),
            entry_reason: Some("Integrity test".to_string()),
            sl_modifications: vec!["test_sl".to_string()],
            tp_50_hit: true,
            trailing_sl: Some(49500.0),
            original_tp: Some(52000.0),
        };

        let order = cvd_trader_rust::persistence::models::DbActiveOrder {
            oid: 12345,
            coin: "INTEGRITY_TEST".to_string(),
            is_buy: true,
            sz: 5.0,
            limit_px: 50000.0,
            order_type: "limit".to_string(),
        };

        // Save both
        repository.save_position(&position).await.unwrap();
        repository.save_active_order(&order).await.unwrap();

        // Verify both exist
        let positions = repository.load_positions().await.unwrap();
        let orders = repository.load_active_orders().await.unwrap();

        assert_eq!(positions.len(), 1);
        assert_eq!(orders.len(), 1);

        let saved_position = positions.get("INTEGRITY_TEST").unwrap();
        let saved_order = orders.get(&12345).unwrap();

        assert_eq!(saved_position.coin, "INTEGRITY_TEST");
        assert_eq!(saved_order.coin, "INTEGRITY_TEST");
        assert_eq!(saved_position.size, saved_order.sz);
        assert_eq!(saved_position.entry_price, saved_order.limit_px);
    }
}