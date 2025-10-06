# Shared Container

Type-safe shared data access for multi-threaded, async, and single-threaded environments.

[![Crates.io](https://img.shields.io/crates/v/shared-container.svg)](https://crates.io/crates/shared-container)
[![Documentation](https://docs.rs/shared-container/badge.svg)](https://docs.rs/shared-container)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`shared-container` provides type-safe abstractions for shared data access across different runtime environments:

- **Synchronous multi-threaded**: `Arc<RwLock<T>>` on native platforms
- **Single-threaded (WebAssembly)**: `Rc<RefCell<T>>`
- **Asynchronous (Tokio)**: `Arc<tokio::sync::RwLock<T>>`

**Version 0.3** introduces type-level separation between sync and async, eliminating runtime surprises and providing
explicit error handling.

## Key Features

- **Type-Level Safety**: Separate types for sync (`Shared<T>`) and async (`AsyncShared<T>`)
- **Platform-Aware**: Automatically selects the right backend based on target
- **Explicit Errors**: `Result<_, AccessError>` instead of `Option` or panics
- **Zero Runtime Overhead**: No blocking operations or runtime initialization
- **Weak References**: Break reference cycles with weak pointers

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
shared-container = "0.3"
```

### Synchronous Usage

```rust
use shared_container::{Shared, SyncAccess};

// Create a new container
let container = Shared::new(42);

// Read access
let guard = container.read().unwrap();
assert_eq!(*guard, 42);
drop(guard);

// Write access
let mut guard = container.write().unwrap();
* guard = 100;
drop(guard);

// Clone the container (both point to the same data)
let container2 = container.clone();

// Get a cloned value
let value = container.get_cloned().unwrap();
assert_eq!(value, 100);
```

### Asynchronous Usage

Enable the `async` feature:

```toml
[dependencies]
shared-container = { version = "0.3.0", features = ["async"] }
```

```rust
use shared_container::{AsyncShared, AsyncAccess};

async fn example() {
    let container = AsyncShared::new(42);

    // Async read access
    let guard = container.read_async().await;
    assert_eq!(*guard, 42);
    drop(guard);

    // Async write access
    let mut guard = container.write_async().await;
    *guard = 100;
    drop(guard);

    // Clone works the same
    let container2 = container.clone();

    // Get cloned value asynchronously
    let value = container.get_cloned_async().await;
    assert_eq!(value, 100);
}
```

### Working with Custom Types

```rust
use shared_container::{Shared, SyncAccess};

#[derive(Debug, Clone)]
struct User {
    id: u64,
    name: String,
}

let container = Shared::new(User {
id: 1,
name: "Alice".to_string(),
});

// Modify the user
let mut guard = container.write().unwrap();
guard.name = "Bob".to_string();
drop(guard);

// Get a snapshot
let user_snapshot = container.get_cloned().unwrap();
println!("User: {:?}", user_snapshot);
```

### Weak References

```rust
use shared_container::{Shared, SyncAccess};

let container = Shared::new(42);
let weak = container.downgrade();

// Try to upgrade
if let Some(strong) = weak.upgrade() {
let guard = strong.read().unwrap();
println ! ("Value: {}", * guard);
} else {
println ! ("Value was dropped");
}

// After dropping all strong references
drop(container);
assert!(weak.upgrade().is_none());
```

## Error Handling

The new API uses `AccessError` enum for explicit error handling:

```rust
use shared_container::{Shared, SyncAccess, AccessError};

let container = Shared::new(42);

match container.read() {
Ok(guard) => println ! ("Value: {}", * guard),
Err(AccessError::Poisoned) => println ! ("Lock was poisoned"),
Err(AccessError::BorrowConflict) => println ! ("Already borrowed"),
Err(AccessError::UnsupportedMode) => println ! ("Wrong container type"),
}
```

### Error Types

- **`Poisoned`**: Lock was poisoned by a panic (multi-threaded only)
- **`BorrowConflict`**: Borrow rules violated (WebAssembly `RefCell` only)
- **`UnsupportedMode`**: Operation not supported for this container type

## Universal Container (Advanced)

For generic code that needs to work with both sync and async containers:

```rust
use shared_container::{SharedAny, Shared, SyncAccess, AccessError};

fn process_container(container: SharedAny<i32>) {
    match container.read() {
        Ok(guard) => println!("Sync read: {}", *guard),
        Err(AccessError::UnsupportedMode) => {
            println!("This is an async container, use read_async() instead");
        }
        Err(e) => println!("Error: {}", e),
    }
}

let sync_container: SharedAny<i32> = Shared::new(42).into();
process_container(sync_container);
```

## Platform-Specific Behavior

| Platform                | Backend                       | Notes                      |
|-------------------------|-------------------------------|----------------------------|
| Native (multi-threaded) | `Arc<std::sync::RwLock<T>>`   | Can be poisoned by panics  |
| WebAssembly             | `Rc<RefCell<T>>`              | Borrow checking at runtime |
| Async (Tokio)           | `Arc<tokio::sync::RwLock<T>>` | Requires `async` feature   |

## Feature Flags

- **`async`**: Enables `AsyncShared<T>` and async trait methods (requires tokio)
- **`std-sync`** (default): Legacy support for `SharedContainer` with std sync primitives
- **`tokio-sync`**: Legacy support for `SharedContainer` with tokio primitives (deprecated)
- **`wasm-sync`**: Legacy support for forcing WebAssembly backend

## Migration from 2.x

Version 0.3 introduces breaking changes with a clearer, type-safe API. The old `SharedContainer<T>` is deprecated but
still available.

### Migration Guide

| Old (0.2.x)                            | New (0.3.x)                                        |
|----------------------------------------|----------------------------------------------------|
| `SharedContainer::new(v)` (std-sync)   | `Shared::new(v)`                                   |
| `SharedContainer::new(v)` (tokio-sync) | `AsyncShared::new(v)` with `async` feature         |
| `container.read()` → `Option<Guard>`   | `container.read()` → `Result<Guard, AccessError>`  |
| `container.write()` → `Option<Guard>`  | `container.write()` → `Result<Guard, AccessError>` |
| `container.read_async().await`         | `container.read_async().await` (same)              |

### Example Migration

**Before (0.2.x):**

```rust
use shared_container::SharedContainer;

let container = SharedContainer::new(42);
if let Some(guard) = container.read() {
println ! ("{}", * guard);
}
```

**After (0.3.x):**

```rust
use shared_container::{Shared, SyncAccess};

let container = Shared::new(42);
match container.read() {
Ok(guard) => println ! ("{}", * guard),
Err(e) => eprintln! ("Error: {}", e),
}
```

For async code with tokio:

**Before (0.2.x with `tokio-sync`):**

```rust
use shared_container::SharedContainer;

let container = SharedContainer::new(42);
let guard = container.read_async().await;
```

**After (0.3.x with `async` feature):**

```rust
use shared_container::{AsyncShared, AsyncAccess};

let container = AsyncShared::new(42);
let guard = container.read_async().await;
```

## Testing

```bash
# Run all tests with default features
cargo test

# Run tests with async support
cargo test --features async

# Run all tests
cargo test --all-features
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
