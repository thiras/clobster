# Clobster AI Coding Instructions

TUI framework for Polymarket prediction markets. Rust 2024 edition with ratatui and polymarket-rs.

## Architecture (Redux/Elm Pattern)

```
Events → Actions → Store::reduce() → UI::render() reads Store
```

**Data Flow**: `EventHandler::handle_key()` → `Action` → `App::handle_action()` (async) or `Store::reduce()` (sync) → `Ui::render()`

### Module Map

| Module | Responsibility | Entry Point |
|--------|---------------|-------------|
| `state` | Centralized store, `Action` enum, domain states | `src/state/mod.rs` |
| `app` | Event loop, terminal setup, async action dispatch | `src/app.rs` |
| `events` | Input → Action mapping with `InputMode` (Normal/Insert/Command/Search) | `src/events/handler.rs` |
| `ui` | Ratatui widgets, view-based rendering | `src/ui/mod.rs` |
| `api` | Polymarket wrapper, `DataConverter` for type transforms | `src/api/client.rs` |
| `strategy` | Trading logic: `Strategy` trait, `Signal`, `RiskGuard` | `src/strategy/traits.rs` |

## Commands

```bash
cargo build --release                    # Production build
cargo test                               # All tests
cargo test strategy::                    # Module-specific
RUST_LOG=clobster=debug cargo run        # Debug logging
cargo clippy -- -D warnings              # Strict lints
```

## Critical Patterns

### State: Actions + Reduce

All mutations through `Action` enum. Async actions use `dispatch()`, sync use `reduce()`:

```rust
// Async (API calls) - goes through channel
self.store.dispatch(Action::RefreshMarkets)?;

// Sync (immediate) - direct state update  
self.store.reduce(Action::MarketsLoaded(markets));
```

**Store composition**: `AppState`, `MarketState`, `OrderState`, `OrderBookState`, `PortfolioState`

### Strategies: Trait + Signal Builder

Implement `Strategy` trait (`src/strategy/traits.rs`). Use builder pattern for `Signal`:

```rust
fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
    Signal::buy(market_id, token_id, dec!(10))
        .with_price(dec!(0.45))
        .with_stop_loss(dec!(0.35))
        .with_reason("momentum breakout")
}
```

**Context methods**: `ctx.markets()`, `ctx.positions()`, `ctx.ema()`, `ctx.total_exposure()`

**Risk validation**: All signals pass through `RiskGuard::check_signal()` before execution

### Error Handling

Use `Error` constructors from `src/error.rs`:

```rust
Error::config("msg")        // Config issues
Error::invalid_input("msg") // Validation
Error::Api(e)               // From polymarket_rs::Error
Error::channel("msg")       // mpsc failures
```

### Financial Values

**Always `Decimal`** (never `f64`). Use `rust_decimal_macros::dec!()`:

```rust
use rust_decimal_macros::dec;
let price = dec!(0.55);  // NOT 0.55_f64
```

## File Conventions

- **State domains**: One file per domain in `src/state/` (e.g., `market_state.rs`)
- **Strategies**: `src/strategy/strategies/{name}.rs` with builder pattern
- **UI widgets**: `src/ui/widgets/{name}.rs`
- **Commits**: Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`)

## Config Schema

`~/.config/clobster/config.toml` - see `src/config/settings.rs`:

```toml
[api]
base_url = "https://clob.polymarket.com"
timeout_secs = 30
rate_limit = 10

[ui]
tick_rate_ms = 250
unicode_symbols = true

[keybindings]  # vim-style
up = "k"
down = "j"
```

## Testing

```bash
cargo test --lib                    # Library only
cargo test strategy::momentum       # Specific module
```

Use `mockall` for API mocking. Tests live alongside code or in `tests/`.
