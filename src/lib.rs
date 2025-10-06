//! # Shared Container - Type-Safe Shared Data Access
//!
//! This library provides type-safe abstractions for shared data access across
//! different runtime environments: synchronous multi-threaded, asynchronous (tokio),
//! and single-threaded (WebAssembly).
//!
//! ## Quick Start
//!
//! ### Synchronous Usage
//!
//! ```rust
//! use shared_container::{Shared, SyncAccess};
//!
//! let container = Shared::new(42);
//!
//! // Read access
//! let guard = container.read().unwrap();
//! assert_eq!(*guard, 42);
//! drop(guard);
//!
//! // Write access
//! let mut guard = container.write().unwrap();
//! *guard = 100;
//! ```
//!
//! ### Asynchronous Usage (with `async` feature)
//!
//! ```rust
//! # #[cfg(feature = "async")]
//! # async fn example() {
//! use shared_container::{AsyncShared, AsyncAccess};
//!
//! let container = AsyncShared::new(42);
//!
//! // Async read access
//! let guard = container.read_async().await;
//! assert_eq!(*guard, 42);
//! drop(guard);
//!
//! // Async write access
//! let mut guard = container.write_async().await;
//! *guard = 100;
//! # }
//! ```
//!
//! ## Key Features
//!
//! - **Type-Level Separation**: `Shared<T>` for sync, `AsyncShared<T>` for async
//! - **Platform-Aware**: Automatically uses the right backend based on target
//!   - Native: `Arc<RwLock<T>>`
//!   - WebAssembly: `Rc<RefCell<T>>`
//!   - Async: `Arc<tokio::sync::RwLock<T>>`
//! - **Explicit Errors**: `Result<_, AccessError>` instead of `Option` or panics
//! - **Zero Runtime Overhead**: No blocking operations or runtime initialization
//!
//! ## Feature Flags
//!
//! - **`async`**: Enables `AsyncShared<T>` and async trait methods (requires tokio)
//! - **`std-sync`** (default): Legacy support for `SharedContainer` with std sync primitives
//! - **`tokio-sync`**: Legacy support for `SharedContainer` with tokio primitives
//! - **`wasm-sync`**: Legacy support for forcing WebAssembly backend
//!
//! ## Migration from 2.x
//!
//! The old `SharedContainer<T>` API is deprecated. Migrate to the new type-safe API:
//!
//! | Old (2.x) | New (3.0) |
//! |-----------|-----------|
//! | `SharedContainer::new(v)` (std-sync) | `Shared::new(v)` |
//! | `SharedContainer::new(v)` (tokio-sync) | `AsyncShared::new(v)` |
//! | `container.read()` → `Option<_>` | `container.read()` → `Result<_, AccessError>` |
//! | `container.read_async().await` | `container.read_async().await` |
//!
//! ## Error Handling
//!
//! The new API uses `AccessError` enum for explicit error handling:
//!
//! ```rust
//! use shared_container::{Shared, SyncAccess, AccessError};
//!
//! let container = Shared::new(42);
//! match container.read() {
//!     Ok(guard) => println!("Value: {}", *guard),
//!     Err(AccessError::Poisoned) => println!("Lock was poisoned"),
//!     Err(AccessError::BorrowConflict) => println!("Already borrowed"),
//!     Err(AccessError::UnsupportedMode) => println!("Wrong container type"),
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::{Deref, DerefMut};

// Standard library synchronization primitives (default)
#[cfg(all(
    feature = "std-sync",
    not(feature = "tokio-sync"),
    not(feature = "wasm-sync")
))]
use std::sync::{Arc, RwLock, Weak};

// Tokio async synchronization primitives
#[cfg(feature = "tokio-sync")]
use std::sync::{Arc, Weak};
#[cfg(feature = "tokio-sync")]
use tokio::sync::RwLock;

// WebAssembly/single-threaded synchronization primitives
#[cfg(any(
    feature = "wasm-sync",
    all(
        target_arch = "wasm32",
        not(feature = "std-sync"),
        not(feature = "tokio-sync")
    )
))]
use std::cell::{Ref, RefCell, RefMut};
#[cfg(any(
    feature = "wasm-sync",
    all(
        target_arch = "wasm32",
        not(feature = "std-sync"),
        not(feature = "tokio-sync")
    )
))]
use std::rc::{Rc, Weak as RcWeak};

