# Quick Start

This guide will get you trading on Polymarket in 5 minutes.

## 1. Set Up Configuration

Create the configuration directory and file:

```bash
mkdir -p ~/.config/clobster
```

Create `~/.config/clobster/config.toml`:

```toml
[api]
base_url = "https://clob.polymarket.com"
ws_url = "wss://ws-subscriptions-clob.polymarket.com/ws"
timeout_secs = 30
rate_limit = 10

[ui]
tick_rate_ms = 100
mouse_support = true
unicode_symbols = true

[keybindings]
quit = "q"
help = "?"
refresh = "r"
```

## 2. Launch Clobster

```bash
clobster
```

## 3. Navigate the Interface

Clobster uses vim-style keybindings:

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Select |
| `Tab` | Switch tabs |
| `?` | Show help |
| `r` | Refresh data |
| `q` | Quit |

## 4. View Markets

The main view shows available markets. Use `j`/`k` to navigate and `Enter` to view details.

## 5. Next Steps

- [Configuration](./configuration.md) - Advanced configuration options
- [Trading Strategies](../strategies/introduction.md) - Automate your trading
- [Architecture Overview](../architecture/overview.md) - Understand the system
