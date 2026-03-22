use cvd_trader_rust::persistence::{Database, Repository};
use cvd_trader_rust::monitoring::{HealthChecker, MetricsCollector};
use cvd_trader_rust::core::state::GlobalState;
use cvd_trader_rust::market_data::candles::CandleBuilder;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

#[cfg(test)]
mod stress_tests {
    use super::*;

    async fn setup_stress_test_env() -> (Arc<Mutex<GlobalState>>, Repository, HealthChecker, MetricsCollector) {
        let db = Database::new(":memory:".to_string());
        db.initialize().unwrap();
        let repository = Repository::new(db);

        let state = Arc::new(Mutex::new(GlobalState::new()));
        let health_checker = HealthChecker::new(Arc::clone(&state), repository.clone());
        let metrics_collector = MetricsCollector::new(repository.clone());

        (state, repository, health_checker, metrics_collector)
    }

    #[tokio::test]
    async fn test_concurrent_position_operations() {
        let (_state, repository, _health, _metrics) = setup_stress_test_env().await;

        const NUM_POSITIONS: usize = 100;
        const NUM_TASKS: usize = 10;

        let start_time = Instant::now();

        // Spawn multiple tasks to create positions concurrently
        let mut handles = vec![];

        for task_id in 0..NUM_TASKS {
            let repo = repository.clone();
            let handle = tokio::spawn(async move {
                let start_idx = task_id * (NUM_POSITIONS / NUM_TASKS);
                let end_idx = start_idx + (NUM_POSITIONS / NUM_TASKS);

                for i in start_idx..end_idx {
                    let position = cvd_trader_rust::persistence::models::DbPosition {
                        coin: format!("STRESS_COIN_{}", i),
                        size: (i as f64 + 1.0),
                        entry_price: 1000.0 + (i as f64 * 10.0),
                        leverage: 5.0,
                        unrealized_pnl: (i as f64 * 25.0),
                        stop_loss: Some(950.0 + (i as f64 * 10.0)),
                        take_profit: Some(1050.0 + (i as f64 * 10.0)),
                        breakeven: 1005.0 + (i as f64 * 10.0),
                        side: if i % 2 == 0 { "LONG" } else { "SHORT" }.to_string(),
                        opened_at: "2024-01-01T12:00:00Z".to_string(),
                        entry_reason: Some(format!("Stress test {}", i)),
                        sl_modifications: vec![format!("mod_{}", i)],
                        tp_50_hit: i % 3 == 0,
                        trailing_sl: Some(975.0 + (i as f64 * 10.0)),
                        original_tp: Some(1100.0 + (i as f64 * 10.0)),
                    };

                    repo.save_position(&position).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start_time.elapsed();
        println!("Concurrent position creation took: {:?}", duration);

        // Verify all positions were saved
        let positions = repository.load_positions().await.unwrap();
        assert_eq!(positions.len(), NUM_POSITIONS);

        // Verify database stats
        let stats = repository.db.get_stats().unwrap();
        assert_eq!(stats.position_count as usize, NUM_POSITIONS);

        // Performance check: should complete within reasonable time
        assert!(duration.as_millis() < 5000, "Concurrent operations took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_high_frequency_trade_processing() {
        let (_state, _repository, _health, _metrics) = setup_stress_test_env().await;

        const NUM_TRADES: usize = 10000;
        let mut builder = CandleBuilder::new(1); // 1-minute candles

        let start_time = Instant::now();

        // Process high volume of trades
        let base_timestamp = 1640995200000i64;

        for i in 0..NUM_TRADES {
            let timestamp = base_timestamp + (i as i64 * 10); // 10ms intervals
            let price = 45000.0 + (i as f64 * 0.1); // Small price movements
            let volume = 1.0 + (i as f64 * 0.01); // Varying volume
            let is_buy = i % 2 == 0;

            builder.process_trade(timestamp, price, volume, is_buy);
        }

        // Complete the candle
        let finished = builder.process_trade(base_timestamp + 60000, 45050.0, 1.0, true);

        let duration = start_time.elapsed();
        println!("High-frequency trade processing took: {:?}", duration);

        // Verify candle was created
        assert!(finished.is_some());

        if let Some(candle) = finished {
            // Verify CVD calculation with high trade volume
            assert!(candle.volume > 0.0);
            assert!(!candle.cvd.is_nan());
            assert!(candle.cvd.is_finite());

            // Performance check
            assert!(duration.as_millis() < 1000, "Trade processing took too long: {:?}", duration);
        }
    }

    #[tokio::test]
    async fn test_concurrent_metrics_collection() {
        let (_state, repository, _health, metrics_collector) = setup_stress_test_env().await;

        const NUM_METRICS: usize = 1000;
        const NUM_TASKS: usize = 5;

        let start_time = Instant::now();

        // Spawn multiple tasks to record metrics concurrently
        let mut handles = vec![];

        for task_id in 0..NUM_TASKS {
            let metrics = Arc::clone(&metrics_collector);
            let repo = repository.clone();

            let handle = tokio::spawn(async move {
                let start_idx = task_id * (NUM_METRICS / NUM_TASKS);
                let end_idx = start_idx + (NUM_METRICS / NUM_TASKS);

                for i in start_idx..end_idx {
                    let coin = format!("COIN_{}", i % 10);
                    let latency = 10.0 + (i as f64 * 0.1);
                    let pnl = (i as f64 * 5.0) - 250.0;

                    metrics.record_market_data_latency(&coin, latency);
                    metrics.record_pnl_update(&coin, pnl, i % 50 == 0); // Some realized P&L

                    // Occasionally record performance metrics
                    if i % 100 == 0 {
                        let perf_metric = cvd_trader_rust::persistence::models::DbPerformanceMetric {
                            id: None,
                            timestamp: chrono::Utc::now(),
                            metric_type: "throughput".to_string(),
                            metric_name: "trades_per_second".to_string(),
                            value: 150.0 + (i as f64 * 0.1),
                            coin: Some(coin.clone()),
                            metadata: Some(serde_json::json!({"batch": i / 100})),
                        };
                        repo.save_performance_metric(&perf_metric).await.unwrap();
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start_time.elapsed();
        println!("Concurrent metrics collection took: {:?}", duration);

        // Verify metrics were saved
        let saved_metrics = repository.get_recent_metrics(100).await.unwrap();
        assert!(saved_metrics.len() > 0);

        // Performance check
        assert!(duration.as_millis() < 3000, "Metrics collection took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_database_concurrent_access() {
        let (_state, repository, _health, _metrics) = setup_stress_test_env().await;

        const NUM_OPERATIONS: usize = 500;
        const NUM_TASKS: usize = 8;

        let start_time = Instant::now();

        // Test concurrent database operations
        let mut handles = vec![];

        for task_id in 0..NUM_TASKS {
            let repo = repository.clone();

            let handle = tokio::spawn(async move {
                for i in 0..(NUM_OPERATIONS / NUM_TASKS) {
                    let operation_id = task_id * (NUM_OPERATIONS / NUM_TASKS) + i;

                    // Mix of different operations
                    match operation_id % 4 {
                        0 => {
                            // Save position
                            let position = cvd_trader_rust::persistence::models::DbPosition {
                                coin: format!("DB_COIN_{}", operation_id),
                                size: operation_id as f64 + 1.0,
                                entry_price: 1000.0,
                                leverage: 5.0,
                                unrealized_pnl: 0.0,
                                stop_loss: Some(950.0),
                                take_profit: Some(1050.0),
                                breakeven: 1005.0,
                                side: "LONG".to_string(),
                                opened_at: "2024-01-01T12:00:00Z".to_string(),
                                entry_reason: Some(format!("DB test {}", operation_id)),
                                sl_modifications: vec![],
                                tp_50_hit: false,
                                trailing_sl: Some(975.0),
                                original_tp: Some(1100.0),
                            };
                            repo.save_position(&position).await.unwrap();
                        }
                        1 => {
                            // Load positions
                            let _positions = repo.load_positions().await.unwrap();
                        }
                        2 => {
                            // Save metric
                            let metric = cvd_trader_rust::persistence::models::DbPerformanceMetric {
                                id: None,
                                timestamp: chrono::Utc::now(),
                                metric_type: "test".to_string(),
                                metric_name: "concurrent".to_string(),
                                value: operation_id as f64,
                                coin: None,
                                metadata: None,
                            };
                            repo.save_performance_metric(&metric).await.unwrap();
                        }
                        3 => {
                            // Get database stats
                            let _stats = repo.db.get_stats().unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start_time.elapsed();
        println!("Concurrent database operations took: {:?}", duration);

        // Verify final state
        let positions = repository.load_positions().await.unwrap();
        let metrics = repository.get_recent_metrics(1000).await.unwrap();
        let stats = repository.db.get_stats().unwrap();

        assert!(positions.len() > 0, "Should have saved positions");
        assert!(metrics.len() > 0, "Should have saved metrics");
        assert_eq!(stats.position_count as usize, positions.len());

        // Performance check for concurrent operations
        assert!(duration.as_millis() < 5000, "Concurrent DB operations took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_memory_pressure_simulation() {
        let (_state, repository, _health, _metrics) = setup_stress_test_env().await;

        const NUM_RECORDS: usize = 10000;

        let start_time = Instant::now();

        // Simulate memory pressure with large dataset
        let mut handles = vec![];

        for batch in 0..10 {
            let repo = repository.clone();
            let handle = tokio::spawn(async move {
                for i in 0..(NUM_RECORDS / 10) {
                    let record_id = batch * (NUM_RECORDS / 10) + i;

                    // Create large metric with metadata
                    let metric = cvd_trader_rust::persistence::models::DbPerformanceMetric {
                        id: None,
                        timestamp: chrono::Utc::now(),
                        metric_type: "memory_test".to_string(),
                        metric_name: format!("large_metric_{}", record_id),
                        value: record_id as f64,
                        coin: Some(format!("MEMORY_COIN_{}", record_id % 100)),
                        metadata: Some(serde_json::json!({
                            "batch": batch,
                            "record_id": record_id,
                            "large_data": "x".repeat(1000), // 1KB of data per record
                            "nested": {
                                "array": [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
                                "object": {
                                    "key1": "value1",
                                    "key2": "value2",
                                    "key3": 12345
                                }
                            }
                        })),
                    };

                    repo.save_performance_metric(&metric).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all operations
        for handle in handles {
            handle.await.unwrap();
        }

        let duration = start_time.elapsed();
        println!("Memory pressure test took: {:?}", duration);

        // Verify all records were saved
        let metrics = repository.get_recent_metrics(NUM_RECORDS).await.unwrap();
        assert_eq!(metrics.len(), NUM_RECORDS);

        // Performance check
        assert!(duration.as_millis() < 15000, "Memory pressure test took too long: {:?}", duration);
    }

    #[tokio::test]
    async fn test_system_recovery_under_load() {
        let (state, repository, health_checker, metrics_collector) = setup_stress_test_env().await;

        // Simulate system under heavy load then recovery
        const NUM_OPERATIONS: usize = 200;

        // Phase 1: Heavy load
        let mut handles = vec![];

        for i in 0..NUM_OPERATIONS {
            let repo = repository.clone();
            let metrics = Arc::clone(&metrics_collector);
            let state = Arc::clone(&state);

            let handle = tokio::spawn(async move {
                // Mixed operations
                let position = cvd_trader_rust::persistence::models::DbPosition {
                    coin: format!("RECOVERY_COIN_{}", i),
                    size: 1.0,
                    entry_price: 1000.0,
                    leverage: 5.0,
                    unrealized_pnl: 0.0,
                    stop_loss: Some(950.0),
                    take_profit: Some(1050.0),
                    breakeven: 1005.0,
                    side: "LONG".to_string(),
                    opened_at: "2024-01-01T12:00:00Z".to_string(),
                    entry_reason: Some(format!("Recovery test {}", i)),
                    sl_modifications: vec![],
                    tp_50_hit: false,
                    trailing_sl: Some(975.0),
                    original_tp: Some(1100.0),
                };

                repo.save_position(&position).await.unwrap();
                metrics.record_trade_execution(&position.coin, 100.0, true);
            });
            handles.push(handle);
        }

        // Wait for load phase
        for handle in handles {
            handle.await.unwrap();
        }

        // Phase 2: System health check under load
        let health_status = health_checker.check_health().await.unwrap();
        assert!(health_status.components.len() > 0);

        // Phase 3: Simulate recovery - clear some data
        repository.cleanup_old_data(0).await.unwrap(); // Keep no data (for test)

        // Phase 4: Verify system remains functional
        let final_health = health_checker.check_health().await.unwrap();
        assert!(final_health.components.len() > 0);

        let final_stats = repository.db.get_stats().unwrap();
        // Should have cleaned up data but database should still be functional
        assert!(final_stats.position_count >= 0);

        println!("System recovery test completed successfully");
    }
}