/// A unified container for shared data that works in both multi-threaded and single-threaded environments.
///
/// **DEPRECATED**: Use [`Shared<T>`](crate::Shared) for synchronous code or
/// `AsyncShared<T>` (with `async` feature) for asynchronous code instead.
///
/// This struct provides an abstraction over different container types:
/// - `Arc<std::sync::RwLock<T>>` (used in standard multi-threaded environments)
/// - `Arc<tokio::sync::RwLock<T>>` (used for async/await support)
/// - `Rc<RefCell<T>>` (used in single-threaded environments like WebAssembly)
///
/// ## Migration Guide
///
/// ```rust
/// // Old (2.x with std-sync)
/// // use shared_container::SharedContainer;
/// // let container = SharedContainer::new(42);
///
/// // New (3.0)
/// use shared_container::{Shared, SyncAccess};
/// let container = Shared::new(42);
/// ```
///
/// For async code with tokio:
/// ```rust
/// # #[cfg(feature = "async")]
/// # async fn example() {
/// // Old (2.x with tokio-sync)
/// // use shared_container::SharedContainer;
/// // let container = SharedContainer::new(42);
///
/// // New (3.0)
/// use shared_container::{AsyncShared, AsyncAccess};
/// let container = AsyncShared::new(42);
/// # }
/// ```
#[deprecated(
    since = "3.0.0",
    note = "Use `Shared<T>` for sync or `AsyncShared<T>` for async instead. See migration guide in docs."
)]
#[derive(Debug)]
pub struct SharedContainer<T> {
    // Standard library thread-safe implementation
    #[cfg(all(
        feature = "std-sync",
        not(feature = "tokio-sync"),
        not(feature = "wasm-sync")
    ))]
    std_inner: Arc<RwLock<T>>,

    // Tokio async implementation
    #[cfg(feature = "tokio-sync")]
    tokio_inner: Arc<RwLock<T>>,

    // Single-threaded implementation for WebAssembly
    #[cfg(any(
        feature = "wasm-sync",
        all(
            target_arch = "wasm32",
            not(feature = "std-sync"),
            not(feature = "tokio-sync")
        )
    ))]
    wasm_inner: Rc<RefCell<T>>,
}

// Implement Send and Sync for SharedContainer only for thread-safe implementations
#[cfg(any(feature = "std-sync", feature = "tokio-sync"))]
unsafe impl<T: Send> Send for SharedContainer<T> {}

#[cfg(any(feature = "std-sync", feature = "tokio-sync"))]
unsafe impl<T: Send + Sync> Sync for SharedContainer<T> {}

/// A weak reference to a `SharedContainer`.
///
/// **DEPRECATED**: Use [`WeakShared<T>`](crate::WeakShared) for synchronous code or
/// `WeakAsyncShared<T>` (with `async` feature) for asynchronous code instead.
///
/// This struct provides an abstraction over different weak reference types:
/// - `Weak<std::sync::RwLock<T>>` (used in standard multi-threaded environments)
/// - `Weak<tokio::sync::RwLock<T>>` (used for async/await support)
/// - `Weak<RefCell<T>>` (used in single-threaded environments like WebAssembly)
///
/// Weak references don't prevent the value from being dropped when no strong references
/// remain. This helps break reference cycles that could cause memory leaks.
#[deprecated(
    since = "3.0.0",
    note = "Use `WeakShared<T>` for sync or `WeakAsyncShared<T>` for async instead."
)]
#[derive(Debug)]
pub struct WeakSharedContainer<T> {
    // Standard library thread-safe implementation
    #[cfg(all(
        feature = "std-sync",
        not(feature = "tokio-sync"),
        not(feature = "wasm-sync")
    ))]
    std_inner: Weak<RwLock<T>>,

    // Tokio async implementation
    #[cfg(feature = "tokio-sync")]
    tokio_inner: Weak<RwLock<T>>,

    // Single-threaded implementation for WebAssembly
    #[cfg(any(
        feature = "wasm-sync",
        all(
            target_arch = "wasm32",
            not(feature = "std-sync"),
            not(feature = "tokio-sync")
        )
    ))]
    wasm_inner: RcWeak<RefCell<T>>,
}

impl<T> Clone for WeakSharedContainer<T> {
    fn clone(&self) -> Self {
        // Different implementations for different platforms
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            WeakSharedContainer {
                std_inner: self.std_inner.clone(),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            WeakSharedContainer {
                tokio_inner: self.tokio_inner.clone(),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            WeakSharedContainer {
                wasm_inner: self.wasm_inner.clone(),
            }
        }
    }
}

impl<T: PartialEq> PartialEq for SharedContainer<T> {
    fn eq(&self, _other: &Self) -> bool {
        #[cfg(feature = "tokio-sync")]
        {
            // Note: PartialEq is not supported with tokio-sync feature
            // as it would require blocking operations in a sync context.
            // Consider comparing values manually using async methods.
            false
        }

        #[cfg(not(feature = "tokio-sync"))]
        {
            match (self.read(), _other.read()) {
                (Some(self_val), Some(other_val)) => *self_val == *other_val,
                _ => false,
            }
        }
    }
}

impl<T> Clone for SharedContainer<T> {
    fn clone(&self) -> Self {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            SharedContainer {
                std_inner: Arc::clone(&self.std_inner),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            SharedContainer {
                tokio_inner: Arc::clone(&self.tokio_inner),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            SharedContainer {
                wasm_inner: Rc::clone(&self.wasm_inner),
            }
        }
    }
}

impl<T: Clone> SharedContainer<T> {
    /// Gets a clone of the contained value.
    ///
    /// This method acquires a read lock, clones the value, and releases the lock.
    ///
    /// # Returns
    /// * `Some(T)`: A clone of the contained value
    /// * `None`: If the lock couldn't be acquired
    ///
    /// # Note
    /// When using the `tokio-sync` feature, this method will try to acquire the lock
    /// in a blocking manner, which may not be ideal for async code. Consider using
    /// `get_cloned_async()` instead.
    #[cfg_attr(
        feature = "tokio-sync",
        doc = "WARNING: This method uses blocking operations when using tokio-sync feature, which is not ideal for async code. Consider using get_cloned_async() instead."
    )]
    pub fn get_cloned(&self) -> Option<T> {
        #[cfg(feature = "tokio-sync")]
        {
            // Note: This method is not recommended with tokio-sync feature.
            // Use get_cloned_async() instead for better async behavior.
            None
        }

        #[cfg(not(feature = "tokio-sync"))]
        {
            let guard = self.read()?;
            Some((*guard).clone())
        }
    }

