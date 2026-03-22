# Changelog

## [2026-03-22] - Production Stabilization & Refactoring

### Fixed
- **Type Safety**: Resolved 40+ compilation errors related to `anyhow` vs `std::error::Error` trait mismatches.
- **Asynchronous Integrity**: Added missing `.await` keywords on all `tokio-rusqlite` call sites and network I/O.
- **Dependency Injection**: Successfully eliminated the `state::STATE` singleton, moving to explicit `Arc<Mutex<GlobalState>>` injection for improved testability and thread safety.
- **Config Access**: Replaced dynamic string-based configuration calls with typed field access (e.g., `config.execution.mode`).
- **Health Monitoring**: Fixed false-negative Unhealthy status by correcting the system's `is_running` flag initialization and URL connectivity checks.

### Changed
- **Error Handling**: Standardized on `anyhow::Result` globally across all modules (`persistence`, `execution`, `market_data`, `health`).
- **API Server**: Corrected address-binding issues and improved error reporting for the `axum` server lifecycle.
- **Market Data**: Enhanced `MarketDataHandler` to participate in global latency tracking.

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