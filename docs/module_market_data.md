# Module: Market Data

## Responsibility
The Market Data module is responsible for real-time ingestion, parsing, and normalization of trade data. It acts as the gateway for all external market information, transforming raw WebSocket JSON into typed, validated `MarketDataEvent` objects.

## Key Logic & Functions

### MarketDataEvent
The primary data exchange format for market updates.
```rust
pub struct MarketDataEvent {
    pub coin: String,
    pub price: f64,
    pub vwap: f64,
    pub indicators: serde_json::Value,
    pub closed_candle_1m: Option<Candle>,
    pub latency_ms: f64,
}
```

### MarketDataHandler
Manages the lifecycle of exchange connections and dispatches events.
- **Dependency Injection**: Now requires `Arc<Mutex<GlobalState>>` to update global latency statistics.
- **WebSocket Heartbeat**: Implements a 50-second `{"method": "ping"}` cycle to prevent connection timeouts from Hyperliquid.
- **State Lock Mitigation**: Explicitly drops the global state lock before calling external callbacks to ensure 100% async safety and prevent deadlocks during high-throughput message bursts.
- **`on_trade`**: Triggered by raw WebSocket updates. It feeds the `CandleBuilder` and emits a `MarketDataEvent`.
- **Latency Tracking**: Appends an arrival timestamp to calculate processing delay and updates `GlobalState.latency_by_coin`.
- **Error Handling**: Uses `anyhow::Result` for the connection lifecycle and message handling.

## Hurdles
* **Network Latency**: WebSocket jitter can cause delayed signal detection.
* **Type Safety**: Raw JSON from Hyperliquid is dynamic; `MarketDataEvent::from_value` uses safe-parsing with defaults to avoid runtime panics.
* **Silent Event Drops**: If the JSON payload emitted by the `MarketDataHandler` lacks a critical required field (like `price`), `MarketDataEvent::from_value` returns `None`, silently dropping the event. This previously caused `candle_closed` events to never reach the strategy unit.

## Future Roadmap
- [ ] Implement L2/L1 Order-book depth events.
- [ ] Add sharding for high-coin-count subscriptions.
- [ ] Formalize `indicators` as a Typed Struct instead of `serde_json::Value`.