    /// Gets a clone of the contained value asynchronously.
    ///
    /// This method is only available when the `tokio-sync` feature is enabled.
    ///
    /// # Returns
    /// A clone of the contained value
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "tokio-sync")]
    /// # async fn example() {
    /// # use shared_container::SharedContainer;
    /// let container = SharedContainer::new(42);
    /// let value = container.get_cloned_async().await;
    /// assert_eq!(value, 42);
    /// # }
    /// ```
    #[cfg(feature = "tokio-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-sync")))]
    pub async fn get_cloned_async(&self) -> T
    where
        T: Clone,
    {
        let guard = self.tokio_inner.read().await;
        (*guard).clone()
    }
}

impl<T> SharedContainer<T> {
    /// Creates a new `SharedContainer` containing the given value.
    ///
    /// # Parameters
    /// * `value`: The value to store in the container
    ///
    /// # Returns
    /// A new `SharedContainer` instance containing the value
    pub fn new(value: T) -> Self {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            SharedContainer {
                std_inner: Arc::new(RwLock::new(value)),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            SharedContainer {
                tokio_inner: Arc::new(RwLock::new(value)),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            SharedContainer {
                wasm_inner: Rc::new(RefCell::new(value)),
            }
        }
    }

    /// Gets a read-only access guard to the contained value.
    ///
    /// # Returns
    /// * `Some(SharedReadGuard<T>)`: A guard allowing read-only access to the value
    /// * `None`: If the lock couldn't be acquired
    ///
    /// # Note
    /// When using the `tokio-sync` feature, this method will always return `None`.
    /// Use `read_async()` instead for async access.
    #[cfg_attr(
        feature = "tokio-sync",
        doc = "WARNING: This method always returns None when using tokio-sync feature. Use read_async() instead."
    )]
    pub fn read(&self) -> Option<SharedReadGuard<'_, T>> {
        #[cfg(feature = "tokio-sync")]
        {
            // Tokio's RwLock doesn't have a non-async read method, so we can't use it here
            // Users should use read_async instead
            return None;
        }

        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            match self.std_inner.read() {
                Ok(guard) => Some(SharedReadGuard::StdSync(guard)),
                Err(_) => None,
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            match self.wasm_inner.try_borrow() {
                Ok(borrow) => Some(SharedReadGuard::Single(borrow)),
                Err(_) => None,
            }
        }
    }

    /// Gets a writable access guard to the contained value.
    ///
    /// # Returns
    /// * `Some(SharedWriteGuard<T>)`: A guard allowing read-write access to the value
    /// * `None`: If the lock couldn't be acquired
    ///
    /// # Note
    /// When using the `tokio-sync` feature, this method will always return `None`.
    /// Use `write_async()` instead for async access.
    #[cfg_attr(
        feature = "tokio-sync",
        doc = "WARNING: This method always returns None when using tokio-sync feature. Use write_async() instead."
    )]
    pub fn write(&self) -> Option<SharedWriteGuard<'_, T>> {
        #[cfg(feature = "tokio-sync")]
        {
            // Tokio's RwLock doesn't have a non-async write method, so we can't use it here
            // Users should use write_async instead
            return None;
        }

        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            match self.std_inner.write() {
                Ok(guard) => Some(SharedWriteGuard::StdSync(guard)),
                Err(_) => None,
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            match self.wasm_inner.try_borrow_mut() {
                Ok(borrow) => Some(SharedWriteGuard::Single(borrow)),
                Err(_) => None,
            }
        }
    }

    /// Creates a weak reference to this container.
    ///
    /// A weak reference doesn't prevent the value from being dropped when no strong
    /// references remain, which helps break reference cycles that could cause memory leaks.
    ///
    /// # Returns
    /// A `WeakSharedContainer` that points to the same data
    pub fn downgrade(&self) -> WeakSharedContainer<T> {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            WeakSharedContainer {
                std_inner: Arc::downgrade(&self.std_inner),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            WeakSharedContainer {
                tokio_inner: Arc::downgrade(&self.tokio_inner),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            WeakSharedContainer {
                wasm_inner: Rc::downgrade(&self.wasm_inner),
            }
        }
    }

    /// Asynchronously gets a read-only access guard to the contained value.
    ///
    /// This method is only available when the `tokio-sync` feature is enabled.
    ///
    /// # Returns
    /// A guard allowing read-only access to the value
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "tokio-sync")]
    /// # async fn example() {
    /// # use shared_container::SharedContainer;
    /// let container = SharedContainer::new(42);
    /// let guard = container.read_async().await;
    /// assert_eq!(*guard, 42);
    /// # }
    /// ```
    #[cfg(feature = "tokio-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-sync")))]
    pub async fn read_async(&self) -> SharedReadGuard<'_, T> {
        let guard = self.tokio_inner.read().await;
        SharedReadGuard::TokioSync(guard)
    }

    /// Asynchronously gets a writable access guard to the contained value.
    ///
    /// This method is only available when the `tokio-sync` feature is enabled.
    ///
    /// # Returns
    /// A guard allowing read-write access to the value
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "tokio-sync")]
    /// # async fn example() {
    /// # use shared_container::SharedContainer;
    /// let container = SharedContainer::new(42);
    /// let mut guard = container.write_async().await;
    /// *guard = 100;
    /// # }
    /// ```
    #[cfg(feature = "tokio-sync")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tokio-sync")))]
    pub async fn write_async(&self) -> SharedWriteGuard<'_, T> {
        let guard = self.tokio_inner.write().await;
        SharedWriteGuard::TokioSync(guard)
    }
}

