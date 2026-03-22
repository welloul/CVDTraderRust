use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub cvd: f64,        // Cumulative Volume Delta: buy_volume - sell_volume
    pub poc: f64,
}

impl Candle {
    pub fn new(timestamp: i64, price: f64, volume: f64, is_buy: bool) -> Self {
        let cvd = if is_buy { volume } else { -volume };
        Self {
            timestamp,
            open: price,
            high: price,
            low: price,
            close: price,
            volume,
            cvd,
            poc: price,  // Simplified POC
        }
    }

    pub fn update(&mut self, price: f64, volume: f64, is_buy: bool) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;

        // Update CVD: buy volume positive, sell volume negative
        if is_buy {
            self.cvd += volume;
        } else {
            self.cvd -= volume;
        }
    }
}

pub struct CandleBuilder {
    current_candle: Option<Candle>,
    interval_ms: i64, // 1 minute = 60000 ms
}

impl CandleBuilder {
    pub fn new(interval_minutes: i64) -> Self {
        Self {
            current_candle: None,
            interval_ms: interval_minutes * 60000,
        }
    }

    pub fn process_trade(&mut self, timestamp_ms: i64, price: f64, volume: f64, is_buy: bool) -> Option<Candle> {
        let candle_start = (timestamp_ms / self.interval_ms) * self.interval_ms;

        match &mut self.current_candle {
            Some(candle) if candle.timestamp == candle_start => {
                // Update current candle
                candle.update(price, volume, is_buy);
                None
            }
            Some(candle) => {
                // Finish current candle and start new one
                let finished = candle.clone();
                *candle = Candle::new(candle_start, price, volume, is_buy);
                Some(finished)
            }
            None => {
                // Start first candle
                self.current_candle = Some(Candle::new(candle_start, price, volume, is_buy));
                None
            }
        }
    }
}