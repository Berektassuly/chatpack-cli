# Contributing to chatpack-cli

Thank you for your interest in contributing to chatpack-cli! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.85 or later
- Git

### Setting Up the Development Environment

```bash
# Clone the repository
git clone https://github.com/Berektassuly/chatpack-cli
cd chatpack-cli

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- tg tests/fixtures/telegram_export.json
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Your Changes

- Write clean, readable code
- Follow Rust conventions and idioms
- Add tests for new functionality
- Update documentation as needed

### 3. Run Quality Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run all tests
cargo test

# Build in release mode to catch optimization issues
cargo build --release
```

### 4. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Features
git commit -m "feat: add support for Signal exports"

# Bug fixes
git commit -m "fix: handle empty messages in WhatsApp parser"

# Documentation
git commit -m "docs: update installation instructions"

# Refactoring
git commit -m "refactor: simplify date parsing logic"

# Tests
git commit -m "test: add integration tests for Discord"

# Chores
git commit -m "chore: update dependencies"
```

### 5. Submit a Pull Request

1. Push your branch to GitHub
2. Open a Pull Request against `main`
3. Fill out the PR template
4. Wait for CI checks to pass
5. Request review

## Code Style

### Rust Guidelines

- Use `rustfmt` for formatting (default settings)
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Prefer explicit error handling over `unwrap()`/`expect()`
- Use `anyhow` for error handling in the CLI
- Document public APIs with doc comments

### Example Code Style

```rust
/// Parses a chat export file and returns messages.
///
/// # Arguments
///
/// * `path` - Path to the export file
/// * `config` - Parser configuration
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn parse_export(path: &Path, config: &Config) -> Result<Vec<Message>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    
    // Implementation...
}
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_telegram_basic

# Integration tests only
cargo test --test integration_tests

# With output
cargo test -- --nocapture
```

### Writing Tests

- Place unit tests in the same file using `#[cfg(test)]`
- Place integration tests in `tests/`
- Use descriptive test names
- Test both success and error cases

```rust
#[test]
fn test_filter_by_date_returns_correct_messages() {
    // Arrange
    let messages = create_test_messages();
    let filter = FilterConfig::new().with_date_from("2024-01-01");
    
    // Act
    let result = apply_filters(messages, &filter);
    
    // Assert
    assert_eq!(result.len(), 3);
    assert!(result.iter().all(|m| m.date >= "2024-01-01"));
}
```

## Adding New Features

### Adding a New Chat Platform

1. Add the platform to the `Source` enum in `main.rs`
2. Implement the parser in the `chatpack` library
3. Add test fixtures in `tests/fixtures/`
4. Add integration tests
5. Update README with usage examples

### Adding a New Output Format

1. Add the format to the `Format` enum
2. Implement the writer function
3. Add tests for the new format
4. Update CLI help text and README

## Release Process

Releases are automated via GitHub Actions when a tag is pushed:

```bash
# Create a new release
git tag v0.2.0
git push origin v0.2.0
```

The CI will:
1. Run all tests
2. Build binaries for all platforms
3. Create a GitHub release
4. Publish to crates.io

## Getting Help

- **Questions?** Open a [Discussion](https://github.com/Berektassuly/chatpack-cli/discussions)
- **Found a bug?** Open an [Issue](https://github.com/Berektassuly/chatpack-cli/issues)
- **Have an idea?** Open a [Feature Request](https://github.com/Berektassuly/chatpack-cli/issues/new?template=feature_request.yml)

## Code of Conduct

Please be respectful and constructive in all interactions. We're all here to make chatpack-cli better!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