impl<T> WeakSharedContainer<T> {
    /// Attempts to create a strong `SharedContainer` from this weak reference.
    ///
    /// This will succeed if the value has not yet been dropped, i.e., if there are
    /// still other strong references to it.
    ///
    /// # Returns
    /// * `Some(SharedContainer<T>)`: If the value still exists
    /// * `None`: If the value has been dropped
    pub fn upgrade(&self) -> Option<SharedContainer<T>> {
        // Different implementations for different platforms
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            self.std_inner
                .upgrade()
                .map(|inner| SharedContainer { std_inner: inner })
        }

        #[cfg(feature = "tokio-sync")]
        {
            self.tokio_inner
                .upgrade()
                .map(|inner| SharedContainer { tokio_inner: inner })
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            self.wasm_inner
                .upgrade()
                .map(|inner| SharedContainer { wasm_inner: inner })
        }
    }
}
/// A read-only guard for accessing data in a `SharedContainer`.
///
/// This type abstracts over the differences between different read guards:
/// - `std::sync::RwLockReadGuard` (used in standard multi-threaded environments)
/// - `tokio::sync::RwLockReadGuard` (used for async/await support)
/// - `std::cell::Ref` (used in single-threaded environments like WebAssembly)
///
/// It implements `Deref` to allow transparent access to the underlying data.
pub enum SharedReadGuard<'a, T> {
    #[cfg(all(
        feature = "std-sync",
        not(feature = "tokio-sync"),
        not(feature = "wasm-sync")
    ))]
    StdSync(std::sync::RwLockReadGuard<'a, T>),

    #[cfg(feature = "tokio-sync")]
    TokioSync(tokio::sync::RwLockReadGuard<'a, T>),

    #[cfg(any(
        feature = "wasm-sync",
        all(
            target_arch = "wasm32",
            not(feature = "std-sync"),
            not(feature = "tokio-sync")
        )
    ))]
    Single(Ref<'a, T>),
}

impl<'a, T> Deref for SharedReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            match self {
                SharedReadGuard::StdSync(guard) => guard.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            match self {
                SharedReadGuard::TokioSync(guard) => guard.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            match self {
                SharedReadGuard::Single(borrow) => borrow.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }
}

/// A writable guard for accessing and modifying data in a `SharedContainer`.
///
/// This type abstracts over the differences between different write guards:
/// - `std::sync::RwLockWriteGuard` (used in standard multi-threaded environments)
/// - `tokio::sync::RwLockWriteGuard` (used for async/await support)
/// - `std::cell::RefMut` (used in single-threaded environments like WebAssembly)
///
/// It implements both `Deref` and `DerefMut` to allow transparent access to the underlying data.
pub enum SharedWriteGuard<'a, T> {
    #[cfg(all(
        feature = "std-sync",
        not(feature = "tokio-sync"),
        not(feature = "wasm-sync")
    ))]
    StdSync(std::sync::RwLockWriteGuard<'a, T>),

    #[cfg(feature = "tokio-sync")]
    TokioSync(tokio::sync::RwLockWriteGuard<'a, T>),

    #[cfg(any(
        feature = "wasm-sync",
        all(
            target_arch = "wasm32",
            not(feature = "std-sync"),
            not(feature = "tokio-sync")
        )
    ))]
    Single(RefMut<'a, T>),
}

impl<'a, T> Deref for SharedWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            match self {
                SharedWriteGuard::StdSync(guard) => guard.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            match self {
                SharedWriteGuard::TokioSync(guard) => guard.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            match self {
                SharedWriteGuard::Single(borrow) => borrow.deref(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }
}

impl<'a, T> DerefMut for SharedWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(all(
            feature = "std-sync",
            not(feature = "tokio-sync"),
            not(feature = "wasm-sync")
        ))]
        {
            match self {
                SharedWriteGuard::StdSync(guard) => guard.deref_mut(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(feature = "tokio-sync")]
        {
            match self {
                SharedWriteGuard::TokioSync(guard) => guard.deref_mut(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }

        #[cfg(any(
            feature = "wasm-sync",
            all(
                target_arch = "wasm32",
                not(feature = "std-sync"),
                not(feature = "tokio-sync")
            )
        ))]
        {
            match self {
                SharedWriteGuard::Single(borrow) => borrow.deref_mut(),
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            }
        }
    }
}

