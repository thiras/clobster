# Installation

## Requirements

- Rust 1.85+ (2024 edition)
- A Polymarket account with API credentials

## From Source

Clone the repository and build:

```bash
git clone https://github.com/thiras/clobster.git
cd clobster
cargo build --release
```

The binary will be at `target/release/clobster`.

## From Crates.io

```bash
cargo install clobster
```

## Development Build

For development with debug symbols:

```bash
cargo build
```

Run with debug logging:

```bash
RUST_LOG=clobster=debug cargo run
```

## Verify Installation

Check that CLOBster is installed correctly:

```bash
clobster --version
```

## Next Steps

- [Quick Start](./quick-start.md) - Get up and running in 5 minutes
- [Configuration](./configuration.md) - Set up your API credentials and preferences
