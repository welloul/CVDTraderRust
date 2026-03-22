# Testing Infrastructure

This directory contains comprehensive testing infrastructure for the CVD Trader Rust project, ensuring reliability, correctness, and performance of the trading system.

## Test Categories

### Unit Tests (`*_tests.rs`)
- **candle_tests.rs**: Core candle building and CVD calculation logic
- **strategy_tests.rs**: Signal detection and divergence analysis
- **core_tests.rs**: Rounding utilities and state management

### Integration Tests
- **integration_tests.rs**: Cross-module interaction testing
- End-to-end data flow validation
- Realistic trading scenario simulation

### Property-Based Tests
- **property_tests.rs**: Proptest-based edge case testing
- Wide input range validation
- Mathematical property verification

### Persistence Tests
- **persistence_tests.rs**: Database layer testing
- CRUD operations for all data models
- Async operation correctness
- Data integrity and consistency

### Monitoring Tests
- **monitoring_tests.rs**: Health checker and metrics validation
- Alert system functionality
- Background monitoring tasks

### API Integration Tests
- **api_integration_tests.rs**: REST endpoint data structure validation
- Response format verification
- Error handling testing

### End-to-End Tests
- **end_to_end_tests.rs**: Complete system workflow testing
- State persistence and recovery
- Full trading lifecycle validation
- System integrity checks

### Stress Tests
- **stress_tests.rs**: High-load and concurrent operation testing
- Memory pressure simulation
- Database concurrent access validation
- System recovery under load

### Mock Infrastructure
- **mocks.rs**: Test doubles for external dependencies
- Exchange API simulation
- Market data mocking utilities

## Running Tests

### All Tests
```bash
cargo test
```

### Specific Test Categories
```bash
# Unit tests
cargo test --test candle_tests
cargo test --test strategy_tests
cargo test --test core_tests

# Integration tests
cargo test --test integration_tests
cargo test --test persistence_tests
cargo test --test monitoring_tests

# Advanced tests
cargo test --test end_to_end_tests
cargo test --test api_integration_tests
cargo test --test stress_tests

# Property-based tests
cargo test property_tests
```

### Performance Benchmarks
```bash
cargo bench
```

## Test Coverage Areas

### Core Trading Logic ✅
- [x] Candle building with proper OHLC tracking
- [x] CVD calculation (buy_volume - sell_volume)
- [x] Trade direction processing and validation
- [x] Interval boundary detection and candle completion

### Strategy Implementation ✅
- [x] Swing high/low detection algorithms
- [x] CVD divergence analysis logic
- [x] Signal generation validation
- [x] Edge case handling (insufficient history, extreme values)

### Data Integrity ✅
- [x] Property-based testing for mathematical correctness
- [x] Floating-point precision validation
- [x] Memory safety guarantees across all operations
- [x] Boundary condition testing

### Persistence Layer ✅
- [x] SQLite database schema initialization
- [x] Async CRUD operations for all models
- [x] Data consistency and referential integrity
- [x] Concurrent database access validation
- [x] Crash recovery and state restoration

### Monitoring System ✅
- [x] Health checker component validation
- [x] Metrics collection and Prometheus export
- [x] Alert system with multi-channel notifications
- [x] Background monitoring task reliability

### API Layer ✅
- [x] REST endpoint data structure validation
- [x] Response format and serialization testing
- [x] Error handling and status code validation
- [x] Configuration management operations

### System Integration ✅
- [x] End-to-end trading workflow validation
- [x] State persistence and recovery testing
- [x] Concurrent operation handling
- [x] Memory pressure and stress testing
- [x] System recovery and error handling

## Performance Benchmarks

### Current Performance Characteristics
```
candle_creation             time:   [5.234 ns 5.312 ns 5.401 ns]
candle_updates_100          time:   [234.5 ns 238.1 ns 242.3 ns]
candle_builder_1000_trades  time:   [45.67 µs 46.23 µs 46.89 µs]
cvd_calculation_mixed       time:   [89.12 ns 90.45 ns 91.78 ns]
strategy_signal_detection   time:   [156.7 ns 159.2 ns 162.1 ns]
database_save_operation     time:   [45.2 µs 46.8 µs 48.5 µs]
health_check_execution      time:   [12.3 ms 13.1 ms 14.2 ms]
```

### Benchmark Categories
- **Candle Operations**: Creation, updates, and completion performance
- **CVD Calculations**: Volume delta computation speed
- **Strategy Logic**: Signal detection and divergence analysis
- **Database Operations**: Persistence layer performance
- **Monitoring Tasks**: Health checks and metrics collection
- **Memory Usage**: Data structure efficiency under load
- **Concurrent Operations**: Multi-threaded performance validation

