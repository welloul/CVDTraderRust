use cvd_trader_rust::monitoring::{
    health::{HealthChecker, ComponentStatus, SystemHealth},
    metrics::MetricsCollector,
    alerts::AlertManager,
};
use cvd_trader_rust::persistence::{Database, Repository};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_components() -> (Arc<Mutex<cvd_trader_rust::core::state::GlobalState>>, HealthChecker, MetricsCollector, Repository) {
        let state = Arc::new(Mutex::new(cvd_trader_rust::core::state::GlobalState::new()));
        let db = Database::new(":memory:".to_string());
        db.initialize().unwrap();
        let repository = Repository::new(db);
        let metrics = MetricsCollector::new(repository.clone());
        let health = HealthChecker::new(Arc::clone(&state), repository.clone());

        (state, health, metrics, repository)
    }

    #[tokio::test]
    async fn test_health_checker_basic() {
        let (_state, health_checker, _metrics, _repo) = setup_test_components().await;

        let status = health_checker.check_health().await.unwrap();
        assert_eq!(status.components.len(), 3); // database, state, memory, network checks

        // Should be healthy initially
        assert!(matches!(status.overall, SystemHealth::Healthy));
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let (_state, _health, metrics, _repo) = setup_test_components().await;

        // Test trade execution metrics
        metrics.record_trade_execution("BTC", 150.0, true);
        metrics.record_trade_execution("ETH", 200.0, false);

        // Test market data latency
        metrics.record_market_data_latency("BTC", 15.5);
        metrics.record_market_data_latency("ETH", 22.3);

        // Test P&L updates
        metrics.record_pnl_update("BTC", 500.0, false);
        metrics.record_pnl_update("ETH", -250.0, true);

        // Test signal generation
        metrics.record_signal_generation("BTC", "long", 0.85);
        metrics.record_signal_generation("ETH", "short", 0.72);

        // Test error recording
        metrics.record_error("strategy", "divergence_calculation");
        metrics.record_error("execution", "order_timeout");

        // These should not panic - actual metric values are tested via integration
    }

    #[tokio::test]
    async fn test_alert_manager_basic() {
        let mut alert_manager = AlertManager::new();

        // Test alert creation
        alert_manager.raise_alert(
            "test_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::PerformanceDegraded,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::Medium,
            "test_component".to_string(),
            "Test alert message".to_string(),
        ).await;

        // Check active alerts
        let active_alerts = alert_manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 1);
        assert_eq!(active_alerts[0].id, "test_alert");
        assert_eq!(active_alerts[0].component, "test_component");

        // Test alert resolution
        alert_manager.resolve_alert("test_alert").await;
        let active_alerts = alert_manager.get_active_alerts();
        assert_eq!(active_alerts.len(), 0);
    }

    #[tokio::test]
    async fn test_alert_manager_severity_filtering() {
        let mut alert_manager = AlertManager::new();

        // Add alerts of different severities
        alert_manager.raise_alert(
            "low_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::PerformanceDegraded,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::Low,
            "component1".to_string(),
            "Low severity alert".to_string(),
        ).await;

        alert_manager.raise_alert(
            "high_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::SystemUnhealthy,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::High,
            "component2".to_string(),
            "High severity alert".to_string(),
        ).await;

        // Test filtering by severity
        let high_alerts = alert_manager.get_alerts_by_severity(cvd_trader_rust::monitoring::alerts::AlertSeverity::High);
        assert_eq!(high_alerts.len(), 1);
        assert_eq!(high_alerts[0].id, "high_alert");

        let low_alerts = alert_manager.get_alerts_by_severity(cvd_trader_rust::monitoring::alerts::AlertSeverity::Low);
        assert_eq!(low_alerts.len(), 1);
        assert_eq!(low_alerts[0].id, "low_alert");
    }

    #[tokio::test]
    async fn test_alert_manager_component_filtering() {
        let mut alert_manager = AlertManager::new();

        // Add alerts for different components
        alert_manager.raise_alert(
            "db_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::DatabaseError,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::High,
            "database".to_string(),
            "Database connection failed".to_string(),
        ).await;

        alert_manager.raise_alert(
            "network_alert".to_string(),
            cvd_trader_rust::monitoring::alerts::AlertType::NetworkError,
            cvd_trader_rust::monitoring::alerts::AlertSeverity::Medium,
            "network".to_string(),
            "Network timeout".to_string(),
        ).await;

        // Test filtering by component
        let db_alerts = alert_manager.get_alerts_by_component("database");
        assert_eq!(db_alerts.len(), 1);
        assert_eq!(db_alerts[0].component, "database");

        let network_alerts = alert_manager.get_alerts_by_component("network");
        assert_eq!(network_alerts.len(), 1);
        assert_eq!(network_alerts[0].component, "network");
    }

    #[tokio::test]
    async fn test_health_status_persistence() {
        let (_state, health_checker, _metrics, repository) = setup_test_components().await;

        // Get health status
        let status = health_checker.check_health().await.unwrap();

        // Persist it
        health_checker.persist_health_status(&status).await.unwrap();

        // Verify it was saved (basic check - we can't easily query without more complex setup)
        // This test mainly ensures the persistence methods don't panic
    }

    #[tokio::test]
    async fn test_metrics_timer_macro() {
        use cvd_trader_rust::monitoring::metrics::Timer;

        // Test timer functionality
        let timer = Timer::new("test_operation");
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        drop(timer); // This should record the metric

        // Timer should have recorded a metric (can't easily verify without metrics exporter)
    }

    #[tokio::test]
    async fn test_health_checker_with_failed_component() {
        let (state, health_checker, _metrics, _repo) = setup_test_components().await;

        // Simulate a system issue by setting bot to not running
        {
            let mut state_guard = state.lock().await;
            state_guard.is_running = false;
        }

        let status = health_checker.check_health().await.unwrap();

        // Should detect the issue
        assert!(!matches!(status.overall, SystemHealth::Healthy));

        // Find the state component
        let state_component = status.components.iter()
            .find(|c| c.name == "state")
            .unwrap();

        assert_eq!(state_component.status, ComponentStatus::Unhealthy);
    }
}