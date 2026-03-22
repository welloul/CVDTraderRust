use cvd_trader_rust::market_data::candles::{Candle, CandleBuilder};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_creation_buy_trade() {
        let candle = Candle::new(1640995200000, 45000.0, 1.5, true);
        assert_eq!(candle.timestamp, 1640995200000);
        assert_eq!(candle.open, 45000.0);
        assert_eq!(candle.high, 45000.0);
        assert_eq!(candle.low, 45000.0);
        assert_eq!(candle.close, 45000.0);
        assert_eq!(candle.volume, 1.5);
        assert_eq!(candle.cvd, 1.5); // Buy trade = positive CVD
    }

    #[test]
    fn test_candle_creation_sell_trade() {
        let candle = Candle::new(1640995200000, 45000.0, 1.5, false);
        assert_eq!(candle.cvd, -1.5); // Sell trade = negative CVD
    }

    #[test]
    fn test_candle_update_buy_trades() {
        let mut candle = Candle::new(1640995200000, 45000.0, 1.0, true);

        // Add another buy trade
        candle.update(45100.0, 2.0, true);
        assert_eq!(candle.high, 45100.0);
        assert_eq!(candle.low, 45000.0);
        assert_eq!(candle.close, 45100.0);
        assert_eq!(candle.volume, 3.0);
        assert_eq!(candle.cvd, 3.0); // 1.0 + 2.0
    }

    #[test]
    fn test_candle_update_sell_trades() {
        let mut candle = Candle::new(1640995200000, 45000.0, 1.0, true);

        // Add sell trade
        candle.update(44900.0, 0.5, false);
        assert_eq!(candle.high, 45000.0);
        assert_eq!(candle.low, 44900.0);
        assert_eq!(candle.close, 44900.0);
        assert_eq!(candle.volume, 1.5);
        assert_eq!(candle.cvd, 0.5); // 1.0 - 0.5
    }

    #[test]
    fn test_candle_update_mixed_trades() {
        let mut candle = Candle::new(1640995200000, 45000.0, 1.0, true);

        // Buy: +1.0, Sell: -2.0, Buy: +0.5
        candle.update(45100.0, 2.0, false); // Sell
        candle.update(45200.0, 0.5, true);  // Buy

        assert_eq!(candle.volume, 3.5);
        assert_eq!(candle.cvd, -0.5); // 1.0 - 2.0 + 0.5
        assert_eq!(candle.high, 45200.0);
        assert_eq!(candle.low, 45000.0);
        assert_eq!(candle.close, 45200.0);
    }

    #[test]
    fn test_candle_builder_initialization() {
        let builder = CandleBuilder::new(1);
        // Test is implicit - if it compiles and creates successfully
        assert_eq!(builder.interval_ms, 60000); // 1 minute
    }

    #[test]
    fn test_candle_builder_single_trade() {
        let mut builder = CandleBuilder::new(1);
        let result = builder.process_trade(1640995200000, 45000.0, 1.0, true);

        assert!(result.is_none()); // No finished candle yet

        // Check internal state
        if let Some(candle) = &builder.current_candle {
            assert_eq!(candle.timestamp, 1640995200000);
            assert_eq!(candle.volume, 1.0);
            assert_eq!(candle.cvd, 1.0);
        } else {
            panic!("Expected candle to be created");
        }
    }

    #[test]
    fn test_candle_builder_multiple_trades_same_candle() {
        let mut builder = CandleBuilder::new(1);

        // All trades in same 1-minute interval
        builder.process_trade(1640995200000, 45000.0, 1.0, true);
        builder.process_trade(1640995200100, 45100.0, 2.0, false);
        builder.process_trade(1640995200200, 44900.0, 0.5, true);

        // Should still have no finished candle
        let result = builder.process_trade(1640995200300, 45000.0, 1.5, true);
        assert!(result.is_none());

        // Check accumulated state
        if let Some(candle) = &builder.current_candle {
            assert_eq!(candle.volume, 5.0); // 1.0 + 2.0 + 0.5 + 1.5
            assert_eq!(candle.cvd, 0.0);   // 1.0 - 2.0 + 0.5 + 1.5
            assert_eq!(candle.high, 45100.0);
            assert_eq!(candle.low, 44900.0);
            assert_eq!(candle.close, 45000.0);
        }
    }

    #[test]
    fn test_candle_builder_candle_completion() {
        let mut builder = CandleBuilder::new(1);

        // First candle
        builder.process_trade(1640995200000, 45000.0, 1.0, true);

        // Next trade in next minute - should complete first candle
        let finished = builder.process_trade(1640995260000, 45100.0, 2.0, false);

        assert!(finished.is_some());
        if let Some(candle) = finished {
            assert_eq!(candle.timestamp, 1640995200000);
            assert_eq!(candle.volume, 1.0);
            assert_eq!(candle.cvd, 1.0);
            assert_eq!(candle.close, 45000.0);
        }

        // Check new candle started
        if let Some(current) = &builder.current_candle {
            assert_eq!(current.timestamp, 1640995260000);
            assert_eq!(current.volume, 2.0);
            assert_eq!(current.cvd, -2.0); // Sell trade
        }
    }

    #[test]
    fn test_candle_builder_different_intervals() {
        let mut builder = CandleBuilder::new(5); // 5-minute candles
        assert_eq!(builder.interval_ms, 300000);

        // Trades across 5-minute boundaries
        builder.process_trade(1640995200000, 45000.0, 1.0, true);  // Minute 0
        builder.process_trade(1640995500000, 45100.0, 2.0, false); // Minute 5 - should complete candle

        let finished = builder.process_trade(1640995500000, 45100.0, 2.0, false);
        assert!(finished.is_some());
    }

    #[test]
    fn test_cvd_divergence_scenarios() {
        // Test scenarios that should produce CVD divergences

        // Bullish divergence: Price makes lower low, CVD makes higher low
        let mut bullish_candle = Candle::new(1640995200000, 45000.0, 5.0, true);
        // Simulate lower low but higher CVD (more buying)
        bullish_candle.update(44900.0, 3.0, true); // Additional buying

        // Bearish divergence: Price makes higher high, CVD makes lower high
        let mut bearish_candle = Candle::new(1640995200000, 45000.0, 5.0, false);
        // Simulate higher high but lower CVD (more selling)
        bearish_candle.update(45100.0, 3.0, false); // Additional selling

        // Verify CVD calculations
        assert_eq!(bullish_candle.cvd, 8.0);  // 5.0 + 3.0
        assert_eq!(bearish_candle.cvd, -8.0); // -5.0 - 3.0
    }

    #[test]
    fn test_zero_volume_edge_cases() {
        let candle = Candle::new(1640995200000, 45000.0, 0.0, true);
        assert_eq!(candle.volume, 0.0);
        assert_eq!(candle.cvd, 0.0);
    }

    #[test]
    fn test_extreme_price_movements() {
        let mut candle = Candle::new(1640995200000, 45000.0, 1.0, true);

        // Extreme price movements
        candle.update(100000.0, 1.0, false); // Massive high
        candle.update(1000.0, 1.0, true);    // Massive low

        assert_eq!(candle.high, 100000.0);
        assert_eq!(candle.low, 1000.0);
        assert_eq!(candle.volume, 3.0);
        assert_eq!(candle.cvd, -1.0); // 1.0 - 1.0 + 1.0
    }
}