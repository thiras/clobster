# Configuration

Clobster loads configuration from `~/.config/clobster/config.toml`.

## Full Configuration Reference

```toml
[api]
# Polymarket CLOB API base URL
base_url = "https://clob.polymarket.com"

# WebSocket URL for real-time updates
ws_url = "wss://ws-subscriptions-clob.polymarket.com/ws"

# Request timeout in seconds
timeout_secs = 30

# Maximum retries for failed requests
max_retries = 3

# Rate limit (requests per second)
rate_limit = 10

# Path to credentials file (optional)
# credentials_path = "/path/to/credentials.json"

[ui]
# UI update tick rate in milliseconds
tick_rate_ms = 100

# Enable mouse support
mouse_support = true

# Use Unicode symbols (disable for compatibility)
unicode_symbols = true

[keybindings]
# Navigation
up = "k"
down = "j"
left = "h"
right = "l"
page_up = "ctrl-u"
page_down = "ctrl-d"
top = "g"
bottom = "G"

# Actions
select = "enter"
back = "esc"
quit = "q"
help = "?"
refresh = "r"

# Tabs
next_tab = "tab"
prev_tab = "shift-tab"

[theme]
# Use custom colors (default: terminal colors)
# primary = "#61afef"
# secondary = "#98c379"
# error = "#e06c75"
# warning = "#e5c07b"
```

## Environment Variables

You can override configuration with environment variables:

```bash
# Override API base URL
export CLOBSTER_API_BASE_URL="https://custom.api.com"

# Enable debug logging
export RUST_LOG=clobster=debug
```

## API Credentials

For authenticated trading, you need Polymarket API credentials. Store them securely:

```bash
# Create credentials file with restricted permissions
touch ~/.config/clobster/credentials.json
chmod 600 ~/.config/clobster/credentials.json
```

Then reference it in your config:

```toml
[api]
credentials_path = "~/.config/clobster/credentials.json"
```

## Multiple Profiles

You can maintain multiple configuration files:

```bash
# Use a specific config file
clobster --config ~/.config/clobster/testnet.toml
```
