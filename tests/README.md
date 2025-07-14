# Testing Shared Container

This directory contains tests for the `shared-container` crate with different feature configurations.

## Test Files

- `std_sync_tests.rs`: Tests for the default implementation using `std::sync::RwLock`
- `tokio_sync_tests.rs`: Tests for the async implementation using `tokio::sync::RwLock`
- `wasm_sync_tests.rs`: Tests for the WebAssembly implementation using `Rc<RefCell<T>>`

## Running Tests

You can run tests with different feature configurations:

```bash
# Run tests with default features (std-sync)
cargo test

# Run tests with tokio-sync feature
cargo test --no-default-features --features tokio-sync

# Run tests with wasm-sync feature
cargo test --no-default-features --features wasm-sync

# Run tests with force-wasm-impl feature
cargo test --no-default-features --features force-wasm-impl
```

## Running All Tests

For convenience, you can use the provided script to run all tests with different feature configurations:

```bash
./tests/run_all_tests.sh
```

This script will run tests with:

1. Default features (std-sync)
2. tokio-sync feature
3. wasm-sync feature
4. force-wasm-impl feature

## Test Coverage

The tests cover:

- Basic container operations (create, read, write)
- Cloning containers
- Weak references
- Thread safety (for std-sync)
- Async operations (for tokio-sync)
- Single-threaded behavior (for wasm-sync and force-wasm-impl)