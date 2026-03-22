use proptest::prelude::*;
use cvd_trader_rust::market_data::candles::{Candle, CandleBuilder};
use cvd_trader_rust::core::rounding::RoundingUtil;

// Property-based tests to ensure correctness across wide input ranges

proptest! {
    #[test]
    fn candle_cvd_never_nan(
        timestamp in 0..86400000i64,  // 1 day in milliseconds
        price in 0.01f64..1000000.0,
        volume in 0.000001f64..1000.0,
        is_buy in proptest::bool::ANY
    ) {
        let candle = Candle::new(timestamp, price, volume, is_buy);
        prop_assert!(!candle.cvd.is_nan(), "CVD should never be NaN");
        prop_assert!(candle.cvd.is_finite(), "CVD should be finite");
    }

    #[test]
    fn candle_volume_always_positive(
        timestamp in 0..86400000i64,
        price in 0.01f64..1000000.0,
        volume in 0.000001f64..1000.0,
        is_buy in proptest::bool::ANY
    ) {
        let candle = Candle::new(timestamp, price, volume, is_buy);
        prop_assert!(candle.volume >= 0.0, "Volume should always be non-negative");
    }

    #[test]
    fn candle_update_preserves_ohlc_properties(
        initial_price in 100.0..100000.0,
        initial_volume in 0.1..100.0,
        updates in prop::collection::vec(
            (100.0..100000.0f64, 0.1..100.0f64, proptest::bool::ANY),
            1..20
        )
    ) {
        let mut candle = Candle::new(1640995200000, initial_price, initial_volume, true);

        let mut min_price = initial_price;
        let mut max_price = initial_price;
        let mut total_volume = initial_volume;
        let mut expected_cvd = initial_volume; // First trade is buy

        for (price, volume, is_buy) in updates {
            min_price = min_price.min(price);
            max_price = max_price.max(price);
            total_volume += volume;

            if is_buy {
                expected_cvd += volume;
            } else {
                expected_cvd -= volume;
            }

            candle.update(price, volume, is_buy);
        }

        // Properties that must hold
        prop_assert_eq!(candle.low, min_price, "Low should be minimum price");
        prop_assert_eq!(candle.high, max_price, "High should be maximum price");
        prop_assert_eq!(candle.volume, total_volume, "Volume should be sum of all trades");
        prop_assert_eq!(candle.cvd, expected_cvd, "CVD should be correct delta calculation");
        prop_assert!(candle.close >= candle.low && candle.close <= candle.high, "Close should be within high-low range");
    }

    #[test]
    fn rounding_util_consistent_precision(
        value in 0.000001f64..1000000.0,
        asset in prop_oneof![
            Just("BTC"),
            Just("ETH"),
            Just("UNKNOWN")
        ]
    ) {
        let util = RoundingUtil::new(None);

        let rounded_size = util.round_size(&asset, value);
        let rounded_price = util.round_price(&asset, value);

        // Should not panic and should return valid strings
        prop_assert!(!rounded_size.is_empty());
        prop_assert!(!rounded_price.is_empty());

        // Should be parseable as floats
        prop_assert!(rounded_size.parse::<f64>().is_ok());
        prop_assert!(rounded_price.parse::<f64>().is_ok());
    }

    #[test]
    fn candle_builder_interval_alignment(
        base_timestamp in 1640995200000..1640995260000i64,  // Within test minute
        trades in prop::collection::vec(
            (100.0..100000.0f64, 0.1..100.0f64, proptest::bool::ANY),
            1..10
        )
    ) {
        let mut builder = CandleBuilder::new(1); // 1-minute candles

        // Process trades within the same minute
        for (price, volume, is_buy) in &trades {
            let timestamp = base_timestamp + (trades.iter().position(|x| x == &(*price, *volume, *is_buy)).unwrap() as i64 * 1000);
            builder.process_trade(timestamp, *price, *volume, *is_buy);
        }

        // Should not have completed any candles yet
        let result = builder.process_trade(base_timestamp, 100.0, 1.0, true);
        prop_assert!(result.is_none(), "Should not complete candle within same interval");

        // Move to next minute - should complete candle
        let next_minute = base_timestamp + 60000;
        let result = builder.process_trade(next_minute, 100.0, 1.0, true);
        prop_assert!(result.is_some(), "Should complete candle when crossing interval boundary");
    }

    #[test]
    fn candle_cvd_conservation_of_momentum(
        trades in prop::collection::vec(
            (100.0..100000.0f64, 0.1..100.0f64, proptest::bool::ANY),
            10..50
        )
    ) {
        let mut builder = CandleBuilder::new(1);
        let base_timestamp = 1640995200000i64;

        // Process all trades in same candle
        for (i, (price, volume, is_buy)) in trades.iter().enumerate() {
            let timestamp = base_timestamp + (i as i64 * 1000);
            builder.process_trade(timestamp, *price, *volume, *is_buy);
        }

        // Complete the candle
        let finished = builder.process_trade(base_timestamp + 60000, 100.0, 1.0, true);
        prop_assert!(finished.is_some());

        if let Some(candle) = finished {
            // CVD should be: sum(buy_volumes) - sum(sell_volumes)
            let expected_cvd: f64 = trades.iter()
                .map(|(_, volume, is_buy)| if *is_buy { *volume } else { -*volume })
                .sum();

            prop_assert!((candle.cvd - expected_cvd).abs() < 0.000001, "CVD should match expected calculation");
        }
    }

    #[test]
    fn rounding_util_api_format_consistency(
        value in 0.000001f64..1000000.0
    ) {
        let util = RoundingUtil::new(None);

        let formatted = util.format_for_api(value);

        // Should not have trailing zeros after decimal
        if formatted.contains('.') {
            prop_assert!(!formatted.ends_with('0'), "Should not end with zero after decimal");
            prop_assert!(!formatted.ends_with('.'), "Should not end with decimal point");
        }

        // Should be parseable back to same value (within floating point precision)
        let parsed = formatted.parse::<f64>().unwrap();
        prop_assert!((parsed - value).abs() < 0.000001, "API format should preserve value");
    }

    #[test]
    fn candle_price_extremes_handled(
        price in prop_oneof![
            0.000001f64..0.01,      // Very small prices
            1000000.0..10000000.0, // Very large prices
            0.01..1000000.0        // Normal range
        ],
        volume in 0.000001f64..1000.0,
        is_buy in proptest::bool::ANY
    ) {
        let candle = Candle::new(1640995200000, price, volume, is_buy);

        // Should handle extreme prices without issues
        prop_assert!(candle.open.is_finite());
        prop_assert!(candle.high.is_finite());
        prop_assert!(candle.low.is_finite());
        prop_assert!(candle.close.is_finite());
        prop_assert!(candle.cvd.is_finite());
    }

    #[test]
    fn multiple_candle_completion(
        candle_count in 5..20usize,
        trades_per_candle in 3..15usize
    ) {
        let mut builder = CandleBuilder::new(1);
        let base_timestamp = 1640995200000i64;
        let mut completed_candles = 0;

        for candle_idx in 0..candle_count {
            let candle_start = base_timestamp + (candle_idx as i64 * 60000);

            // Add trades to this candle
            for trade_idx in 0..trades_per_candle {
                let timestamp = candle_start + (trade_idx as i64 * 1000);
                let price = 100.0 + (candle_idx as f64 * 10.0) + (trade_idx as f64);
                let volume = 1.0 + (trade_idx as f64 * 0.1);
                let is_buy = trade_idx % 2 == 0;

                let result = builder.process_trade(timestamp, price, volume, is_buy);

                // Should not complete candle until we finish this minute
                if trade_idx < trades_per_candle - 1 {
                    prop_assert!(result.is_none(), "Should not complete candle mid-minute");
                }
            }

            // Finish this candle by moving to next minute
            let next_candle_start = candle_start + 60000;
            let result = builder.process_trade(next_candle_start, 100.0, 1.0, true);

            if candle_idx < candle_count - 1 {
                prop_assert!(result.is_some(), "Should complete candle when crossing boundary");
                completed_candles += 1;
            }
        }

        prop_assert_eq!(completed_candles, candle_count - 1, "Should complete correct number of candles");
    }
}