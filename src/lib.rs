//! # Shared Container Module
//!
//! This module provides a unified abstraction over different container types
//! used for shared data access with interior mutability in different contexts.
//!
//! It abstracts over the differences between thread-safe `Arc<RwLock<T>>` used in
//! multi-threaded environments and `Rc<RefCell<T>>` used in single-threaded
//! environments like WebAssembly.
//!
//! This allows code using these containers to be written once but work efficiently
//! in both contexts.

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

// Native platforms use thread-safe types
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

// WebAssembly uses single-threaded types
#[cfg(target_arch = "wasm32")]
use std::cell::{Ref, RefCell, RefMut};
#[cfg(target_arch = "wasm32")]
use std::rc::{Rc, Weak};


/// A unified container for shared data that works in both multi-threaded and single-threaded environments.
///
/// This struct provides an abstraction over `Arc<RwLock<T>>` (used in multi-threaded environments)
/// and `Rc<RefCell<T>>` (used in single-threaded environments like WebAssembly).
///
/// It allows code to be written once but compile to the most efficient implementation
/// based on the environment where it will run.
#[derive(Debug)]
pub struct SharedContainer<T: Debug> {
    // Thread-safe implementation for native platforms
    #[cfg(not(target_arch = "wasm32"))]
    inner: Arc<RwLock<T>>,

    // Single-threaded implementation for WebAssembly
    #[cfg(target_arch = "wasm32")]
    inner: Rc<RefCell<T>>,
}

// Implement Send and Sync for SharedContainer only for non-wasm builds
#[cfg(not(target_arch = "wasm32"))]
unsafe impl<T: Debug + Send> Send for SharedContainer<T> {}

#[cfg(not(target_arch = "wasm32"))]
unsafe impl<T: Debug + Send + Sync> Sync for SharedContainer<T> {}

/// A weak reference to a `SharedContainer`.
///
/// This struct provides an abstraction over `Weak<RwLock<T>>` (used in multi-threaded environments)
/// and `Weak<RefCell<T>>` (used in single-threaded environments like WebAssembly).
///
/// Weak references don't prevent the value from being dropped when no strong references
/// remain. This helps break reference cycles that could cause memory leaks.
#[derive(Debug)]
pub struct WeakSharedContainer<T: Debug> {
    // Thread-safe implementation for native platforms
    #[cfg(not(target_arch = "wasm32"))]
    inner: Weak<RwLock<T>>,

    // Single-threaded implementation for WebAssembly
    #[cfg(target_arch = "wasm32")]
    inner: Weak<RefCell<T>>,
}

impl<T: Debug> Clone for WeakSharedContainer<T> {
    fn clone(&self) -> Self {
        // Same implementation for both platforms, but different underlying types
        WeakSharedContainer {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Debug + PartialEq> PartialEq for SharedContainer<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.read(), other.read()) {
            (Some(self_val), Some(other_val)) => *self_val == *other_val,
            _ => false,
        }
    }
}

impl<T: Debug> Clone for SharedContainer<T> {
    fn clone(&self) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            SharedContainer {
                inner: Arc::clone(&self.inner),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            SharedContainer {
                inner: Rc::clone(&self.inner),
            }
        }
    }
}

impl<T: Debug + Clone> SharedContainer<T> {
    /// Gets a clone of the contained value.
    ///
    /// This method acquires a read lock, clones the value, and releases the lock.
    ///
    /// # Returns
    /// * `Some(T)`: A clone of the contained value
    /// * `None`: If the lock couldn't be acquired
    pub fn get_cloned(&self) -> Option<T> {
        let guard = self.read()?;
        Some((*guard).clone())
    }
}

