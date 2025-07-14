#!/bin/bash

# Run tests with default features (std-sync)
echo "Running tests with default features (std-sync)..."
cargo test

# Run tests with tokio-sync feature
echo "Running tests with tokio-sync feature..."
cargo test --no-default-features --features tokio-sync

# Run tests with wasm-sync feature
echo "Running tests with wasm-sync feature..."
cargo test --no-default-features --features wasm-sync

# Run tests with force-wasm-impl feature (should be the same as wasm-sync)
echo "Running tests with force-wasm-impl feature..."
cargo test --no-default-features --features force-wasm-impl

echo "All tests completed!"