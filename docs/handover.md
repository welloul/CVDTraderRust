## Current Project Status
- **Status:** **Stabilization Phase Complete**
- **Stability:** **High** (All core compilation errors resolved, unified Error handling via Anyhow)
- **Phase:** **Transition to Live Execution Readiness**

## Context Injection (Critical Decisions)
1.  **Unified State Ownership:** All split state "sources of truth" have been unified into a single `Arc<Mutex<GlobalState>>`. Do NOT re-introduce `lazy_static` singletons; the code follows explicit Dependency Injection principles.
2.  **Typed Market Events:** Market data parsing is centralized in `src/market_data/event.rs`. MarketDataHandler now requires `GlobalState` injection to track per-coin latencies accurately.
3.  **Unified Error Domain:** The codebase now uses `anyhow::Result` globally for fallible operations. When adding new modules, utilize `.context()` to preserve the stack-trace/context without boilerplate trait implementations.
4.  **Observable Configuration:** Strategy logic now reads parameters directly from `GlobalState.config`. The `Config` struct is accessed via typed fields (e.g., `config.execution.mode`) rather than string-key `get()` calls.
5.  **Non-blocking Database**: Database interactions are fully asynchronous via `tokio-rusqlite`. The `Repository` layer abstracts all SQL logic, including periodic context-aware cleanup (TTL-based).

## Next Steps
- Implement **Exchange Authentication** for real order signing (currently in dryrun mode).
- Finalize the **OrderTTLTracker** logic (struct exists, but logic is pending full integration).
- Complete the **sync_state** and **sync_main_wallet_balance** TODOs in `src/core/state.rs`.
- Implement full **Hyperliquid API** authentication in `client.rs`.