## Test Data and Fixtures

### Test Data Generators
```rust
// Generate realistic test trades
let trade = create_test_trade(45000.0, 1.5, true, Some(1640995200000));

// Create market data events
let event = create_test_market_data_event("BTC", 45000.0, 1.5, true, 15.5);

// Generate candle events
let candle_event = create_test_candle_event("BTC", timestamp, open, high, low, close, volume, cvd);

// Create test positions
let position = create_test_position("BTC", 1.5, 45000.0, "LONG");
```

### Test Scenarios
- **Normal Trading**: Standard buy/sell activity with realistic volumes
- **High Volatility**: Extreme price movements and gaps
- **Low Liquidity**: Small volume trades and sparse data
- **Market Events**: Large institutional trades and order book impact
- **Edge Cases**: Zero volume, extreme prices, boundary conditions
- **Error Conditions**: Network failures, database errors, invalid data
- **Concurrent Load**: Multiple operations simultaneously
- **Memory Pressure**: Large datasets and high-frequency operations

## Continuous Integration

### GitHub Actions Setup
```yaml
name: Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
      - run: cargo bench
      - run: cargo test property_tests
```

### Local CI Simulation
```bash
# Run complete test suite
cargo test --all-features

# Run benchmarks with baseline comparison
cargo bench

# Check code formatting
cargo fmt --check

# Lint code
cargo clippy -- -D warnings

# Generate coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Test Organization Best Practices

### Unit Test Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange: Set up test data and preconditions
        let input = setup_test_data();

        // Act: Execute the code under test
        let result = function_under_test(input);

        // Assert: Verify expected outcomes
        assert_eq!(result, expected_value);
        assert!(additional_conditions);
    }
}
```

### Integration Test Pattern
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_end_to_end_flow() {
        // Setup: Initialize test environment
        let (state, repository, health, metrics) = setup_test_environment().await;

        // Execute: Run complete workflow
        let result = execute_complete_workflow().await;

        // Verify: Check all components interacted correctly
        assert!(result.is_success());
        assert!(system_state_is_consistent().await);
    }
}
```

### Property-Based Test Pattern
```rust
proptest! {
    #[test]
    fn property_mathematical_correctness(input_params) {
        // Test mathematical properties across wide input ranges
        prop_assert!(mathematical_invariant_holds(input_params));
        prop_assert!(edge_cases_handled_properly(input_params));
    }
}
```

### Stress Test Pattern
```rust
#[tokio::test]
async fn test_concurrent_operations() {
    const NUM_OPERATIONS: usize = 1000;
    const NUM_TASKS: usize = 10;

    // Setup concurrent tasks
    let mut handles = vec![];
    for task_id in 0..NUM_TASKS {
        let handle = tokio::spawn(async move {
            for i in 0..(NUM_OPERATIONS / NUM_TASKS) {
                execute_operation_under_load(task_id, i).await;
            }
        });
        handles.push(handle);
    }

    // Execute and measure
    let start_time = Instant::now();
    for handle in handles {
        handle.await.unwrap();
    }
    let duration = start_time.elapsed();

    // Verify performance and correctness
    assert!(duration < MAX_DURATION);
    assert!(system_state_valid().await);
}
```

## Test Maintenance

### When to Update Tests
- **New Features**: Add tests for new functionality
- **Bug Fixes**: Add regression tests for fixed issues
- **Performance Changes**: Update benchmarks for significant changes
- **API Changes**: Update integration tests for modified interfaces
- **Data Schema Changes**: Update persistence tests for schema modifications

### Test Quality Metrics
- **Coverage**: Aim for >90% line coverage and >95% branch coverage
- **Performance**: Tests should complete within 5 minutes total
- **Reliability**: Tests should be deterministic and not flaky
- **Maintainability**: Tests should be readable and well-documented
- **Speed**: Unit tests <100ms, integration tests <1s, e2e tests <30s

## Troubleshooting Test Issues

### Common Test Failures
- **Async Timing**: Use proper async test setup and synchronization
- **Database State**: Ensure test isolation with unique data or cleanup
- **Resource Contention**: Avoid shared state between concurrent tests
- **Time-Dependent Tests**: Mock time or use relative comparisons

### Debugging Tests
```bash
# Run single test with output
cargo test test_name -- --nocapture

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test failing_test

# Debug specific module
cargo test --test specific_test -- --nocapture
```

### Performance Debugging
```bash
# Profile slow tests
cargo test --release slow_test
perf record target/release/deps/slow_test-*
perf report

# Benchmark specific functions
cargo bench function_name
```

This comprehensive testing infrastructure ensures the CVD Trader maintains high reliability, performance, and correctness across all system components and usage scenarios.