use cvd_trader_rust::market_data::candles::{Candle, CandleBuilder};
use cvd_trader_rust::core::rounding::RoundingUtil;
use serde_json::json;

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_candle_builder_with_rounding_util() {
        let mut builder = CandleBuilder::new(1);
        let rounding = RoundingUtil::new(None);

        // Process trades with realistic sizes
        builder.process_trade(1640995200000, 45000.0, 1.23456, true);
        builder.process_trade(1640995201000, 45100.0, 2.78912, false);
        builder.process_trade(1640995202000, 44900.0, 0.56789, true);

        // Complete the candle
        let finished = builder.process_trade(1640995260000, 45000.0, 1.11111, false);
        assert!(finished.is_some());

        if let Some(candle) = finished {
            // Verify CVD calculation
            assert_eq!(candle.cvd, 1.23456 - 2.78912 + 0.56789 - 1.11111);

            // Test rounding the CVD for display/API
            let rounded_cvd = rounding.format_for_api(candle.cvd);
            assert!(!rounded_cvd.is_empty());
        }
    }

    #[test]
    fn test_market_data_event_processing() {
        // Simulate market data event processing pipeline
        let event = json!({
            "type": "market_data",
            "coin": "BTC",
            "price": 45000.50,
            "size": 1.23456,
            "is_buy": true,
            "timestamp": 1640995200.123,
            "latency_ms": 15.5
        });

        // Extract and validate data
        assert_eq!(event["type"], "market_data");
        assert_eq!(event["coin"], "BTC");
        assert_eq!(event["price"], 45000.50);
        assert_eq!(event["size"], 1.23456);
        assert_eq!(event["is_buy"], true);
        assert_eq!(event["latency_ms"], 15.5);
    }

    #[test]
    fn test_candle_event_creation() {
        // Test candle closed event structure
        let candle = Candle {
            timestamp: 1640995200000,
            open: 45000.0,
            high: 45100.0,
            low: 44900.0,
            close: 45050.0,
            volume: 10.5,
            cvd: 2.5,
            poc: 45025.0,
        };

        let event = json!({
            "type": "candle_closed",
            "coin": "BTC",
            "closed_candle_1m": {
                "start_time": candle.timestamp,
                "open": candle.open,
                "high": candle.high,
                "low": candle.low,
                "close": candle.close,
                "volume": candle.volume,
                "cvd": candle.cvd,
                "poc": candle.poc
            },
            "vwap": 0.0,
            "indicators": {}
        });

        // Validate event structure
        assert_eq!(event["type"], "candle_closed");
        assert_eq!(event["coin"], "BTC");
        assert_eq!(event["closed_candle_1m"]["cvd"], 2.5);
        assert_eq!(event["closed_candle_1m"]["volume"], 10.5);
    }

    #[test]
    fn test_realistic_trading_scenario() {
        // Simulate a realistic 1-minute candle with mixed buy/sell activity
        let mut builder = CandleBuilder::new(1);

        // Simulate market activity: mix of buys and sells
        let trades = vec![
            (1640995200000, 45000.0, 1.2, true),   // Buy
            (1640995201000, 45010.0, 0.8, false),  // Sell
            (1640995202000, 45005.0, 2.1, true),   // Buy
            (1640995203000, 45015.0, 1.5, false),  // Sell
            (1640995204000, 45008.0, 0.9, true),   // Buy
            (1640995205000, 45012.0, 1.8, false),  // Sell
        ];

        // Process all trades in same minute
        for (timestamp, price, size, is_buy) in trades {
            builder.process_trade(timestamp, price, size, is_buy);
        }

        // Complete candle
        let finished = builder.process_trade(1640995260000, 45010.0, 1.0, true);
        assert!(finished.is_some());

        if let Some(candle) = finished {
            // Calculate expected CVD: 1.2 - 0.8 + 2.1 - 1.5 + 0.9 - 1.8 = 0.1
            let expected_cvd = 1.2 - 0.8 + 2.1 - 1.5 + 0.9 - 1.8;
            assert_eq!(candle.cvd, expected_cvd);

            // Total volume should be sum of all trades
            let expected_volume = 1.2 + 0.8 + 2.1 + 1.5 + 0.9 + 1.8;
            assert_eq!(candle.volume, expected_volume);

            // Price range should be correct
            assert_eq!(candle.high, 45015.0);
            assert_eq!(candle.low, 45000.0);
            assert_eq!(candle.close, 45010.0);
        }
    }

    #[test]
    fn test_cvd_divergence_integration() {
        // Test CVD divergence detection with realistic candle data

        // Create candles showing bearish divergence:
        // Price makes higher high, CVD makes lower high
        let candles = vec![
            create_candle(1, 44900.0, 45000.0, 44800.0, 44950.0, 10.0, 3.0),
            create_candle(2, 44950.0, 45050.0, 44900.0, 45000.0, 12.0, 4.0),
            create_candle(3, 45000.0, 45100.0, 44950.0, 45050.0, 15.0, 2.0), // Higher high, lower CVD
            create_candle(4, 45050.0, 45080.0, 44980.0, 45020.0, 11.0, -1.0),
            create_candle(5, 45020.0, 45060.0, 44990.0, 45030.0, 9.0, 0.5),
        ];

        // Test swing detection
        assert!(detect_swing_high(&candles));

        // Test CVD divergence (bearish)
        assert!(is_cvd_exhaustion(&candles, true));

        // Test bullish divergence scenario
        let bullish_candles = vec![
            create_candle(1, 45100.0, 45200.0, 45000.0, 45150.0, 10.0, -3.0),
            create_candle(2, 45150.0, 45250.0, 45100.0, 45200.0, 12.0, -4.0),
            create_candle(3, 45200.0, 45200.0, 44900.0, 45000.0, 15.0, -1.0), // Lower low, higher CVD
            create_candle(4, 45000.0, 45100.0, 44950.0, 45050.0, 11.0, 1.0),
            create_candle(5, 45050.0, 45100.0, 45000.0, 45070.0, 9.0, 0.5),
        ];

        assert!(detect_swing_low(&bullish_candles));
        assert!(is_cvd_exhaustion(&bullish_candles, false));
    }

    // Helper functions for integration tests
    fn create_candle(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64, cvd: f64) -> Candle {
        Candle {
            timestamp: timestamp * 60000, // Convert to milliseconds
            open,
            high,
            low,
            close,
            volume,
            cvd,
            poc: close,
        }
    }

    fn detect_swing_high(history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_highs: Vec<f64> = history.iter().rev().skip(1).take(4).map(|c| c.high).collect();

        prev_highs.iter().all(|&h| current.high > h)
    }

    fn detect_swing_low(history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_lows: Vec<f64> = history.iter().rev().skip(1).take(4).map(|c| c.low).collect();

        prev_lows.iter().all(|&l| current.low < l)
    }

    fn is_cvd_exhaustion(history: &[Candle], is_high_swing: bool) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_candles: Vec<&Candle> = history.iter().rev().skip(1).take(4).collect();

        if is_high_swing {
            let price_high_swing = prev_candles.iter().all(|c| current.high > c.high);
            let cvd_lower_high = prev_candles.iter().all(|c| current.cvd < c.cvd);
            price_high_swing && cvd_lower_high
        } else {
            let price_low_swing = prev_candles.iter().all(|c| current.low < c.low);
            let cvd_higher_low = prev_candles.iter().all(|c| current.cvd > c.cvd);
            price_low_swing && cvd_higher_low
        }
    }
}