// ============================================================================
// New 3.0 API - Type-level separation of sync and async
// ============================================================================

/// Errors that can occur when accessing shared containers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessError {
    /// The requested operation is not supported for this container type.
    ///
    /// This typically occurs when trying to use synchronous methods on an async container,
    /// or vice versa.
    UnsupportedMode,

    /// A borrow conflict occurred (for single-threaded RefCell-based containers).
    ///
    /// This happens when trying to acquire a write lock while a read lock exists,
    /// or when trying to acquire any lock while a write lock exists.
    BorrowConflict,

    /// The lock was poisoned due to a panic while holding the lock.
    ///
    /// This only occurs with multi-threaded RwLock-based containers.
    Poisoned,
}

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessError::UnsupportedMode => {
                write!(f, "operation not supported for this container mode")
            }
            AccessError::BorrowConflict => {
                write!(f, "borrow conflict: lock already held")
            }
            AccessError::Poisoned => {
                write!(f, "lock poisoned by panic")
            }
        }
    }
}

impl std::error::Error for AccessError {}

/// Trait for synchronous access to shared containers.
pub trait SyncAccess<T> {
    /// Acquires a read lock on the container.
    fn read(&self) -> Result<SyncReadGuard<'_, T>, AccessError>;

    /// Acquires a write lock on the container.
    fn write(&self) -> Result<SyncWriteGuard<'_, T>, AccessError>;

    /// Gets a clone of the contained value.
    fn get_cloned(&self) -> Result<T, AccessError>
    where
        T: Clone;
}

/// Trait for asynchronous access to shared containers.
#[cfg(feature = "async")]
pub trait AsyncAccess<T> {
    /// Asynchronously acquires a read lock on the container.
    fn read_async<'a>(&'a self) -> impl std::future::Future<Output = AsyncReadGuard<'a, T>> + Send
    where
        T: 'a;

    /// Asynchronously acquires a write lock on the container.
    fn write_async<'a>(
        &'a self,
    ) -> impl std::future::Future<Output = AsyncWriteGuard<'a, T>> + Send
    where
        T: 'a;

    /// Asynchronously gets a clone of the contained value.
    fn get_cloned_async(&self) -> impl std::future::Future<Output = T> + Send
    where
        T: Clone;
}

/// Read guard for synchronous access.
#[derive(Debug)]
pub enum SyncReadGuard<'a, T> {
    #[cfg(not(target_arch = "wasm32"))]
    Std(std::sync::RwLockReadGuard<'a, T>),
    #[cfg(target_arch = "wasm32")]
    Wasm(Ref<'a, T>),
}

impl<'a, T> Deref for SyncReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            SyncReadGuard::Std(guard) => guard.deref(),
            #[cfg(target_arch = "wasm32")]
            SyncReadGuard::Wasm(guard) => guard.deref(),
        }
    }
}

/// Write guard for synchronous access.
#[derive(Debug)]
pub enum SyncWriteGuard<'a, T> {
    #[cfg(not(target_arch = "wasm32"))]
    Std(std::sync::RwLockWriteGuard<'a, T>),
    #[cfg(target_arch = "wasm32")]
    Wasm(RefMut<'a, T>),
}

impl<'a, T> Deref for SyncWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            SyncWriteGuard::Std(guard) => guard.deref(),
            #[cfg(target_arch = "wasm32")]
            SyncWriteGuard::Wasm(guard) => guard.deref(),
        }
    }
}

impl<'a, T> DerefMut for SyncWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            SyncWriteGuard::Std(guard) => guard.deref_mut(),
            #[cfg(target_arch = "wasm32")]
            SyncWriteGuard::Wasm(guard) => guard.deref_mut(),
        }
    }
}

/// Read guard for asynchronous access.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncReadGuard<'a, T>(tokio::sync::RwLockReadGuard<'a, T>);

#[cfg(feature = "async")]
impl<'a, T> Deref for AsyncReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// Write guard for asynchronous access.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncWriteGuard<'a, T>(tokio::sync::RwLockWriteGuard<'a, T>);

#[cfg(feature = "async")]
impl<'a, T> Deref for AsyncWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

#[cfg(feature = "async")]
impl<'a, T> DerefMut for AsyncWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

/// A synchronous shared container that works across platforms.
///
/// On wasm32 targets: uses `Rc<RefCell<T>>`
/// On other targets: uses `Arc<RwLock<T>>`
#[derive(Debug)]
pub struct Shared<T> {
    #[cfg(target_arch = "wasm32")]
    inner: Rc<RefCell<T>>,

    #[cfg(not(target_arch = "wasm32"))]
    inner: std::sync::Arc<std::sync::RwLock<T>>,
}

/// A weak reference to a `Shared<T>`.
#[derive(Debug)]
pub struct WeakShared<T> {
    #[cfg(target_arch = "wasm32")]
    inner: RcWeak<RefCell<T>>,

    #[cfg(not(target_arch = "wasm32"))]
    inner: std::sync::Weak<std::sync::RwLock<T>>,
}

/// An asynchronous shared container using tokio primitives.
///
/// Only available with the `async` feature flag.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncShared<T> {
    inner: Arc<tokio::sync::RwLock<T>>,
}

#[cfg(feature = "async")]
unsafe impl<T: Send> Send for AsyncShared<T> {}

