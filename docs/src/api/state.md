# State Module

The state module provides centralized state management for Clobster.

## Store

The `Store` holds all application state:

```rust
pub struct Store {
    pub app: AppState,
    pub markets: MarketState,
    pub orders: OrderState,
    pub orderbooks: OrderBookState,
    pub portfolio: PortfolioState,
}
```

### Methods

```rust
impl Store {
    /// Create a new store with default state
    pub fn new(action_tx: mpsc::UnboundedSender<Action>) -> Self;
    
    /// Dispatch an async action
    pub fn dispatch(&self, action: Action) -> Result<()>;
    
    /// Reduce (apply) an action to update state
    pub fn reduce(&mut self, action: Action);
}
```

## Actions

All state mutations go through the `Action` enum:

```rust
pub enum Action {
    // Navigation
    SetView(View),
    SetInputMode(InputMode),
    SetAppMode(AppMode),

    // Market actions
    LoadMarkets,
    MarketsLoaded(Vec<Market>),
    SelectMarket(usize),
    SearchMarkets(String),
    FilterMarkets(MarketStatus),
    ClearMarketFilter,

    // Order actions
    LoadOrders,
    OrdersLoaded(Vec<Order>),
    SelectOrder(usize),
    PlaceOrder(OrderRequest),
    CancelOrder(String),
    OrderPlaced(Order),
    OrderCancelled(String),

    // Portfolio actions
    LoadPortfolio,
    PortfolioLoaded(PortfolioState),
    LoadPositions,
    PositionsLoaded(Vec<Position>),

    // Order book actions
    LoadOrderBook(String),           // token_id
    OrderBookLoaded(OrderBookDepth),
    SelectOrderBook(String),         // token_id
    ClearOrderBook(String),          // token_id
    ClearAllOrderBooks,
    SetOrderBookDepth(usize),        // display depth
    RefreshOrderBook(String),        // token_id

    // UI actions
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoToTop,
    GoToBottom,
    ToggleHelp,
    ShowNotification(Notification),
    DismissNotification,

    // Data refresh
    RefreshAll,
    RefreshMarkets,
    RefreshOrders,
    RefreshPortfolio,

    // Error handling
    SetError(String),
    ClearError,

    // Connection status
    SetConnected(bool),
    SetLoading(bool),

    // Quit
    Quit,
}
```

## AppState

Application-level state:

```rust
pub struct AppState {
    /// Whether the application is running
    pub running: bool,
    /// Current view/screen
    pub current_view: View,
    /// Input mode (Normal/Insert/Command)
    pub input_mode: InputMode,
    /// Application mode (Trading/Backtesting/etc)
    pub mode: AppMode,
    /// Current error message
    pub error: Option<String>,
    /// Current notification
    pub notification: Option<Notification>,
    /// Loading state
    pub loading: bool,
    /// Connection status
    pub connected: bool,
}
```

### View

```rust
pub enum View {
    Markets,
    MarketDetail,
    Orders,
    Portfolio,
    Strategies,
    Settings,
    Help,
}
```

### InputMode

```rust
pub enum InputMode {
    Normal,
    Insert,
    Command,
}
```

### AppMode

```rust
pub enum AppMode {
    Live,
    Paper,
    Backtest,
}
```

## MarketState

Market data state:

```rust
pub struct MarketState {
    /// All loaded markets
    pub items: Vec<Market>,
    /// Currently selected market index
    pub selected: Option<usize>,
    /// Search query filter
    pub search_query: String,
    /// Status filter
    pub filter: Option<MarketStatus>,
    /// Loading indicator
    pub loading: bool,
    /// Scroll position
    pub scroll: usize,
}
```

## OrderState

Order tracking state:

```rust
pub struct OrderState {
    /// All orders
    pub items: Vec<Order>,
    /// Currently selected order index
    pub selected: Option<usize>,
    /// Pending order IDs (being processed)
    pub pending: Vec<String>,
    /// Scroll position
    pub scroll: usize,
}
```

## PortfolioState

Portfolio and position state:

```rust
pub struct PortfolioState {
    /// Account balance
    pub balance: Balance,
    /// Current positions
    pub positions: Vec<Position>,
}

pub struct Balance {
    pub total: Decimal,
    pub available: Decimal,
    pub locked: Decimal,
}

pub struct Position {
    pub market_id: String,
    pub token_id: String,
    pub outcome_name: String,
    pub size: Decimal,
    pub avg_price: Decimal,
    pub current_price: Decimal,
    pub unrealized_pnl: Decimal,
}
```

## Usage Example

```rust
use clobster::state::{Store, Action, View};

// Create store
let (action_tx, mut action_rx) = mpsc::unbounded_channel();
let mut store = Store::new(action_tx);

// Dispatch async action
store.dispatch(Action::RefreshMarkets)?;

// Synchronous state update
store.reduce(Action::SetView(View::Orders));

// Read state
if store.app.loading {
    println!("Loading...");
}

for market in &store.markets.items {
    println!("{}: {}", market.question, market.outcomes[0].price);
}
```
