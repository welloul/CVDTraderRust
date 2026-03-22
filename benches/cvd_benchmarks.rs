use criterion::{black_box, criterion_group, criterion_main, Criterion};
use cvd_trader_rust::market_data::candles::{Candle, CandleBuilder};
use cvd_trader_rust::core::rounding::RoundingUtil;

fn bench_candle_creation(c: &mut Criterion) {
    c.bench_function("candle_creation", |b| {
        b.iter(|| {
            let candle = Candle::new(
                black_box(1640995200000),
                black_box(45000.0),
                black_box(1.5),
                black_box(true)
            );
            black_box(candle);
        })
    });
}

fn bench_candle_updates(c: &mut Criterion) {
    c.bench_function("candle_updates_100", |b| {
        b.iter(|| {
            let mut candle = Candle::new(1640995200000, 45000.0, 1.0, true);

            for i in 0..100 {
                let price = 45000.0 + (i as f64 * 10.0);
                let volume = 1.0 + (i as f64 * 0.1);
                let is_buy = i % 2 == 0;
                candle.update(price, volume, is_buy);
            }

            black_box(candle);
        })
    });
}

fn bench_candle_builder_processing(c: &mut Criterion) {
    let trades: Vec<(i64, f64, f64, bool)> = (0..1000)
        .map(|i| {
            let timestamp = 1640995200000 + (i * 1000);
            let price = 45000.0 + (i as f64 * 0.1);
            let volume = 1.0 + (i as f64 * 0.01);
            let is_buy = i % 2 == 0;
            (timestamp, price, volume, is_buy)
        })
        .collect();

    c.bench_function("candle_builder_1000_trades", |b| {
        b.iter(|| {
            let mut builder = CandleBuilder::new(1);

            for (timestamp, price, volume, is_buy) in &trades {
                builder.process_trade(*timestamp, *price, *volume, *is_buy);
            }

            black_box(builder);
        })
    });
}

fn bench_candle_completion(c: &mut Criterion) {
    c.bench_function("candle_completion_cycle", |b| {
        b.iter(|| {
            let mut builder = CandleBuilder::new(1);
            let base_time = 1640995200000;

            // Fill a candle with trades
            for i in 0..50 {
                let timestamp = base_time + (i * 1000);
                let price = 45000.0 + (i as f64);
                let volume = 1.0;
                let is_buy = i % 2 == 0;
                builder.process_trade(timestamp, price, volume, is_buy);
            }

            // Complete the candle
            let finished = builder.process_trade(base_time + 60000, 45050.0, 1.0, true);
            black_box(finished);
        })
    });
}

fn bench_rounding_operations(c: &mut Criterion) {
    let util = RoundingUtil::new(None);

    c.bench_function("rounding_size_btc", |b| {
        b.iter(|| {
            let result = util.round_size(black_box("BTC"), black_box(1.23456789));
            black_box(result);
        })
    });

    c.bench_function("rounding_price_btc", |b| {
        b.iter(|| {
            let result = util.round_price(black_box("BTC"), black_box(45000.123456789));
            black_box(result);
        })
    });

    c.bench_function("format_for_api", |b| {
        b.iter(|| {
            let result = util.format_for_api(black_box(1.234567890000));
            black_box(result);
        })
    });
}

fn bench_cvd_calculations(c: &mut Criterion) {
    c.bench_function("cvd_calculation_mixed_trades", |b| {
        b.iter(|| {
            let mut builder = CandleBuilder::new(1);
            let base_time = 1640995200000;

            // Simulate realistic trading activity with CVD pressure
            let trades = [
                (1.2, true),   // Buy
                (0.8, false),  // Sell
                (2.1, true),   // Buy
                (1.5, false),  // Sell
                (0.9, true),   // Buy
                (1.8, false),  // Sell
            ];

            for (i, (volume, is_buy)) in trades.iter().enumerate() {
                let timestamp = base_time + (i as i64 * 1000);
                let price = 45000.0 + (i as f64 * 5.0);
                builder.process_trade(timestamp, price, *volume, *is_buy);
            }

            // Complete candle to get final CVD
            let finished = builder.process_trade(base_time + 60000, 45030.0, 1.0, true);
            black_box(finished);
        })
    });
}

fn bench_memory_usage(c: &mut Criterion) {
    c.bench_function("memory_candle_array_1000", |b| {
        b.iter(|| {
            let mut candles = Vec::new();

            for i in 0..1000 {
                let candle = Candle::new(
                    1640995200000 + (i as i64 * 60000),
                    45000.0 + (i as f64),
                    10.0 + (i as f64 * 0.1),
                    i % 2 == 0
                );
                candles.push(candle);
            }

            black_box(candles);
        })
    });
}

fn bench_strategy_signal_detection(c: &mut Criterion) {
    // Create test data for signal detection
    let candles: Vec<Candle> = (0..20)
        .map(|i| {
            let timestamp = 1640995200000 + (i as i64 * 60000);
            let open = 45000.0 + (i as f64 * 10.0);
            let high = open + 50.0;
            let low = open - 50.0;
            let close = open + (i as f64 * 5.0);
            let volume = 100.0 + (i as f64 * 10.0);
            let cvd = if i % 2 == 0 { volume * 0.3 } else { -volume * 0.3 };

            Candle {
                timestamp,
                open,
                high,
                low,
                close,
                volume,
                cvd,
                poc: close,
            }
        })
        .collect();

    c.bench_function("swing_detection_20_candles", |b| {
        b.iter(|| {
            // Test swing detection logic
            let result_high = detect_swing_high(&candles);
            let result_low = detect_swing_low(&candles);
            black_box((result_high, result_low));
        })
    });

    c.bench_function("cvd_divergence_detection", |b| {
        b.iter(|| {
            let result = is_cvd_exhaustion(&candles, true);
            black_box(result);
        })
    });
}

// Helper functions for benchmarks (duplicated from tests for isolation)
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

criterion_group!(
    benches,
    bench_candle_creation,
    bench_candle_updates,
    bench_candle_builder_processing,
    bench_candle_completion,
    bench_rounding_operations,
    bench_cvd_calculations,
    bench_memory_usage,
    bench_strategy_signal_detection
);

criterion_main!(benches);