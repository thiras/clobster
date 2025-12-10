# State Management

Clobster uses a centralized state management pattern inspired by Redux and Elm.

## The Store

All application state lives in a single `Store` struct:

```rust
pub struct Store {
    pub app: AppState,
    pub markets: MarketState,
    pub orders: OrderState,
    pub portfolio: PortfolioState,
    action_tx: mpsc::UnboundedSender<Action>,
}
```

## Actions

State mutations happen exclusively through `Action` variants:

```rust
pub enum Action {
    // Navigation
    SetView(View),
    SetInputMode(InputMode),
    
    // Market actions
    LoadMarkets,
    MarketsLoaded(Vec<Market>),
    SelectMarket(usize),
    SearchMarkets(String),
    
    // Order actions
    PlaceOrder(OrderRequest),
    OrderPlaced(Order),
    CancelOrder(String),
    
    // Portfolio actions
    LoadPortfolio,
    PortfolioLoaded(PortfolioState),
    
    // ... more actions
}
```

## The Reduce Function

The `Store::reduce()` method handles synchronous state updates:

```rust
impl Store {
    pub fn reduce(&mut self, action: Action) {
        match action {
            Action::SetView(view) => {
                self.app.current_view = view;
            }
            Action::MarketsLoaded(markets) => {
                self.markets.items = markets;
                self.markets.loading = false;
            }
            Action::SelectMarket(idx) => {
                self.markets.selected = Some(idx);
            }
            // ... handle other actions
        }
    }
}
```

## Dispatching Actions

### Synchronous Dispatch

For immediate state updates:

```rust
store.reduce(Action::SetView(View::Markets));
```

### Async Dispatch

For operations that need async processing:

```rust
// Send action through channel
store.dispatch(Action::RefreshMarkets)?;

// App handles async work and dispatches result
async fn handle_action(&mut self, action: Action) {
    match action {
        Action::RefreshMarkets => {
            let markets = self.api.fetch_markets().await?;
            self.store.reduce(Action::MarketsLoaded(markets));
        }
        // ...
    }
}
```

## State Domains

### AppState

Application-level state:

```rust
pub struct AppState {
    pub running: bool,
    pub current_view: View,
    pub input_mode: InputMode,
    pub mode: AppMode,
    pub error: Option<String>,
    pub notification: Option<Notification>,
}
```

### MarketState

Market data and selection:

```rust
pub struct MarketState {
    pub items: Vec<Market>,
    pub selected: Option<usize>,
    pub search_query: String,
    pub filter: Option<MarketStatus>,
    pub loading: bool,
}
```

### OrderState

Order tracking:

```rust
pub struct OrderState {
    pub items: Vec<Order>,
    pub selected: Option<usize>,
    pub pending: Vec<String>,
}
```

### PortfolioState

Portfolio and positions:

```rust
pub struct PortfolioState {
    pub balance: Balance,
    pub positions: Vec<Position>,
}
```

## Benefits of This Pattern

1. **Predictability** - All state changes go through one path
2. **Debuggability** - Easy to log and trace actions
3. **Testability** - Reduce function is pure and testable
4. **Time Travel** - Can implement undo/redo by storing action history
