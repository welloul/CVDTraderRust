# Module: Main

## Responsibility

The Main module serves as the application entry point and orchestrates the initialization and execution of all trading system components. It handles environment configuration, dependency injection, and the main event loop that keeps the system running.

### Why
- **System Bootstrap**: Coordinates startup of all subsystems
- **Configuration Management**: Environment variable processing and defaults
- **Lifecycle Management**: Clean startup and graceful operation
- **Integration Point**: Connects all modules into cohesive system

### What
- main.rs: Application entry point with system initialization
- lib.rs: Library declarations and module exports

## Key Logic & Functions

### main.rs
**Execution Flow:**
1. **Logging Initialization**: Sets up structured logging
2. **Environment Loading**: Loads .env file with dotenvy
3. **Configuration Loading**: Loads TOML config file or uses defaults
4. **Configuration Parsing**: Reads execution mode, strategy, latency limits
5. **State Initialization**: Creates global state with defaults
6. **Exchange Setup**: Initializes Hyperliquid clients (Account/Exchange/Info)
7. **Metadata Fetching**: Retrieves exchange metadata for rounding
8. **State Sync**: Syncs initial wallet and position data
9. **Component Creation**: Instantiates risk manager, execution gateway, strategy with config
10. **Market Data Setup**: Creates WebSocket handlers for configured coins
11. **Background Tasks**: Spawns state streamer and periodic sync tasks
12. **Event Loop**: Waits for market data tasks to complete

**Configuration Parameters:**
- `EXECUTION_MODE`: dryrun/testnet/live
- `ACTIVE_STRATEGY`: Strategy selection (currently "delta_poc")
- `MAX_LATENCY_MS`: Latency threshold (default 5000ms)
- `TARGET_COINS`: Comma-separated coin list (default BTC,ETH,SOL)
- `HYPERLIQUID_SECRET_KEY`: Trading credentials
- `HYPERLIQUID_WALLET_ADDRESS`: Wallet address
- `HYPERLIQUID_MAIN_WALLET_ADDRESS`: Main wallet for balance tracking

**Key Functions:**
- `main() -> Result<(), Box<dyn std::error::Error>>`: Application entry point
- `start_bot_loop() -> Result<(), Box<dyn std::error::Error>>`: Core initialization logic

### lib.rs
**Module Exports:**
```rust
pub mod core;
pub mod market_data;
pub mod execution;
pub mod risk;
pub mod strategy;
pub mod api;
pub mod hyperliquid;
```

**Purpose:** Library interface for external usage (currently unused)

## Hurdles

### Bugs
- **Exchange Initialization**: Exchange client creation fails silently with placeholders
- **State Sync**: sync_state() and sync_main_wallet_balance() are unimplemented
- **Error Propagation**: Some initialization errors may not prevent startup
- **Configuration Validation**: Limited validation of environment variables

### Race Conditions
- **Background Tasks**: State sync task runs concurrently with market data processing
- **Shared State**: Multiple tasks access global state simultaneously
- **Task Coordination**: No synchronization between market data handlers and main loop

### Technical Debt
- **Hardcoded Defaults**: Many configuration values are hardcoded
- **No Graceful Shutdown**: No signal handling for clean shutdown
- **Limited Error Recovery**: Failures in one component may not affect others appropriately
- **Configuration Management**: No configuration file support, env-only
- **Dependency Injection**: Tight coupling between component creation
- **Logging Context**: Limited structured logging in initialization

## Future Roadmap

### Immediate (Next Sprint)
- **Complete State Sync**: Implement real Hyperliquid API integration for state loading
- **Exchange Authentication**: Proper wallet and signing setup
- **Configuration Validation**: Runtime validation with helpful error messages
- **Graceful Shutdown**: Signal handling and clean component teardown

### Short Term (1-2 weeks)
- **Configuration Files**: YAML/TOML configuration file support
- **Health Checks**: Startup health validation for all components
- **Dependency Injection**: Proper DI container for better testability
- **Logging Enhancement**: Structured logging throughout initialization

### Medium Term (1-2 months)
- **Multi-Environment**: Environment-specific configurations
- **Hot Reload**: Runtime configuration updates without restart
- **Monitoring Integration**: Startup metrics and health monitoring
- **Containerization**: Proper Docker support with health checks

### Long Term (3-6 months)
- **Orchestration**: Kubernetes deployment with service mesh
- **Auto-scaling**: Dynamic component scaling based on load
- **Multi-region**: Cross-region deployment and synchronization
- **Disaster Recovery**: Automatic failover and state recovery