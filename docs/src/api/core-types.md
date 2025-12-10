# Core Types

This section documents the fundamental types used throughout Clobster.

## Error Handling

### Error Enum

```rust
pub enum Error {
    Io(std::io::Error),
    Terminal(String),
    Api(polymarket_rs::Error),
    Config(String),
    Serialization(serde_json::Error),
    Channel(String),
    Auth(String),
    Wallet(String),
    InvalidInput(String),
    Network(String),
    RateLimited(u64),
    Application(String),
}
```

### Constructor Helpers

```rust
// Create errors with helper methods
Error::terminal("Failed to initialize terminal")
Error::config("Invalid configuration value")
Error::channel("Action dispatch failed")
Error::auth("Invalid credentials")
Error::wallet("Failed to sign transaction")
Error::invalid_input("Invalid order size")
Error::network("Connection timeout")
Error::application("Unexpected error")
```

### Result Type

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Financial Types

### Decimal

All financial values use `rust_decimal::Decimal`:

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

let price = dec!(0.45);
let size = dec!(100.0);
let total = price * size;  // dec!(45.0)
```

**Never use `f64` for money!** Floating point leads to rounding errors.

## Order Types

### OrderSide

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}
```

### OrderType

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    Market,
}
```

### OrderStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}
```

### OrderRequest

```rust
pub struct OrderRequest {
    pub market_id: String,
    pub token_id: String,
    pub side: OrderSide,
    pub price: Option<Decimal>,  // None for market orders
    pub size: Decimal,
    pub order_type: OrderType,
}
```

## Market Types

### MarketStatus

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Active,
    Paused,
    Closed,
    Resolved,
}
```

### Market

```rust
pub struct Market {
    pub id: String,
    pub question: String,
    pub description: String,
    pub outcomes: Vec<Outcome>,
    pub status: MarketStatus,
    pub volume: Decimal,
    pub liquidity: Decimal,
    pub end_date: Option<DateTime<Utc>>,
}
```

### Outcome

```rust
pub struct Outcome {
    pub token_id: String,
    pub name: String,
    pub price: Decimal,
    pub bid: Decimal,
    pub ask: Decimal,
}
```

## Notification Types

### NotificationLevel

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}
```

### Notification

```rust
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub duration_secs: u64,
}

// Constructors
Notification::info("Market data refreshed")
Notification::success("Order placed successfully")
Notification::warning("Rate limit approaching")
Notification::error("Failed to connect to API")
```
