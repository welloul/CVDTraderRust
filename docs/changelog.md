# Changelog

## [2026-03-22] - Production Stabilization & Refactoring

### Fixed
- **Market Data Event Parsing**: Fixed a critical bug in `MarketDataHandler` where dispatched `candle_closed` JSON events lacked the `price` field, causing silent parsing failures in `MarketDataEvent::from_value` and preventing candle historical buildup in the strategy module.
- **Type Safety**: Resolved 40+ compilation errors related to `anyhow` vs `std::error::Error` trait mismatches.
- **Asynchronous Integrity**: Added missing `.await` keywords on all `tokio-rusqlite` call sites and network I/O.
- **Dependency Injection**: Successfully eliminated the `state::STATE` singleton, moving to explicit `Arc<Mutex<GlobalState>>` injection for improved testability and thread safety.
- **Config Access**: Replaced dynamic string-based configuration calls with typed field access (e.g., `config.execution.mode`).
- **Health Monitoring**: Fixed false-negative Unhealthy status by correcting the system's `is_running` flag initialization and URL connectivity checks.

### Changed
- **Error Handling**: Standardized on `anyhow::Result` globally across all modules (`persistence`, `execution`, `market_data`, `health`).
- **API Server**: Corrected address-binding issues and improved error reporting for the `axum` server lifecycle.

## [0.1.2] - 2026-03-22 - WebSocket Stability & Heartbeat Sprint
**🎯 Objective: Solve frequent disconnections and ensure data continuity for strategy execution.**

- **WebSocket Heartbeat**: Implemented `{"method": "ping"}` logic in `MarketDataHandler` to prevent 30-second timeouts from Hyperliquid.
- **Deadlock Mitigation**: Identified and resolved a critical async deadlock in `MarketDataHandler` where the the `GlobalState` Mutex was held through long-running callbacks.
- **TLS Support**: Enabled `native-tls` in `Cargo.toml` to support `wss://` (WebSocket Secure) and `https://` requests on the production server.
- **Improved Logging**:
    - Added "Warmup" progress indicators for strategy lookback (e.g., `Warming up SOL... 5/10 candles`).
    - Added `1m Candle Closed` and trade activity logs to verify real-time data flow.
- **JSON Export Persistence**: Ensured `backend/data/` is created automatically, providing a stable path for exporting closed trade JSON files.
- **Deployment & Setup**: Created a high-fidelity `docs/setup.md` for AWS Amazon Linux deployment, including a Systemd service template.
- **Lookback Optimization**: Reduced default development lookback to 10 minutes for faster functional verification of strategy signals.

### Modules Updated
- `src/persistence/repository.rs`: Now fully uses `anyhow` for context-aware database errors.
- `src/execution/gateway.rs`: Refactored to handle both real and dryrun modes with shared state.
- `src/core/state.rs`: Unified GlobalState initialization and trade loading logic.
- `src/monitoring/health.rs`: Improved trait implementations and async consistency for all checks.

---

## [Previously Included]
- **State Persistence System**: Complete database layer with SQLite backend
- **Production Monitoring Infrastructure**: Comprehensive health and metrics system
- **Enhanced API Server**: Production-ready monitoring and control endpoints
... (previous entries preserved)