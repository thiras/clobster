# CLOBster

[![Crates.io](https://img.shields.io/crates/v/clobster.svg)](https://crates.io/crates/clobster)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

**CLOBster** is a terminal user interface (TUI) framework for [Polymarket](https://polymarket.com) prediction markets. Built with [Rust](https://rust-lang.org), [ratatui](https://ratatui.rs), and [polymarket-rs](https://crates.io/crates/polymarket-rs).

<!-- ![Clobster Demo](docs/demo.gif) -->

## Features

- 🖥️ **Modern TUI** — Beautiful terminal interface with vim-style navigation
- 📊 **Real-time Data** — Live market updates via WebSocket
- 🤖 **Programmable Strategies** — Build and deploy custom trading strategies
- ⚡ **High Performance** — Built in Rust for speed and reliability
- 🛡️ **Risk Management** — Built-in guards and position limits
- ⚙️ **Configurable** — TOML-based configuration with sensible defaults

## Installation

### Requirements

- Rust 1.85+ (2024 edition)
- A Polymarket account with API credentials

### From Crates.io

```bash
cargo install clobster
```

### From Source

```bash
git clone https://github.com/thiras/clobster.git
cd clobster
cargo build --release
```

The binary will be at `target/release/clobster`.

## Quick Start

1. **Create a configuration file** at `~/.config/clobster/config.toml`:

```toml
[api]
base_url = "https://clob.polymarket.com"
ws_url = "wss://ws-subscriptions-clob.polymarket.com/ws"
timeout_secs = 30

[ui]
tick_rate_ms = 100
mouse_support = true
unicode_symbols = true

[keybindings]
up = "k"
down = "j"
left = "h"
right = "l"
quit = "q"
help = "?"
refresh = "r"
```

2. **Run CLOBster**:

```bash
clobster
```

3. **Enable debug logging** (optional):

```bash
RUST_LOG=clobster=debug clobster
```

## Architecture

CLOBster follows a **unidirectional data flow** pattern (Redux/Elm-inspired):

```
Events → Actions → Store (reduce) → UI renders from Store
```

### Core Modules

| Module | Purpose |
|--------|---------|
| `app` | Event loop, terminal setup, async action handling |
| `state` | Centralized state with `Store`, `Action` enum, and domain states |
| `ui` | Ratatui rendering, layout, widgets |
| `events` | Input handling, key bindings → Action dispatch |
| `api` | Polymarket API wrapper via `polymarket-rs` |
| `strategy` | Programmable trading strategies with signals and risk management |

### Data Flow

1. User presses key → `EventHandler::handle_key()` returns `Action`
2. `App::handle_action()` processes async actions (API calls) or delegates to `Store::reduce()`
3. `Store::reduce()` updates state immutably
4. `Ui::render()` reads from `Store` and draws widgets

## Trading Strategies

CLOBster provides a powerful framework for building automated trading strategies.

### Quick Example

```rust
use clobster::strategy::{Strategy, StrategyContext, Signal};
use rust_decimal_macros::dec;

struct MyStrategy {
    threshold: Decimal,
}

impl Strategy for MyStrategy {
    fn name(&self) -> &str { "my_strategy" }

    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let mut signals = vec![];
        for market in ctx.markets() {
            if let Some(outcome) = market.outcomes.first() {
                if outcome.price < self.threshold {
                    signals.push(Signal::buy(
                        market.id.clone(),
                        outcome.token_id.clone(),
                        dec!(0.10),
                    ));
                }
            }
        }
        signals
    }
}
```

### Built-in Strategies

- **Momentum** — Trend-following based on price movement
- **Mean Reversion** — Capitalize on price deviations from historical averages
- **Spread** — Market-making by capturing bid-ask spreads

### Risk Management

All strategies pass through a risk guard before execution:

```rust
pub struct RiskConfig {
    pub max_position_size: Decimal,
    pub max_order_size: Decimal,
    pub max_daily_loss: Decimal,
    pub max_open_orders: usize,
}
```

## Development

### Build Commands

```bash
cargo build                          # Debug build
cargo build --release                # Optimized release build
cargo test                           # Run all tests
cargo clippy                         # Run lints
cargo doc --open                     # Generate and view documentation
```

### Run with Debug Logging

```bash
RUST_LOG=clobster=debug cargo run
```

### Project Structure

```
src/
├── app.rs              # Application lifecycle
├── lib.rs              # Public API exports
├── main.rs             # Entry point
├── error.rs            # Error types
├── api/                # Polymarket API client
├── config/             # Configuration management
├── events/             # Input handling
├── state/              # State management (Store, Actions)
├── strategy/           # Trading strategy framework
│   └── strategies/     # Built-in strategy implementations
└── ui/                 # Terminal UI rendering
    └── widgets/        # Reusable UI components
```

## Documentation

Full documentation is available at [thiras.github.io/clobster](https://thiras.github.io/clobster) or build locally:

```bash
cd docs
mdbook serve
```

## Contributing

Contributions are welcome! Please follow [Conventional Commits](https://www.conventionalcommits.org) for commit messages:

- `feat:` new features
- `fix:` bug fixes
- `refactor:` code restructuring
- `test:` test additions
- `docs:` documentation

## License

CLOBster is licensed under the [MIT License](LICENSE).

## Acknowledgments

- [ratatui](https://ratatui.rs) — Terminal UI framework
- [polymarket-rs](https://crates.io/crates/polymarket-rs) — Polymarket API client
- [Polymarket](https://polymarket.com) — Prediction market platform
