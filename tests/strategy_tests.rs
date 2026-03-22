use cvd_trader_rust::market_data::candles::Candle;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test candles
    fn create_test_candle(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64, cvd: f64) -> Candle {
        Candle {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            cvd,
            poc: close, // Simplified
        }
    }

    #[test]
    fn test_detect_swing_high() {
        // Create test candles with swing high pattern
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 2.0),
            create_test_candle(2, 44200.0, 44800.0, 44000.0, 44600.0, 12.0, 3.0),
            create_test_candle(3, 44600.0, 45200.0, 44400.0, 45000.0, 15.0, 4.0), // Swing high
            create_test_candle(4, 45000.0, 44800.0, 44200.0, 44500.0, 11.0, -1.0),
            create_test_candle(5, 44500.0, 44700.0, 44300.0, 44600.0, 9.0, 0.5),
        ];

        // Test swing high detection (needs at least 5 candles)
        assert!(detect_swing_high(&candles));
    }

    #[test]
    fn test_detect_swing_low() {
        // Create test candles with swing low pattern
        let candles = vec![
            create_test_candle(1, 46000.0, 46500.0, 45500.0, 46200.0, 10.0, -2.0),
            create_test_candle(2, 46200.0, 46800.0, 46000.0, 46600.0, 12.0, -3.0),
            create_test_candle(3, 46600.0, 45200.0, 44800.0, 45000.0, 15.0, -4.0), // Swing low
            create_test_candle(4, 45000.0, 45800.0, 44800.0, 45500.0, 11.0, 1.0),
            create_test_candle(5, 45500.0, 45700.0, 45300.0, 45600.0, 9.0, -0.5),
        ];

        assert!(detect_swing_low(&candles));
    }

    #[test]
    fn test_no_swing_high() {
        // Create candles without swing high pattern
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 2.0),
            create_test_candle(2, 44200.0, 45000.0, 44000.0, 44800.0, 12.0, 3.0), // Higher high
            create_test_candle(3, 44800.0, 44900.0, 44400.0, 44600.0, 15.0, 4.0), // No swing
            create_test_candle(4, 44600.0, 44800.0, 44200.0, 44500.0, 11.0, -1.0),
            create_test_candle(5, 44500.0, 44700.0, 44300.0, 44600.0, 9.0, 0.5),
        ];

        assert!(!detect_swing_high(&candles));
    }

    #[test]
    fn test_bullish_cvd_divergence() {
        // Price makes lower low, CVD makes higher low (bullish divergence)
        let candles = vec![
            create_test_candle(1, 46000.0, 46500.0, 45500.0, 46200.0, 10.0, -5.0),
            create_test_candle(2, 46200.0, 46800.0, 46000.0, 46600.0, 12.0, -6.0),
            create_test_candle(3, 46600.0, 46200.0, 45000.0, 45500.0, 15.0, -2.0), // Lower low, higher CVD
            create_test_candle(4, 45500.0, 45800.0, 45200.0, 45600.0, 11.0, 1.0),
            create_test_candle(5, 45600.0, 45900.0, 45400.0, 45700.0, 9.0, 0.5),
        ];

        assert!(is_cvd_exhaustion(&candles, false)); // Bullish divergence
    }

    #[test]
    fn test_bearish_cvd_divergence() {
        // Price makes higher high, CVD makes lower high (bearish divergence)
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 5.0),
            create_test_candle(2, 44200.0, 44800.0, 44000.0, 44600.0, 12.0, 6.0),
            create_test_candle(3, 44600.0, 45200.0, 44400.0, 45000.0, 15.0, 2.0), // Higher high, lower CVD
            create_test_candle(4, 45000.0, 44800.0, 44200.0, 44500.0, 11.0, -1.0),
            create_test_candle(5, 44500.0, 44700.0, 44300.0, 44600.0, 9.0, 0.5),
        ];

        assert!(is_cvd_exhaustion(&candles, true)); // Bearish divergence
    }

    #[test]
    fn test_no_cvd_divergence() {
        // Price and CVD move in same direction (no divergence)
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 5.0),
            create_test_candle(2, 44200.0, 44800.0, 44000.0, 44600.0, 12.0, 6.0),
            create_test_candle(3, 44600.0, 45200.0, 44400.0, 45000.0, 15.0, 7.0), // Higher high, higher CVD
            create_test_candle(4, 45000.0, 44800.0, 44200.0, 44500.0, 11.0, -1.0),
            create_test_candle(5, 44500.0, 44700.0, 44300.0, 44600.0, 9.0, 0.5),
        ];

        assert!(!is_cvd_exhaustion(&candles, true)); // No bearish divergence
    }

    #[test]
    fn test_insufficient_history() {
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 5.0),
            create_test_candle(2, 44200.0, 44800.0, 44000.0, 44600.0, 12.0, 6.0),
        ];

        assert!(!detect_swing_high(&candles));
        assert!(!detect_swing_low(&candles));
        assert!(!is_cvd_exhaustion(&candles, true));
    }

    #[test]
    fn test_extreme_cvd_values() {
        // Test with extreme CVD values
        let candles = vec![
            create_test_candle(1, 44000.0, 44500.0, 43500.0, 44200.0, 10.0, 1000.0),
            create_test_candle(2, 44200.0, 44800.0, 44000.0, 44600.0, 12.0, 2000.0),
            create_test_candle(3, 44600.0, 45200.0, 44400.0, 45000.0, 15.0, 500.0), // Lower CVD
            create_test_candle(4, 45000.0, 44800.0, 44200.0, 44500.0, 11.0, -1.0),
            create_test_candle(5, 44500.0, 44700.0, 44300.0, 44600.0, 9.0, 0.5),
        ];

        assert!(is_cvd_exhaustion(&candles, true)); // Bearish divergence with extreme values
    }

    // Helper functions (duplicating strategy logic for testing)
    fn detect_swing_high(history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_highs: Vec<f64> = history.iter().rev().skip(1).take(4).map(|c| c.high).collect();

        // Current high is higher than previous 4 highs
        prev_highs.iter().all(|&h| current.high > h)
    }

    fn detect_swing_low(history: &[Candle]) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_lows: Vec<f64> = history.iter().rev().skip(1).take(4).map(|c| c.low).collect();

        // Current low is lower than previous 4 lows
        prev_lows.iter().all(|&l| current.low < l)
    }

    fn is_cvd_exhaustion(history: &[Candle], is_high_swing: bool) -> bool {
        if history.len() < 5 {
            return false;
        }

        let current = history.last().unwrap();
        let prev_candles: Vec<&Candle> = history.iter().rev().skip(1).take(4).collect();

        if is_high_swing {
            // For short signals: Price higher high + CVD lower high (bearish divergence)
            let price_high_swing = prev_candles.iter().all(|c| current.high > c.high);
            let cvd_lower_high = prev_candles.iter().all(|c| current.cvd < c.cvd);

            price_high_swing && cvd_lower_high
        } else {
            // For long signals: Price lower low + CVD higher low (bullish divergence)
            let price_low_swing = prev_candles.iter().all(|c| current.low < c.low);
            let cvd_higher_low = prev_candles.iter().all(|c| current.cvd > c.cvd);

            price_low_swing && cvd_higher_low
        }
    }
}