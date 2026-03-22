## Development Environment

### IDE Setup
- **VS Code** with rust-analyzer extension
- **CLion** with Rust plugin
- Enable "Check on save" for faster feedback

### Testing Infrastructure
The project includes comprehensive automated testing:

#### Test Categories
```bash
# Unit Tests (Isolated functionality)
cargo test --test candle_tests     # CVD calculations
cargo test --test strategy_tests   # Signal generation
cargo test --test core_tests       # Utilities

# Integration Tests (Cross-module)
cargo test --test integration_tests

# Property-Based Tests (Mathematical correctness)
cargo test property_tests

# All Tests
cargo test
```

#### Performance Validation
```bash
# Benchmarks with Criterion
cargo bench

# Typical results:
# candle_creation: ~5ns
# candle_builder_1000_trades: ~46μs
# cvd_calculation: ~90ns
```

### Debugging
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Profile performance
cargo build --release
perf record ./target/release/cvd_trader_rust
perf report
```

### Configuration
The application uses TOML-based configuration for runtime tunability:

```toml
# config.toml - Place in project root
[strategy]
lookback = 20
fixed_fee_rate = 0.0003

[risk]
max_allowed_latency_ms = 1000.0

[general]
target_coins = ["SOL", "ZEC", "HYPE", "XMR", "LINK", "XLM", "AVAX", "TON", "TAO"]
```

Configuration loads from `config.toml` or falls back to sensible defaults. No restart required for parameter changes.

### Code Quality
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Generate coverage (requires tarpaulin)
cargo tarpaulin --out Html