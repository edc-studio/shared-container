## Async Support with Tokio

This library provides optional support for async/await with Tokio through the `tokio-sync` feature:

```toml
[dependencies]
shared-container = { version = "0.1.1", features = ["tokio-sync"] }
```

When the `tokio-sync` feature is enabled, the library uses `Arc<tokio::sync::RwLock<T>>` internally, and provides async methods for read and write access:

```rust
use shared_container::SharedContainer;

async fn example() {
    let container = SharedContainer::new(42);
    
    // Synchronous methods return None with tokio-sync
    assert!(container.read().is_none());
    assert!(container.write().is_none());
    
    // Use async methods instead
    let guard = container.read_async().await;
    assert_eq!(*guard, 42);
    
    // Async write access
    {
        let mut guard = container.write_async().await;
        *guard = 100;
    }
    
    // Verify change
    let guard = container.read_async().await;
    assert_eq!(*guard, 100);
}
```

## Comprehensive Testing

The library includes comprehensive tests for all features. To run the tests:

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

Alternatively, you can use the provided script to run all tests:

```bash
./tests/run_all_tests.sh
```