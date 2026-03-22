## 🔒 Production Features

### State Persistence
- **SQLite Database**: Crash-resistant state storage with automatic recovery
- **Persistent Configuration**: Settings survive restarts and updates
- **Trade History**: Complete audit trail of all trading activity
- **Position Recovery**: Automatic restoration of active positions on startup

### Production Monitoring
- **Health Checks**: Real-time system component monitoring
- **Metrics Collection**: Performance tracking with Prometheus-compatible exports
- **Alert System**: Configurable notifications for system issues
- **API Endpoints**: Comprehensive monitoring and control interface

```bash
# Health check
curl http://localhost:8000/health

# System metrics
curl http://localhost:8000/metrics

# Trading control
curl -X POST http://localhost:8000/control/start
curl -X POST http://localhost:8000/control/stop
```

## 🗂️ Project Structure

```
cvd_trader_rust/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Module declarations
│   ├── core/                   # State management, logging, utilities
│   ├── market_data/            # WebSocket feeds, candle building
│   ├── strategy/               # CVD analysis, signal generation
│   ├── execution/              # Order management, Hyperliquid API
│   ├── risk/                   # Circuit breakers, position limits
│   ├── api/                    # REST API, monitoring endpoints
│   ├── hyperliquid/            # Exchange integration
│   ├── persistence/            # Database layer, state persistence
│   └── monitoring/             # Health checks, metrics, alerting
├── tests/                      # Comprehensive test suite
├── benches/                    # Performance benchmarks
├── docs/                       # Complete documentation
└── Cargo.toml                  # Dependencies and configuration
```

## 📊 Architecture

### Data Flow
```
Market Data → CVD Analysis → Signal Generation → Risk Check → Order Execution
    ↓            ↓            ↓               ↓           ↓
 Persistence ← Monitoring ← Alerting ← Health Checks ← Metrics
```

### Key Components
- **Persistence Layer**: SQLite-based state management with async operations
- **Monitoring System**: Health checks, metrics collection, and alerting
- **API Server**: REST endpoints for monitoring, control, and data access
- **Background Tasks**: Health monitoring, metrics collection, state sync
- **Alert Handlers**: Console, email, and webhook notifications

## 🔧 Configuration

### Environment Variables
```bash
# Trading
EXECUTION_MODE=dryrun|testnet|live
TARGET_COINS=BTC,ETH,SOL
ACTIVE_STRATEGY=cvd_exhaustion

# Persistence
DATABASE_PATH=cvd_trader.db

# Monitoring
ALERT_EMAIL_RECIPIENTS=user@example.com,admin@example.com
ALERT_WEBHOOK_URL=https://hooks.example.com/webhook

# Hyperliquid
HYPERLIQUID_SECRET_KEY=your_key
HYPERLIQUID_WALLET_ADDRESS=your_address
```

### Database Schema
The system automatically creates and manages:
- **Configuration**: Key-value settings storage
- **Positions**: Active position tracking
- **Orders**: Active order management
- **Trades**: Complete trade history
- **Metrics**: Performance data collection
- **Health**: System health monitoring

## 🚀 Advanced Usage

### Monitoring Dashboard
Access real-time system status:
- **Health Status**: Component-level system health
- **Performance Metrics**: Latency, throughput, error rates
- **Active Positions**: Current holdings and P&L
- **Trade History**: Complete audit trail
- **System Configuration**: Runtime settings management

### Alert Configuration
Configure multi-channel alerting:
```bash
# Email alerts for critical issues
ALERT_EMAIL_RECIPIENTS=trader@example.com,admin@example.com

# Webhook for integration with monitoring systems
ALERT_WEBHOOK_URL=https://api.monitoring.com/alerts
```

### Database Management
```bash
# View database statistics
curl http://localhost:8000/status

# Backup database
cp cvd_trader.db cvd_trader_backup.db

# Query trade history
sqlite3 cvd_trader.db "SELECT * FROM closed_trades ORDER BY closed_at DESC LIMIT 10;"