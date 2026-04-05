# Radio-CLI Justfile
# Usage: just <command>

# List all recipes
default:
    @just --list

# Format all code
format:
    cargo fmt

# Check formatting without modifying files
format-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy

# Check code quality (both formatting and clippy)
check: format-check lint
    echo "All checks passed!"

# Build the project
build:
    cargo build --bin radio_cli

# Run the project
run:
    cargo run --bin radio_cli

# Run with visualizations enabled
run-vis:
    cargo run --bin radio_cli -- --vis

# Build release version
release:
    cargo build --release --bin radio_cli

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Setup git hooks (run once after cloning)
setup-hooks:
    #!/bin/bash
    echo "Setting up git hooks..."
    chmod +x .git/hooks/pre-commit
    echo "Git hooks setup complete."