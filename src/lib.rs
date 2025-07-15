//! # Shared Container Module
//!
//! This module provides a unified abstraction over different container types
//! used for shared data access with interior mutability in different contexts.
//!
//! It abstracts over the differences between:
//! - `Arc<std::sync::RwLock<T>>` (used in standard multi-threaded environments)
//! - `Arc<tokio::sync::RwLock<T>>` (used for async/await support)
//! - `Rc<RefCell<T>>` (used in single-threaded environments like WebAssembly)
//!
//! This allows code using these containers to be written once but work efficiently
//! in different contexts.
//!
//! ## Feature Flags
//!
//! This library provides several feature flags to customize its behavior:
//!
//! - **std-sync** (default): Uses `Arc<std::sync::RwLock<T>>` for thread-safe access
//! - **tokio-sync**: Uses `Arc<tokio::sync::RwLock<T>>` for async/await support
//! - **wasm-sync**: Uses `Rc<RefCell<T>>` for single-threaded environments
//! - **force-wasm-impl**: Legacy feature, equivalent to wasm-sync
//!
//! ## Async Support
//!
//! When the `tokio-sync` feature is enabled, the library provides async versions of the read and write methods:
//!
//! ```rust
//! # #[cfg(feature = "tokio-sync")]
//! # async fn example() {
//! # use shared_container::SharedContainer;
//! let container = SharedContainer::new(42);
//!
//! // Read access
//! let guard = container.read_async().await;
//! println!("Value: {}", *guard);
//!
//! // Write access
//! let mut guard = container.write_async().await;
//! *guard = 100;
//! # }
//! ```
//!
//! Note that when using the `tokio-sync` feature, the synchronous `read()` and `write()` methods
//! will always return `None`. You should use the async methods instead.

#![cfg_attr(docsrs, feature(doc_cfg))]

use std::ops::{Deref, DerefMut};

/// Custom attribute to warn when using synchronous methods with tokio-sync
///
/// This attribute will generate a warning during compilation when the `tokio-sync` feature
/// is enabled, but won't mark the function as deprecated in the documentation.
#[cfg(feature = "tokio-sync")]
#[doc(hidden)]
macro_rules! tokio_sync_warning {
    ($msg:expr) => {
        #[doc(hidden)]
        #[allow(deprecated)]
        struct __TokioSyncWarning;

        impl __TokioSyncWarning {
            #[deprecated(note = $msg)]
            #[doc(hidden)]
            fn __warn() {}
        }

        // Call the deprecated function to trigger the warning
        let _ = __TokioSyncWarning::__warn();
    };
}

// #[cfg(not(feature = "tokio-sync"))]
// #[doc(hidden)]
// #[allow(dead_code)]
// macro_rules! tokio_sync_warning {
//     ($msg:expr) => {};
// }

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
/// This struct provides an abstraction over different container types:
/// - `Arc<std::sync::RwLock<T>>` (used in standard multi-threaded environments)
/// - `Arc<tokio::sync::RwLock<T>>` (used for async/await support)
/// - `Rc<RefCell<T>>` (used in single-threaded environments like WebAssembly)
///
/// It allows code to be written once but compile to the most efficient implementation
/// based on the environment where it will run and the features enabled.
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
/// This struct provides an abstraction over different weak reference types:
/// - `Weak<std::sync::RwLock<T>>` (used in standard multi-threaded environments)
/// - `Weak<tokio::sync::RwLock<T>>` (used for async/await support)
/// - `Weak<RefCell<T>>` (used in single-threaded environments like WebAssembly)
///
/// Weak references don't prevent the value from being dropped when no strong references
/// remain. This helps break reference cycles that could cause memory leaks.
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
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "tokio-sync")]
        {
            // For tokio-sync, we need to block on the async read
            use std::sync::Arc;
            let self_inner = Arc::clone(&self.tokio_inner);
            let other_inner = Arc::clone(&other.tokio_inner);

            tokio::task::block_in_place(|| {
                // Create a new runtime for this blocking operation
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let self_val = self_inner.read().await;
                    let other_val = other_inner.read().await;
                    *self_val == *other_val
                })
            })
        }

        #[cfg(not(feature = "tokio-sync"))]
        {
            match (self.read(), other.read()) {
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
            tokio_sync_warning!(
                "This method uses blocking operations when using tokio-sync feature, which is not ideal for async code. Consider using get_cloned_async() instead."
            );
            // For tokio-sync, we need to block on the async read
            // This is not ideal for async code, but it allows the method to work
            // in both sync and async contexts
            use std::sync::Arc;
            let inner = Arc::clone(&self.tokio_inner);
            let value = tokio::task::block_in_place(|| {
                // Create a new runtime for this blocking operation
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    let guard = inner.read().await;
                    (*guard).clone()
                })
            });
            Some(value)
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
    pub fn read(&self) -> Option<SharedReadGuard<T>> {
        #[cfg(feature = "tokio-sync")]
        {
            tokio_sync_warning!(
                "This method always returns None when using tokio-sync feature. Use read_async() instead."
            );
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
    pub fn write(&self) -> Option<SharedWriteGuard<T>> {
        #[cfg(feature = "tokio-sync")]
        {
            tokio_sync_warning!(
                "This method always returns None when using tokio-sync feature. Use write_async() instead."
            );
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