#[cfg(feature = "async")]
unsafe impl<T: Send + Sync> Sync for AsyncShared<T> {}

/// A weak reference to an `AsyncShared<T>`.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct WeakAsyncShared<T> {
    inner: Weak<tokio::sync::RwLock<T>>,
}

/// A universal container that can hold either sync or async variants.
///
/// This enum allows writing generic code that works with both sync and async containers,
/// but requires explicit handling of the mode mismatch via `Result`.
#[derive(Debug)]
pub enum SharedAny<T> {
    Sync(Shared<T>),
    #[cfg(feature = "async")]
    Async(AsyncShared<T>),
}

/// A weak reference to a `SharedAny<T>`.
#[derive(Debug)]
pub enum WeakSharedAny<T> {
    Sync(WeakShared<T>),
    #[cfg(feature = "async")]
    Async(WeakAsyncShared<T>),
}

// ============================================================================
// Basic constructors and conversions
// ============================================================================

impl<T> Shared<T> {
    /// Creates a new synchronous shared container.
    pub fn new(value: T) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Shared {
                inner: Rc::new(RefCell::new(value)),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Shared {
                inner: std::sync::Arc::new(std::sync::RwLock::new(value)),
            }
        }
    }

    /// Creates a weak reference to this container.
    pub fn downgrade(&self) -> WeakShared<T> {
        #[cfg(target_arch = "wasm32")]
        {
            WeakShared {
                inner: Rc::downgrade(&self.inner),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            WeakShared {
                inner: std::sync::Arc::downgrade(&self.inner),
            }
        }
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Shared {
                inner: Rc::clone(&self.inner),
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Shared {
                inner: std::sync::Arc::clone(&self.inner),
            }
        }
    }
}

impl<T> WeakShared<T> {
    /// Attempts to upgrade the weak reference to a strong reference.
    pub fn upgrade(&self) -> Option<Shared<T>> {
        #[cfg(target_arch = "wasm32")]
        {
            self.inner.upgrade().map(|inner| Shared { inner })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.upgrade().map(|inner| Shared { inner })
        }
    }
}

impl<T> Clone for WeakShared<T> {
    fn clone(&self) -> Self {
        WeakShared {
            inner: self.inner.clone(),
        }
    }
}

// ============================================================================
// SyncAccess implementation for Shared<T>
// ============================================================================

impl<T> SyncAccess<T> for Shared<T> {
    fn read(&self) -> Result<SyncReadGuard<'_, T>, AccessError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner
                .read()
                .map(SyncReadGuard::Std)
                .map_err(|_| AccessError::Poisoned)
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.inner
                .try_borrow()
                .map(SyncReadGuard::Wasm)
                .map_err(|_| AccessError::BorrowConflict)
        }
    }

    fn write(&self) -> Result<SyncWriteGuard<'_, T>, AccessError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner
                .write()
                .map(SyncWriteGuard::Std)
                .map_err(|_| AccessError::Poisoned)
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.inner
                .try_borrow_mut()
                .map(SyncWriteGuard::Wasm)
                .map_err(|_| AccessError::BorrowConflict)
        }
    }

    fn get_cloned(&self) -> Result<T, AccessError>
    where
        T: Clone,
    {
        let guard = self.read()?;
        Ok((*guard).clone())
    }
}

#[cfg(feature = "async")]
impl<T> AsyncShared<T> {
    /// Creates a new asynchronous shared container.
    pub fn new(value: T) -> Self {
        AsyncShared {
            inner: Arc::new(tokio::sync::RwLock::new(value)),
        }
    }