impl<T: Debug> SharedContainer<T> {
    /// Creates a new `SharedContainer` containing the given value.
    ///
    /// # Parameters
    /// * `value`: The value to store in the container
    ///
    /// # Returns
    /// A new `SharedContainer` instance containing the value
    pub fn new(value: T) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            SharedContainer {
                inner: Arc::new(RwLock::new(value)),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            SharedContainer {
                inner: Rc::new(RefCell::new(value)),
            }
        }
    }

    /// Gets a read-only access guard to the contained value.
    ///
    /// # Returns
    /// * `Some(SharedReadGuard<T>)`: A guard allowing read-only access to the value
    /// * `None`: If the lock couldn't be acquired (in multi-threaded mode)
    pub fn read(&self) -> Option<SharedReadGuard<T>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self.inner.read() {
                Ok(guard) => Some(SharedReadGuard::Multi(guard)),
                Err(_) => None,
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            match self.inner.try_borrow() {
                Ok(borrow) => Some(SharedReadGuard::Single(borrow)),
                Err(_) => None,
            }
        }
    }

    /// Gets a writable access guard to the contained value.
    ///
    /// # Returns
    /// * `Some(SharedWriteGuard<T>)`: A guard allowing read-write access to the value
    /// * `None`: If the lock couldn't be acquired (in multi-threaded mode)
    pub fn write(&self) -> Option<SharedWriteGuard<T>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self.inner.write() {
                Ok(guard) => Some(SharedWriteGuard::Multi(guard)),
                Err(_) => None,
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            match self.inner.try_borrow_mut() {
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            WeakSharedContainer {
                inner: Arc::downgrade(&self.inner),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            WeakSharedContainer {
                inner: Rc::downgrade(&self.inner),
            }
        }
    }
}

impl<T: Debug> WeakSharedContainer<T> {
    /// Attempts to create a strong `SharedContainer` from this weak reference.
    ///
    /// This will succeed if the value has not yet been dropped, i.e., if there are
    /// still other strong references to it.
    ///
    /// # Returns
    /// * `Some(SharedContainer<T>)`: If the value still exists
    /// * `None`: If the value has been dropped
    pub fn upgrade(&self) -> Option<SharedContainer<T>> {
        // Code is the same for both platforms, but types are different
        self.inner.upgrade().map(|inner| SharedContainer { inner })
    }
}
/// A read-only guard for accessing data in a `SharedContainer`.
///
/// This type abstracts over the differences between `RwLockReadGuard` (used in multi-threaded environments)
/// and `Ref` (used in single-threaded environments like WebAssembly).
///
/// It implements `Deref` to allow transparent access to the underlying data.
pub enum SharedReadGuard<'a, T: Debug> {
    #[cfg(not(target_arch = "wasm32"))]
    Multi(RwLockReadGuard<'a, T>),

    #[cfg(target_arch = "wasm32")]
    Single(Ref<'a, T>),
}

impl<'a, T: Debug> Deref for SharedReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self {
                SharedReadGuard::Multi(guard) => guard.deref(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            match self {
                SharedReadGuard::Single(borrow) => borrow.deref(),
            }
        }
    }
}

/// A writable guard for accessing and modifying data in a `SharedContainer`.
///
/// This type abstracts over the differences between `RwLockWriteGuard` (used in multi-threaded environments)
/// and `RefMut` (used in single-threaded environments like WebAssembly).
///
/// It implements both `Deref` and `DerefMut` to allow transparent access to the underlying data.
pub enum SharedWriteGuard<'a, T: Debug> {
    #[cfg(not(target_arch = "wasm32"))]
    Multi(RwLockWriteGuard<'a, T>),

    #[cfg(target_arch = "wasm32")]
    Single(RefMut<'a, T>),
}

impl<'a, T: Debug> Deref for SharedWriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self {
                SharedWriteGuard::Multi(guard) => guard.deref(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            match self {
                SharedWriteGuard::Single(borrow) => borrow.deref(),
            }
        }
    }
}

impl<'a, T: Debug> DerefMut for SharedWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match self {
                SharedWriteGuard::Multi(guard) => guard.deref_mut(),
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            match self {
                SharedWriteGuard::Single(borrow) => borrow.deref_mut(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestStruct {
        value: i32,
    }

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
