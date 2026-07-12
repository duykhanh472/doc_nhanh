#!/usr/bin/env bash

# Exit immediately on error, unset variables, or pipeline failures
set -euo pipefail

# Define paths
DEST_DIR="$HOME/.local/bin"

echo "=== 1. Formatting Check ==="
cargo fmt -- --check

echo "=== 2. Linting (Clippy) ==="
cargo clippy -- -D warnings

# echo "=== 3. Test ==="
# cargo test

echo "=== 4. Compiling Release Binary ==="
cargo build --release

echo "=== 5. Deploying Binary ==="
# Ensure the destination directory exists
mkdir -p "$DEST_DIR"

# Parse the binary name from Cargo.toml and copy it
BIN_NAME=$(cargo metadata --no-deps --format-version 1 | grep -o '"name":"[^"]*"' | head -n 1 | cut -d'"' -f4)
TARGET_PATH="target/release/$BIN_NAME"

if [ -f "$TARGET_PATH" ]; then
    cp "$TARGET_PATH" "$DEST_DIR/"
    echo "Successfully deployed '$BIN_NAME' to $DEST_DIR/"
else
    echo "Error: Binary not found at $TARGET_PATH"
    exit 1
fi

echo "=== Build and Deployment Completed! ==="
