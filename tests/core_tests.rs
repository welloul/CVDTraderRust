use cvd_trader_rust::core::rounding::RoundingUtil;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rounding_util_initialization() {
        let meta = json!({
            "universe": [{
                "name": "BTC",
                "szDecimals": 2,
                "tickSize": 0.01
            }]
        });

        let util = RoundingUtil::new(Some(meta));

        // Test successful creation
        assert_eq!(util.round_size("BTC", 1.23456), "1.23");
        assert_eq!(util.round_price("BTC", 45000.12345), "45000.12");
    }

    #[test]
    fn test_rounding_util_defaults() {
        // Test with no metadata - should use defaults
        let util = RoundingUtil::new(None);

        assert_eq!(util.round_size("BTC", 1.23456), "1.23");
        assert_eq!(util.round_price("BTC", 45000.12345), "45000.12");
    }

    #[test]
    fn test_round_size_btc() {
        let util = RoundingUtil::new(None);
        assert_eq!(util.round_size("BTC", 1.23456), "1.23");
        assert_eq!(util.round_size("BTC", 1.99999), "2.00");
        assert_eq!(util.round_size("BTC", 0.00123), "0.00"); // Rounds to 2 decimals
    }

    #[test]
    fn test_round_size_eth() {
        let util = RoundingUtil::new(None);
        assert_eq!(util.round_size("ETH", 1.234567), "1.2346"); // 4 decimals
        assert_eq!(util.round_size("ETH", 0.0000123), "0.0000");
    }

    #[test]
    fn test_round_price_btc() {
        let util = RoundingUtil::new(None);

        // BTC tick size 0.01
        assert_eq!(util.round_price("BTC", 45000.12345), "45000.12");
        assert_eq!(util.round_price("BTC", 45000.00123), "45000.00");
        assert_eq!(util.round_price("BTC", 45000.99999), "45100.00"); // Rounds up
    }

    #[test]
    fn test_round_price_large_numbers() {
        let util = RoundingUtil::new(None);

        // Test with large numbers (no decimals)
        let meta = json!({
            "universe": [{
                "name": "TEST",
                "szDecimals": 0,
                "tickSize": 1.0
            }]
        });

        let util = RoundingUtil::new(Some(meta));
        assert_eq!(util.round_price("TEST", 45000.12345), "45000");
        assert_eq!(util.round_price("TEST", 45000.99999), "45100");
    }

    #[test]
    fn test_format_for_api() {
        let util = RoundingUtil::new(None);

        assert_eq!(util.format_for_api(1.23450), "1.2345");
        assert_eq!(util.format_for_api(1.00000), "1");
        assert_eq!(util.format_for_api(1.234567), "1.234567");
    }

    #[test]
    fn test_unknown_asset_fallback() {
        let util = RoundingUtil::new(None);

        // Unknown asset should return unrounded values
        assert_eq!(util.round_size("UNKNOWN", 1.23456), "1.23456");
        assert_eq!(util.round_price("UNKNOWN", 45000.12345), "45000.12345");
    }

    #[test]
    fn test_precision_edge_cases() {
        let util = RoundingUtil::new(None);

        // Test very small numbers
        assert_eq!(util.round_size("BTC", 0.0000001), "0.00");

        // Test very large numbers
        assert_eq!(util.round_size("BTC", 999999.999999), "1000000.00");

        // Test zero
        assert_eq!(util.round_size("BTC", 0.0), "0.00");
        assert_eq!(util.round_price("BTC", 0.0), "0.00");
    }
}

#[cfg(test)]
mod latency_tests {
    use cvd_trader_rust::core::state::GlobalState;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_latency_tracking() {
        let state = GlobalState::new();
        let mut state = Arc::new(Mutex::new(state)).lock().await;

        // Test initial empty state
        let stats = state.get_latency_stats();
        assert!(stats.is_empty());

        // Add latency samples
        state.update_latency("BTC", 15.5);
        state.update_latency("BTC", 20.3);
        state.update_latency("BTC", 12.8);

        let stats = state.get_latency_stats();
        assert!(stats.contains_key("BTC"));

        let btc_stats = &stats["BTC"];
        assert_eq!(btc_stats["samples"], 3.0);
        assert!(btc_stats["avg_ms"] > 0.0);
        assert_eq!(btc_stats["min_ms"], 12.8);
        assert_eq!(btc_stats["max_ms"], 20.3);
    }

    #[tokio::test]
    async fn test_latency_outlier_filtering() {
        let state = GlobalState::new();
        let mut state = Arc::new(Mutex::new(state)).lock().await;

        // Add normal latencies
        state.update_latency("BTC", 15.0);
        state.update_latency("BTC", 20.0);
        state.update_latency("BTC", 25.0);

        // Add extreme outlier (should be filtered)
        state.update_latency("BTC", -100000.0);

        let stats = state.get_latency_stats();
        let btc_stats = &stats["BTC"];

        // Should only count valid samples
        assert_eq!(btc_stats["samples"], 3.0); // Outlier filtered out
        assert!(btc_stats["avg_ms"] >= 15.0 && btc_stats["avg_ms"] <= 25.0);
    }

    #[tokio::test]
    async fn test_latency_sample_rotation() {
        let state = GlobalState::new();
        let mut state = Arc::new(Mutex::new(state)).lock().await;

        // Add 101 samples (limit is 100)
        for i in 0..101 {
            state.update_latency("BTC", i as f64);
        }

        let stats = state.get_latency_stats();
        let btc_stats = &stats["BTC"];

        // Should only keep last 100 samples
        assert_eq!(btc_stats["samples"], 100.0);
        // Average should be from 1.0 to 100.0 (first sample 0.0 removed)
        assert!(btc_stats["avg_ms"] > 50.0);
    }
}

#[cfg(test)]
mod logger_tests {
    use cvd_trader_rust::core::logger;

    #[test]
    fn test_logger_macro_compilation() {
        // Test that logging macros compile and don't panic
        // (We can't easily test the actual output without complex setup)
        logger::init(); // Should not panic
    }
}