use metrics::{counter, gauge, histogram};
use std::time::Instant;
use crate::persistence::Repository;

pub struct MetricsCollector {
    repository: Repository,
}

impl MetricsCollector {
    pub fn new(repository: Repository) -> Self {
        Self { repository }
    }

    pub fn init() {
        // Initialize Prometheus exporter
        metrics_exporter_prometheus::PrometheusBuilder::new()
            .install()
            .expect("Failed to install Prometheus metrics exporter");
    }

    pub fn record_trade_execution(&self, coin: &str, execution_time_ms: f64, success: bool) {
        histogram!("trade_execution_duration", "coin" => coin.to_owned())
            .record(execution_time_ms);

        if success {
            counter!("trade_executions_success", "coin" => coin.to_owned()).increment(1);
        } else {
            counter!("trade_executions_failed", "coin" => coin.to_owned()).increment(1);
        }
    }

    pub fn record_market_data_latency(&self, coin: &str, latency_ms: f64) {
        histogram!("market_data_latency", "coin" => coin.to_owned())
            .record(latency_ms);

        gauge!("market_data_latency_current", "coin" => coin.to_owned())
            .set(latency_ms);
    }

    pub fn record_pnl_update(&self, coin: &str, pnl: f64, is_realized: bool) {
        if is_realized {
            histogram!("realized_pnl", "coin" => coin.to_owned()).record(pnl);
            counter!("trades_closed", "coin" => coin.to_owned()).increment(1);
        } else {
            gauge!("unrealized_pnl", "coin" => coin.to_owned()).set(pnl);
        }
    }

    pub fn record_signal_generation(&self, coin: &str, signal_type: &str, confidence: f64) {
        counter!("signals_generated", "coin" => coin.to_owned(), "type" => signal_type.to_owned())
            .increment(1);

        histogram!("signal_confidence", "coin" => coin.to_owned(), "type" => signal_type.to_owned())
            .record(confidence);
    }

    pub fn record_error(&self, component: &str, error_type: &str) {
        counter!("errors_total", "component" => component.to_owned(), "type" => error_type.to_owned())
            .increment(1);
    }

    pub fn record_api_request(&self, endpoint: &str, method: &str, status_code: u16, duration_ms: f64) {
        histogram!("api_request_duration", "endpoint" => endpoint.to_owned(), "method" => method.to_owned())
            .record(duration_ms);

        counter!("api_requests_total",
            "endpoint" => endpoint.to_owned(),
            "method" => method.to_owned(),
            "status" => status_code.to_string()
        ).increment(1);
    }

    pub fn record_websocket_connection(&self, coin: &str, connected: bool) {
        if connected {
            gauge!("websocket_connections", "coin" => coin.to_owned()).increment(1.0);
            counter!("websocket_reconnects", "coin" => coin.to_owned()).increment(1);
        } else {
            gauge!("websocket_connections", "coin" => coin.to_owned()).decrement(1.0);
        }
    }

    pub fn record_memory_usage(&self) {
        // Record basic memory stats (would need more sophisticated memory tracking in production)
        gauge!("memory_usage_bytes").set(0.0); // Placeholder
    }

    pub fn record_system_load(&self) {
        // Record system load (simplified)
        gauge!("system_load_average").set(0.0); // Placeholder
    }

    pub async fn persist_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This would collect current metrics and persist them
        // For now, we'll just log that persistence is working
//         println!("[INFO] "Metrics persistence completed");
        Ok(())
    }

    pub fn start_background_collection(&self) {
        let collector = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Collect system metrics
                collector.record_memory_usage();
                collector.record_system_load();

                // Persist metrics periodically
                if let Err(e) = collector.persist_metrics().await {
// //                     eprintln!("[ERROR]",  "Failed to persist metrics", error = %e);
                }
            }
        });
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            repository: Repository::new(self.repository.db.clone()),
        }
    }
}

// Timer helper for measuring operation duration
pub struct Timer {
    start: Instant,
    operation: String,
}

impl Timer {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_millis() as f64
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.elapsed_ms();
        histogram!("operation_duration", "operation" => self.operation.clone())
            .record(duration);
    }
}

// Convenience macros for timing operations
#[macro_export]
macro_rules! time_operation {
    ($operation:expr) => {
        let _timer = $crate::monitoring::metrics::Timer::new($operation);
    };
}

#[macro_export]
macro_rules! record_metric {
    (counter, $name:expr $(, $label_key:expr => $label_value:expr)* $(,)?) => {
        metrics::counter!($name $(, $label_key => $label_value)*).increment(1);
    };
    (gauge, $name:expr, $value:expr $(, $label_key:expr => $label_value:expr)* $(,)?) => {
        metrics::gauge!($name $(, $label_key => $label_value)*).set($value);
    };
    (histogram, $name:expr, $value:expr $(, $label_key:expr => $label_value:expr)* $(,)?) => {
        metrics::histogram!($name $(, $label_key => $label_value)*).record($value);
    };
}