    /// Creates a weak reference to this container.
    pub fn downgrade(&self) -> WeakAsyncShared<T> {
        WeakAsyncShared {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

#[cfg(feature = "async")]
impl<T> Clone for AsyncShared<T> {
    fn clone(&self) -> Self {
        AsyncShared {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(feature = "async")]
impl<T> WeakAsyncShared<T> {
    /// Attempts to upgrade the weak reference to a strong reference.
    pub fn upgrade(&self) -> Option<AsyncShared<T>> {
        self.inner.upgrade().map(|inner| AsyncShared { inner })
    }
}

#[cfg(feature = "async")]
impl<T> Clone for WeakAsyncShared<T> {
    fn clone(&self) -> Self {
        WeakAsyncShared {
            inner: self.inner.clone(),
        }
    }
}

// ============================================================================
// AsyncAccess implementation for AsyncShared<T>
// ============================================================================

#[cfg(feature = "async")]
impl<T: Send + Sync> AsyncAccess<T> for AsyncShared<T> {
    async fn read_async<'a>(&'a self) -> AsyncReadGuard<'a, T>
    where
        T: 'a,
    {
        AsyncReadGuard(self.inner.read().await)
    }

    async fn write_async<'a>(&'a self) -> AsyncWriteGuard<'a, T>
    where
        T: 'a,
    {
        AsyncWriteGuard(self.inner.write().await)
    }

    async fn get_cloned_async(&self) -> T
    where
        T: Clone,
    {
        let guard = self.inner.read().await;
        (*guard).clone()
    }
}

// ============================================================================
// Conversions for SharedAny
// ============================================================================

// Conversions for SharedAny
impl<T> From<Shared<T>> for SharedAny<T> {
    fn from(shared: Shared<T>) -> Self {
        SharedAny::Sync(shared)
    }
}

#[cfg(feature = "async")]
impl<T> From<AsyncShared<T>> for SharedAny<T> {
    fn from(shared: AsyncShared<T>) -> Self {
        SharedAny::Async(shared)
    }
}

impl<T> Clone for SharedAny<T> {
    fn clone(&self) -> Self {
        match self {
            SharedAny::Sync(s) => SharedAny::Sync(s.clone()),
            #[cfg(feature = "async")]
            SharedAny::Async(a) => SharedAny::Async(a.clone()),
        }
    }
}

impl<T> SharedAny<T> {
    /// Creates a weak reference to this container.
    pub fn downgrade(&self) -> WeakSharedAny<T> {
        match self {
            SharedAny::Sync(s) => WeakSharedAny::Sync(s.downgrade()),
            #[cfg(feature = "async")]
            SharedAny::Async(a) => WeakSharedAny::Async(a.downgrade()),
        }
    }
}

impl<T> WeakSharedAny<T> {
    /// Attempts to upgrade the weak reference to a strong reference.
    pub fn upgrade(&self) -> Option<SharedAny<T>> {
        match self {
            WeakSharedAny::Sync(w) => w.upgrade().map(SharedAny::Sync),
            #[cfg(feature = "async")]
            WeakSharedAny::Async(w) => w.upgrade().map(SharedAny::Async),
        }
    }
}

impl<T> Clone for WeakSharedAny<T> {
    fn clone(&self) -> Self {
        match self {
            WeakSharedAny::Sync(w) => WeakSharedAny::Sync(w.clone()),
            #[cfg(feature = "async")]
            WeakSharedAny::Async(w) => WeakSharedAny::Async(w.clone()),
        }
    }
}

// ============================================================================
// Trait implementations for SharedAny<T>
// ============================================================================

impl<T> SyncAccess<T> for SharedAny<T> {
    fn read(&self) -> Result<SyncReadGuard<'_, T>, AccessError> {
        match self {
            SharedAny::Sync(s) => s.read(),
            #[cfg(feature = "async")]
            SharedAny::Async(_) => Err(AccessError::UnsupportedMode),
        }
    }

    fn write(&self) -> Result<SyncWriteGuard<'_, T>, AccessError> {
        match self {
            SharedAny::Sync(s) => s.write(),
            #[cfg(feature = "async")]
            SharedAny::Async(_) => Err(AccessError::UnsupportedMode),
        }
    }

    fn get_cloned(&self) -> Result<T, AccessError>
    where
        T: Clone,
    {
        match self {
            SharedAny::Sync(s) => s.get_cloned(),
            #[cfg(feature = "async")]
            SharedAny::Async(_) => Err(AccessError::UnsupportedMode),
        }
    }
}

#[cfg(feature = "async")]
impl<T: Send + Sync> AsyncAccess<T> for SharedAny<T> {
    async fn read_async<'a>(&'a self) -> AsyncReadGuard<'a, T>
    where
        T: 'a,
    {
        match self {
            SharedAny::Async(a) => a.read_async().await,
            SharedAny::Sync(_) => {
                // This branch should not be reachable in normal usage,
                // as the type system should prevent it. However, we need
                // to provide a return value for the compiler.
                unreachable!("Cannot call async methods on sync container")
            }
        }
    }

