## Core Components

### 1. Core Module
- **GlobalState**: Centralized state management for positions, orders, trades, and unified configuration. Distributed via Dependency Injection (Arc<Mutex<State>>).
- **Logger**: Macro-based logging interface with structured output.
- **RoundingUtil**: Asset-specific precision handling for size and price formatting.

### 2. Market Data Module
- **MarketDataHandler**: WebSocket connection management with trade processing and event dispatch.
- **MarketDataEvent**: Typed abstraction for market updates, centralizing JSON parsing.
- **CandleBuilder**: OHLCV candle construction with CVD (cumulative volume delta) calculations.
- **Real-time Processing**: Sub-millisecond latency market data ingestion.

### 3. Strategy Module
- **StrategyModule**: CVD-based signal generation with divergence detection
- **Position Management**: Automated entry/exit with stop loss and take profit
- **P&L Tracking**: Real-time profit/loss calculation and reporting

### 4. Execution Module
- **ExecutionGateway**: Order placement, cancellation, and amendment via Hyperliquid API
- **OrderTTLTracker**: Time-based order lifecycle management (framework ready)
- **Post-only Orders**: Market maker style execution to avoid taker fees

### 5. Risk Module
- **RiskManager**: Circuit breaker system with latency monitoring and position limits
- **Pre-trade Validation**: Size, price, and risk checks before order submission
- **Automatic Protection**: Trading suspension on critical failures

### 6. API Module (Framework)
- **Server**: Axum-based HTTP server with WebSocket streaming capabilities
- **Health Endpoints**: System monitoring and status reporting
- **REST API**: Framework ready for external integrations

### 7. Hyperliquid Module
- **Exchange**: Trading operations with authentication framework
- **Info**: Market data and account information retrieval
- **WebSocket Integration**: Real-time trade and order book data

### 8. Testing Infrastructure
- **Scenario Tests**: Reproducible signal detection validation using hand-crafted candle sequences.
- **Unit Tests**: 20+ comprehensive tests for core functionality.
- **Integration Tests**: End-to-end data flow validation.
- **Property-Based Tests**: Proptest validation for mathematical correctness.
- **Performance Benchmarks**: Criterion-based speed and efficiency testing.
- **Mock Framework**: External dependency simulation for reliable testing.