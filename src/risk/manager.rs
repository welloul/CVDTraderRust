
use crate::core::config::RiskConfig;

pub struct RiskManager {
    pub circuit_breaker_active: bool,
    pub consecutive_failures: i32,
    max_allowed_latency_ms: f64,
    consecutive_failures_threshold: i32,
}

impl RiskManager {
    pub fn new(config: &RiskConfig) -> Self {
        Self {
            circuit_breaker_active: false,
            consecutive_failures: 0,
            max_allowed_latency_ms: config.max_allowed_latency_ms,
            consecutive_failures_threshold: config.consecutive_failures_threshold,
        }
    }

    pub fn check_latency(&mut self, latency_ms: f64) {
        if latency_ms > self.max_allowed_latency_ms {
            self.consecutive_failures += 1;
//             println!("[WARN]",  "High latency detected", latency_ms = latency_ms, failures = self.consecutive_failures);

            if self.consecutive_failures >= 3 {
                self.circuit_breaker_active = true;
// //                 eprintln!("[ERROR]",  "Circuit breaker activated due to high latency");
            }
        } else if self.consecutive_failures > 0 {
            self.consecutive_failures -= 1;
            if self.consecutive_failures <= self.consecutive_failures_threshold - 1 {
                self.circuit_breaker_active = false;
//                 println!("[INFO] "Circuit breaker reset");
            }
        }
    }

    pub fn check_pre_trade(&self, coin: &str, size: f64, price: f64) -> bool {
        if self.circuit_breaker_active {
//             println!("[WARN]",  "Circuit breaker active, rejecting trade", coin = %coin);
            return false;
        }

        // Basic size and price validation
        if size <= 0.0 || price <= 0.0 {
//             println!("[WARN]",  "Invalid trade parameters", coin = %coin, size = size, price = price);
            return false;
        }

        // TODO: Add position size limits, drawdown checks
        true
    }

    pub fn record_order_result(&mut self, success: bool) {
        if success {
            self.consecutive_failures = 0;
            if self.circuit_breaker_active {
                self.circuit_breaker_active = false;
//                 println!("[INFO] "Circuit breaker reset after successful order");
            }
        } else {
            self.consecutive_failures += 1;
        }
    }
}