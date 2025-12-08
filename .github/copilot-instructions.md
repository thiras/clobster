# Clobster AI Coding Instructions

A production-grade TUI framework for Polymarket prediction markets built with Rust, ratatui, and polymarket-rs.

## Architecture Overview

Clobster follows a **unidirectional data flow** pattern (Redux/Elm-inspired):

```
Events → Actions → Store (reduce) → UI renders from Store
```

### Core Components

| Module | Purpose | Key Files |
|--------|---------|-----------|
| `app` | Event loop, terminal setup, async action handling | `src/app.rs` |
| `state` | Centralized state with `Store`, `Action` enum, and domain states | `src/state/mod.rs` |
| `ui` | Ratatui rendering, layout, widgets | `src/ui/mod.rs` |
| `events` | Input handling, key bindings → Action dispatch | `src/events/handler.rs` |
| `api` | Polymarket API wrapper via `polymarket-rs` | `src/api/client.rs` |
| `strategy` | Programmable trading strategies with signals and risk management | `src/strategy/` |

### Data Flow Example
1. User presses key → `EventHandler::handle_key()` returns `Action`
2. `App::handle_action()` processes async actions (API calls) or delegates to `Store::reduce()`
3. `Store::reduce()` updates state immutably
4. `Ui::render()` reads from `Store` and draws widgets

## Development Commands

```bash
cargo build                    # Debug build
cargo build --release          # Optimized release build
cargo test                     # Run all tests
cargo doc --open               # Generate and view documentation
cargo clippy                   # Run lints
RUST_LOG=clobster=debug cargo run  # Run with debug logging
```

## Key Patterns

### State Management
- All state lives in `Store` (composed of `AppState`, `MarketState`, `OrderState`, `OrderBookState`, `PortfolioState`)
- Mutations only via `Action` enum and `Store::reduce()`
- Async operations dispatch actions through `mpsc::UnboundedSender<Action>`

```rust
// Dispatch async action
self.store.dispatch(Action::RefreshMarkets)?;

// Synchronous state update
self.store.reduce(Action::MarketsLoaded(markets));
```

### Strategy Implementation
Implement the `Strategy` trait in `src/strategy/traits.rs`:

```rust
#[async_trait]
impl Strategy for MyStrategy {
    fn name(&self) -> &str { "my_strategy" }
    
    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        // Access ctx.markets(), ctx.positions(), ctx.available_balance
        // Return Signal::buy() or Signal::sell() with builder pattern
    }
}
```

Built-in strategies in `src/strategy/strategies/`: `MomentumStrategy`, `MeanReversionStrategy`, `SpreadStrategy`

### Error Handling
Use the custom `Error` enum from `src/error.rs` with constructor helpers:

```rust
use crate::error::{Error, Result};

Error::config("message")       // Configuration errors
Error::invalid_input("msg")    // Validation errors
Error::Api(polymarket_err)     // Wrap API errors (via From)
```

### API Integration
- `ApiClient` wraps `polymarket-rs` crate
- `DataConverter` in `src/api/converter.rs` transforms API types → internal state types
- Rate limiting built into client via `RateLimiter`
- `fetch_orderbook()` / `fetch_orderbooks()` for market depth data

## Conventions

### Code Style
- Use `rust_decimal::Decimal` for all financial values (never `f64`)
- Builder pattern for complex structs (see `Signal`, `MomentumStrategy`)
- Async functions use `async_trait` macro for trait methods

### File Organization
- One domain per state file: `market_state.rs`, `order_state.rs`, `orderbook_state.rs`, `portfolio_state.rs`
- UI widgets in `src/ui/widgets/` as separate modules
- Strategy implementations in `src/strategy/strategies/`

### Commit Messages
Follow [Conventional Commits](https://www.conventionalcommits.org) - see `cliff.toml` for categories:
- `feat:` new features
- `fix:` bug fixes
- `refactor:` code restructuring
- `test:` test additions
- `docs:` documentation

## Testing

```bash
cargo test                           # All tests
cargo test --lib                     # Library tests only
cargo test strategy::               # Strategy module tests
```

Use `mockall` for mocking (see `dev-dependencies` in Cargo.toml).

## Configuration

Config loads from `~/.config/clobster/config.toml` with TOML format. See `src/config/settings.rs` for schema:
- `api`: Base URL, timeouts, rate limits
- `ui`: Tick rate, mouse support, Unicode settings
- `keybindings`: Vim-style navigation customization