    async fn write_async<'a>(&'a self) -> AsyncWriteGuard<'a, T>
    where
        T: 'a,
    {
        match self {
            SharedAny::Async(a) => a.write_async().await,
            SharedAny::Sync(_) => {
                unreachable!("Cannot call async methods on sync container")
            }
        }
    }

    async fn get_cloned_async(&self) -> T
    where
        T: Clone,
    {
        match self {
            SharedAny::Async(a) => a.get_cloned_async().await,
            SharedAny::Sync(_) => {
                unreachable!("Cannot call async methods on sync container")
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[derive(Debug, Clone, PartialEq)]
    #[allow(dead_code)]
    struct TestStruct {
        value: i32,
    }

    // Skip synchronous tests when tokio-sync is enabled
    #[cfg(not(feature = "tokio-sync"))]
    mod sync_tests {
        use super::*;
        use crate::SharedContainer;

        #[test]
        fn test_read_access() {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Read access
            let guard = container.read().unwrap();
            assert_eq!(guard.value, 42);
        }

        #[test]
        fn test_write_access() {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Write access
            {
                let mut guard = container.write().unwrap();
                guard.value = 100;
            }

            // Verify change
            let guard = container.read().unwrap();
            assert_eq!(guard.value, 100);
        }

        #[test]
        fn test_clone_container() {
            let container1 = SharedContainer::new(TestStruct { value: 42 });
            let container2 = container1.clone();

            // Modify through container2
            {
                let mut guard = container2.write().unwrap();
                guard.value = 100;
            }

            // Verify change visible through container1
            let guard = container1.read().unwrap();
            assert_eq!(guard.value, 100);
        }

        #[test]
        fn test_get_cloned() {
            let container = SharedContainer::new(TestStruct { value: 42 });
            let cloned = container.get_cloned().unwrap();
            assert_eq!(cloned, TestStruct { value: 42 });

            // Modify original
            {
                let mut guard = container.write().unwrap();
                guard.value = 100;
            }

            // Cloned value should not change
            assert_eq!(cloned, TestStruct { value: 42 });

            // And we can get a new clone with updated value
            let new_clone = container.get_cloned().unwrap();
            assert_eq!(new_clone, TestStruct { value: 100 });
        }

        #[test]
        fn test_weak_ref() {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Create a weak reference
            let weak = container.downgrade();

            // Weak reference can be upgraded to a strong reference
            let container2 = weak.upgrade().unwrap();

            // Both containers point to the same data
            {
                let mut guard = container2.write().unwrap();
                guard.value = 100;
            }

            // Change visible through first container
            {
                let guard = container.read().unwrap();
                assert_eq!(guard.value, 100);
            }
            // Drop all strong references
            drop(container);
            drop(container2);

            // Weak reference can no longer be upgraded
            assert!(weak.upgrade().is_none());
        }

        #[test]
        fn test_weak_clone() {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Create a weak reference and clone it
            let weak1 = container.downgrade();
            let weak2 = weak1.clone();

            // Both weak references can be upgraded
            let container1 = weak1.upgrade().unwrap();
            let container2 = weak2.upgrade().unwrap();

            // Modify through container2
            {
                let mut guard = container2.write().unwrap();
                guard.value = 100;
            }

            // Change visible through container1
            {
                let guard = container1.read().unwrap();
                assert_eq!(guard.value, 100);
            }

            // Drop all strong references
            drop(container);
            drop(container1);
            drop(container2);

            // Neither weak reference can be upgraded
            assert!(weak1.upgrade().is_none());
            assert!(weak2.upgrade().is_none());
        }
    }
}

// Tests specifically for the tokio async implementation
#[cfg(test)]
#[cfg(feature = "tokio-sync")]
mod tokio_tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        value: i32,
    }

    #[test]
    fn test_tokio_read_access() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Synchronous read should return None with tokio-sync
            assert!(container.read().is_none());

            // Async read access
            let guard = container.read_async().await;
            assert_eq!(guard.value, 42);
        });
    }

    #[test]
    fn test_tokio_write_access() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Synchronous write should return None with tokio-sync
            assert!(container.write().is_none());

            // Async write access
            {
                let mut guard = container.write_async().await;
                guard.value = 100;
            }

            // Verify change
            let guard = container.read_async().await;
            assert_eq!(guard.value, 100);
        });
    }

    #[test]
    fn test_tokio_clone_container() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container1 = SharedContainer::new(TestStruct { value: 42 });
            let container2 = container1.clone();

            // Modify through container2
            {
                let mut guard = container2.write_async().await;
                guard.value = 100;
            }

            // Verify change visible through container1
            let guard = container1.read_async().await;
            assert_eq!(guard.value, 100);
        });
    }

    #[test]
    fn test_tokio_weak_ref() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let container = SharedContainer::new(TestStruct { value: 42 });

            // Create a weak reference
            let weak = container.downgrade();

            // Weak reference can be upgraded to a strong reference
            let container2 = weak.upgrade().unwrap();

            // Both containers point to the same data
            {
                let mut guard = container2.write_async().await;
                guard.value = 100;
            }

            // Change visible through first container
            {
                let guard = container.read_async().await;
                assert_eq!(guard.value, 100);
            }

            // Drop all strong references
            drop(container);
            drop(container2);

            // Weak reference can no longer be upgraded
            assert!(weak.upgrade().is_none());
        });
    }
}

// Tests specifically for the WebAssembly implementation
// These tests can be run on any platform by enabling the force-wasm-impl feature
#[cfg(test)]
#[cfg(any(target_arch = "wasm32", feature = "force-wasm-impl"))]
mod wasm_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        value: i32,
    }

    #[test]
    fn test_wasm_read_access() {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Read access
        let guard = container.read().unwrap();
        assert_eq!(guard.value, 42);
    }

    #[test]
    fn test_wasm_write_access() {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Write access
        {
            let mut guard = container.write().unwrap();
            guard.value = 100;
        }

        // Verify change
        let guard = container.read().unwrap();
        assert_eq!(guard.value, 100);
    }

    #[test]
    fn test_wasm_borrow_conflict() {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Get a read borrow
        let _guard = container.read().unwrap();

        // Trying to get a write borrow while a read borrow exists should fail
        assert!(container.write().is_none());
    }

    #[test]
    fn test_wasm_multiple_reads() {
        let container = SharedContainer::new(TestStruct { value: 42 });

        // Multiple read borrows should work
        let _guard1 = container.read().unwrap();
        let guard2 = container.read().unwrap();

        assert_eq!(guard2.value, 42);
    }

    #[test]
    fn test_wasm_weak_ref() {
        let container = SharedContainer::new(TestStruct { value: 42 });
        let weak = container.downgrade();

        // Upgrade should work
        let container2 = weak.upgrade().unwrap();
        assert_eq!(container2.read().unwrap().value, 42);

        // After dropping all strong references, upgrade should fail
        drop(container);
        drop(container2);
        assert!(weak.upgrade().is_none());
    }
}
