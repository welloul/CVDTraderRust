# Module: Hyperliquid

## Responsibility

The Hyperliquid module provides integration with the Hyperliquid perpetual futures exchange, handling API communication, authentication, and data retrieval. It abstracts the exchange's REST API into Rust-native interfaces for trading operations and market data access.

### Why
- **Exchange Integration**: Direct connection to Hyperliquid's trading infrastructure
- **API Abstraction**: Clean Rust interfaces for complex exchange APIs
- **Authentication**: Secure credential management for trading operations
- **Data Access**: Unified access to market data, account state, and order information

### What
- Exchange: Trading operations (orders, cancellations)
- Info: Read-only market data and account information
- Account: Credential management and authentication
- Constants: Network endpoints for mainnet/testnet

## Key Logic & Functions

### Exchange
**Core Data Structures:**
```rust
pub struct Exchange {
    client: Client,
    account: Account,
    base_url: String,
}
```

**Key Methods:**
- `new(account: Account, base_url: &str) -> Self`: Initializes trading client
- `place_order(params: Value) -> Result<Value, Box<dyn std::error::Error>>`: Submits orders to exchange
- `cancel_order(params: Value) -> Result<Value, Box<dyn std::error::Error>>`: Cancels existing orders

**API Endpoints:**
- `POST /{base_url}/exchange`: Order placement and cancellation

### Info
**Core Data Structures:**
```rust
pub struct Info {
    client: Client,
    base_url: String,
}
```

**Key Methods:**
- `new(base_url: &str) -> Option<Self>`: Creates info client (always succeeds)
- `meta() -> Result<Value, Box<dyn std::error::Error>>`: Retrieves exchange metadata (assets, fees, limits)
- `user_state(address: &str) -> Result<Value, Box<dyn std::error::Error>>`: Gets account positions and balances
- `open_orders(address: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>>`: Retrieves active orders
- `spot_user_state(address: &str) -> Result<Value, Box<dyn std::error::Error>>`: Gets spot account state

**API Endpoints:**
- `POST /{base_url}/info`: All read operations

### Account
**Core Data Structures:**
```rust
pub struct Account {
    pub address: String,
    pub secret_key: String,
}
```

**Key Methods:**
- `from_key(secret_key: &str) -> Self`: Creates account from secret key (placeholder implementation)

**Authentication:**
- Currently placeholder - no actual key derivation or signing

### Constants
**Network Endpoints:**
```rust
pub const MAINNET_API_URL: &str = "https://api.hyperliquid.xyz";
pub const TESTNET_API_URL: &str = "https://api.hyperliquid-testnet.xyz";
```

## Hurdles

### Bugs
- **Account Implementation**: from_key() returns placeholder values, no real key handling
- **Authentication**: No signature generation for API requests
- **Error Handling**: Generic error types without specific exchange error codes
- **Rate Limiting**: No rate limit handling or backoff logic

### Race Conditions
- **Shared Client**: Reqwest client shared across instances without synchronization
- **Concurrent Requests**: Multiple API calls may exceed rate limits

### Technical Debt
- **Incomplete Auth**: No cryptographic signing or wallet integration
- **No Signing**: API requests lack required signatures for authenticated endpoints
- **Limited Coverage**: Only basic endpoints implemented
- **No Websockets**: No WebSocket client for real-time data
- **No Testing**: No integration tests with actual API
- **Error Types**: Generic error handling without exchange-specific errors

## Future Roadmap

### Immediate (Next Sprint)
- **Cryptographic Signing**: Implement proper ECDSA signing for API authentication
- **Wallet Integration**: Real wallet address derivation from private keys
- **Error Handling**: Specific error types for different API failures
- **Rate Limiting**: Request throttling and exponential backoff

### Short Term (1-2 weeks)
- **WebSocket Client**: Real-time order book and trade data
- **Advanced Orders**: Stop orders, OCO, bracket orders support
- **Position Management**: Bulk position closing and adjustment
- **Historical Data**: Trade history and funding rate retrieval

### Medium Term (1-2 months)
- **Multi-signature**: Hardware wallet and multi-sig support
- **Order Management**: Advanced order types and conditional orders
- **Risk Integration**: Pre-trade risk checks with exchange position limits
- **Performance**: Connection pooling and request optimization

### Long Term (3-6 months)
- **Exchange Features**: New Hyperliquid features as they become available
- **Multi-exchange**: Unified interface for multiple DEXes
- **DeFi Integration**: Cross-protocol trading and liquidity management
- **Compliance**: Regulatory reporting and audit trail features