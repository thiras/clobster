# Contributing

Thank you for your interest in contributing to CLOBster!

## Code of Conduct

Be respectful and constructive. We're all here to build great software.

## Getting Started

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/clobster.git
   cd clobster
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/thiras/clobster.git
   ```
4. Create a feature branch:
   ```bash
   git checkout -b feat/my-feature
   ```

## Development Setup

### Prerequisites

- Rust 1.85+ (2024 edition)
- Git with GPG signing configured (for commits)

### Build and Test

```bash
# Build
cargo build

# Run tests
cargo test

# Run lints
cargo clippy

# Format code
cargo fmt

# Generate docs
cargo doc --open
```

### Pre-commit Hooks

We use pre-commit hooks to ensure code quality. Install them:

```bash
pip install pre-commit
pre-commit install
```

## Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Formatting, no code change |
| `refactor` | Code restructuring |
| `perf` | Performance improvement |
| `test` | Adding tests |
| `chore` | Maintenance tasks |

### Examples

```
feat(strategy): add RSI-based strategy

Implements a relative strength index strategy that generates
buy signals when RSI drops below 30 and sell signals above 70.

Closes #42
```

```
fix(api): handle rate limit errors gracefully

- Add exponential backoff on 429 responses
- Display user-friendly notification

Fixes #123
```

## Pull Request Process

1. **Update your branch** with latest upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Ensure all checks pass**:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```

3. **Write meaningful PR description**:
   - What does this change?
   - Why is it needed?
   - How was it tested?

4. **Request review** from maintainers

5. **Address feedback** and push updates

## Code Style

### Rust Guidelines

- Use `rust_decimal::Decimal` for financial values
- Prefer builder pattern for complex structs
- Use `async_trait` for async trait methods
- Document public APIs with `///` comments
- Add unit tests for new functionality

### File Organization

- One domain per state file
- Widgets in `src/ui/widgets/`
- Strategy implementations in `src/strategy/strategies/`

### Error Handling

Use the custom `Error` enum with constructor helpers:

```rust
use crate::error::{Error, Result};

fn validate_order(size: Decimal) -> Result<()> {
    if size <= Decimal::ZERO {
        return Err(Error::invalid_input("Order size must be positive"));
    }
    Ok(())
}
```

## Adding a New Strategy

1. Create `src/strategy/strategies/my_strategy.rs`
2. Implement the `Strategy` trait
3. Export from `src/strategy/strategies/mod.rs`
4. Add documentation in `docs/src/strategies/`
5. Add tests

## Questions?

- Open an issue for bugs or feature requests
- Start a discussion for questions
