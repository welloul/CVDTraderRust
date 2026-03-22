## Current Project Status
- **Status:** **Ready for VPS Deployment**
- **Stability:** **Production-Grade** (WebSocket heartbeats implemented, Deadlocks resolved, TLS support added)
- **Phase:** **Deployment & Scaling**

## Context Injection (Critical Decisions)
1.  **Unified State Ownership:** All split state "sources of truth" have been unified into a single `Arc<Mutex<GlobalState>>`. Do NOT re-introduce `lazy_static` singletons; the code follows explicit Dependency Injection principles.
2.  **WebSocket Persistence:** Managed via a 50-second ping cycle. Do not reduce the ping interval below 20 seconds to avoid Hyperliquid rate-limits.
3.  **TLS and Environment:** The production environment (AWS/Linux) requires `openssl-devel` and `sqlite-devel` to compile the `native-tls` and `bundled` rusqlite features.
4.  **Signal Lookback:** The current `lookback` is set to 10 for rapid verification. Restoration to 20 or higher is recommended for production signal quality once functionality is confirmed on VPS.

## Known Failures & Edge Cases
- **Silent Deserialization Failures**: Event payloads (like candle closures) passed dynamically via `serde_json::Value` callbacks can be silently dropped by `MarketDataEvent::from_value` if critical fields like `price` are missing. We patched this by decorating `candle_closed` events with `price` and `latency_ms` properties, but any future events emitted from the `MarketDataHandler` must structurally adhere to the minimal requirements of `MarketDataEvent`.

## Next Steps
- **Deploy to VPS**: Use `docs/setup.md` to move the bot to a persistent Amazon Linux environment.
- **Implement Exchange Authentication**: Enable real order signing (currently in dryrun mode).
- **Finalize OrderTTLTracker**: Complete the logic integration to manage order time-to-live.
- **WebSocket Sharding**: If expanding beyond 10 coins, implement a connection aggregator to avoid per-coin connection overhead.