# Shared Container

A unified abstraction for shared data access in both multi-threaded and single-threaded environments.

[![Crates.io](https://img.shields.io/crates/v/shared-container.svg)](https://crates.io/crates/shared-container)
[![Documentation](https://docs.rs/shared-container/badge.svg)](https://docs.rs/shared-container)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`shared-container` provides a unified abstraction over different container types used for shared data access with
interior mutability in different contexts. It abstracts over the differences between:

- Thread-safe `Arc<RwLock<T>>` used in multi-threaded environments
- `Rc<RefCell<T>>` used in single-threaded environments like WebAssembly

This allows code using these containers to be written once but work efficiently in both contexts.

## Features

- **Platform-aware implementation**: Automatically uses the most efficient implementation based on the target platform
- **Unified API**: Same API for both multi-threaded and single-threaded environments
- **Read/Write access**: Provides both read-only and read-write access to the contained data
- **Weak references**: Supports weak references to prevent reference cycles
- **Clone support**: Containers can be cloned to create multiple references to the same data
- **Transparent access**: Uses Rust's deref mechanism for ergonomic access to the contained data
- **Async support**: Optional support for async/await with Tokio

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
shared-container = "0.2.0"
```

### Basic Example

```rust
use shared_container::SharedContainer;

// Create a new container with a value
let container = SharedContainer::new(42);

// Read access
if let Some(guard) = container.read() {
    println!("Value: {}", *guard);
}

// Write access
if let Some(mut guard) = container.write() {
    *guard = 100;
}

// Clone the container (both point to the same data)
let container2 = container.clone();

// Changes through one container are visible through the other
if let Some(guard) = container2.read() {
    assert_eq!(*guard, 100);
}

// Create a weak reference
let weak = container.downgrade();

// Upgrade weak reference to strong reference
if let Some(container3) = weak.upgrade() {
    // Use container3...
}
```

### Working with Custom Types

```rust
use shared_container::SharedContainer;
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
}

let user = User {
    id: 1,
    name: "Alice".to_string(),
};

let container = SharedContainer::new(user);

// Get a clone of the contained value
if let Some(user_clone) = container.get_cloned() {
    println!("User: {:?}", user_clone);
}

// Modify the user
if let Some(mut guard) = container.write() {
    guard.name = "Bob".to_string();
}
```

## Async Support with Tokio

This library provides optional support for async/await with Tokio through the `tokio-sync` feature:

```toml
[dependencies]
shared-container = { version = "0.2.0", features = ["tokio-sync"] }
```

When the `tokio-sync` feature is enabled, the library uses `Arc<tokio::sync::RwLock<T>>` internally, and provides async
methods for read and write access:

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

## Platform-specific Behavior

- On native platforms, `SharedContainer<T>` uses `Arc<RwLock<T>>` internally
- On WebAssembly (`wasm32` target), it uses `Rc<RefCell<T>>` internally
- With the `tokio-sync` feature, it uses `Arc<tokio::sync::RwLock<T>>` for async support
- The API remains the same, but the behavior differs slightly:
    - On native platforms, read/write operations can fail if the lock is poisoned
    - On WebAssembly, read/write operations can fail if there's already a borrow
    - With `tokio-sync`, synchronous methods return `None` and you should use async methods instead

## Testing WebAssembly Compatibility

This library includes a feature flag to help test WebAssembly compatibility even on native platforms:

```toml
[dependencies]
shared-container = { version = "0.2.0", features = ["force-wasm-impl"] }
```

When the `force-wasm-impl` feature is enabled, the library will use the WebAssembly implementation (`Rc<RefCell<T>>`)
even when compiling for native platforms. This allows you to test WebAssembly-specific behavior without actually
compiling to WebAssembly.

To run the WebAssembly-specific tests:

```bash
cargo test --features force-wasm-impl
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
