# Module: Core

## Responsibility

The Core module provides foundational infrastructure for the trading system, managing global application state, logging, and asset-specific precision handling. It serves as the central nervous system, maintaining consistency across all trading operations and providing utilities for safe financial calculations.

### Why
- **Centralized State**: Single source of truth for positions, orders, and configuration prevents data inconsistencies
- **Precision Safety**: Asset-specific rounding prevents order rejections and slippage
- **Observability**: Structured logging enables debugging and monitoring of trading decisions
- **Performance Tracking**: Latency measurement ensures execution quality

### What
- GlobalState: Thread-safe shared state with positions, orders, trades, and configuration.
- Logger: `init_logger()` function and `log!` macro-based interface using `tracing-subscriber`.
- RoundingUtil: Hyperliquid-specific price and size formatting utilities.
- Config: TOML-based configuration system with runtime loading (now uses typed fields instead of dynamic get).
- Error Handling: Global integration of `anyhow` for context-aware error propagation.

## Key Logic & Functions

### GlobalState
**Core Data Structures:**
```rust
pub struct GlobalState {
    pub is_running: bool,
    pub config: Config, // Structured Config struct instead of HashMap
    pub positions: HashMap<String, Position>,
    pub active_orders: HashMap<i64, ActiveOrder>,
    pub closed_trades: Vec<ClosedTrade>,
    pub market_data: HashMap<String, HashMap<String, Vec<f64>>>,
    pub wallet_balance: f64,
    pub main_wallet_balance: f64,
    pub logs: Vec<LogEntry>,
    pub latency_by_coin: HashMap<String, Vec<f64>>,
}
```

**Key Methods:**
- `new()` → Self: Initializes with default config and loads closed trades from disk
- `update_latency(&mut self, coin: &str, latency_ms: f64)`: Tracks execution latency per coin (keeps last 100 samples)
- `get_latency_stats(&self) -> HashMap<String, HashMap<String, f64>>`: Calculates avg/min/max/median latency with outlier filtering
- `add_log(&mut self, level: &str, message: &str, extra: HashMap<String, serde_json::Value>)`: Adds structured log entry (max 50 entries)
- `start_bot/stop_bot(&mut self)`: Command & control interface (currently placeholder)

**Side Effects:**
- File I/O for trade persistence (`backend/data/trades.json`)
- In-memory log rotation (FIFO when >50 entries)
- Latency sample rotation (FIFO when >100 samples)

### RoundingUtil
**Core Data Structures:**
```rust
pub struct AssetInfo {
    pub sz_decimals: i32,    // Size precision
    pub px_decimals: i32,    // Price precision
    pub tick_size: f64,      // Minimum price increment
}
```

**Key Methods:**
- `new(meta_info: Option<Value>) -> Self`: Parses Hyperliquid universe metadata or uses defaults
- `round_size(&self, coin: &str, sz: f64) -> String`: Formats position size to asset precision
- `round_price(&self, coin: &str, px: f64) -> String`: Formats price to tick size
- `format_for_api(&self, num: f64) -> String`: Trims trailing zeros for API compatibility

**Side Effects:**
- Fallback to default precision when asset metadata unavailable
- String formatting for API compatibility (no trailing zeros)

### Config
**Core Data Structures:**
```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub strategy: StrategyConfig,
    pub risk: RiskConfig,
    pub execution: ExecutionConfig,
    pub general: GeneralConfig,
}
```

**Key Methods:**
- `load() -> Self`: Loads configuration from config.toml or uses defaults
- `from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>>`: Loads from specific file

**Configuration Sections:**
- **Strategy**: Algorithm parameters (lookback, ratios, fees)
- **Risk**: Safety thresholds (latency limits, failure counts)
- **Execution**: Trading parameters (slippage)
- **General**: System settings (latency, target coins)

**Side Effects:**
- File I/O for config.toml loading
- Fallback to compile-time defaults when file missing

### Logger
**Key Interface:**
```rust
log!(info, "message {}", var);
log!(warn, "warning: {:?}", error);
```

**Side Effects:**
- Output to configured tracing subscriber (console by default)

## Hurdles

### Bugs
- **State Persistence**: Trade saving errors logged but not handled - potential data loss
- **Latency Filtering**: Outlier filter range (-50000..=50000) may exclude valid high-latency samples
- **Clock Offset**: Median latency labeled as "clock_offset_ms" but actually median latency

### Race Conditions
- **Shared State Access**: All GlobalState methods require external Mutex locking
- **Log Rotation**: Concurrent add_log calls may cause inconsistent log ordering
- **Latency Updates**: update_latency not thread-safe without external synchronization

### Technical Debt
- **Dependency Injection**: Successfully replaced the legacy `STATE` singleton with explicit `Arc<Mutex<GlobalState>>` parameter passing across `MarketDataHandler`, `ExecutionGateway`, and `StrategyModule`.
- **In-Memory Only**: No persistence for active positions/orders (only closed trades)
- **Magic Numbers**: Hardcoded limits (50 logs, 100 latency samples)
- **Error Handling**: load_trades/save_trades swallow errors after logging
- **Sync Methods**: sync_state/sync_main_wallet_balance are placeholders returning no errors

## Future Roadmap

### Immediate (Next Sprint)
- **Implement State Sync**: Complete Hyperliquid API integration for real state synchronization
- **Add Persistence**: Database storage for all state (positions, orders, config)
- **Thread Safety**: Audit all state access patterns for race conditions
- **Error Recovery**: Implement retry logic for failed state sync operations

### Short Term (1-2 weeks)
- **Configuration Validation**: Runtime config validation with sensible defaults
- **Metrics Export**: Prometheus-compatible metrics endpoint
- **State Backup**: Automatic state snapshots for crash recovery
- **Memory Bounds**: Configurable limits for collections with LRU eviction

### Medium Term (1-2 months)
- **State Sharding**: Partition state by coin for better concurrency
- **Audit Logging**: Immutable log chain for regulatory compliance
- **Real-time Sync**: WebSocket-based state synchronization
- **State Compression**: Efficient serialization for large state objects

### Long Term (3-6 months)
- **Distributed State**: Multi-node state replication for HA
- **State Analytics**: Historical state analysis for strategy optimization
- **Dynamic Config**: Runtime configuration updates without restart (via GlobalState.config)
- **State Migration**: Versioned state schema with automatic migration