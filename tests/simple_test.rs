use cvd_trader_rust::market_data::candles::{Candle, calculate_cvd};

#[test]
fn test_basic_cvd_calculation() {
    let mut candles = vec![
        Candle {
            timestamp: 1000,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
            cvd: 0.0,
            poc: 101.0,
        },
        Candle {
            timestamp: 2000,
            open: 102.0,
            high: 108.0,
            low: 98.0,
            close: 106.0,
            volume: 1200.0,
            cvd: 0.0,
            poc: 105.0,
        },
    ];

    calculate_cvd(&mut candles);

    // Check CVD calculation - first candle should be volume since it's the first
    // Second candle: if close > open (bullish), cvd += volume, else cvd -= volume
    // 106 > 102, so bullish: cvd should be 1200
    assert_eq!(candles[0].cvd, 1000.0); // First candle
    assert_eq!(candles[1].cvd, 2200.0); // 1000 + 1200 (cumulative)
}

#[test]
fn test_cvd_bearish() {
    let mut candles = vec![
        Candle {
            timestamp: 1000,
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
            cvd: 0.0,
            poc: 101.0,
        },
        Candle {
            timestamp: 2000,
            open: 106.0,
            high: 108.0,
            low: 98.0,
            close: 100.0, // Close < Open = bearish
            volume: 800.0,
            cvd: 0.0,
            poc: 105.0,
        },
    ];

    calculate_cvd(&mut candles);

    // First candle: 1000
    // Second candle: bearish (100 < 106), so cvd -= 800 = 1000 - 800 = 200
    assert_eq!(candles[0].cvd, 1000.0);
    assert_eq!(candles[1].cvd, 200.0);
}